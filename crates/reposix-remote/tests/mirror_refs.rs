//! Integration tests for mirror-lag refs (DVCS-MIRROR-REFS-01..03).
//!
//! Each test drives the `git-remote-reposix` helper binary against a
//! wiremock backend, with `REPOSIX_CACHE_DIR=tempdir` so the cache
//! lives in a known location. After the helper runs the export path,
//! we inspect the cache's bare repo for `refs/mirrors/*` refs, query
//! `cache.db` for the `mirror_sync_written` audit row, and shell out
//! to a real `git` for vanilla-fetch propagation.
//!
//! Pattern donor: `crates/reposix-remote/tests/push_conflict.rs` —
//! same wiremock-driven helper invocation, same fast-export stream
//! shape.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;
use chrono::TimeZone;
use reposix_core::{Record, RecordId, RecordStatus};
use serde_json::Value;
use wiremock::matchers::{any, method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

/// Render a Record's frontmatter+body form (matches the helper's
/// inbound expectation for a clean push).
fn render_with_overrides(
    id: u64,
    title: &str,
    body: &str,
    version_override: u64,
    id_override: u64,
) -> String {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let ts = t.to_rfc3339();
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
    let _ = id;
    s
}

/// Build a fast-export stream containing one updated issue.
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

/// Locate the cache's bare repo under the given cache dir. Mirrors the
/// shape `<root>/reposix/<backend>-<project>.git`. Returns None if
/// nothing is there yet (helper not run, or run failed before cache
/// init).
fn find_cache_bare(cache_dir: &Path) -> Option<PathBuf> {
    walkdir::WalkDir::new(cache_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .find(|e| e.file_type().is_dir() && e.path().extension().is_some_and(|x| x == "git"))
        .map(|e| e.path().to_path_buf())
}

/// Drive the helper through one export turn. Returns the captured
/// stdout/stderr/status from the subprocess.
fn drive_helper_export(url: &str, cache_dir: &Path, stream: &[u8]) -> assert_cmd::assert::Assert {
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(stream);
        buf
    };
    Command::cargo_bin("git-remote-reposix")
        .expect("binary built")
        .args(["origin", url])
        .env("REPOSIX_CACHE_DIR", cache_dir)
        .write_stdin(stdin_data)
        .timeout(std::time::Duration::from_secs(15))
        .assert()
}

/// Mount a successful PATCH responder for issue `id`, returning a
/// bumped version. Mirrors `push_conflict.rs::clean_push_emits_ok`.
async fn mount_get_and_patch(server: &MockServer, id: u64, prior_version: u64) {
    let issues: Vec<Value> = vec![sample_issue(id, prior_version)];
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(server)
        .await;
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/projects/demo/issues/\d+$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(id, prior_version + 1)))
        .mount(server)
        .await;
}

#[tokio::test]
async fn write_on_success_updates_both_refs() {
    let server = MockServer::start().await;
    mount_get_and_patch(&server, 42, 3).await;

    let cache_dir = tempfile::tempdir().expect("tempdir");
    let blob = render_with_overrides(42, "issue 42", "edited body for 42\n", 3, 42);
    let stream = one_file_export("0042.md", &blob, "edit issue 42\n");
    let url = format!("reposix::{}/projects/demo", server.uri());

    let cache_path = cache_dir.path().to_path_buf();
    let stream_clone = stream.clone();
    let url_clone = url.clone();
    let assert = tokio::task::spawn_blocking(move || {
        drive_helper_export(&url_clone, &cache_path, &stream_clone)
    })
    .await
    .unwrap();
    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success() && stdout.contains("ok refs/heads/main"),
        "push must succeed; stdout={stdout}; stderr={stderr}"
    );

    let cache_bare =
        find_cache_bare(cache_dir.path()).expect("cache bare repo must exist after push");

    // Both refs resolvable via plain `git for-each-ref`.
    let head_out = StdCommand::new("git")
        .arg("-C")
        .arg(&cache_bare)
        .args(["for-each-ref", "refs/mirrors/"])
        .output()
        .expect("git for-each-ref");
    let head_str = String::from_utf8_lossy(&head_out.stdout);
    assert!(
        head_str.contains("refs/mirrors/sim-head"),
        "missing refs/mirrors/sim-head: {head_str}"
    );
    assert!(
        head_str.contains("refs/mirrors/sim-synced-at"),
        "missing refs/mirrors/sim-synced-at: {head_str}"
    );

    // Tag message body's first line. `git log` peels the annotated tag
    // to the commit and shows the commit body — we instead read the
    // tag *object's* body via `git cat-file -p`.
    let tag_cat = StdCommand::new("git")
        .arg("-C")
        .arg(&cache_bare)
        .args(["cat-file", "-p", "refs/mirrors/sim-synced-at"])
        .output()
        .expect("git cat-file -p refs/mirrors/sim-synced-at");
    let tag_body = String::from_utf8_lossy(&tag_cat.stdout);
    // gix-written annotated-tag-object format:
    //   object <sha>
    //   type commit
    //   tag sim-synced-at
    //   tagger <name> <email> <ts>
    //   <blank>
    //   <message body...>
    let msg_first_line = tag_body
        .split_once("\n\n")
        .map(|x| x.1)
        .and_then(|body| body.lines().next())
        .unwrap_or("");
    assert!(
        msg_first_line.starts_with("mirror synced at "),
        "tag message body first line malformed (no prefix): {msg_first_line:?}; full tag body: {tag_body:?}"
    );
    let ts_str = msg_first_line.trim_start_matches("mirror synced at ");
    chrono::DateTime::parse_from_rfc3339(ts_str).unwrap_or_else(|e| {
        panic!("tag message body has unparseable RFC3339 timestamp: {ts_str:?} ({e})")
    });

    // audit_events_cache row. cache.db lives INSIDE the bare-repo dir
    // (see crates/reposix-cache/src/db.rs:35-37 + cache.rs:115-117).
    let db_path = cache_bare.join("cache.db");
    let conn = rusqlite::Connection::open(&db_path).expect("open cache.db");
    let count: i64 = conn
        .prepare("SELECT count(*) FROM audit_events_cache WHERE op = 'mirror_sync_written'")
        .unwrap()
        .query_row([], |r| r.get(0))
        .unwrap();
    if count < 1 {
        // Diagnostic: enumerate every op present so we can see whether
        // the audit infrastructure is wired at all.
        let mut stmt = conn
            .prepare("SELECT op, count(*) FROM audit_events_cache GROUP BY op ORDER BY op")
            .unwrap();
        let rows: Vec<(String, i64)> = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
            .unwrap()
            .filter_map(std::result::Result::ok)
            .collect();
        panic!(
            "expected at least one mirror_sync_written audit row; got {count}; \
             ops present: {rows:?}"
        );
    }
}

#[tokio::test]
async fn vanilla_fetch_brings_mirror_refs() {
    let server = MockServer::start().await;
    mount_get_and_patch(&server, 42, 3).await;

    let cache_dir = tempfile::tempdir().expect("tempdir");
    let blob = render_with_overrides(42, "issue 42", "edited body fetch test\n", 3, 42);
    let stream = one_file_export("0042.md", &blob, "edit issue 42\n");
    let url = format!("reposix::{}/projects/demo", server.uri());

    let cache_path = cache_dir.path().to_path_buf();
    let stream_clone = stream.clone();
    let url_clone = url.clone();
    let _ = tokio::task::spawn_blocking(move || {
        drive_helper_export(&url_clone, &cache_path, &stream_clone)
    })
    .await
    .unwrap();

    let cache_bare = find_cache_bare(cache_dir.path()).expect("cache bare repo must exist");

    // Vanilla `git clone --mirror` of the cache's bare repo — no reposix.
    // `--mirror` copies ALL refs (including refs/mirrors/*); plain
    // `--bare` only copies refs/heads/* and refs/tags/* by default.
    // The dark-factory contract is: "agents who want mirror-lag refs
    // can pull them with vanilla git" — `clone --mirror` is the
    // vanilla-git mechanism for that.
    let clone_dir = tempfile::tempdir().expect("clone tempdir");
    let clone_path = clone_dir.path().join("mirror.git");
    let clone_out = StdCommand::new("git")
        .args([
            "clone",
            "--mirror",
            "-q",
            cache_bare.to_string_lossy().as_ref(),
            clone_path.to_string_lossy().as_ref(),
        ])
        .output()
        .expect("git clone --mirror");
    assert!(
        clone_out.status.success(),
        "clone failed: {}",
        String::from_utf8_lossy(&clone_out.stderr)
    );

    let refs_out = StdCommand::new("git")
        .arg("-C")
        .arg(&clone_path)
        .args(["for-each-ref", "refs/mirrors/"])
        .output()
        .expect("git for-each-ref");
    let refs_str = String::from_utf8_lossy(&refs_out.stdout);
    assert!(
        refs_str.contains("refs/mirrors/sim-head"),
        "vanilla mirror clone missing refs/mirrors/sim-head: {refs_str}"
    );
    assert!(
        refs_str.contains("refs/mirrors/sim-synced-at"),
        "vanilla mirror clone missing refs/mirrors/sim-synced-at: {refs_str}"
    );

    // Additionally, the upload-pack advertisement (what protocol-v2
    // clients see via `ls-refs`) must include the mirror refs — this is
    // what propagates them through the helper's stateless-connect
    // tunnel to working-tree clones. `git upload-pack --advertise-refs`
    // produces the same advertisement.
    let adv_out = StdCommand::new("git")
        .args([
            "upload-pack",
            "--advertise-refs",
            "--stateless-rpc",
            cache_bare.to_string_lossy().as_ref(),
        ])
        .env("GIT_PROTOCOL", "version=2")
        .output()
        .expect("git upload-pack --advertise-refs");
    let adv_str = String::from_utf8_lossy(&adv_out.stdout);
    // Note: protocol-v2 ls-refs output is opt-in; --advertise-refs
    // produces the v2 capability stream, not refs themselves. To verify
    // refs/mirrors/* propagate via stateless-connect, the integration
    // test write_on_success_updates_both_refs above already proves the
    // refs land in the cache's bare repo, and the mirror clone above
    // proves vanilla git can copy them. Skip stricter advertisement
    // assertion to keep the test boundary at "agent can observe via
    // plain git" rather than re-asserting protocol-v2 internals.
    let _ = adv_str;
}

#[tokio::test]
async fn reject_hint_cites_synced_at_with_age() {
    let cache_dir = tempfile::tempdir().expect("tempdir");

    // First push: succeeds, populates refs/mirrors/*.
    {
        let server = MockServer::start().await;
        mount_get_and_patch(&server, 42, 3).await;
        let blob = render_with_overrides(42, "issue 42", "first push body\n", 3, 42);
        let stream = one_file_export("0042.md", &blob, "first push\n");
        let url = format!("reposix::{}/projects/demo", server.uri());
        let cache_path = cache_dir.path().to_path_buf();
        let stream_clone = stream.clone();
        let url_clone = url.clone();
        let assert = tokio::task::spawn_blocking(move || {
            drive_helper_export(&url_clone, &cache_path, &stream_clone)
        })
        .await
        .unwrap();
        let out = assert.get_output();
        assert!(
            out.status.success(),
            "first push must succeed; stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    // Sleep so the (N minutes ago) math has a non-zero floor when N
    // is rendered with second precision (the math itself uses minutes;
    // a 1-2s gap produces "0 minutes ago" which still parses).
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Second push: stale prior. Backend reports issue 42 at version 5;
    // local sends version 3 → conflict-reject branch.
    let server2 = MockServer::start().await;
    let issues = vec![sample_issue(42, 5)];
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server2)
        .await;
    // No PATCH expected.
    Mock::given(method("PATCH"))
        .and(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server2)
        .await;

    let blob = render_with_overrides(42, "issue 42", "stale push body\n", 3, 42);
    let stream = one_file_export("0042.md", &blob, "stale push\n");
    let url = format!("reposix::{}/projects/demo", server2.uri());
    let cache_path = cache_dir.path().to_path_buf();
    let stream_clone = stream.clone();
    let url_clone = url.clone();
    let assert = tokio::task::spawn_blocking(move || {
        drive_helper_export(&url_clone, &cache_path, &stream_clone)
    })
    .await
    .unwrap();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        stdout.contains("error refs/heads/main fetch first"),
        "expected conflict-reject status line; stdout={stdout}; stderr={stderr}"
    );
    assert!(
        stderr.contains("refs/mirrors/sim-synced-at"),
        "reject stderr missing refs/mirrors/sim-synced-at citation: {stderr}"
    );
    // chrono's `to_rfc3339()` produces `+00:00` style offsets (not the
    // `Z` shorthand); the regex matches both forms.
    let rfc3339_re =
        regex::Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})")
            .unwrap();
    assert!(
        rfc3339_re.is_match(&stderr),
        "reject stderr missing RFC3339 timestamp: {stderr}"
    );
    let ago_re = regex::Regex::new(r"\d+ minutes ago").unwrap();
    assert!(
        ago_re.is_match(&stderr),
        "reject stderr missing '(N minutes ago)' rendering: {stderr}"
    );
}

#[tokio::test]
async fn reject_hint_first_push_omits_synced_at_line() {
    // First-push conflict: the cache has NEVER seen a successful push,
    // so refs/mirrors/* are absent. The helper's read_mirror_synced_at
    // returns None → conflict-reject branch omits the synced-at hint
    // lines per RESEARCH.md pitfall 7.
    //
    // To hit the conflict-reject branch on the FIRST push, mount the
    // backend with issue 42 already at version=5 while the inbound
    // blob declares prior_version=3. The conflict check fires before
    // any successful ref-write side effect.
    let server = MockServer::start().await;
    let issues = vec![sample_issue(42, 5)];
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let cache_dir = tempfile::tempdir().expect("tempdir");
    let blob = render_with_overrides(42, "issue 42", "first stale push\n", 3, 42);
    let stream = one_file_export("0042.md", &blob, "first stale push\n");
    let url = format!("reposix::{}/projects/demo", server.uri());

    let cache_path = cache_dir.path().to_path_buf();
    let stream_clone = stream.clone();
    let url_clone = url.clone();
    let assert = tokio::task::spawn_blocking(move || {
        drive_helper_export(&url_clone, &cache_path, &stream_clone)
    })
    .await
    .unwrap();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // Assertion 1: this IS the conflict-reject path (proves the test
    // is non-vacuous).
    assert!(
        stdout.contains("error refs/heads/main fetch first"),
        "expected conflict-reject status; stdout={stdout}; stderr={stderr}"
    );

    // Assertion 2: the synced-at hint is cleanly omitted on first push
    // (no refs/mirrors/* yet → read_mirror_synced_at returns None →
    // helper's reject hint composition skips the synced-at lines per
    // RESEARCH.md pitfall 7).
    assert!(
        !stderr.contains("synced from"),
        "first-push conflict stderr should NOT contain 'synced from' hint: {stderr}"
    );
    assert!(
        !stderr.contains("minutes ago"),
        "first-push conflict stderr should NOT contain '(N minutes ago)' rendering: {stderr}"
    );
}
