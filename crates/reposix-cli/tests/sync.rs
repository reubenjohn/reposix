//! Smoke test for `reposix sync --reconcile` (DVCS-PERF-L1-02).
//!
//! Asserts the command exists, accepts --reconcile, and advances the
//! cache's `meta.last_fetched_at` cursor against a wiremock-backed
//! sim. Mirrors the wiremock setup in `tests/history.rs` (no shared
//! `mod common;` across test files in cargo's test harness).

#![allow(clippy::missing_panics_doc)]

use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use assert_cmd::Command as AssertCmd;
use chrono::TimeZone;
use reposix_cache::Cache;
use reposix_core::backend::sim::SimBackend;
use reposix_core::{BackendConnector, Record, RecordId, RecordStatus};
use tempfile::tempdir;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Process-global lock for `REPOSIX_CACHE_DIR` mutation. Mirrors the
/// pattern in `tests/history.rs` so independent tests don't race.
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct CacheDirGuard<'a> {
    _guard: MutexGuard<'a, ()>,
    prev: Option<String>,
}

impl<'a> CacheDirGuard<'a> {
    fn new(path: &Path) -> Self {
        let guard = env_lock().lock().unwrap_or_else(|p| p.into_inner());
        let prev = std::env::var("REPOSIX_CACHE_DIR").ok();
        std::env::set_var("REPOSIX_CACHE_DIR", path);
        Self {
            _guard: guard,
            prev,
        }
    }
}

impl Drop for CacheDirGuard<'_> {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => std::env::set_var("REPOSIX_CACHE_DIR", v),
            None => std::env::remove_var("REPOSIX_CACHE_DIR"),
        }
    }
}

fn sample_issues(n: usize) -> Vec<Record> {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    (1..=n)
        .map(|i| Record {
            id: RecordId(i as u64),
            title: format!("issue {i}"),
            status: RecordStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: t,
            updated_at: t,
            version: 1,
            body: format!("body {i}"),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        })
        .collect()
}

async fn seed_mock(server: &MockServer, project: &str, issues: &[Record]) {
    let list_body: Vec<serde_json::Value> = issues
        .iter()
        .map(|i| {
            serde_json::json!({
                "id": i.id.0,
                "title": i.title,
                "status": i.status.as_str(),
                "assignee": i.assignee,
                "labels": i.labels,
                "created_at": i.created_at.to_rfc3339(),
                "updated_at": i.updated_at.to_rfc3339(),
                "version": i.version,
                "body": i.body,
            })
        })
        .collect();
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .respond_with(ResponseTemplate::new(200).set_body_json(list_body))
        .mount(server)
        .await;
    for issue in issues {
        let id = issue.id.0;
        Mock::given(method("GET"))
            .and(path(format!("/projects/{project}/issues/{id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": issue.id.0,
                "title": issue.title,
                "status": issue.status.as_str(),
                "assignee": issue.assignee,
                "labels": issue.labels,
                "created_at": issue.created_at.to_rfc3339(),
                "updated_at": issue.updated_at.to_rfc3339(),
                "version": issue.version,
                "body": issue.body,
            })))
            .mount(server)
            .await;
    }
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues/\d+$")))
        .respond_with(ResponseTemplate::new(404))
        .mount(server)
        .await;
}

fn git_init(path: &Path) {
    let out = Command::new("git")
        .args(["init", "-q"])
        .arg(path)
        .output()
        .unwrap();
    assert!(out.status.success(), "git init failed: {out:?}");
}

fn git_config_set(path: &Path, key: &str, val: &str) {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["config", key, val])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git config {key} {val} failed: {out:?}"
    );
}

#[tokio::test]
async fn sync_reconcile_advances_cursor() {
    let server = MockServer::start().await;
    let project = "demo";
    let issues = sample_issues(3);
    seed_mock(&server, project, &issues).await;

    // Isolate the cache dir for this test (per-test tempdir keeps the
    // CacheDirGuard scoped tightly so concurrent tests don't trample
    // each other's `REPOSIX_CACHE_DIR`).
    let cache_root = tempdir().expect("tempdir");
    let _env = CacheDirGuard::new(cache_root.path());

    // First seed the cache via in-process Cache::sync so we have a
    // baseline `last_fetched_at = T1`. The CLI subcommand under test
    // is then expected to advance the cursor.
    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(server.uri()).expect("SimBackend::new"));
    let cache = Cache::open(backend, "sim", project).expect("Cache::open");
    cache.sync().await.expect("seed sync");
    let t1 = cache
        .read_last_fetched_at()
        .expect("read cursor")
        .expect("cursor present after seed sync");

    // Sleep so RFC3339 second-granularity ticks forward, otherwise the
    // bumped cursor compares equal to the seed cursor.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    // Working tree with a remote.origin.url that resolves to the same
    // (sim, demo) cache. The handler reads remote.origin.url, derives
    // backend=sim + project=demo, and constructs a SimBackend pointed
    // at REPOSIX_SIM_ORIGIN.
    let work = tempdir().expect("work tempdir");
    git_init(work.path());
    git_config_set(
        work.path(),
        "remote.origin.url",
        &format!("reposix::{}/projects/{project}", server.uri()),
    );

    // Drive the CLI subcommand. The subprocess inherits
    // REPOSIX_CACHE_DIR from this process (set above by CacheDirGuard)
    // and reads REPOSIX_SIM_ORIGIN to point its SimBackend at the
    // wiremock server.
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache_root.path())
        .env("REPOSIX_SIM_ORIGIN", server.uri())
        .args(["sync", "--reconcile"])
        .arg(work.path())
        .timeout(std::time::Duration::from_secs(15))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "sync --reconcile failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    let t2 = cache
        .read_last_fetched_at()
        .expect("read cursor after sync")
        .expect("cursor present after --reconcile");

    assert!(
        t2 > t1,
        "expected cursor to advance after --reconcile; got t1 = {t1}, t2 = {t2}"
    );
}

#[test]
fn sync_help_renders() {
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .args(["sync", "--reconcile", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success(), "sync --help failed: {out:?}");
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("--reconcile"),
        "sync --help missing --reconcile: {s}"
    );
}

#[test]
fn sync_bare_form_prints_hint() {
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .args(["sync"])
        .output()
        .unwrap();
    assert!(out.status.success(), "sync (bare) should exit 0: {out:?}");
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("--reconcile"),
        "sync (bare) should print hint pointing at --reconcile: {s}"
    );
}
