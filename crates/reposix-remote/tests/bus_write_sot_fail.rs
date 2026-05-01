//! Bus write fan-out fault-injection (b) integration test (DVCS-BUS-WRITE-06).
//!
//! RESEARCH.md § "Test (b)": confluence write fails mid-stream (5xx on
//! second PATCH). Helper must:
//!
//! - exit non-zero with `error refs/heads/main some-actions-failed`.
//! - NO mirror push attempted.
//! - mirror baseline preserved (mirror's main ref absent — empty bare).
//! - `helper_push_accepted` count == 0 (didn't reach success branch).
//! - `helper_push_partial_fail_mirror_lag` count == 0 (mirror never attempted).
//! - wiremock saw exactly 2 PATCH requests (id=1 → 200; id=2 → 500).
//!
//! D-09 / Pitfall 3: SoT partial state is observable on wiremock's
//! request log (id=1 server-side updated; id=2 unchanged). The test
//! does NOT assert all-or-nothing — that's not what the helper does.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)] // test-internal doc comments cite SoT/refs/audit ops verbatim
#![allow(clippy::too_many_lines)] // narrow integration-test setup
#![allow(clippy::unnecessary_debug_formatting)]

use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::Arc;

use assert_cmd::Command as AssertCommand;
use chrono::TimeZone;
use reposix_cache::Cache;
use reposix_core::BackendConnector;
use serde_json::json;
use wiremock::matchers::{method, path_regex};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

mod common;
use common::{count_audit_cache_rows, sample_issues, seed_mock, sim_backend, CacheDirGuard};

struct HasSinceQueryParam;
impl Match for HasSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().any(|(k, _)| k == "since")
    }
}

fn run_git_in(dir: &Path, args: &[&str]) -> String {
    let out = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap_or_else(|e| panic!("spawn git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} in {dir:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

fn render_issue_blob(id: u64, version: u64, body: &str) -> String {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let ts = t.to_rfc3339();
    let mut s = String::new();
    s.push_str("---\n");
    writeln!(&mut s, "id: {id}").unwrap();
    writeln!(&mut s, "title: issue {id} in demo").unwrap();
    s.push_str("status: open\n");
    writeln!(&mut s, "created_at: {ts}").unwrap();
    writeln!(&mut s, "updated_at: {ts}").unwrap();
    writeln!(&mut s, "version: {version}").unwrap();
    s.push_str("---\n");
    s.push_str(body);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s
}

fn multi_file_export(entries: &[(&str, String)], msg: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    writeln!(&mut out, "feature done").unwrap();
    let base_mark: u64 = 100;
    for (i, (_, blob)) in entries.iter().enumerate() {
        writeln!(&mut out, "blob").unwrap();
        writeln!(&mut out, "mark :{}", base_mark + i as u64).unwrap();
        writeln!(&mut out, "data {}", blob.len()).unwrap();
        out.extend_from_slice(blob.as_bytes());
        out.push(b'\n');
    }
    writeln!(&mut out, "commit refs/heads/main").unwrap();
    writeln!(&mut out, "mark :1").unwrap();
    writeln!(&mut out, "committer test <t@t> 0 +0000").unwrap();
    let bytes = msg.as_bytes();
    writeln!(&mut out, "data {}", bytes.len()).unwrap();
    out.extend_from_slice(bytes);
    out.push(b'\n');
    for (i, (path, _)) in entries.iter().enumerate() {
        writeln!(&mut out, "M 100644 :{} {path}", base_mark + i as u64).unwrap();
    }
    writeln!(&mut out, "done").unwrap();
    out
}

fn find_cache_bare(cache_dir: &Path) -> Option<PathBuf> {
    walkdir::WalkDir::new(cache_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .find(|e| e.file_type().is_dir() && e.path().extension().is_some_and(|x| x == "git"))
        .map(|e| e.path().to_path_buf())
}

/// Build a SYNCED file:// mirror fixture (PRECHECK A passes; `git push`
/// would succeed if attempted). Returns
/// `(working_tree_dir, bare_mirror_dir, mirror_url)`.
///
/// This is the "passing" sibling of `make_failing_mirror_fixture` —
/// we use it for SoT-fail tests where the helper MUST NOT reach the
/// mirror push step. If the helper bypasses the SoT-fail branch, the
/// mirror push would land successfully (because the hook is the default
/// no-op), and the test's "mirror baseline preserved" assertion would
/// catch the regression.
fn make_synced_mirror_fixture() -> (tempfile::TempDir, tempfile::TempDir, String) {
    let mirror = tempfile::tempdir().expect("mirror tempdir");
    let wtree = tempfile::tempdir().expect("wtree tempdir");
    let scratch = tempfile::tempdir().expect("scratch tempdir");

    run_git_in(mirror.path(), &["init", "--bare", "."]);
    run_git_in(scratch.path(), &["init", "."]);
    run_git_in(scratch.path(), &["config", "user.email", "p83-02@example"]);
    run_git_in(scratch.path(), &["config", "user.name", "P83-02 Test"]);
    run_git_in(scratch.path(), &["checkout", "-b", "main"]);
    std::fs::write(scratch.path().join("seed.txt"), "seed").unwrap();
    run_git_in(scratch.path(), &["add", "seed.txt"]);
    run_git_in(scratch.path(), &["commit", "-m", "seed"]);
    let synced_sha = run_git_in(scratch.path(), &["rev-parse", "HEAD"]);

    let mirror_url = format!("file://{}", mirror.path().display());
    run_git_in(scratch.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(scratch.path(), &["push", "mirror", "HEAD:refs/heads/main"]);

    run_git_in(wtree.path(), &["init", "."]);
    run_git_in(wtree.path(), &["config", "user.email", "p83-02@example"]);
    run_git_in(wtree.path(), &["config", "user.name", "P83-02 Test"]);
    run_git_in(wtree.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(wtree.path(), &["fetch", "mirror"]);
    run_git_in(
        wtree.path(),
        &["update-ref", "refs/remotes/mirror/main", &synced_sha],
    );
    run_git_in(
        wtree.path(),
        &["update-ref", "refs/heads/main", &synced_sha],
    );
    run_git_in(wtree.path(), &["symbolic-ref", "HEAD", "refs/heads/main"]);

    (wtree, mirror, mirror_url)
}

#[tokio::test(flavor = "multi_thread")]
async fn bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit() {
    let server = MockServer::start().await;
    let project = "demo";
    let issues = sample_issues(project, 3);

    // 1. Setup-phase mocks (priority 5): seed list + per-id GETs.
    seed_mock(&server, project, &issues).await;

    let cache_root = tempfile::tempdir().expect("cache_root");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync (warm cache cursor)");
    drop(cache);

    // 2. ASSERTION-PHASE mocks (priority=1):
    //    - list_changed_since returns [] (PRECHECK B Stable).
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues$")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .with_priority(1)
        .mount(&server)
        .await;

    // 3. PATCH /1 → 200 (id=1 succeeds server-side).
    //    Mock::expect(1) — exactly 1 PATCH for id=1.
    Mock::given(method("PATCH"))
        .and(path_regex(format!(r"^/projects/{project}/issues/1$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 1, "title": "issue 1 in demo", "status": "open",
            "assignee": null, "labels": [],
            "created_at": "2026-04-13T00:00:00Z",
            "updated_at": "2026-05-01T00:00:00Z",
            "version": 2, "body": "edited body for 1\n"
        })))
        .with_priority(1)
        .expect(1)
        .mount(&server)
        .await;

    // 4. PATCH /2 → 500 (mid-stream fail).
    //    Mock::expect(1) — exactly 1 PATCH for id=2 (loop bails at this error).
    Mock::given(method("PATCH"))
        .and(path_regex(format!(r"^/projects/{project}/issues/2$")))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "error": "internal_server_error"
        })))
        .with_priority(1)
        .expect(1)
        .mount(&server)
        .await;

    // 5. Build PASSING file:// mirror fixture.
    //    (If the helper incorrectly reaches the mirror-push step despite
    //    the SoT mid-stream fail, the mirror's main ref would advance
    //    past the seed SHA — the test's "mirror baseline preserved"
    //    assertion catches that regression.)
    let (wtree, mirror_bare, mirror_url) = make_synced_mirror_fixture();
    let mirror_baseline_sha = {
        let out = StdCommand::new("git")
            .arg("-C")
            .arg(mirror_bare.path())
            .args(["rev-parse", "main"])
            .output()
            .expect("rev-parse mirror baseline");
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    };

    let bus_url = format!(
        "reposix::{}/projects/{project}?mirror={}",
        server.uri(),
        mirror_url
    );

    // 6. Fast-export with id=1 AND id=2 bodies changed → 2 PATCHes.
    //    id=3 unchanged → plan() skips.
    let blob1 = render_issue_blob(1, 1, "edited body for 1\n");
    let blob2 = render_issue_blob(2, 1, "edited body for 2\n");
    let blob3 = render_issue_blob(3, 1, "body of issue 3");
    let entries: Vec<(&str, String)> =
        vec![("0001.md", blob1), ("0002.md", blob2), ("0003.md", blob3)];
    let stream = multi_file_export(&entries, "edit issues 1+2\n");
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "capabilities").unwrap();
        writeln!(&mut buf).unwrap();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&stream);
        buf
    };

    let cache_path = cache_root.path().to_path_buf();
    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", &bus_url])
        .write_stdin(stdin_data)
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("REPOSIX_CACHE_DIR", &cache_path)
        .timeout(std::time::Duration::from_secs(30))
        .output()
        .expect("run helper");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // ASSERTION 1: helper exited non-zero.
    assert!(
        !out.status.success(),
        "helper must exit non-zero on SoT mid-stream fail; \
         stdout={stdout}, stderr={stderr}"
    );

    // ASSERTION 2: stdout contains `error refs/heads/main some-actions-failed`.
    assert!(
        stdout.contains("error refs/heads/main some-actions-failed"),
        "expected `error refs/heads/main some-actions-failed`; \
         stdout={stdout}, stderr={stderr}"
    );

    // ASSERTION 3 (implicit): wiremock's Drop check enforces the
    //  Mock::expect(1) on each PATCH route.

    // 7. Audit assertions.
    let cache_bare =
        find_cache_bare(cache_root.path()).expect("cache bare dir must exist after push");
    let db_path = cache_bare.join("cache.db");

    // ASSERTION 4: helper_push_accepted == 0 (didn't reach success branch).
    let accepted = count_audit_cache_rows(&db_path, "helper_push_accepted");
    assert_eq!(
        accepted, 0,
        "expected 0 helper_push_accepted rows on SoT mid-stream fail, got {accepted}"
    );

    // ASSERTION 5: helper_push_partial_fail_mirror_lag == 0 (mirror never attempted).
    let partial = count_audit_cache_rows(&db_path, "helper_push_partial_fail_mirror_lag");
    assert_eq!(
        partial, 0,
        "expected 0 helper_push_partial_fail_mirror_lag rows on SoT-fail (mirror never attempted), got {partial}"
    );

    // ASSERTION 6: helper_push_started == 1 (bus_handler wrote the started
    //  row before reaching apply_writes; the mid-stream fail happens AFTER
    //  the started row but BEFORE the accepted row).
    let started = count_audit_cache_rows(&db_path, "helper_push_started");
    assert_eq!(
        started, 1,
        "expected 1 helper_push_started row, got {started}"
    );

    // ASSERTION 7: mirror baseline ref unchanged.
    let mirror_after = StdCommand::new("git")
        .arg("-C")
        .arg(mirror_bare.path())
        .args(["rev-parse", "main"])
        .output()
        .expect("rev-parse mirror after");
    let mirror_after_sha = String::from_utf8_lossy(&mirror_after.stdout)
        .trim()
        .to_owned();
    assert_eq!(
        mirror_after_sha, mirror_baseline_sha,
        "mirror's main ref must be unchanged when SoT fails (no mirror push attempted)"
    );

    // ASSERTION 8: refs/mirrors/<sot>-head and -synced-at unchanged.
    //  On SoT-fail the helper bypasses both writes (apply_writes returns
    //  before mirror_head update; bus_handler's mirror_sync_written /
    //  partial_fail audit-write blocks are inside the SotOk arm).
    let refs_out = StdCommand::new("git")
        .arg("-C")
        .arg(&cache_bare)
        .args(["for-each-ref", "refs/mirrors/"])
        .output()
        .expect("git for-each-ref");
    let refs_str = String::from_utf8_lossy(&refs_out.stdout);
    assert!(
        !refs_str.contains("refs/mirrors/sim-synced-at"),
        "refs/mirrors/sim-synced-at must be ABSENT on SoT-fail; got: {refs_str}"
    );

    // Suppress unused warnings on tempdir handles.
    let _ = (wtree, &mirror_bare);
}
