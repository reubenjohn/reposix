//! Phase 33 headline integration test — delta sync against `reposix-sim`.
//!
//! Flow exercised:
//! 1. Start an in-process `reposix-sim` on `127.0.0.1:0` (random port).
//! 2. Seed `n` issues in project "demo" via the sim's HTTP surface.
//! 3. Open a fresh `Cache`; call `sync()` → seed path → `tree_sync` audit
//!    + `meta.last_fetched_at = T1`.
//! 4. Mutate one issue via PATCH → sim updates its `updated_at` to T2.
//! 5. Sleep 1100ms so T2 > T1 at second granularity.
//! 6. Call `sync()` again → delta path:
//!    - `list_changed_since(T1)` returns exactly `[mutated_id]`.
//!    - `oid_map` contains a new row for the mutated issue's new blob.
//!    - `audit_events_cache` has one new `delta_sync` row with `bytes=1`.
//!    - `meta.last_fetched_at` is bumped.
//!    - The new commit's tree differs from the seed commit's tree only
//!      at the mutated issue's blob OID.
//!
//! Ground-truth assertion: across the two sync commits, exactly ONE blob
//! OID changes (the mutated issue's). Other blob OIDs are pin-equal.

#![allow(clippy::missing_panics_doc)]

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;

use chrono::{Timelike, Utc};
use reposix_cache::Cache;
use reposix_core::backend::sim::SimBackend;
use reposix_core::BackendConnector;

/// Process-global lock for `REPOSIX_CACHE_DIR` mutation. Mirrors the
/// pattern in `tests/common/mod.rs` so independent tests don't race.
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
/// The handle is held by the test for the duration of the test; the
/// server quits when the process exits.
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
    // Wait until /healthz responds, with a 5s budget. Use the workspace
    // HttpClient so this test runs through the same SG-01 allowlist as
    // production code (loopback is allowlisted by default).
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
                    ("X-Reposix-Agent", "delta-sync-test"),
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

/// GET a single issue and return its `updated_at` as a parsed
/// `DateTime<Utc>`. Used by the D-P92-03 coherence regression to pin the
/// cache cursor into the same wall-clock second as a backend write.
async fn fetch_issue_updated_at(origin: &str, project: &str, id: u64) -> chrono::DateTime<Utc> {
    let client =
        reposix_core::http::client(reposix_core::http::ClientOpts::default()).expect("http client");
    let url = format!("{origin}/projects/{project}/issues/{id}");
    let resp = client.get(url.as_str()).await.expect("GET issue");
    assert!(
        resp.status().is_success(),
        "GET issue {id}: {}",
        resp.status()
    );
    let v: serde_json::Value = resp.json().await.expect("issue json");
    let raw = v["updated_at"].as_str().expect("updated_at is a string");
    chrono::DateTime::parse_from_rfc3339(raw)
        .expect("updated_at is RFC3339")
        .with_timezone(&Utc)
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
                ("X-Reposix-Agent", "delta-sync-test"),
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

fn blob_oid_in_tree_at_commit(
    repo_path: &std::path::Path,
    commit: gix::ObjectId,
    filename: &str,
) -> Option<gix::ObjectId> {
    let repo = gix::open(repo_path).ok()?;
    let commit_obj = repo.find_object(commit).ok()?.try_into_commit().ok()?;
    let tree = commit_obj.tree().ok()?;
    // Find `issues` subtree OID inside a scoped block so the iterator
    // borrow on `tree` is dropped before we call `find_object` again.
    let issues_oid = {
        let entry = tree.iter().flatten().find(|e| e.filename() == "issues")?;
        entry.oid().to_owned()
    };
    let issues_tree = repo.find_object(issues_oid).ok()?.try_into_tree().ok()?;
    let target_oid = {
        let entry = issues_tree
            .iter()
            .flatten()
            .find(|e| e.filename() == filename)?;
        entry.oid().to_owned()
    };
    Some(target_oid)
}

#[derive(Debug)]
struct AuditRow {
    op: String,
    bytes: i64,
    reason: String,
}

fn read_audit_rows(repo_path: &std::path::Path, op_filter: &str) -> Vec<AuditRow> {
    let conn = rusqlite::Connection::open(repo_path.join("cache.db")).unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT op, COALESCE(bytes, 0), COALESCE(reason, '') \
             FROM audit_events_cache WHERE op = ?1 ORDER BY id ASC",
        )
        .unwrap();
    stmt.query_map([op_filter], |r| {
        Ok(AuditRow {
            op: r.get(0)?,
            bytes: r.get(1)?,
            reason: r.get(2)?,
        })
    })
    .unwrap()
    .map(std::result::Result::unwrap)
    .collect()
}

fn read_meta(repo_path: &std::path::Path, key: &str) -> Option<String> {
    let conn = rusqlite::Connection::open(repo_path.join("cache.db")).unwrap();
    conn.query_row("SELECT value FROM meta WHERE key = ?1", [key], |r| {
        r.get::<_, String>(0)
    })
    .ok()
}

#[tokio::test(flavor = "multi_thread")]
async fn delta_sync_updates_only_changed_issue() {
    let (origin, _sim) = spawn_sim().await;
    seed_demo_issues(&origin, 5).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(origin.clone()).expect("SimBackend"));
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");

    // Seed sync: no prior cursor → forwards to build_from internally.
    let r1 = cache.sync().await.expect("seed sync");
    assert!(r1.since.is_none(), "seed sync has no prior cursor");
    let seed_commit = r1.new_commit.expect("seed produces a commit");

    // Pin: each of the 5 seeded blobs appears in the seed commit's tree.
    let mut seed_oids: Vec<gix::ObjectId> = Vec::new();
    for id in 1u64..=5 {
        let fname = format!("{id}.md");
        let oid = blob_oid_in_tree_at_commit(cache.repo_path(), seed_commit, &fname)
            .unwrap_or_else(|| panic!("seed tree missing issues/{fname}"));
        seed_oids.push(oid);
    }

    // Sleep so the patched issue's updated_at is strictly > seed timestamp.
    tokio::time::sleep(Duration::from_millis(1100)).await;
    patch_issue_title(&origin, "demo", 3, "CHANGED").await;

    // Delta sync.
    let r2 = cache.sync().await.expect("delta sync");
    assert_eq!(
        r2.changed_ids.len(),
        1,
        "exactly one issue changed, got {:?}",
        r2.changed_ids
    );
    assert_eq!(r2.changed_ids[0].0, 3);
    assert!(r2.since.is_some(), "delta sync carries prior cursor");
    let post_commit = r2.new_commit.expect("delta produces a commit");

    // Ground-truth: issue 3's blob OID changed; the others did not.
    let post_oid_for_3 = blob_oid_in_tree_at_commit(cache.repo_path(), post_commit, "3.md")
        .expect("post-delta tree has issues/3.md");
    let seed_oid_for_3 = seed_oids[2];
    assert_ne!(
        seed_oid_for_3, post_oid_for_3,
        "issue 3's blob oid must change across delta sync"
    );

    for id in [1u64, 2, 4, 5] {
        let fname = format!("{id}.md");
        let before = blob_oid_in_tree_at_commit(cache.repo_path(), seed_commit, &fname)
            .expect("seed tree has issue");
        let after = blob_oid_in_tree_at_commit(cache.repo_path(), post_commit, &fname)
            .expect("post-delta tree has issue");
        assert_eq!(
            before, after,
            "issue {id}'s blob oid must NOT change across delta sync"
        );
    }

    // Audit: exactly ONE delta_sync row with bytes=1 and reason starting "since=".
    let audit = read_audit_rows(cache.repo_path(), "delta_sync");
    assert_eq!(audit.len(), 1, "exactly one delta_sync row, got {audit:?}");
    assert_eq!(
        audit[0].bytes, 1,
        "delta_sync bytes must be the changed-id count"
    );
    assert!(
        audit[0].reason.starts_with("since="),
        "delta_sync reason must start with 'since=', got {:?}",
        audit[0].reason
    );
    assert_eq!(audit[0].op, "delta_sync");

    // Cursor advanced.
    let cursor_before = r1
        .since
        .map(|t| t.to_rfc3339())
        .unwrap_or_else(|| "<seed>".into());
    let cursor_after = read_meta(cache.repo_path(), "last_fetched_at").expect("cursor present");
    assert_ne!(
        cursor_after, cursor_before,
        "last_fetched_at must advance across a successful delta sync"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delta_sync_empty_delta_still_writes_audit_and_bumps_cursor() {
    let (origin, _sim) = spawn_sim().await;
    seed_demo_issues(&origin, 3).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(origin.clone()).expect("SimBackend"));
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");
    let _ = cache.sync().await.expect("seed sync");
    let cursor_before = read_meta(cache.repo_path(), "last_fetched_at").expect("cursor seeded");

    // No mutations, but sleep enough that any newly-issued cursor differs
    // from the seed cursor at second granularity.
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let r = cache.sync().await.expect("empty delta sync");
    assert_eq!(r.changed_ids.len(), 0, "no mutations → empty delta");
    assert!(r.since.is_some(), "non-seed sync carries the prior cursor");

    // Audit: one delta_sync row with bytes=0.
    let audit = read_audit_rows(cache.repo_path(), "delta_sync");
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].bytes, 0);

    let cursor_after = read_meta(cache.repo_path(), "last_fetched_at").expect("cursor present");
    assert_ne!(
        cursor_after, cursor_before,
        "empty-delta sync must still bump last_fetched_at"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delta_sync_atomic_on_backend_error_midsync() {
    // Atomicity proof: when list_changed_since fails (network error), the
    // SQLite transaction holding `meta.last_fetched_at` + `oid_map` writes
    // + the `delta_sync` audit row must roll back. Cursor stays put;
    // the next sync retries the same window.
    //
    // Setup: open a Cache with a SimBackend pointed at a dead port. The
    // first sync attempts list_records → 502/connection-refused → returns
    // Err. We then poke the cache.db's meta.last_fetched_at to a known
    // value (simulating a previously-successful seed) and call sync()
    // again — this time list_changed_since fails and we assert the meta
    // row is unchanged.
    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());

    // Bind a fresh listener on 127.0.0.1:0, capture its port, drop the
    // listener. Subsequent connects to that port get connection-refused
    // (for the test's lifetime — the kernel only reuses the ephemeral
    // port range after a quiescence window).
    let dead_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind dead port");
    let dead_addr = dead_listener.local_addr().expect("local_addr");
    drop(dead_listener);
    let dead_origin = format!("http://{dead_addr}");

    let dead_backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(dead_origin).expect("SimBackend (dead)"));
    let cache = Cache::open(dead_backend, "sim", "demo").expect("Cache::open");

    // First sync against the dead backend should fail (no seed possible).
    let first = cache.sync().await;
    assert!(
        first.is_err(),
        "seed sync against dead backend must fail, got {first:?}"
    );

    // Manually plant a `last_fetched_at` cursor to simulate a successful
    // seed in the past. This drives the second sync onto the delta path.
    let cursor_before = "2026-04-01T00:00:00+00:00".to_owned();
    {
        let conn = rusqlite::Connection::open(cache.repo_path().join("cache.db")).unwrap();
        conn.execute(
            "INSERT INTO meta (key, value, updated_at) VALUES ('last_fetched_at', ?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            rusqlite::params![&cursor_before, &cursor_before],
        )
        .unwrap();
    }

    // Second sync: delta path, list_changed_since fails on the dead origin.
    let result = cache.sync().await;
    assert!(
        result.is_err(),
        "delta sync against dead backend must fail, got {result:?}"
    );

    let cursor_after = read_meta(cache.repo_path(), "last_fetched_at").expect("cursor present");
    assert_eq!(
        cursor_before, cursor_after,
        "last_fetched_at must NOT advance on failed sync (atomicity)"
    );

    // No delta_sync audit row should have been written (rollback proof).
    let audit = read_audit_rows(cache.repo_path(), "delta_sync");
    assert_eq!(
        audit.len(),
        0,
        "failed delta sync must not leak a delta_sync audit row"
    );
}

/// RED regression for D-P92-03 (P93 cache-coherence) — delta-sync builds a
/// tree entry it cannot serve.
///
/// PROVE-BEFORE-FIX: this test is deliberately RED against current `main`. It
/// is `#[ignore]`d so it does not break the default CI gate; the fix is
/// coordinator-gated (do NOT apply it in the investigation lane). Un-ignore it
/// (drop the `#[ignore]`) in the same commit that lands the coherence fix.
///
/// ## The defect
///
/// `Cache::sync` (crates/reposix-cache/src/builder.rs) sources the git TREE
/// from `list_records` (the FULL current backend state, Step 4) while it only
/// materializes blob objects + upserts `oid_map` for the `list_changed_since`
/// delta (Steps 3 & 5). When the two disagree, the tree references a blob OID
/// that was neither written to the object store nor recorded in `oid_map` — a
/// dangling entry. A partial-clone lazy fetch of that OID hits
/// `Cache::read_blob` → `Error::UnknownOid` → the helper leaves the `want` for
/// `git upload-pack`, which rejects it: `fatal: git upload-pack: not our ref
/// <oid>` / `could not fetch <oid> from promisor remote`.
///
/// ## The trigger (executed side-by-side in the container litmus)
///
/// The `reposix-sim` stores `updated_at` at SECONDS precision
/// (`now_rfc3339` = `SecondsFormat::Secs`) and filters `list_changed_since`
/// with a seconds-truncated cursor under a STRICT `updated_at > ?` compare
/// (crates/reposix-sim/src/routes/issues.rs). Any backend write whose
/// truncated second is `<=` the cursor's truncated second is therefore invisible
/// to `list_changed_since`, even though `list_records` (unfiltered) still returns
/// its new content. This is exactly what happens when a second writer's cache
/// cursor and the first writer's push land in the same wall-clock second — the
/// D-P92-03 two-writer `git pull --rebase` scenario. Full executed transcripts
/// (git 2.54 container, "not our ref" reproduced 4/4 same-second runs; clean
/// `CONFLICT (content)` in the 2s-gap negative control):
/// `.planning/milestones/v0.13.0-phases/93-cache-coherence/93-DP2-REPRO-NOTES.md`.
///
/// Here we reproduce the trigger deterministically at the cache layer by
/// pinning the cursor into the write's second, then assert the coherence
/// invariant the helper relies on: EVERY blob OID the HEAD tree references
/// must be resolvable by `read_blob` (no `UnknownOid`).
// GREEN as of ADR-010 / RBF-LR-01: `Cache::sync` Step 5 now upserts oid_map
// for the FULL `list_records` set, so the HEAD tree entry for a same-second
// under-reported change is resolvable by `read_blob` (was RED — `not our
// ref <oid>` — before the coherence fix). See
// docs/decisions/010-l2-l3-cache-coherence.md.
#[tokio::test(flavor = "multi_thread")]
async fn delta_sync_tree_references_only_resolvable_oids() {
    let (origin, _sim) = spawn_sim().await;
    seed_demo_issues(&origin, 3).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(origin.clone()).expect("SimBackend"));
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");

    // Seed sync (build_from): tree + oid_map + blobs for the 3 seeds.
    let _ = cache.sync().await.expect("seed sync");

    // A second writer changes issue 2 on the backend ("writer A pushes").
    patch_issue_title(&origin, "demo", 2, "CHANGED-BY-A").await;

    // Pin THIS cache's cursor into issue 2's write-second, sub-second max —
    // exactly the same-wall-clock-second race the litmus hits naturally.
    // The sim truncates the cursor to seconds and compares `updated_at > cursor`
    // strictly, so issue 2's write (updated_at == that second) is dropped.
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

    // Delta sync. The boundary bug makes list_changed_since under-report:
    // 0 changed, even though issue 2's content is new.
    let r = cache.sync().await.expect("delta sync");
    assert_eq!(
        r.changed_ids.len(),
        0,
        "same-second boundary: list_changed_since must under-report here (got {:?}) — \
         if this is non-empty the sim's cursor semantics changed and the trigger no longer holds",
        r.changed_ids
    );
    let commit = r.new_commit.expect("delta sync writes a commit");

    // The HEAD tree STILL reflects issue 2's NEW content (built from
    // list_records), so it references the new blob OID...
    let tree_oid = blob_oid_in_tree_at_commit(cache.repo_path(), commit, "2.md")
        .expect("post-sync tree has issues/2.md");

    // ...but the coherence invariant is violated: read_blob cannot resolve it
    // (no oid_map row was written for the un-detected change), which is the
    // precise precondition for `git upload-pack: not our ref <oid>`.
    let resolved = cache.read_blob(tree_oid).await;
    assert!(
        resolved.is_ok(),
        "COHERENCE VIOLATION (D-P92-03): HEAD tree references blob {tree_oid} that \
         read_blob cannot resolve — a partial-clone lazy fetch of this OID dies \
         `git upload-pack: not our ref {tree_oid}`. Got: {resolved:?}"
    );
}
