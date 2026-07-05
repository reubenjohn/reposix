//! Post-sync cache-coherence invariant (ADR-010 / RBF-LR-01, closes D-P92-03).
//!
//! The invariant under test: **every blob OID the HEAD tree references must be
//! resolvable by `Cache::read_blob`.** `Cache::sync` builds the git tree from
//! `list_records` (the full current backend state) but historically populated
//! `oid_map` only for the `list_changed_since` delta. When those two sources
//! disagreed — e.g. a write landing in the same wall-clock second as the
//! cache cursor, dropped by a seconds-resolution `updated_at` filter — the
//! tree referenced an OID with no `oid_map` row: a dangling entry. A puller's
//! partial-clone lazy fetch of that OID reached `read_blob` → `UnknownOid` →
//! the helper left the `want` for `git upload-pack`, which rejected it:
//! `fatal: git upload-pack: not our ref <oid>`.
//!
//! `RBF-LR-02`'s gate runs `cargo test -p reposix-cache --test cache_coherence`.
//! Both tests walk the HEAD tree after a sync and assert `read_blob` resolves
//! every referenced blob OID. The seed-sync test is a positive control
//! (`build_from` was always coherent); the same-second delta-sync test is the
//! D-P92-03 regression the coherence fix closes. Full executed repro:
//! `.planning/phases/93-cache-coherence/93-DP2-REPRO-NOTES.md`.

#![allow(clippy::missing_panics_doc)]

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

use chrono::{Timelike, Utc};
use reposix_cache::Cache;
use reposix_core::backend::sim::SimBackend;
use reposix_core::BackendConnector;

/// Process-global lock for `REPOSIX_CACHE_DIR` mutation. Mirrors the
/// pattern in `delta_sync.rs` so independent test binaries don't race.
fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct CacheDirGuard<'a> {
    _guard: MutexGuard<'a, ()>,
    prev: Option<String>,
}

impl<'a> CacheDirGuard<'a> {
    fn new(path: &std::path::Path) -> Self {
        let guard = env_lock().lock().unwrap_or_else(|p| p.into_inner());
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

/// Spawn `reposix-sim` on a random port. Returns `(origin, JoinHandle)`.
async fn spawn_sim() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind 127.0.0.1:0");
    let addr = listener.local_addr().expect("local_addr");
    let origin = format!("http://{addr}");

    let cfg = reposix_sim::SimConfig {
        bind: addr,
        db_path: std::path::PathBuf::from(":memory:"),
        seed: false,
        seed_file: None,
        ephemeral: true,
        rate_limit_rps: 1000,
    };
    let handle = tokio::spawn(async move {
        let _ = reposix_sim::run_with_listener(listener, cfg).await;
    });
    let client =
        reposix_core::http::client(reposix_core::http::ClientOpts::default()).expect("http client");
    for _ in 0..50 {
        if let Ok(r) = client.get(format!("{origin}/healthz")).await {
            if r.status().is_success() {
                return (origin, handle);
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("sim did not become healthy at {origin}");
}

async fn seed_demo_issues(origin: &str, n: u64) {
    let client =
        reposix_core::http::client(reposix_core::http::ClientOpts::default()).expect("http client");
    for i in 1..=n {
        let body = serde_json::json!({
            "title": format!("issue-{i}"),
            "body": "",
            "status": "open",
            "labels": [],
        });
        let body_bytes = serde_json::to_vec(&body).unwrap();
        let url = format!("{origin}/projects/demo/issues");
        let resp = client
            .request_with_headers_and_body(
                reqwest::Method::POST,
                url.as_str(),
                &[
                    ("Content-Type", "application/json"),
                    ("X-Reposix-Agent", "cache-coherence-test"),
                ],
                Some(body_bytes),
            )
            .await
            .unwrap();
        assert!(
            resp.status().is_success(),
            "seed issue {i} failed: {}",
            resp.status()
        );
    }
}

async fn patch_issue_title(origin: &str, project: &str, id: u64, new_title: &str) {
    let client =
        reposix_core::http::client(reposix_core::http::ClientOpts::default()).expect("http client");
    let body = serde_json::json!({ "title": new_title });
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let url = format!("{origin}/projects/{project}/issues/{id}");
    let resp = client
        .request_with_headers_and_body(
            reqwest::Method::PATCH,
            url.as_str(),
            &[
                ("Content-Type", "application/json"),
                ("X-Reposix-Agent", "cache-coherence-test"),
            ],
            Some(body_bytes),
        )
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "patch issue {id} failed: {}",
        resp.status()
    );
}

/// GET a single issue's `updated_at` — used to pin the cursor into a write's
/// wall-clock second (the D-P92-03 trigger).
async fn fetch_issue_updated_at(origin: &str, project: &str, id: u64) -> chrono::DateTime<Utc> {
    let client =
        reposix_core::http::client(reposix_core::http::ClientOpts::default()).expect("http client");
    let url = format!("{origin}/projects/{project}/issues/{id}");
    let resp = client.get(url.as_str()).await.expect("GET issue");
    assert!(resp.status().is_success(), "GET issue {id}");
    let v: serde_json::Value = resp.json().await.expect("issue json");
    let raw = v["updated_at"].as_str().expect("updated_at is a string");
    chrono::DateTime::parse_from_rfc3339(raw)
        .expect("updated_at is RFC3339")
        .with_timezone(&Utc)
}

/// Walk the HEAD tree's record bucket (`issues/`) and return every blob OID it
/// references. This is the exact set the partial-clone `stateless-connect`
/// handler will try to serve on a puller's lazy fetch — so every one of them
/// MUST be resolvable by `read_blob`.
fn head_tree_blob_oids(repo_path: &std::path::Path) -> Vec<gix::ObjectId> {
    let repo = gix::open(repo_path).expect("open cache bare repo");
    let commit_id = repo
        .find_reference("refs/heads/main")
        .expect("refs/heads/main present after sync")
        .peel_to_id()
        .expect("peel main to commit id")
        .detach();
    let commit = repo
        .find_object(commit_id)
        .expect("commit object")
        .try_into_commit()
        .expect("HEAD is a commit");
    let tree = commit.tree().expect("commit tree");

    // Find the `issues` bucket subtree (the sim backend's record bucket).
    let bucket_oid = {
        let entry = tree
            .iter()
            .flatten()
            .find(|e| e.filename() == "issues")
            .expect("HEAD tree has an `issues/` bucket");
        entry.oid().to_owned()
    };
    let bucket_tree = repo
        .find_object(bucket_oid)
        .expect("find issues subtree")
        .try_into_tree()
        .expect("issues entry is a tree");

    bucket_tree
        .iter()
        .flatten()
        .map(|e| e.oid().to_owned())
        .collect()
}

/// Assert the coherence invariant: every blob OID the HEAD tree references
/// resolves via `read_blob` (no `UnknownOid`).
async fn assert_head_tree_coherent(cache: &Cache) {
    let oids = head_tree_blob_oids(cache.repo_path());
    assert!(
        !oids.is_empty(),
        "HEAD tree references zero blobs — test setup is wrong"
    );
    for oid in oids {
        let resolved = cache.read_blob(oid).await;
        assert!(
            resolved.is_ok(),
            "COHERENCE VIOLATION: HEAD tree references blob {oid} that read_blob \
             cannot resolve — a partial-clone lazy fetch of this OID dies \
             `git upload-pack: not our ref {oid}`. Got: {resolved:?}"
        );
    }
}

/// Positive control: `build_from` (the seed path) has always populated
/// `oid_map` for every listed record, so its HEAD tree is coherent by
/// construction.
#[tokio::test(flavor = "multi_thread")]
async fn head_tree_blobs_resolvable_after_seed_sync() {
    let (origin, _sim) = spawn_sim().await;
    seed_demo_issues(&origin, 4).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(origin.clone()).expect("SimBackend"));
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");

    cache.sync().await.expect("seed sync");
    assert_head_tree_coherent(&cache).await;
}

/// The D-P92-03 regression: a same-wall-clock-second write is invisible to
/// `list_changed_since` (0 changed), but `list_records` still reflects its
/// new content, so the HEAD tree references the new blob OID. Before the
/// ADR-010 fix, `oid_map` covered only the (empty) delta and `read_blob`
/// could not resolve that OID. After the fix, `Cache::sync` upserts `oid_map`
/// for the full `list_records` set, restoring the invariant.
#[tokio::test(flavor = "multi_thread")]
async fn head_tree_blobs_resolvable_after_same_second_delta_sync() {
    let (origin, _sim) = spawn_sim().await;
    seed_demo_issues(&origin, 3).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(origin.clone()).expect("SimBackend"));
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");

    // Seed sync (build_from): coherent baseline.
    cache.sync().await.expect("seed sync");

    // A second writer changes issue 2 on the backend.
    patch_issue_title(&origin, "demo", 2, "CHANGED-BY-A").await;

    // Pin THIS cache's cursor into issue 2's write-second (sub-second max) —
    // the sim truncates the cursor to seconds and compares `updated_at >
    // cursor` strictly, so issue 2's write is dropped from list_changed_since.
    let upd = fetch_issue_updated_at(&origin, "demo", 2).await;
    let pinned = upd
        .with_nanosecond(999_999_999)
        .expect("valid nanosecond")
        .to_rfc3339();
    {
        let conn = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
        conn.execute(
            "INSERT INTO meta (key, value, updated_at) VALUES ('last_fetched_at', ?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            rusqlite::params![&pinned, &pinned],
        )
        .unwrap();
    }

    // Delta sync: list_changed_since under-reports (0 changed) but the tree
    // still reflects issue 2's new content built from list_records.
    let r = cache.sync().await.expect("delta sync");
    assert_eq!(
        r.changed_ids.len(),
        0,
        "same-second boundary must make list_changed_since under-report here \
         (got {:?}); if non-empty the sim's cursor semantics changed and the \
         trigger no longer holds",
        r.changed_ids
    );

    // Invariant: every HEAD-tree blob OID — including issue 2's new,
    // un-detected-as-changed OID — resolves via read_blob.
    assert_head_tree_coherent(&cache).await;
}
