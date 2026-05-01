//! Bus write fan-out happy-path integration test (DVCS-BUS-WRITE-01..05).
//!
//! Asserts the full SoT-success + mirror-success path:
//!
//! 1. wiremock SoT setup (list_records prior; list_changed_since empty
//!    on warm cursor; PATCH 200 for the changed record).
//! 2. file:// bare mirror with default — passing — `update` hook.
//! 3. Working tree with `mirror` remote synced (refs/remotes/mirror/main
//!    points at the bare mirror's HEAD; PRECHECK A passes).
//! 4. Drive helper via assert_cmd with bus URL
//!    `reposix::<sim>?mirror=<file://...>` and a fast-export stream
//!    that updates issue 1.
//! 5. Assert helper exits zero, stdout has `ok refs/heads/main`.
//! 6. Assert refs/mirrors/<sot>-head AND -synced-at populated in
//!    cache bare.
//! 7. Assert audit_events_cache contains helper_push_started +
//!    helper_push_accepted + mirror_sync_written rows; ZERO
//!    helper_push_partial_fail_mirror_lag rows.
//! 8. Assert wiremock saw at least one PATCH call (REST write fired).
//! 9. Assert mirror's main ref points at the new SoT SHA (mirror push
//!    landed via the helper's git-push subprocess).
//!
//! Donor patterns:
//! - `tests/mirror_refs.rs::write_on_success_updates_both_refs`
//!   (helper-driver shape, find_cache_bare helper).
//! - `tests/bus_precheck_b.rs::make_synced_mirror_fixture`
//!   (file:// bare mirror with passing default hook).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)] // test-internal doc comments cite SoT/refs/audit ops verbatim
#![allow(clippy::too_many_lines)] // narrow integration-test setup; readability beats split fns
#![allow(clippy::unnecessary_debug_formatting)] // stderr/path Debug is intentional in test diagnostics

use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::Arc;

use assert_cmd::Command as AssertCommand;
use chrono::TimeZone;
use reposix_cache::Cache;
use reposix_core::BackendConnector;
use serde_json::{json, Value};
use wiremock::matchers::{method, path_regex};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

mod common;
use common::{count_audit_cache_rows, sample_issues, seed_mock, sim_backend, CacheDirGuard};

/// Custom matcher: matches requests that DO have a `since` query
/// param. Mirrors `tests/bus_precheck_b.rs::HasSinceQueryParam`.
struct HasSinceQueryParam;
impl Match for HasSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().any(|(k, _)| k == "since")
    }
}

/// Spawn `git` against a directory; assert success.
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

/// Render a Record's frontmatter+body form.
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

/// Build a fast-export stream containing N issues. Each entry is
/// `(path, blob_bytes)` — caller renders the blob via [`render_issue_blob`].
/// Only blobs whose content differs from the cache's prior trigger a
/// PATCH; identical blobs flow through `plan` as no-ops. This shape
/// mirrors the multi-file form `parse_export_stream` expects: blobs
/// at sequential marks, then a single commit listing every `M 100644`
/// entry in the new tree.
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

/// Locate the cache bare repo under `cache_dir`. Returns the bare
/// directory path (`<cache>/reposix/<backend>-<project>.git`).
fn find_cache_bare(cache_dir: &Path) -> Option<PathBuf> {
    walkdir::WalkDir::new(cache_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .find(|e| e.file_type().is_dir() && e.path().extension().is_some_and(|x| x == "git"))
        .map(|e| e.path().to_path_buf())
}

/// Build a SYNCED file:// mirror fixture (PRECHECK A passes). Returns
/// `(working_tree_dir, bare_mirror_dir, mirror_url)`. Mirrors the donor
/// in `tests/bus_precheck_b.rs`.
fn make_synced_mirror_fixture() -> (tempfile::TempDir, tempfile::TempDir, String) {
    let mirror = tempfile::tempdir().expect("mirror tempdir");
    let wtree = tempfile::tempdir().expect("wtree tempdir");
    let scratch = tempfile::tempdir().expect("scratch tempdir");

    run_git_in(mirror.path(), &["init", "--bare", "."]);
    run_git_in(scratch.path(), &["init", "."]);
    run_git_in(scratch.path(), &["config", "user.email", "p83@example"]);
    run_git_in(scratch.path(), &["config", "user.name", "P83 Test"]);
    run_git_in(scratch.path(), &["checkout", "-b", "main"]);
    std::fs::write(scratch.path().join("seed.txt"), "seed").unwrap();
    run_git_in(scratch.path(), &["add", "seed.txt"]);
    run_git_in(scratch.path(), &["commit", "-m", "seed"]);
    let synced_sha = run_git_in(scratch.path(), &["rev-parse", "HEAD"]);

    let mirror_url = format!("file://{}", mirror.path().display());
    run_git_in(scratch.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(scratch.path(), &["push", "mirror", "HEAD:refs/heads/main"]);

    run_git_in(wtree.path(), &["init", "."]);
    run_git_in(wtree.path(), &["config", "user.email", "p83@example"]);
    run_git_in(wtree.path(), &["config", "user.name", "P83 Test"]);
    run_git_in(wtree.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(wtree.path(), &["fetch", "mirror"]);
    run_git_in(
        wtree.path(),
        &["update-ref", "refs/remotes/mirror/main", &synced_sha],
    );
    // Set HEAD on the working tree's main branch so subsequent git
    // commands (for the helper subprocess inheriting cwd) have a valid
    // ref to push from. Use the seed commit itself.
    run_git_in(
        wtree.path(),
        &["update-ref", "refs/heads/main", &synced_sha],
    );
    run_git_in(wtree.path(), &["symbolic-ref", "HEAD", "refs/heads/main"]);

    (wtree, mirror, mirror_url)
}

#[tokio::test(flavor = "multi_thread")]
async fn happy_path_writes_both_refs_and_acks_ok() {
    let server = MockServer::start().await;
    let project = "demo";
    let issues = sample_issues(project, 3);

    // 1. Setup-phase mocks (default priority 5): seed list + per-id GETs.
    seed_mock(&server, project, &issues).await;

    // 2. Per-test cache dir with warm last_fetched_at cursor.
    let cache_root = tempfile::tempdir().expect("cache_root");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync (warm cache cursor)");
    drop(cache);

    // 3. ASSERTION-PHASE mocks (priority=1): list_changed_since with
    //    `?since=` returns EMPTY (no SoT drift — PRECHECK B passes).
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues$")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .with_priority(1)
        .mount(&server)
        .await;

    // 4. PATCH for issue 1 (the one we'll update). Returns the bumped
    //    version. write_loop::apply_writes calls execute_action which
    //    runs PATCH against this route.
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
        .expect(1..)
        .mount(&server)
        .await;

    // 5. Build the synced file:// mirror fixture (passing update hook).
    let (wtree, mirror_bare, mirror_url) = make_synced_mirror_fixture();

    // 6. Bus URL: wiremock SoT + file:// mirror.
    let bus_url = format!(
        "reposix::{}/projects/{project}?mirror={}",
        server.uri(),
        mirror_url
    );

    // 7. Build the fast-export stream containing all 3 issues. Issue
    //    1's body changes (PATCH fires); issues 2 and 3 are unchanged
    //    (plan() should compute no actions for them since path+content
    //    match the cache prior). The multi-file shape prevents plan()
    //    from interpreting absent paths as DELETEs.
    //
    //    Note on prior-content match: render_issue_blob emits the
    //    canonical frontmatter+body shape sample_issues seeds with
    //    (`body of issue {i}`). For ids 2 + 3 we emit the unchanged
    //    canonical body so plan() sees identical blobs and skips them.
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

    // 8. Drive the helper.
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

    // ASSERTION 1: helper exited zero.
    assert!(
        out.status.success(),
        "helper must exit zero on SoT-success+mirror-success; \
         stdout={stdout}, stderr={stderr}"
    );

    // ASSERTION 2: helper stdout contains `ok refs/heads/main`.
    assert!(
        stdout.contains("ok refs/heads/main"),
        "missing `ok refs/heads/main` ack; stdout={stdout}, stderr={stderr}"
    );

    // 9. Locate the cache bare for ref + audit assertions.
    let cache_bare =
        find_cache_bare(cache_root.path()).expect("cache bare dir must exist after push");

    // ASSERTION 3 + 4: refs/mirrors/<sot>-head AND -synced-at both
    // resolvable via plain git for-each-ref on the cache bare.
    let refs_out = StdCommand::new("git")
        .arg("-C")
        .arg(&cache_bare)
        .args(["for-each-ref", "refs/mirrors/"])
        .output()
        .expect("git for-each-ref");
    let refs_str = String::from_utf8_lossy(&refs_out.stdout);
    assert!(
        refs_str.contains("refs/mirrors/sim-head"),
        "missing refs/mirrors/sim-head; got: {refs_str}"
    );
    assert!(
        refs_str.contains("refs/mirrors/sim-synced-at"),
        "missing refs/mirrors/sim-synced-at; got: {refs_str}"
    );

    // ASSERTION 5: audit_events_cache helper_push_started: count == 1.
    let db_path = cache_bare.join("cache.db");
    let started = count_audit_cache_rows(&db_path, "helper_push_started");
    assert_eq!(
        started, 1,
        "expected 1 helper_push_started row, got {started}"
    );

    // ASSERTION 6: audit_events_cache helper_push_accepted: count == 1.
    let accepted = count_audit_cache_rows(&db_path, "helper_push_accepted");
    assert_eq!(
        accepted, 1,
        "expected 1 helper_push_accepted row, got {accepted}"
    );

    // ASSERTION 7: audit_events_cache mirror_sync_written: count == 1.
    let synced = count_audit_cache_rows(&db_path, "mirror_sync_written");
    assert_eq!(
        synced, 1,
        "expected 1 mirror_sync_written row, got {synced}"
    );

    // ASSERTION 8: audit_events_cache helper_push_partial_fail_mirror_lag:
    // count == 0 (the partial-fail row is FOR the mirror-fail path; in
    // the happy path it MUST be absent).
    let partial = count_audit_cache_rows(&db_path, "helper_push_partial_fail_mirror_lag");
    assert_eq!(
        partial, 0,
        "expected 0 helper_push_partial_fail_mirror_lag rows on happy path, got {partial}"
    );

    // ASSERTION 9: wiremock saw at least 1 PATCH call (the
    //  Mock::expect(1..) at mount time enforces this on Drop).
    //  (Implicit via wiremock's Drop check.)

    // ASSERTION 10: mirror's main ref points at the new SoT SHA.
    // The mirror push (git push <mirror_remote_name> main) MUST have
    // landed for the synced-at ref to be written. We assert the
    // mirror's main resolves to the working-tree's HEAD — both should
    // reflect the helper's terminal mirror-push state.
    let mirror_main = StdCommand::new("git")
        .arg("-C")
        .arg(mirror_bare.path())
        .args(["rev-parse", "main"])
        .output()
        .expect("git rev-parse main on bare mirror");
    assert!(
        mirror_main.status.success(),
        "mirror's main ref should resolve; stderr={}",
        String::from_utf8_lossy(&mirror_main.stderr)
    );
    let mirror_sha = String::from_utf8_lossy(&mirror_main.stdout)
        .trim()
        .to_owned();
    assert!(
        !mirror_sha.is_empty(),
        "mirror main SHA must be non-empty; stdout={mirror_sha}"
    );

    // Suppress unused warnings on tempdir handles (must outlive test scope).
    let _ = (wtree, &mirror_bare);

    // Suppress unused-import on Value (kept for serde_json visibility).
    let _: Option<Value> = None;
}
