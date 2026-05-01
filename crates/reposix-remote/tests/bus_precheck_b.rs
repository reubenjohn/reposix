//! PRECHECK B — `SoT` drift via list_changed_since (DVCS-BUS-PRECHECK-02).
//!
//! Fixture strategy: wiremock SoT (P81 donor pattern from
//! `tests/perf_l1.rs`) + synced file:// mirror (P82 donor pattern
//! from `tests/bus_precheck_a.rs::make_drifting_mirror_fixture`).
//! Drifted: wiremock returns non-empty `?since=` response → helper
//! emits `error refs/heads/main fetch first`. Stable: wiremock
//! returns `[]` on `?since=` → helper passes PRECHECK B and emits
//! the D-02 deferred-shipped error.

#![allow(clippy::missing_panics_doc)]

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use assert_cmd::Command as AssertCommand;
use reposix_cache::Cache;
use reposix_core::BackendConnector;
use serde_json::json;
use wiremock::matchers::{method, path_regex};
use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

mod common;
use common::{sample_issues, seed_mock, sim_backend, CacheDirGuard};

/// Custom matcher (verbatim from `tests/perf_l1.rs`): matches
/// requests that DO have a `since` query param. wiremock 0.6's
/// `query_param(K, V)` is byte-exact; there is no `query_param_exists`
/// or wildcard-value form. A custom `Match` impl is the canonical idiom.
struct HasSinceQueryParam;
impl Match for HasSinceQueryParam {
    fn matches(&self, req: &Request) -> bool {
        req.url.query_pairs().any(|(k, _)| k == "since")
    }
}

/// Spawn `git` against a directory; assert success. Mirrors the helper
/// from `bus_precheck_a.rs` verbatim.
fn run_git_in(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
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

/// Build a SYNCED file:// mirror fixture: bare mirror with one commit;
/// working tree with `refs/remotes/mirror/main` pointing at that same
/// commit (PRECHECK A passes). Returns
/// `(working_tree_dir, mirror_bare_dir, mirror_url)`.
fn make_synced_mirror_fixture() -> (tempfile::TempDir, tempfile::TempDir, String) {
    let mirror = tempfile::tempdir().expect("mirror tempdir");
    let wtree = tempfile::tempdir().expect("wtree tempdir");
    let scratch = tempfile::tempdir().expect("scratch tempdir");

    run_git_in(mirror.path(), &["init", "--bare", "."]);
    run_git_in(scratch.path(), &["init", "."]);
    run_git_in(scratch.path(), &["config", "user.email", "p82@example"]);
    run_git_in(scratch.path(), &["config", "user.name", "P82 Test"]);
    run_git_in(scratch.path(), &["checkout", "-b", "main"]);
    std::fs::write(scratch.path().join("seed.txt"), "seed").unwrap();
    run_git_in(scratch.path(), &["add", "seed.txt"]);
    run_git_in(scratch.path(), &["commit", "-m", "seed"]);
    let synced_sha = run_git_in(scratch.path(), &["rev-parse", "HEAD"]);

    let mirror_url = format!("file://{}", mirror.path().display());
    run_git_in(scratch.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(scratch.path(), &["push", "mirror", "HEAD:refs/heads/main"]);

    // Working tree: init + add the mirror remote + fetch (populate
    // object DB) + write refs/remotes/mirror/main pointing at the
    // SAME commit (synced). The intermediate `git fetch mirror` is
    // required because `update-ref` refuses to point at a SHA the
    // local object DB doesn't have — without the fetch, only the
    // bare mirror has the seed object.
    run_git_in(wtree.path(), &["init", "."]);
    run_git_in(wtree.path(), &["config", "user.email", "p82@example"]);
    run_git_in(wtree.path(), &["config", "user.name", "P82 Test"]);
    run_git_in(wtree.path(), &["remote", "add", "mirror", &mirror_url]);
    run_git_in(wtree.path(), &["fetch", "mirror"]);
    run_git_in(
        wtree.path(),
        &["update-ref", "refs/remotes/mirror/main", &synced_sha],
    );

    (wtree, mirror, mirror_url)
}

#[tokio::test(flavor = "multi_thread")]
async fn bus_precheck_b_emits_fetch_first_on_sot_drift() {
    let server = MockServer::start().await;
    let project = "demo";
    let issues = sample_issues(project, 3);

    // Setup-phase mocks (default priority 5): seed list + per-id GETs
    // so warm_cache populates the last_fetched_at cursor.
    seed_mock(&server, project, &issues).await;

    // Per-test cache dir.
    let cache_root = tempfile::tempdir().expect("cache_root");
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync (warm cache cursor)");
    drop(cache);

    // ASSERTION-PHASE mock (priority=1, beats setup): wiremock returns
    // a non-empty `?since=` response — PRECHECK B sees Drifted.
    // Body must be a full Record array (sim's decode_issues consumes it).
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues$")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 5, "title": "drift", "status": "open",
             "assignee": null, "labels": [],
             "created_at": "2026-04-13T00:00:00Z",
             "updated_at": "2026-05-01T00:00:00Z",
             "version": 2, "body": "drift body"}
        ])))
        .with_priority(1)
        .mount(&server)
        .await;

    // Per-id GET for the drifted record (defensive — currently unused
    // since PRECHECK B bails on count alone, but a future hardening
    // could call get_record for hint composition).
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues/5$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 5, "title": "drift", "status": "open",
            "assignee": null, "labels": [],
            "created_at": "2026-04-13T00:00:00Z",
            "updated_at": "2026-05-01T00:00:00Z",
            "version": 2, "body": "drift body"
        })))
        .with_priority(1)
        .mount(&server)
        .await;

    // Build the synced file:// mirror fixture (PRECHECK A passes).
    let (wtree, _mirror_bare, mirror_url) = make_synced_mirror_fixture();

    // Bus URL: wiremock SoT + file:// mirror.
    let bus_url = format!(
        "reposix::{}/projects/{project}?mirror={}",
        server.uri(),
        mirror_url
    );

    // Drive the helper. write_stdin uses the same shape as
    // `bus_precheck_a.rs`: capabilities + export verb. The helper
    // reaches PRECHECK B (PRECHECK A passes since mirror is synced),
    // sees Drifted, emits the fetch-first reject before reading the
    // export stream.
    let cache_path = cache_root.path().to_path_buf();
    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", &bus_url])
        .write_stdin("capabilities\n\nexport\n\n")
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("REPOSIX_CACHE_DIR", &cache_path)
        .output()
        .expect("run helper");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // Assertion 1: fetch-first protocol error on stdout.
    assert!(
        stdout.contains("error refs/heads/main fetch first"),
        "expected fetch-first protocol error on stdout; got stdout={stdout}, stderr={stderr}"
    );

    // Assertion 2: stderr names SoT drift + mentions `git pull --rebase`
    // + (when populated by P80) cites refs/mirrors/<sot>-synced-at.
    // For this test the synced-at ref is NOT populated (no prior P80
    // push happened), so we only assert the always-on hint substrings.
    assert!(
        stderr.contains("git pull --rebase"),
        "expected stderr to suggest `git pull --rebase`; got: {stderr}"
    );
    assert!(
        stderr.contains("PRECHECK B") || stderr.contains("change(s) since"),
        "expected stderr to name SoT drift / PRECHECK B; got: {stderr}"
    );

    // Assertion 3: helper exited non-zero (precheck reject).
    assert!(!out.status.success(), "expected helper to exit non-zero");
}

#[tokio::test(flavor = "multi_thread")]
async fn bus_precheck_b_passes_when_sot_stable() {
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

    // ASSERTION-PHASE mock (priority=1): wiremock returns EMPTY on
    // `?since=` — PRECHECK B sees Stable, helper proceeds to write
    // fan-out (P83-01 T04).
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues$")))
        .and(HasSinceQueryParam)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .with_priority(1)
        .mount(&server)
        .await;

    // PATCH backstop: this test sends an empty fast-export stream
    // (`capabilities\n\nexport\n\n`), so plan() may compute deletes
    // for prior records. With sim's GET returning the seeded issues,
    // execute_action's DELETE leg runs against simulator's DELETE
    // route — accept any number of calls (we don't assert PATCH/DELETE
    // counts here, that's bus_write_happy.rs's job in P83-01 T05).
    Mock::given(method("PATCH"))
        .and(path_regex(format!(r"^/projects/{project}/issues/\d+$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 1, "version": 2})))
        .with_priority(2)
        .mount(&server)
        .await;

    let (wtree, _mirror_bare, mirror_url) = make_synced_mirror_fixture();
    let bus_url = format!(
        "reposix::{}/projects/{project}?mirror={}",
        server.uri(),
        mirror_url
    );

    let cache_path = cache_root.path().to_path_buf();
    let out = AssertCommand::cargo_bin("git-remote-reposix")
        .expect("cargo bin")
        .args(["origin", &bus_url])
        .write_stdin("capabilities\n\nexport\n\n")
        .current_dir(wtree.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("REPOSIX_CACHE_DIR", &cache_path)
        .output()
        .expect("run helper");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // PRIMARY ASSERTION (post-P83-01): NO fetch-first signal.
    // PRECHECK B passed → helper proceeded into the write fan-out
    // path. The deferred-shipped stub from P82 was removed in
    // P83-01 T04; the test's intent is now "did PRECHECK B reject?"
    // and the answer must be NO.
    assert!(
        !stdout.contains("fetch first"),
        "PRECHECK B incorrectly tripped on stable SoT; stdout={stdout}, stderr={stderr}"
    );

    // Regression assertion: the P82 deferred-shipped stub is GONE.
    // If a regression re-introduces it, P83's write fan-out is being
    // bypassed and this test should fail RED.
    assert!(
        !stdout.contains("bus-write-not-yet-shipped"),
        "P82 deferred-shipped stub re-appeared after P83-01 — write fan-out bypassed; \
         stdout={stdout}, stderr={stderr}"
    );
}
