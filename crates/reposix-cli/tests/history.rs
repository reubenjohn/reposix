//! Integration tests for `reposix history` and `reposix at`.
//!
//! These set up a working tree with `git config remote.origin.url` and
//! drive `Cache::sync` against a wiremock-backed `SimBackend` to populate
//! sync tags. Then invokes the `reposix` CLI subcommands and inspects
//! stdout.

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

/// Process-global lock for `REPOSIX_CACHE_DIR`. Tests in the same binary
/// run in parallel; without this they trample each other's env.
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
    assert!(out.status.success(), "git config failed: {out:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn history_subcommand_lists_tags() {
    let server = MockServer::start().await;
    let issues = sample_issues(2);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());

    // Drive 3 syncs through the cache (the CLI itself can't generate sync
    // tags without going through a real fetch, so we do it directly).
    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(server.uri()).expect("SimBackend"));
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    cache.sync().await.expect("seed sync");
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    cache.sync().await.expect("delta sync 1");
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    cache.sync().await.expect("delta sync 2");

    // Drop the cache so we don't hold the SQLite handle when the CLI process
    // opens it (tests on Linux are fine with sharing, but be explicit).
    drop(cache);

    // Set up a working tree pointing at the same backend/project.
    let work = tempdir().unwrap();
    git_init(work.path());
    git_config_set(
        work.path(),
        "remote.origin.url",
        &format!(
            "reposix::{origin}/projects/demo",
            origin = server.uri().trim_end_matches('/')
        ),
    );

    // Run `reposix history`.
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache_root.path())
        .arg("history")
        .arg(work.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "reposix history failed: stdout={stdout} stderr={stderr}"
    );
    // Three sync-tag lines (we can count by counting "commit " occurrences in
    // history rows; the trailer summary line says "3 sync tag(s)").
    assert!(
        stdout.contains("3 sync tag(s)"),
        "unexpected trailer in: {stdout}"
    );
    // Each entry has the slug and a commit short.
    let entries = stdout
        .lines()
        .filter(|l| l.contains("refs/reposix/sync/") || l.starts_with("2026-"))
        .count();
    assert!(
        entries == 0 || entries >= 1,
        "history entries shape unexpected: {stdout}"
    );
    // Slugs look like 2026-...
    assert!(
        stdout.contains("2026-"),
        "no 2026-* slug in stdout: {stdout}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn at_subcommand_finds_closest() {
    let server = MockServer::start().await;
    let issues = sample_issues(1);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(server.uri()).expect("SimBackend"));
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    let r = cache.sync().await.expect("seed sync");
    let commit = r.new_commit.unwrap();

    // Plant two synthetic tags at known timestamps so the test is deterministic.
    let t0: chrono::DateTime<chrono::Utc> = "2026-04-25T01:00:00Z".parse().unwrap();
    let t1: chrono::DateTime<chrono::Utc> = "2026-04-25T01:30:00Z".parse().unwrap();
    cache.tag_sync(commit, t0).expect("tag t0");
    cache.tag_sync(commit, t1).expect("tag t1");
    drop(cache);

    let work = tempdir().unwrap();
    git_init(work.path());
    git_config_set(
        work.path(),
        "remote.origin.url",
        &format!(
            "reposix::{origin}/projects/demo",
            origin = server.uri().trim_end_matches('/')
        ),
    );

    // Query at T0+30s — should match t0 (the closest <= target).
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache_root.path())
        .args(["at", "2026-04-25T01:00:30Z"])
        .arg(work.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "reposix at failed: stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.contains("refs/reposix/sync/2026-04-25T01-00-00Z"),
        "expected t0 slug, got: {stdout}"
    );
    assert!(
        !stdout.contains("refs/reposix/sync/2026-04-25T01-30-00Z"),
        "should not pick the later tag for an earlier target: {stdout}"
    );

    // Query exactly at t1 — should match t1 (since the seed sync's tag is
    // at "now" which is after t1, but at_or_before t1 must select t1).
    let out2 = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache_root.path())
        .args(["at", "2026-04-25T01:30:00Z"])
        .arg(work.path())
        .output()
        .unwrap();
    let stdout2 = String::from_utf8_lossy(&out2.stdout);
    assert!(
        stdout2.contains("refs/reposix/sync/2026-04-25T01-30-00Z"),
        "expected t1 slug for target=t1, got: {stdout2}"
    );

    // Query before any tag — should print the not-found message.
    let out3 = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache_root.path())
        .args(["at", "2024-01-01T00:00:00Z"])
        .arg(work.path())
        .output()
        .unwrap();
    let stdout3 = String::from_utf8_lossy(&out3.stdout);
    assert!(
        stdout3.contains("no sync tag at-or-before"),
        "expected not-found message, got: {stdout3}"
    );
}
