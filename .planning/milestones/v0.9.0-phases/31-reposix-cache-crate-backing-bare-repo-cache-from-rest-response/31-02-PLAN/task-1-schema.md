← [back to index](./index.md) · phase 31 plan 02

# Task 1: Schema + Audit + Metadata modules

Land `cache_schema.sql`, `db.rs`, `audit.rs`, `meta.rs` modules and append-only trigger test.

## Files

- `crates/reposix-cache/fixtures/cache_schema.sql`
- `crates/reposix-cache/src/db.rs`
- `crates/reposix-cache/src/audit.rs`
- `crates/reposix-cache/src/meta.rs`
- `crates/reposix-cache/src/lib.rs` (module declarations)
- `crates/reposix-cache/src/error.rs` (add Error variants)
- `crates/reposix-cache/src/cache.rs` (type hints)
- `crates/reposix-cache/tests/audit_is_append_only.rs`

## Read first

- `crates/reposix-cache/src/lib.rs` — current module structure.
- `crates/reposix-core/src/audit.rs` — DEFENSIVE flag and load_schema patterns.
- `crates/reposix-core/fixtures/audit.sql` — append-only trigger shape.

## Behavior

The cache holds three tables: `audit_events_cache` (append-only, audits all blob materializations + denials), `meta` (single row, tracks last fetch time + optional commit SHA), and `oid_map` (many rows, maps git OID → issue ID for deduplication on rebuild).

**Schema shape:**

```sql
CREATE TABLE IF NOT EXISTS audit_events_cache (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ts TEXT NOT NULL,         -- RFC3339 timestamp of audit event
  op TEXT NOT NULL,         -- 'materialize' | 'egress_denied' | 'tree_sync'
  oid TEXT,                 -- git object ID (NULL for tree_sync)
  issue_id TEXT,            -- issue ID if applicable (NULL for some ops)
  backend_name TEXT,        -- backend slug (sim, github, confluence, jira)
  details TEXT              -- JSON or free text for context
);
-- Indexes on (op), (ts), (issue_id) for query performance.

CREATE TABLE IF NOT EXISTS meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
-- Single row: key='last_fetched_at', value=RFC3339 timestamp.

CREATE TABLE IF NOT EXISTS oid_map (
  oid TEXT PRIMARY KEY,
  issue_id TEXT NOT NULL,
  backend_name TEXT NOT NULL
);
-- Reverse index: oid → issue_id for dedup on rebuild (see RESEARCH §Pitfall 2).
```

**`db.rs` exports:**
- `pub fn open_cache_db(path: &Path) -> Result<reposix_cache::db::CacheDb>` — opens `cache.db` with mode 0o600, DEFENSIVE flag, WAL enabled, and loads the schema from `cache_schema.sql`.
- `pub struct CacheDb(rusqlite::Connection)` — opaque handle.
- Implicit: `impl AsRef<rusqlite::Connection> for CacheDb` or explicit `fn conn(&self)` getter for tests.

**`audit.rs` exports:**
- `pub fn log_materialize(db: &CacheDb, oid: gix::ObjectId, issue_id: &str, backend_name: &str) -> Result<()>` — INSERT into `audit_events_cache` with `op='materialize'`. Returns `()` on success; best-effort WARN on INSERT failure (disk full, SQLite busy) — never poison the user flow per CONTEXT §Atomicity.
- `pub fn log_egress_denied(db: &CacheDb, origin: &str, backend_name: &str) -> Result<()>` — INSERT with `op='egress_denied'`.
- `pub fn log_tree_sync(db: &CacheDb, backend_name: &str) -> Result<()>` — INSERT with `op='tree_sync'`, `oid=NULL`.
- Use `let now = chrono::Utc::now().to_rfc3339();` for the `ts` column.

**`meta.rs` exports:**
- `pub fn get_meta(db: &CacheDb, key: &str) -> Result<Option<String>>` — SELECT from `meta` WHERE key=?.
- `pub fn set_meta(db: &CacheDb, key: &str, value: &str) -> Result<()>` — INSERT OR REPLACE.
- `pub fn put_oid_mapping(db: &CacheDb, oid: gix::ObjectId, issue_id: &str, backend_name: &str) -> Result<()>` — INSERT OR REPLACE into `oid_map`.
- `pub fn get_issue_for_oid(db: &CacheDb, oid: gix::ObjectId) -> Result<Option<(String, String)>>` — SELECT (issue_id, backend_name) for a given OID.

**`lib.rs` changes:**
- Add `pub mod db;`, `pub mod audit;`, `pub mod meta;` (alphabetically).
- Re-export key symbols: `pub use db::{open_cache_db, CacheDb}; pub use audit::{log_materialize, log_egress_denied, log_tree_sync};` etc.

**`error.rs` changes:**
- Add `SQLiteConstraintViolation(String)` variant if not present (for append-only trigger fires).
- Derive Display / Debug / thiserror as needed.

**`tests/audit_is_append_only.rs`:**

```rust
#[test]
fn audit_events_cache_rejects_update() {
    let tmp = tempdir().unwrap();
    let db = open_cache_db(tmp.path()).unwrap();
    
    // Insert one row via the helper.
    log_materialize(&db, some_oid, "issue-1", "sim").unwrap();
    
    // Attempt UPDATE on that row — must fail with CONSTRAINT.
    let result = db.conn().execute(
        "UPDATE audit_events_cache SET ts = ? WHERE op = ?",
        ["2099-01-01T00:00:00Z", "materialize"],
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("append-only"));
}

#[test]
fn audit_events_cache_rejects_delete() {
    let tmp = tempdir().unwrap();
    let db = open_cache_db(tmp.path()).unwrap();
    
    log_materialize(&db, some_oid, "issue-1", "sim").unwrap();
    
    // Attempt DELETE — must fail with CONSTRAINT.
    let result = db.conn().execute(
        "DELETE FROM audit_events_cache WHERE op = ?",
        ["materialize"],
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("append-only"));
}
```

## Action

**Step 1 — Create `crates/reposix-cache/fixtures/cache_schema.sql`.**

Write the DDL with three tables + indexes + append-only triggers. Trigger format (imitated from `reposix-core/fixtures/audit.sql`):

```sql
DROP TRIGGER IF EXISTS audit_cache_no_update;
CREATE TRIGGER audit_cache_no_update BEFORE UPDATE ON audit_events_cache
    BEGIN SELECT RAISE(ABORT, 'audit_events_cache is append-only'); END;
DROP TRIGGER IF EXISTS audit_cache_no_delete;
CREATE TRIGGER audit_cache_no_delete BEFORE DELETE ON audit_events_cache
    BEGIN SELECT RAISE(ABORT, 'audit_events_cache is append-only'); END;
```

Wrap the whole thing in `BEGIN; ... COMMIT;` to ensure atomicity on load.

**Step 2 — Create `crates/reposix-cache/src/db.rs`.**

Implement `open_cache_db(path: &Path) -> Result<CacheDb>`:
1. Open or create a `rusqlite::Connection` at `path/cache.db` with `OpenFlags::SQLITE_OPEN_READ_WRITE | SQLITE_OPEN_CREATE`.
2. Set file mode to `0o600` via `fs::set_permissions` (after the file is created).
3. Call `reposix_core::audit::enable_defensive(&conn)` to set `SQLITE_DBCONFIG_DEFENSIVE`.
4. Execute `PRAGMA journal_mode = WAL;`.
5. Load the schema from the `cache_schema.sql` fixture (read the file, execute as one statement via `conn.execute_batch`).
6. Return `Ok(CacheDb(conn))`.

Use `anyhow::Context` or map errors through `reposix_cache::error::Error`. Wrap rusqlite errors as `Error::Sqlite(String)` or similar.

**Step 3 — Create `crates/reposix-cache/src/audit.rs`.**

Implement the three logging functions. Each function:
- Computes `chrono::Utc::now().to_rfc3339()` for the `ts` column.
- INSERTs into `audit_events_cache` (the full row for each op type).
- Returns `Result<()>`.
- On INSERT failure: emit `tracing::warn!(target: "reposix_cache::audit_failure", ...)` with the error details; return `Ok(())` (best-effort, don't poison the flow).

Example for `log_materialize`:
```rust
pub fn log_materialize(db: &CacheDb, oid: gix::ObjectId, issue_id: &str, backend_name: &str) -> Result<()> {
    let ts = chrono::Utc::now().to_rfc3339();
    let oid_str = oid.to_string();
    
    db.conn().execute(
        "INSERT INTO audit_events_cache (ts, op, oid, issue_id, backend_name) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![ts, "materialize", oid_str, issue_id, backend_name],
    ).map(|_| ())
    .or_else(|e| {
        tracing::warn!(target: "reposix_cache::audit_failure", 
                       op="materialize", issue_id, backend_name, 
                       error=%e, "audit log INSERT failed");
        Ok(())
    })
}
```

**Step 4 — Create `crates/reposix-cache/src/meta.rs`.**

Implement the four metadata functions. Follow the same best-effort-warn pattern for failures.

**Step 5 — Update `crates/reposix-cache/src/lib.rs`.**

Add module declarations:
```rust
pub mod audit;
pub mod db;
pub mod meta;
```

And re-exports:
```rust
pub use db::{open_cache_db, CacheDb};
pub use audit::{log_materialize, log_egress_denied, log_tree_sync};
pub use meta::{get_meta, set_meta, put_oid_mapping, get_issue_for_oid};
```

**Step 6 — Update `crates/reposix-cache/src/error.rs`.**

Add error variants for SQLite constraints:
```rust
#[error("SQLite constraint violation: {0}")]
SqliteConstraint(String),
```

Map rusqlite errors from `db.rs` and other modules via `From<rusqlite::Error>` conversion.

**Step 7 — Write `crates/reposix-cache/tests/audit_is_append_only.rs`.**

See Behavior section for the test shape. Verify both UPDATE and DELETE reject with the expected message.

**Step 8 — Run and verify:**

```bash
cargo test -p reposix-cache --test audit_is_append_only
cargo test -p reposix-cache  # run all unit tests in db, audit, meta modules
cargo clippy -p reposix-cache --all-targets -- -D warnings
```

All must pass with zero warnings.

## Acceptance criteria

- `grep -q "CREATE TABLE IF NOT EXISTS audit_events_cache" crates/reposix-cache/fixtures/cache_schema.sql` returns 0.
- `grep -q "RAISE(ABORT, 'audit_events_cache is append-only')" crates/reposix-cache/fixtures/cache_schema.sql` returns 0.
- `grep -q "pub fn open_cache_db" crates/reposix-cache/src/db.rs` returns 0.
- `grep -q "pub fn log_materialize" crates/reposix-cache/src/audit.rs` returns 0.
- `grep -q "pub fn set_meta" crates/reposix-cache/src/meta.rs` returns 0.
- `grep -q "pub mod db" crates/reposix-cache/src/lib.rs` returns 0.
- `grep -q "SQLITE_DBCONFIG_DEFENSIVE" crates/reposix-cache/src/db.rs` returns 0.
- `grep -q "journal_mode = WAL" crates/reposix-cache/src/db.rs` returns 0.
- `cargo test -p reposix-cache --test audit_is_append_only` exits 0.
- `cargo test -p reposix-cache` (full crate) exits 0.
- `cargo clippy -p reposix-cache --all-targets -- -D warnings` exits 0.

## Verify

Automated:
```
cargo test -p reposix-cache && cargo clippy -p reposix-cache --all-targets -- -D warnings
```

Manual (optional):
```
sqlite3 runtime/test.db '.read crates/reposix-cache/fixtures/cache_schema.sql'
# Verify CREATE TABLE statements succeed.
```

## Done

Three tables exist with append-only enforcement on `audit_events_cache`. Open, metadata, and audit helpers are wired. The append-only trigger test passes.
