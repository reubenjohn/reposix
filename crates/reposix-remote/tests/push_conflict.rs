//! Phase 34 Plan 02 push-path integration tests.
//!
//! Three scenarios end-to-end against a wiremock backend:
//!
//! 1. **`stale_base_push_emits_fetch_first_and_writes_no_rest`** —
//!    ARCH-08 regression. The agent's local base version of issue 2 is
//!    stale (1 vs backend's 2); the helper must reject with the canned
//!    `error refs/heads/main fetch first` status, write a stderr
//!    diagnostic mentioning the issue id and `git pull --rebase`, and
//!    NOT make any PATCH/POST/DELETE call.
//! 2. **`clean_push_emits_ok_and_mutates_backend`** — happy-path
//!    regression. Local base matches backend; the helper writes
//!    `ok refs/heads/main` and the backend sees a PATCH for the
//!    changed issue.
//! 3. **`frontmatter_strips_server_controlled_fields`** — ARCH-10
//!    regression. The inbound blob has `id: 999999` and `version: 999`
//!    overrides; the PATCH body sent to the backend must contain the
//!    server-authoritative id (42) and NOT contain `version: 999`.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::fmt::Write as _;
use std::io::Write;
use std::sync::Arc;

use assert_cmd::Command;
use chrono::TimeZone;
use reposix_cache::Cache;
use reposix_core::{BackendConnector, Record, RecordId, RecordStatus};
use serde_json::Value;
use wiremock::matchers::{any, method, path_regex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

mod common;
use common::{sim_backend, CacheDirGuard};

fn sample_issue(id: u64, version: u64) -> Value {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let i = Record {
        id: RecordId(id),
        title: format!("issue {id}"),
        status: RecordStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: t,
        updated_at: t,
        version,
        body: format!("body of {id}\n"),
        parent_id: None,
        extensions: std::collections::BTreeMap::new(),
    };
    serde_json::to_value(i).unwrap()
}

/// Render `Record` to its on-disk frontmatter+body form, then override
/// the `version` field in the YAML so we can simulate stale-base or
/// hijacked-version pushes. `version_override` and `id_override` let
/// the caller forge those server-controlled fields.
fn render_with_overrides(
    id: u64,
    title: &str,
    body: &str,
    version_override: u64,
    id_override: u64,
) -> String {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let ts = t.to_rfc3339();
    // Hand-roll the YAML so we can control field order and override
    // server-controlled fields exactly.
    let mut s = String::new();
    s.push_str("---\n");
    writeln!(&mut s, "id: {id_override}").unwrap();
    writeln!(&mut s, "title: {title}").unwrap();
    s.push_str("status: open\n");
    writeln!(&mut s, "created_at: {ts}").unwrap();
    writeln!(&mut s, "updated_at: {ts}").unwrap();
    writeln!(&mut s, "version: {version_override}").unwrap();
    s.push_str("---\n");
    s.push_str(body);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    let _ = id; // logical id passed for clarity; emitted via path
    s
}

/// Build a fast-export stream containing one updated issue.
/// `path` is e.g. `issues/42.md`; `blob` is the rendered frontmatter+body.
fn one_file_export(path: &str, blob: &str, msg: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    writeln!(&mut out, "feature done").unwrap();
    writeln!(&mut out, "blob").unwrap();
    writeln!(&mut out, "mark :100").unwrap();
    writeln!(&mut out, "data {}", blob.len()).unwrap();
    out.extend_from_slice(blob.as_bytes());
    out.push(b'\n');
    writeln!(&mut out, "commit refs/heads/main").unwrap();
    writeln!(&mut out, "mark :1").unwrap();
    writeln!(&mut out, "committer test <t@t> 0 +0000").unwrap();
    let bytes = msg.as_bytes();
    writeln!(&mut out, "data {}", bytes.len()).unwrap();
    out.extend_from_slice(bytes);
    out.push(b'\n');
    writeln!(&mut out, "M 100644 :100 {path}").unwrap();
    writeln!(&mut out, "done").unwrap();
    out
}

/// ARCH-08 regression: stale-base push (local version=1, backend
/// version=2) must reject with canned `fetch first` and write zero
/// PATCH/POST/DELETE calls.
#[tokio::test]
// test-name-honesty: ok — drives helper export via stdin against wiremock; genuine push-path coverage
async fn stale_base_push_emits_fetch_first_and_writes_no_rest() {
    let server = MockServer::start().await;
    // Backend has issue 2 at version=2 — local will claim version=1.
    let issues: Vec<Value> = vec![sample_issue(1, 1), sample_issue(2, 2)];
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    // P81 L1 precheck does ONE backend.get_record(id) per record in
    // changed_set ∩ push_set on the cursor-present hot path. Mount
    // per-issue GETs so the precheck gets the v2 response and emits
    // the correct fetch-first reject.
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues/1$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(1, 1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues/2$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(2, 2)))
        .mount(&server)
        .await;
    // Strict expectation: NO writes should fire.
    Mock::given(method("PATCH"))
        .and(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(any())
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(any())
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;

    let blob = render_with_overrides(2, "issue 2", "edited body\n", 1, 2);
    let stream = one_file_export("issues/2.md", &blob, "edit issue 2\n");

    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&stream);
        buf
    };
    // Per-test cache_dir isolation — under cargo llvm-cov, tests in the
    // same workspace can share host-level cache state via the default
    // REPOSIX_CACHE_DIR, polluting each other's prior trees. Pinning a
    // tempdir per test keeps the cache deterministically empty/built.
    let cache_dir = tempfile::tempdir().expect("cache_dir tempdir");
    let cache_path = cache_dir.path().to_path_buf();
    let assert = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .write_stdin(stdin_data)
            .timeout(std::time::Duration::from_secs(15))
            .assert()
    })
    .await
    .unwrap();
    drop(cache_dir);
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "stale-base push must fail; stderr: {stderr}"
    );
    assert!(
        stdout.contains("error refs/heads/main fetch first"),
        "stdout missing canned status: {stdout}"
    );
    assert!(
        stderr.contains("issue 2"),
        "stderr missing issue id: {stderr}"
    );
    assert!(
        stderr.contains("git pull --rebase"),
        "stderr missing git-pull recovery hint: {stderr}"
    );
    // P121 W3.5: the human-facing conflict diag carries its stable RPX-0505 tag
    // (on STDERR), while the git-parsed `fetch first` status on STDOUT stays
    // untagged — tagging the protocol line would break the helper.
    assert!(
        stderr.contains("[RPX-0505]"),
        "stderr conflict diag missing RPX-0505 tag: {stderr}"
    );
    assert!(
        stderr.contains("reposix explain RPX-0505"),
        "stderr conflict diag missing `reposix explain RPX-0505` nudge: {stderr}"
    );
    assert!(
        !stdout.contains("RPX-0505"),
        "the git-parsed `fetch first` protocol line on STDOUT must NOT carry an \
         RPX code (it would corrupt the push status git parses): {stdout}"
    );
}

/// SC3 / DRAIN-12 regression: when the mirror-lag ref (`refs/mirrors/<sot>-synced-at`)
/// is populated, a stale-base push must reject with a mirror-lag hint that (a)
/// recommends `reposix sync --reconcile` — the real cache-refresh command, NOT the
/// no-op bare `reposix sync` — and (b) warns that on a `reposix attach` (Pattern-C)
/// tree a bare `git pull`/`git rebase` reads the ORIGIN MIRROR by default, pointing
/// at a remote-explicit rebase against the SoT-backed bus remote. The load-bearing
/// pinned `git pull --rebase` substring must survive both before and after the fix.
///
/// Setup drives the branch honestly: an in-process `Cache` warms the same
/// `REPOSIX_CACHE_DIR` the subprocess consumes, then `write_mirror_synced_at`
/// populates the ref so `read_mirror_synced_at` returns `Some` inside `write_loop.rs`
/// (the branch under test fires ONLY on that condition — see `write_loop.rs:208`).
#[tokio::test]
// test-name-honesty: ok — drives helper export via stdin against wiremock; genuine mirror-lag-hint coverage
async fn mirror_lag_reject_hint_recommends_reconcile_and_remote_explicit_rebase() {
    let server = MockServer::start().await;
    // Backend has issue 2 at version=2 — local push will claim version=1.
    let issues: Vec<Value> = vec![sample_issue(1, 1), sample_issue(2, 2)];
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues/1$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(1, 1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues/2$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(2, 2)))
        .mount(&server)
        .await;
    // Strict expectation: the reject must fire BEFORE any write reaches the backend.
    Mock::given(method("PATCH"))
        .and(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(any())
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(any())
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;

    // Per-test cache dir — the SAME path the subprocess will consume below.
    let cache_dir = tempfile::tempdir().expect("cache_dir tempdir");
    let cache_path = cache_dir.path().to_path_buf();

    // Load-bearing new step vs the sibling test: pre-populate the mirror-lag ref so
    // the `if let Ok(Some(synced_at)) = c.read_mirror_synced_at(backend_name)` arm in
    // write_loop.rs actually fires. Open an in-process Cache on the same dir, warm it
    // (sync() commits refs/heads/main + sets HEAD — the tag-object target
    // write_mirror_synced_at needs), then stamp the synced-at ref 5 minutes in the
    // past. Backend host is `"sim"` (loopback origin classifies as BackendKind::Sim,
    // so the subprocess helper reads refs/mirrors/sim-synced-at). Mirrors
    // bus_precheck_b.rs:110-115.
    {
        let _env = CacheDirGuard::new(&cache_path);
        let backend: Arc<dyn BackendConnector> = sim_backend(&server);
        let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
        cache
            .sync()
            .await
            .expect("seed sync (warm cache cursor + HEAD for synced-at target)");
        cache
            .write_mirror_synced_at("sim", chrono::Utc::now() - chrono::Duration::minutes(5))
            .expect("populate refs/mirrors/sim-synced-at");
        drop(cache); // release the cache lock + (on _env drop) restore the env var
    }

    // Drive the stale-base push exactly as the sibling test does.
    let blob = render_with_overrides(2, "issue 2", "edited body\n", 1, 2);
    let stream = one_file_export("issues/2.md", &blob, "edit issue 2\n");
    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&stream);
        buf
    };
    let cache_path_sub = cache_path.clone();
    let assert = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .env("REPOSIX_CACHE_DIR", &cache_path_sub)
            .write_stdin(stdin_data)
            .timeout(std::time::Duration::from_secs(15))
            .assert()
    })
    .await
    .unwrap();
    drop(cache_dir);
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        !out.status.success(),
        "stale-base push must fail; stderr: {stderr}"
    );
    assert!(
        stdout.contains("error refs/heads/main fetch first"),
        "stdout missing canned conflict status: {stdout}"
    );
    // Prove the mirror-lag branch actually fired (guards against a mis-setup
    // false-negative that would otherwise look like a missing-impl failure):
    assert!(
        stderr.contains("last synced from"),
        "mirror-lag branch did not fire — refs/mirrors/<sot>-synced-at was not populated: {stderr}"
    );
    // SC3 bug (a): the corrected `--reconcile` flag on the cache-refresh hint. The
    // bare `reposix sync` is a no-op (per sync.rs's own doc comment) — the recovery
    // must name the flag that actually rebuilds the LOCAL cache.
    assert!(
        stderr.contains("reposix sync --reconcile"),
        "stderr still recommends the no-op bare `reposix sync` (missing --reconcile): {stderr}"
    );
    // SC3 bug (b): the Pattern-C remote-explicit augmentation — a bare pull on an
    // attach tree reads the stale origin mirror, so the recovery names the bus remote.
    assert!(
        stderr.contains("git pull --rebase <reposix-remote-name> main")
            || stderr.contains("reposix attach"),
        "stderr missing the Pattern-C bare-pull-reads-stale-mirror augmentation hint: {stderr}"
    );
    // Pin survives (TRUE both before and after the fix — never removed):
    assert!(
        stderr.contains("git pull --rebase"),
        "pinned `git pull --rebase` substring vanished: {stderr}"
    );
}

/// P121 W3.5: a single-backend push whose L1 precheck cannot reach the `SoT`
/// (here a dead loopback port → connection refused) rejects with the git-parsed
/// `error refs/heads/main backend-unreachable` status on STDOUT and a human-facing
/// diag carrying the stable RPX-0504 tag on STDERR. Reality-first: drives the REAL
/// helper against an unreachable backend, no mock. Hermetic — loopback:9 refuses
/// instantly, isolated cache, no shared repo.
#[tokio::test]
// test-name-honesty: ok — drives the real helper export against an unreachable loopback SoT and asserts the RPX-0504 tag on the stderr diag + the untagged protocol status on stdout
async fn backend_unreachable_push_tags_diag_rpx_0504_not_protocol_line() {
    let blob = render_with_overrides(1, "issue 1", "edited body\n", 1, 1);
    let stream = one_file_export("issues/1.md", &blob, "edit issue 1\n");
    // Nothing listening on loopback:9 → the precheck's REST call is refused.
    let url = "reposix::http://127.0.0.1:9/projects/demo".to_string();
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&stream);
        buf
    };
    let cache_dir = tempfile::tempdir().expect("cache_dir tempdir");
    let cache_path = cache_dir.path().to_path_buf();
    let assert = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .write_stdin(stdin_data)
            .timeout(std::time::Duration::from_secs(15))
            .assert()
    })
    .await
    .unwrap();
    drop(cache_dir);
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "unreachable-backend push must fail; stderr: {stderr}"
    );
    assert!(
        stdout.contains("error refs/heads/main backend-unreachable"),
        "stdout missing the git-parsed backend-unreachable status: {stdout}"
    );
    assert!(
        stderr.contains("[RPX-0504]"),
        "stderr precheck diag missing RPX-0504 tag: {stderr}"
    );
    assert!(
        stderr.contains("reposix explain RPX-0504"),
        "stderr precheck diag missing `reposix explain RPX-0504` nudge: {stderr}"
    );
    assert!(
        !stdout.contains("RPX-0504"),
        "the git-parsed protocol status on STDOUT must NOT carry an RPX code: {stdout}"
    );
}

/// P121 W3.6 (FIX 4, P121-review): the FETCH/import sibling of the RPX-0504
/// push test above. An `import` batch whose `list_records` cannot reach the `SoT`
/// (here a dead loopback port → connection refused) rejects with the git-parsed
/// `error refs/heads/main backend-unreachable` status on STDOUT and a human-facing
/// diag carrying the stable RPX-0507 tag on STDERR. Closes the gap the committed
/// `import_unreachable_detail_renders_rpx0507_...` string-builder unit test left:
/// this drives the REAL helper `import` command end-to-end, verifying the live
/// path actually emits the tag (not just that the string builder can). Reality-
/// first: no mock — loopback:9 refuses instantly. Hermetic, isolated cache.
#[tokio::test]
// test-name-honesty: ok — drives the real helper import command against an unreachable loopback SoT and asserts the RPX-0507 tag on the stderr diag + the untagged backend-unreachable protocol status on stdout
async fn backend_unreachable_import_tags_diag_rpx_0507_not_protocol_line() {
    // Nothing listening on loopback:9 → the import's list_records is refused.
    let url = "reposix::http://127.0.0.1:9/projects/demo".to_string();
    // `import refs/heads/main` then a blank line terminates the import batch;
    // handle_import_batch then calls list_records, which fails → RPX-0507.
    let stdin_data = "import refs/heads/main\n\n".to_owned();
    let cache_dir = tempfile::tempdir().expect("cache_dir tempdir");
    let cache_path = cache_dir.path().to_path_buf();
    let assert = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .write_stdin(stdin_data)
            .timeout(std::time::Duration::from_secs(15))
            .assert()
    })
    .await
    .unwrap();
    drop(cache_dir);
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "unreachable-backend import must fail; stderr: {stderr}"
    );
    assert!(
        stdout.contains("error refs/heads/main backend-unreachable"),
        "stdout missing the git-parsed backend-unreachable status: {stdout}"
    );
    assert!(
        stderr.contains("[RPX-0507]"),
        "stderr import diag missing RPX-0507 tag: {stderr}"
    );
    assert!(
        stderr.contains("reposix explain RPX-0507"),
        "stderr import diag missing `reposix explain RPX-0507` nudge: {stderr}"
    );
    assert!(
        !stdout.contains("RPX-0507"),
        "the git-parsed protocol status on STDOUT must NOT carry an RPX code \
         (it would corrupt the status git parses): {stdout}"
    );
}

/// Happy-path regression: local base matches backend, body change goes
/// through. Helper emits `ok refs/heads/main`; backend sees one PATCH.
#[tokio::test]
// test-name-honesty: ok — drives helper export via stdin against wiremock; genuine push-path coverage
async fn clean_push_emits_ok_and_mutates_backend() {
    let server = MockServer::start().await;
    let issues: Vec<Value> = vec![sample_issue(42, 3)];
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    // P81 L1 precheck does ONE backend.get_record(id) per record in
    // changed_set ∩ push_set on the cursor-present hot path.
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues/42$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(42, 3)))
        .mount(&server)
        .await;
    // Backend echoes back an updated issue — version bumped to 4.
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/projects/demo/issues/42$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(42, 4)))
        .expect(1)
        .mount(&server)
        .await;

    let blob = render_with_overrides(42, "issue 42", "edited body for 42\n", 3, 42);
    let stream = one_file_export("issues/42.md", &blob, "edit issue 42\n");

    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&stream);
        buf
    };
    // Per-test cache_dir isolation — under cargo llvm-cov, tests in the
    // same workspace can share host-level cache state via the default
    // REPOSIX_CACHE_DIR, polluting each other's prior trees. Pinning a
    // tempdir per test keeps the cache deterministically empty/built.
    let cache_dir = tempfile::tempdir().expect("cache_dir tempdir");
    let cache_path = cache_dir.path().to_path_buf();
    let assert = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .write_stdin(stdin_data)
            .timeout(std::time::Duration::from_secs(15))
            .assert()
    })
    .await
    .unwrap();
    drop(cache_dir);
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "clean push must succeed; stderr: {stderr}"
    );
    assert!(
        stdout.contains("ok refs/heads/main"),
        "stdout missing ok: {stdout}"
    );
}

/// ARCH-10 regression: an inbound blob with `id: 999999` and
/// `version: 999` does NOT carry those values into the PATCH body.
/// The helper's `sanitize()` step replaces them with the server-trusted
/// id and `prior_version` BEFORE serializing the request body.
///
/// Strategy: capture the PATCH body via wiremock. The sim's
/// `PatchIssueBody` has `deny_unknown_fields` and only carries the
/// mutable-field subset, so server-controlled fields are stripped at
/// the wire boundary. The ARCH-10 assertion is that no attacker-supplied
/// `999_999` / `999` value leaks into ANY field of the PATCH body.
#[tokio::test]
async fn frontmatter_strips_server_controlled_fields() {
    let server = MockServer::start().await;
    let issues: Vec<Value> = vec![sample_issue(42, 3)];
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    // P81 L1 precheck does ONE backend.get_record(id) per record in
    // changed_set ∩ push_set on the cursor-present hot path.
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues/42$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(42, 3)))
        .mount(&server)
        .await;
    let captured: Arc<std::sync::Mutex<Vec<Value>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_clone = captured.clone();
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/projects/demo/issues/42$"))
        .respond_with(move |req: &Request| {
            if let Ok(v) = serde_json::from_slice::<Value>(&req.body) {
                captured_clone.lock().unwrap().push(v);
            }
            ResponseTemplate::new(200).set_body_json(sample_issue(42, 4))
        })
        .expect(1)
        .mount(&server)
        .await;

    // Inbound blob: id_override=999999, version=3 (matches backend so
    // conflict-check passes), but if the *bytes that hit PATCH* include
    // 999999 anywhere, sanitize is broken.
    let blob = render_with_overrides(
        42,
        "issue 42",
        "edited body sanitize regression\n",
        3,       // version: matches backend so conflict-check passes
        999_999, // id override — server must replace with 42
    );
    let stream = one_file_export("issues/42.md", &blob, "sanitize regression\n");

    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&stream);
        buf
    };
    // Per-test cache_dir isolation — under cargo llvm-cov, tests in the
    // same workspace can share host-level cache state via the default
    // REPOSIX_CACHE_DIR, polluting each other's prior trees. Pinning a
    // tempdir per test keeps the cache deterministically empty/built.
    let cache_dir = tempfile::tempdir().expect("cache_dir tempdir");
    let cache_path = cache_dir.path().to_path_buf();
    let assert = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .write_stdin(stdin_data)
            .timeout(std::time::Duration::from_secs(15))
            .assert()
    })
    .await
    .unwrap();
    drop(cache_dir);
    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "sanitize-regression push must succeed; stderr: {stderr}"
    );

    let captured = captured.lock().unwrap();
    assert_eq!(captured.len(), 1, "exactly one PATCH should fire");
    let body = &captured[0];
    // The sim's PATCH body has `deny_unknown_fields` and only carries
    // the mutable-field subset (title/body/status/assignee/labels).
    // Server-controlled fields (id/created_at/updated_at/version) are
    // stripped by construction at the wire boundary. The ARCH-10
    // guarantee is therefore: no attacker-supplied 999_999 / 999 leaks
    // into ANY field of the PATCH body.
    let body_str = body.to_string();
    assert!(
        !body_str.contains("999999"),
        "attacker id=999999 leaked into PATCH body: {body_str}"
    );
    assert!(
        !body_str.contains("\"version\""),
        "PATCH body must not include `version` field at all (server-controlled): {body_str}"
    );
    assert!(
        !body_str.contains("\"id\""),
        "PATCH body must not include `id` field at all (server-controlled, lives in URL path): {body_str}"
    );
    // And the URL was the canonical /issues/42 endpoint (proves the
    // helper used the server-trusted id derived from the path, not the
    // hijacked value from the frontmatter).
}
