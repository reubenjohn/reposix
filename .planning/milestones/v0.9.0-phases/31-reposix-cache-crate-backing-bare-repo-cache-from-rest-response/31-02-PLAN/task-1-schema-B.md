← [back to index](./index.md)

# Task 1 (continued): Cache::open, build_from updates, lib.rs, audit_is_append_only test (Steps 6–10)

*Steps 1–5: [task-1-schema-A.md](./task-1-schema-A.md)*

## Action

Step 6 — Update `crates/reposix-cache/src/cache.rs` to store a `rusqlite::Connection`:
```rust
// Replace the existing `pub struct Cache { ... }` with:
pub struct Cache {
    pub(crate) backend: Arc<dyn BackendConnector>,
    pub(crate) backend_name: String,
    pub(crate) project: String,
    pub(crate) path: PathBuf,
    pub(crate) repo: gix::Repository,
    /// Wrapped in Mutex because rusqlite::Connection is !Send-safe across
    /// await points — we need interior mutability for the async methods.
    pub(crate) db: std::sync::Mutex<rusqlite::Connection>,
}

impl Cache {
    pub fn open(...) -> Result<Self> {
        // (existing body)
        let db = crate::db::open_cache_db(&path)?;
        // Cache-collision detection: if meta has a `backend`/`project`
        // row mismatching the args, error out (Plan 02 scaffolds; full
        // enforcement lands in Plan 03 or Phase 33).
        let expected = format!("{backend_name}:{project}");
        if let Some(found) = crate::meta::get_meta(&db, "identity")? {
            if found != expected {
                return Err(Error::CacheCollision { expected, found });
            }
        } else {
            crate::meta::set_meta(&db, "identity", &expected)?;
        }
        Ok(Self { backend, backend_name, project, path, repo, db: std::sync::Mutex::new(db) })
    }
}
```

Step 7 — Update `crates/reposix-cache/src/builder.rs::build_from` to record tree_sync + oid_map + last_fetched_at (does NOT yet add read_blob — that is Task 2):
```rust
// After computing `entries: Vec<(path, oid)>`, BEFORE writing the tree:
let db = self.db.lock().expect("cache db mutex poisoned");
for (path, oid) in &entries {
    // Extract issue_id from the path "issues/<id>.md" — simplest is to
    // key the oid_map on issue.id.0 directly. Pull the issue index from
    // the outer loop so we have both (issue, oid).
}
// Actually: refactor so the entries loop keeps the issue_id alongside:
let mut entries: Vec<(String, gix::ObjectId, String)> = Vec::with_capacity(issues.len());
for issue in &issues {
    let rendered = frontmatter::render(issue)?;
    let bytes = rendered.into_bytes();
    let oid = compute_blob_oid(&self.repo, &bytes)?;
    entries.push((format!("issues/{}.md", issue.id.0), oid, issue.id.0.to_string()));
}
// Populate oid_map + fire tree_sync audit row
for (_, oid, issue_id) in &entries {
    crate::meta::put_oid_mapping(&db, &self.backend_name, &self.project, &oid.to_hex().to_string(), issue_id)?;
}
crate::audit::log_tree_sync(&db, &self.backend_name, &self.project, entries.len());
// ... then continue with tree write + commit (unchanged from Plan 01).

// After commit succeeds, upsert last_fetched_at:
crate::meta::set_meta(&db, "last_fetched_at", &Utc::now().to_rfc3339())?;
```

Adjust the surrounding Plan 01 code so the tree-edit loop works off `entries` (use `(path, oid)` pairs — drop the `issue_id` for the upsert loop). Keep the same HEAD-overwrite step.

Step 8 — Update `crates/reposix-cache/src/lib.rs`:
```rust
pub mod audit;
pub mod builder;
pub mod cache;
pub mod db;
pub mod error;
pub mod meta;
pub mod path;

pub use cache::Cache;
pub use error::{Error, Result};
pub use path::{resolve_cache_path, CACHE_DIR_ENV};
```

Step 9 — Create `crates/reposix-cache/tests/audit_is_append_only.rs`:
```rust
//! ARCH-02: audit_events_cache is strictly append-only.

use rusqlite::params;
use tempfile::tempdir;

#[test]
fn update_and_delete_on_audit_table_both_fail() {
    let tmp = tempdir().unwrap();
    let conn = reposix_cache::db::open_cache_db(tmp.path()).unwrap();

    // Seed one row.
    conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project) VALUES (?1, 'tree_sync', 'sim', 'proj')",
        params!["2026-04-24T12:00:00Z"],
    ).unwrap();

    // UPDATE must fail with trigger message.
    let upd = conn.execute(
        "UPDATE audit_events_cache SET ts = 'tampered' WHERE id = 1",
        [],
    );
    let err = upd.expect_err("UPDATE must fail");
    let msg = err.to_string();
    assert!(msg.contains("append-only"),
            "expected trigger abort, got: {msg}");

    // DELETE must fail with trigger message.
    let del = conn.execute("DELETE FROM audit_events_cache WHERE id = 1", []);
    let err = del.expect_err("DELETE must fail");
    let msg = err.to_string();
    assert!(msg.contains("append-only"),
            "expected trigger abort, got: {msg}");

    // Row is still there.
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_events_cache", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}
```

Step 10 — Run:
```bash
cargo check -p reposix-cache
cargo clippy -p reposix-cache --all-targets -- -D warnings
cargo test -p reposix-cache --test audit_is_append_only
cargo test -p reposix-cache   # full crate including Plan 01 tests still green
```

## Acceptance Criteria

- `test -f crates/reposix-cache/fixtures/cache_schema.sql` returns 0.
- `grep -qE "RAISE\\(ABORT.*audit_events_cache is append-only" crates/reposix-cache/fixtures/cache_schema.sql` returns 0.
- `grep -qE "CREATE INDEX.*idx_oid_map_issue" crates/reposix-cache/fixtures/cache_schema.sql` returns 0.
- `grep -q "pub fn open_cache_db" crates/reposix-cache/src/db.rs` returns 0.
- `grep -q "mode(0o600)" crates/reposix-cache/src/db.rs` returns 0.
- `grep -q "enable_defensive" crates/reposix-cache/src/db.rs` returns 0.
- `grep -q "pub fn log_materialize" crates/reposix-cache/src/audit.rs` returns 0.
- `grep -q "pub fn log_egress_denied" crates/reposix-cache/src/audit.rs` returns 0.
- `grep -q "pub fn log_tree_sync" crates/reposix-cache/src/audit.rs` returns 0.
- `grep -q "pub fn set_meta\|pub fn get_meta\|pub fn put_oid_mapping\|pub fn get_issue_for_oid" crates/reposix-cache/src/meta.rs | wc -l` returns 4 (or multiple pipeline — ensure all four functions are defined).
- `grep -q "Egress(String)" crates/reposix-cache/src/error.rs` returns 0.
- `grep -q "UnknownOid" crates/reposix-cache/src/error.rs` returns 0.
- `grep -q "OidDrift" crates/reposix-cache/src/error.rs` returns 0.
- `cargo test -p reposix-cache --test audit_is_append_only` exits 0.
- `cargo test -p reposix-cache` (full crate) exits 0 — Plan 01 regression tests (`tree_contains_all_issues`, `blobs_are_lazy`) still pass with the extended `Cache::open`.
- `cargo clippy -p reposix-cache --all-targets -- -D warnings` exits 0.

## Verify

```
cargo test -p reposix-cache && cargo clippy -p reposix-cache --all-targets -- -D warnings
```

## Done

cache.db is opened with 0o600 + DEFENSIVE + WAL; schema loads cleanly; append-only triggers fire on UPDATE and DELETE attempts; `Cache::open` writes an identity row, and `Cache::build_from` records tree_sync + oid_map + last_fetched_at. Plan 01 integration tests still pass.
