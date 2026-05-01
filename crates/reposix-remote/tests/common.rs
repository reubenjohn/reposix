//! Shared test harness: spin up a wiremock server that satisfies the
//! sim's `GET /projects/<p>/issues` + `GET /projects/<p>/issues/<id>`
//! routes so `reposix_core::backend::sim::SimBackend` can be pointed at
//! it. Used by every integration test in this crate.

#![allow(dead_code)]

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use reposix_core::backend::sim::SimBackend;
use reposix_core::BackendConnector;
use reposix_core::{Record, RecordId, RecordStatus};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Process-global lock serializing `REPOSIX_CACHE_DIR` mutation. Tests
/// within the same binary run in parallel by default; without this,
/// two tests racing on `set_var` drop each other's cache-dir settings
/// and the Cache picks up a path that doesn't exist. Guard survives
/// for the test's lifetime and restores the previous value on drop.
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Test-scoped RAII guard for the `REPOSIX_CACHE_DIR` env var.
/// Tests that do any `Cache::open` MUST hold one of these for the
/// duration of the open / build_from / read_blob calls.
pub struct CacheDirGuard<'a> {
    _guard: MutexGuard<'a, ()>,
    prev: Option<String>,
}

impl CacheDirGuard<'_> {
    pub fn new(path: &std::path::Path) -> Self {
        let guard = env_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prev = std::env::var(reposix_cache::CACHE_DIR_ENV).ok();
        std::env::set_var(reposix_cache::CACHE_DIR_ENV, path);
        Self {
            _guard: guard,
            prev,
        }
    }
}

impl Drop for CacheDirGuard<'_> {
    fn drop(&mut self) {
        match &self.prev {
            Some(v) => std::env::set_var(reposix_cache::CACHE_DIR_ENV, v),
            None => std::env::remove_var(reposix_cache::CACHE_DIR_ENV),
        }
    }
}

/// Build `n` deterministic test issues with ids 1..=n.
#[must_use]
pub fn sample_issues(project: &str, n: usize) -> Vec<Record> {
    use chrono::TimeZone;
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    (1..=n)
        .map(|i| Record {
            id: RecordId(i as u64),
            title: format!("issue {i} in {project}"),
            status: RecordStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: t,
            updated_at: t,
            version: 1,
            body: format!("body of issue {i}"),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        })
        .collect()
}

/// Seed a wiremock server so that
/// `GET /projects/<project>/issues` returns the provided list and
/// `GET /projects/<project>/issues/<id>` returns the matching single
/// issue (or 404).
pub async fn seed_mock(server: &MockServer, project: &str, issues: &[Record]) {
    // List route.
    let list_body: Vec<serde_json::Value> = issues.iter().map(issue_to_json).collect();
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .respond_with(ResponseTemplate::new(200).set_body_json(list_body))
        .mount(server)
        .await;

    // Per-issue routes.
    for issue in issues {
        let id = issue.id.0;
        Mock::given(method("GET"))
            .and(path(format!("/projects/{project}/issues/{id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue_to_json(issue)))
            .mount(server)
            .await;
    }

    // Catch-all 404 for unknown ids (so tests fail fast if the cache
    // requests an OID it hasn't populated).
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues/\d+$")))
        .respond_with(ResponseTemplate::new(404))
        .mount(server)
        .await;
}

fn issue_to_json(issue: &Record) -> serde_json::Value {
    serde_json::json!({
        "id": issue.id.0,
        "title": issue.title,
        "status": issue.status.as_str(),
        "assignee": issue.assignee,
        "labels": issue.labels,
        "created_at": issue.created_at.to_rfc3339(),
        "updated_at": issue.updated_at.to_rfc3339(),
        "version": issue.version,
        "body": issue.body,
    })
}

/// Build an Arc-wrapped `SimBackend` pointed at `server`.
#[must_use]
pub fn sim_backend(server: &MockServer) -> Arc<dyn BackendConnector> {
    Arc::new(SimBackend::new(server.uri()).expect("SimBackend::new"))
}

/// Build a `file://` bare mirror whose `update` hook always fails
/// with exit 1. Used by mirror-fail fault tests (P83-02 T02 ships
/// the consumer test). Returns:
///
/// 1. the tempdir handle (KEEP IN SCOPE for the test's lifetime —
///    drop removes the dir);
/// 2. the `file://` URL pointing at the bare mirror's path.
///
/// Gated `#[cfg(unix)]` per D-04 RATIFIED — the `update`-hook +
/// `chmod 0o755` pattern is POSIX-specific. Reposix CI is Linux-only
/// at this phase; macOS dev workflow honors the same hook semantics.
/// Windows hosts that try to run this fail at compile time, which
/// is the intended behavior (the bus write fan-out's mirror push is
/// a `git` shell-out — same Unix-shaped contract).
#[cfg(unix)]
#[must_use]
pub fn make_failing_mirror_fixture() -> (tempfile::TempDir, String) {
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    let mirror = tempfile::tempdir().expect("mirror tempdir");
    let status = Command::new("git")
        .args(["init", "--bare", "."])
        .current_dir(mirror.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()
        .expect("spawn git init --bare");
    assert!(status.success(), "git init --bare failed in mirror fixture");

    // Override `core.hooksPath` in the bare repo's *local* config to
    // point at the per-repo `hooks/` directory. Without this override,
    // a user-global `core.hooksPath = /home/.../.git-hooks` (set by
    // some agent / dev environments) wins and the per-repo update hook
    // we install below NEVER fires — making the failing-mirror fixture
    // silently a passing-mirror fixture. P83-02 T02 surfaced this as a
    // real bug (not a planning oversight). Fix is local-config override
    // because GIT_CONFIG_NOSYSTEM only disables `/etc/gitconfig`, not
    // the user's `~/.gitconfig`.
    let hooks_dir = mirror.path().join("hooks");
    let status = Command::new("git")
        .args(["config", "core.hooksPath"])
        .arg(&hooks_dir)
        .current_dir(mirror.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .status()
        .expect("spawn git config core.hooksPath");
    assert!(
        status.success(),
        "git config core.hooksPath failed in mirror fixture"
    );

    let hook = hooks_dir.join("update");
    std::fs::write(
        &hook,
        "#!/bin/sh\necho \"intentional fail for fault test\" >&2\nexit 1\n",
    )
    .expect("write update hook");
    let mut perms = std::fs::metadata(&hook)
        .expect("stat update hook")
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&hook, perms).expect("chmod update hook");

    let url = format!("file://{}", mirror.path().display());
    (mirror, url)
}

/// Open the cache.db at `cache_db_path` and count rows matching `op`.
/// Used by audit-completeness assertions in P83-01 + P83-02 tests.
///
/// `cache_db_path` is the full path to the SQLite file — typically
/// `<cache-bare-repo>/cache.db` (locate via the `find_cache_bare`
/// helper in tests/mirror_refs.rs OR by walking the cache root for
/// a `.git` directory).
#[must_use]
pub fn count_audit_cache_rows(cache_db_path: &std::path::Path, op: &str) -> i64 {
    let conn = rusqlite::Connection::open(cache_db_path).expect("open cache.db");
    conn.query_row(
        "SELECT COUNT(*) FROM audit_events_cache WHERE op = ?1",
        rusqlite::params![op],
        |r| r.get::<_, i64>(0),
    )
    .expect("count audit rows")
}
