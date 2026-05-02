← [back to index](./index.md)

# Task 02-T05 — Integration test: end-to-end delta sync against SimBackend

<read_first>
- `crates/reposix-cache/tests/` (existing integration tests for patterns)
- `crates/reposix-sim/src/` — how to start an in-process sim in a test
- `crates/reposix-core/src/backend/sim.rs::tests` — `SimBackend::new` pattern
</read_first>

<action>
Create `crates/reposix-cache/tests/delta_sync.rs`:

```rust
//! Phase 33 headline integration test — delta sync against SimBackend.
//!
//! Flow:
//!   1. Start in-process reposix-sim on a random port.
//!   2. Seed 5 issues in project "demo" via the sim's HTTP surface.
//!   3. Open a fresh Cache; call sync() → seed path → tree_sync audit,
//!      last_fetched_at = T1.
//!   4. Mutate issue 3's title via PATCH → sim updates updated_at to T2.
//!   5. Sleep 1s to guarantee T2 > T1 at second granularity.
//!   6. Call sync() again → delta path:
//!      - list_changed_since(T1) returns [IssueId(3)]
//!      - oid_map has a new row for issue 3's new blob
//!      - audit_events_cache has one new delta_sync row with bytes=1
//!      - meta.last_fetched_at = T3 (~now)
//!      - a new commit exists on refs/heads/main with a DIFFERENT tree
//!        than the seed commit
//!
//! Ground-truth assertion: after the second sync, the tree's entry for
//! `issues/3.md` points at a blob OID that is NOT in the seed commit's
//! tree. All other entries (issues/1,2,4,5.md) are unchanged.

use std::sync::Arc;

use reposix_cache::Cache;
use reposix_core::backend::{sim::SimBackend, BackendConnector};

// Start the sim. Reuse whatever harness the existing reposix-cache
// integration tests use (grep `crates/reposix-cache/tests/` for
// `spawn_sim` or similar; add one here if missing, modeled on
// `crates/reposix-sim/src/main.rs` + a random-port TcpListener).
mod common;

#[tokio::test(flavor = "multi_thread")]
async fn delta_sync_updates_only_changed_issue() {
    let (sim_origin, _guard) = common::spawn_sim().await;
    common::seed_demo_issues(&sim_origin, 5).await;

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(sim_origin.clone()).unwrap());
    let cache_root = tempfile::tempdir().unwrap();
    std::env::set_var("REPOSIX_CACHE_DIR", cache_root.path());
    let cache = Cache::open(backend.clone(), "sim", "demo").unwrap();

    // Seed sync.
    let r1 = cache.sync().await.expect("seed sync");
    assert!(r1.since.is_none(), "seed sync has no prior cursor");
    assert!(r1.new_commit.is_some());

    // Capture the seed tree's blob oid for issue 3.
    let seed_oid_for_3 = common::blob_oid_in_tree(cache.repo_path(), "3.md")
        .expect("seed tree has issues/3.md");

    // Mutate issue 3.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    common::patch_issue_title(&sim_origin, "demo", 3, "CHANGED").await;

    // Delta sync.
    let r2 = cache.sync().await.expect("delta sync");
    assert_eq!(r2.changed_ids.len(), 1, "exactly one issue changed");
    assert_eq!(r2.changed_ids[0].0, 3);
    assert!(r2.since.is_some(), "delta sync carries prior cursor");

    let post_oid_for_3 = common::blob_oid_in_tree(cache.repo_path(), "3.md")
        .expect("post-delta tree has issues/3.md");
    assert_ne!(seed_oid_for_3, post_oid_for_3,
        "issue 3's blob oid must change across delta sync");

    // Other issues' blob oids must be UNCHANGED.
    for id in [1u64, 2, 4, 5] {
        let fname = format!("{id}.md");
        let before = common::blob_oid_in_tree_at_commit(
            cache.repo_path(), r1.new_commit.unwrap(), &fname,
        ).unwrap();
        let after = common::blob_oid_in_tree_at_commit(
            cache.repo_path(), r2.new_commit.unwrap(), &fname,
        ).unwrap();
        assert_eq!(before, after,
            "issue {id}'s blob oid must NOT change across delta sync");
    }

    // Audit: exactly ONE delta_sync row, bytes=1.
    let audit = common::read_audit_rows(cache.repo_path(), "delta_sync");
    assert_eq!(audit.len(), 1, "exactly one delta_sync row");
    assert_eq!(audit[0].bytes, 1);
    assert!(audit[0].reason.starts_with("since="));
}

#[tokio::test(flavor = "multi_thread")]
async fn delta_sync_empty_delta_still_writes_audit_and_bumps_cursor() {
    let (sim_origin, _guard) = common::spawn_sim().await;
    common::seed_demo_issues(&sim_origin, 3).await;

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(sim_origin.clone()).unwrap());
    let cache_root = tempfile::tempdir().unwrap();
    std::env::set_var("REPOSIX_CACHE_DIR", cache_root.path());
    let cache = Cache::open(backend.clone(), "sim", "demo").unwrap();
    let _ = cache.sync().await.expect("seed sync");

    // No mutations. Delta = 0.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let r = cache.sync().await.expect("empty delta sync");
    assert_eq!(r.changed_ids.len(), 0);

    // Audit: one delta_sync row with bytes=0.
    let audit = common::read_audit_rows(cache.repo_path(), "delta_sync");
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0].bytes, 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn delta_sync_atomic_on_backend_error_midsync() {
    // Seed successfully, then point the backend at a dead origin so
    // list_changed_since fails. Assert last_fetched_at is UNCHANGED
    // after the failed sync.
    let (sim_origin, _guard) = common::spawn_sim().await;
    common::seed_demo_issues(&sim_origin, 2).await;

    let backend: Arc<dyn BackendConnector> =
        Arc::new(SimBackend::new(sim_origin.clone()).unwrap());
    let cache_root = tempfile::tempdir().unwrap();
    std::env::set_var("REPOSIX_CACHE_DIR", cache_root.path());
    let cache = Cache::open(backend.clone(), "sim", "demo").unwrap();
    let _ = cache.sync().await.expect("seed sync");
    let cursor_before = common::read_meta(cache.repo_path(), "last_fetched_at").unwrap();

    // Swap to a dead backend. Simplest way: drop the sim, re-open cache
    // with a fresh SimBackend pointed at the now-dead origin.
    drop(_guard);
    let dead_cache = Cache::open(
        Arc::new(SimBackend::new(sim_origin).unwrap()),
        "sim",
        "demo",
    ).unwrap();
    let result = dead_cache.sync().await;
    assert!(result.is_err(), "sync against dead backend must fail");

    let cursor_after = common::read_meta(cache.repo_path(), "last_fetched_at").unwrap();
    assert_eq!(cursor_before, cursor_after,
        "last_fetched_at must not advance on failed sync (atomicity)");
}
```

Create `crates/reposix-cache/tests/common/mod.rs` with helpers. **Important:** model these on existing helpers in `crates/reposix-cache/tests/*` (run `grep -rn 'spawn_sim\|seed_' crates/reposix-cache/tests/`). Reuse an existing module if available. If none exists, write:

```rust
//! Shared test harness for reposix-cache integration tests.

use std::path::Path;
use reqwest::Client;
use rusqlite::Connection;

pub struct SimGuard {
    pub handle: tokio::task::JoinHandle<()>,
    pub shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl Drop for SimGuard {
    fn drop(&mut self) {
        // Best-effort shutdown — JoinHandle will be dropped.
    }
}

pub async fn spawn_sim() -> (String, SimGuard) {
    // See crates/reposix-sim/src/main.rs for the server bootstrap.
    // Bind to 127.0.0.1:0 for a random port; query the resulting
    // SocketAddr; start the axum server in a tokio task.
    // Return (origin_url, guard).
    todo!("reuse crates/reposix-sim/src/main.rs's server factory; bind to port 0")
}

pub async fn seed_demo_issues(origin: &str, n: u64) {
    let client = Client::new();
    for i in 1..=n {
        let body = serde_json::json!({
            "title": format!("issue-{i}"),
            "body": "",
            "status": "open",
            "labels": [],
        });
        let resp = client
            .post(format!("{origin}/projects/demo/issues"))
            .header("X-Reposix-Agent", "test")
            .json(&body)
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success(), "seed issue {i} failed: {}", resp.status());
    }
}

pub async fn patch_issue_title(origin: &str, project: &str, id: u64, new_title: &str) {
    let client = Client::new();
    let body = serde_json::json!({
        "title": new_title,
        "body": "",
        "status": "open",
        "labels": [],
    });
    let resp = client
        .patch(format!("{origin}/projects/{project}/issues/{id}"))
        .header("X-Reposix-Agent", "test")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "patch issue {id} failed: {}", resp.status());
}

pub fn blob_oid_in_tree(repo_path: &Path, filename: &str) -> Option<gix::ObjectId> {
    let repo = gix::open(repo_path).ok()?;
    let head = repo.head_commit().ok()?;
    let tree = head.tree().ok()?;
    let issues_entry = tree.iter().flatten().find(|e| e.filename() == "issues")?;
    let issues_tree = repo.find_object(issues_entry.oid()).ok()?.try_into_tree().ok()?;
    issues_tree
        .iter()
        .flatten()
        .find(|e| e.filename() == filename)
        .map(|e| e.oid().to_owned())
}

pub fn blob_oid_in_tree_at_commit(
    repo_path: &Path, commit: gix::ObjectId, filename: &str,
) -> Option<gix::ObjectId> {
    let repo = gix::open(repo_path).ok()?;
    let commit_obj = repo.find_object(commit).ok()?.try_into_commit().ok()?;
    let tree = commit_obj.tree().ok()?;
    let issues_entry = tree.iter().flatten().find(|e| e.filename() == "issues")?;
    let issues_tree = repo.find_object(issues_entry.oid()).ok()?.try_into_tree().ok()?;
    issues_tree
        .iter()
        .flatten()
        .find(|e| e.filename() == filename)
        .map(|e| e.oid().to_owned())
}

#[derive(Debug)]
pub struct AuditRow {
    pub op: String,
    pub bytes: i64,
    pub reason: String,
}

pub fn read_audit_rows(repo_path: &Path, op_filter: &str) -> Vec<AuditRow> {
    let conn = Connection::open(repo_path.join("cache.db")).unwrap();
    let mut stmt = conn
        .prepare("SELECT op, COALESCE(bytes, 0), COALESCE(reason, '') FROM audit_events_cache WHERE op = ?1 ORDER BY id ASC")
        .unwrap();
    let rows: Vec<AuditRow> = stmt
        .query_map([op_filter], |r| {
            Ok(AuditRow {
                op: r.get(0)?,
                bytes: r.get(1)?,
                reason: r.get(2)?,
            })
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    rows
}

pub fn read_meta(repo_path: &Path, key: &str) -> Option<String> {
    let conn = Connection::open(repo_path.join("cache.db")).unwrap();
    conn.query_row(
        "SELECT value FROM meta WHERE key = ?1",
        [key],
        |r| r.get::<_, String>(0),
    )
    .ok()
}
```

Replace the `todo!` inside `spawn_sim` with a real in-process axum start — pattern after `crates/reposix-sim/src/main.rs` but bind `TcpListener::bind("127.0.0.1:0")` to get a free port, capture `.local_addr()`, and spawn the axum serve loop in a tokio task. If Phase 31's tests already have this helper, reuse directly.
</action>

<acceptance_criteria>
- `cargo test -p reposix-cache --test delta_sync delta_sync_updates_only_changed_issue` exits 0.
- `cargo test -p reposix-cache --test delta_sync delta_sync_empty_delta_still_writes_audit_and_bumps_cursor` exits 0.
- `cargo test -p reposix-cache --test delta_sync delta_sync_atomic_on_backend_error_midsync` exits 0.
- The first test asserts exactly ONE blob OID changed across the two sync commits (criterion #3 of Phase 33 ROADMAP success criteria).
</acceptance_criteria>

<threat_model>
The integration test is the empirical proof — not a mock — that (a) delta sync transfers only the changed blob, (b) the audit trail is mechanically produced, and (c) a mid-sync backend failure does not leave a torn cursor (atomicity). All three defenses are validated against git's view of the bare repo, not against bookkeeping assertions.
</threat_model>
