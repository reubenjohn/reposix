//! Bus write fan-out fault-injection (c) integration test (DVCS-BUS-WRITE-06).
//!
//! RESEARCH.md § "Test (c)": confluence 409 after PRECHECK B passed.
//! `list_changed_since` returns `[]` (PRECHECK B Stable) but PATCH
//! returns 409 with version-mismatch body — the SoT changed between
//! the precheck and the write (race that PRECHECK B can't catch on
//! its own; D-09 / Pitfall 3 documented behavior).
//!
//! Helper must:
//!
//! - exit non-zero with `error refs/heads/main some-actions-failed`.
//! - NO mirror push attempted.
//! - stderr names the failing record id.
//! - `helper_push_accepted` count == 0.
//! - `helper_push_partial_fail_mirror_lag` count == 0.
//! - mirror baseline preserved (mirror's main ref unchanged).
//! - wiremock saw exactly 1 PATCH (the one that 409'd) AND exactly 1
//!   `list_changed_since` (PRECHECK B Stable).
//!
//! D-02 RATIFIED: test does NOT assert per-failure REST audit row
//! (deferred to v0.14.0 GOOD-TO-HAVE).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::too_many_lines)]
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
async fn bus_write_post_precheck_conflict_409_no_mirror_push() {
    let server = MockServer::start().await;
    let project = "demo";
    let issues = sample_issues(project, 3);

    seed_mock(&server, project, &issues).await;

    let cache_root = tempfile::tempdir().expect("cache_root");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync (warm cache cursor)");
    drop(cache);

    // PRECHECK B: list_changed_since returns [] → Stable.
    //  Mock::expect(2) — bus_handler's PRECHECK B fires once; the
    //  inner L1 precheck inside write_loop::apply_writes
    //  (precheck_export_against_changed_set) fires a second time on
    //  the same `?since=` route because the helper consults the
    //  changed-set TWICE (coarse drift then fine per-record). Both
    //  return [] → both Stable.
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues$")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .with_priority(1)
        .expect(2)
        .mount(&server)
        .await;

    // PATCH /1 → 409 with version-mismatch JSON body (donor shape from
    //  push_conflict.rs::stale_base_push_emits_fetch_first_and_writes_no_rest
    //  + sim's 409 body shape from backend/sim.rs:566).
    //  Mock::expect(1) — exactly 1 PATCH for id=1 (loop bails on first error).
    Mock::given(method("PATCH"))
        .and(path_regex(format!(r"^/projects/{project}/issues/1$")))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "version_mismatch",
            "current": 2,
            "requested": 1
        })))
        .with_priority(1)
        .expect(1)
        .mount(&server)
        .await;

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

    // Update id=1 only (single PATCH expected); ids 2+3 unchanged.
    let blob1 = render_issue_blob(1, 1, "edited body for 1\n");
    let blob2 = render_issue_blob(2, 1, "body of issue 2");
    let blob3 = render_issue_blob(3, 1, "body of issue 3");
    let entries: Vec<(&str, String)> =
        vec![("0001.md", blob1), ("0002.md", blob2), ("0003.md", blob3)];
    let stream = multi_file_export(&entries, "edit issue 1\n");
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
        "helper must exit non-zero on post-precheck 409; \
         stdout={stdout}, stderr={stderr}"
    );

    // ASSERTION 2: stdout contains some-actions-failed (the apply_writes
    //  SotPartialFail path emits this — execute_action propagates the
    //  409 as Err which the loop counts as failure).
    assert!(
        stdout.contains("error refs/heads/main some-actions-failed"),
        "expected `error refs/heads/main some-actions-failed`; \
         stdout={stdout}, stderr={stderr}"
    );

    // ASSERTION 3: stderr names the failing record id. The execute_action
    //  Update arm wraps the error with `format!("patch issue {}", id.0)`
    //  via .with_context; apply_writes' loop emits via crate::diag.
    assert!(
        stderr.contains("issue 1") || stderr.contains("issues/1"),
        "expected stderr to name failing record id (issue 1); got: {stderr}"
    );

    // ASSERTION 4 (implicit): wiremock's Drop check enforces the
    //  Mock::expect(1) on PATCH /1 + list_changed_since.

    // ASSERTION 5: mirror baseline ref unchanged.
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
        "mirror's main ref must be unchanged when SoT 409s post-precheck \
         (no mirror push attempted)"
    );

    // ASSERTION 6: cache audit assertions.
    let cache_bare =
        find_cache_bare(cache_root.path()).expect("cache bare dir must exist after push");
    let db_path = cache_bare.join("cache.db");

    let accepted = count_audit_cache_rows(&db_path, "helper_push_accepted");
    assert_eq!(
        accepted, 0,
        "expected 0 helper_push_accepted rows on post-precheck 409, got {accepted}"
    );

    let partial = count_audit_cache_rows(&db_path, "helper_push_partial_fail_mirror_lag");
    assert_eq!(
        partial, 0,
        "expected 0 helper_push_partial_fail_mirror_lag rows on SoT-fail, got {partial}"
    );

    // ASSERTION 7: refs/mirrors/<sot>-synced-at unchanged.
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
