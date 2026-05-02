← [back to index](./index.md)

# Task 1: Land `cache_schema.sql` + `db.rs` + `audit.rs` + `meta.rs` modules and append-only trigger test

**Files:**
```
crates/reposix-cache/fixtures/cache_schema.sql,
crates/reposix-cache/src/db.rs,
crates/reposix-cache/src/audit.rs,
crates/reposix-cache/src/meta.rs,
crates/reposix-cache/src/lib.rs,
crates/reposix-cache/src/error.rs,
crates/reposix-cache/src/cache.rs,
crates/reposix-cache/tests/audit_is_append_only.rs
```

**Read first:**
```
crates/reposix-cache/src/lib.rs,
crates/reposix-cache/src/cache.rs,
crates/reposix-cache/src/error.rs,
crates/reposix-core/fixtures/audit.sql,
crates/reposix-core/src/audit.rs,
crates/reposix-cli/src/cache_db.rs,
.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-RESEARCH.md
```

## Behavior

- `crates/reposix-cache/fixtures/cache_schema.sql` defines three tables (`audit_events_cache`, `meta`, `oid_map`) wrapped in `BEGIN; ... COMMIT;`. Two BEFORE UPDATE / BEFORE DELETE triggers on `audit_events_cache` raise `ABORT` with message containing `"audit_events_cache is append-only"`. `meta` and `oid_map` are normal tables (mutable). An index `idx_oid_map_issue(backend, project, issue_id)` is present.
- `reposix_cache::db::open_cache_db(cache_dir)` creates `<cache_dir>/cache.db` with file mode `0o600`, opens with WAL mode, enables `SQLITE_DBCONFIG_DEFENSIVE` by calling through to `reposix_core::audit::enable_defensive`, then executes `cache_schema.sql`.
- `reposix_cache::audit::log_materialize(&conn, backend, project, issue_id, oid_hex, bytes)` INSERTs one row with `op='materialize'`, current UTC timestamp (RFC 3339).
- `reposix_cache::audit::log_egress_denied(&conn, backend, project, issue_id, reason)` INSERTs one row with `op='egress_denied'` and a `reason` string.
- `reposix_cache::audit::log_tree_sync(&conn, backend, project, items)` INSERTs one row with `op='tree_sync'` and `bytes=items` (item count).
- `reposix_cache::meta::set_meta(&conn, key, value)` / `get_meta(&conn, key) -> Option<String>` — upsert/lookup in the `meta` table with `updated_at = now()`.
- `reposix_cache::meta::put_oid_mapping(&conn, backend, project, oid_hex, issue_id)` / `get_issue_for_oid(&conn, oid_hex) -> Option<IssueId>` — the `oid_map` CRUD.
- Integration test `audit_is_append_only.rs` opens a fresh cache.db, inserts one audit row, then attempts (a) `UPDATE audit_events_cache SET ts='x' WHERE id=1` and (b) `DELETE FROM audit_events_cache WHERE id=1` — both must return an error whose string contains `"append-only"`.
- `Cache::open` is extended to also open `cache.db` at `<cache_path>/cache.db` and store the `rusqlite::Connection` in the struct.
- `Cache::build_from` (Plan 01) is UPDATED to insert one `op=tree_sync` row + one row per issue into `oid_map` + upsert `meta` key `last_fetched_at = now()` — all inside the same sync operation (not a single SQLite transaction yet; full atomicity is Phase 33's job per CONTEXT §Atomicity).

## Action

Step 1 — Create `crates/reposix-cache/fixtures/cache_schema.sql` verbatim from RESEARCH §Code Example 1:
```sql
-- Source: pattern from crates/reposix-core/fixtures/audit.sql (append-only triggers
-- + idempotent DROP TRIGGER IF EXISTS pattern lifted verbatim).
-- Phase 31 Plan 02 — audit_events_cache schema. SG-06 append-only invariant
-- enforced via BEFORE UPDATE/DELETE triggers; paired with DEFENSIVE flag
-- in db.rs::open_cache_db to block writable_schema bypass.

BEGIN;

CREATE TABLE IF NOT EXISTS audit_events_cache (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    ts            TEXT    NOT NULL,
    op            TEXT    NOT NULL CHECK (op IN ('materialize','egress_denied','tree_sync')),
    backend       TEXT    NOT NULL,
    project       TEXT    NOT NULL,
    issue_id      TEXT,
    oid           TEXT,
    bytes         INTEGER,
    reason        TEXT
);

CREATE TABLE IF NOT EXISTS meta (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS oid_map (
    oid       TEXT PRIMARY KEY,
    issue_id  TEXT NOT NULL,
    backend   TEXT NOT NULL,
    project   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_oid_map_issue
    ON oid_map(backend, project, issue_id);

DROP TRIGGER IF EXISTS audit_cache_no_update;
CREATE TRIGGER audit_cache_no_update BEFORE UPDATE ON audit_events_cache
    BEGIN
        SELECT RAISE(ABORT, 'audit_events_cache is append-only');
    END;

DROP TRIGGER IF EXISTS audit_cache_no_delete;
CREATE TRIGGER audit_cache_no_delete BEFORE DELETE ON audit_events_cache
    BEGIN
        SELECT RAISE(ABORT, 'audit_events_cache is append-only');
    END;

COMMIT;
```

Step 2 — Create `crates/reposix-cache/src/db.rs`:
```rust
//! cache.db — audit + meta + oid_map storage for the reposix-cache crate.
//!
//! Opens `<cache-dir>/cache.db` with:
//! - File created at mode `0o600` (mitigates T-31-02-01 local-user read).
//! - `SQLite` DEFENSIVE flag to block `writable_schema` attacks (H-02).
//! - WAL journal mode for concurrent-reader friendliness.
//! - Schema loaded from `fixtures/cache_schema.sql`.
//!
//! We deliberately do NOT use EXCLUSIVE locking here (unlike
//! `crates/reposix-cli/src/cache_db.rs`), because the cache DB is read
//! from multiple concurrent code paths (the helper's Phase 32
//! `stateless-connect` handler reads from `oid_map` while `build_from`
//! writes). Concurrency safety is Phase 33's job.

use std::os::unix::fs::OpenOptionsExt as _;
use std::path::Path;

use rusqlite::Connection;

use crate::error::{Error, Result};

const CACHE_SCHEMA_SQL: &str = include_str!("../fixtures/cache_schema.sql");

/// Open `<cache_dir>/cache.db` with 0o600 permissions, DEFENSIVE flag,
/// WAL, and the cache schema loaded.
///
/// # Errors
/// - [`Error::Io`] for directory/file creation failure.
/// - [`Error::Sqlite`] for rusqlite failures or schema load failures.
pub fn open_cache_db(cache_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(cache_dir)?;
    let path = cache_dir.join("cache.db");

    // Pre-create the file with 0o600 (mirrors crates/reposix-cli/src/cache_db.rs).
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&path)?;

    let conn = Connection::open(&path)
        .map_err(|e| Error::Sqlite(format!("open {}: {e}", path.display())))?;

    // Enable DEFENSIVE before any schema statement (same ordering as
    // reposix_core::audit::open_audit_db).
    reposix_core::audit::enable_defensive(&conn)
        .map_err(|e| Error::Sqlite(format!("enable_defensive: {e}")))?;

    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| Error::Sqlite(format!("WAL: {e}")))?;

    conn.execute_batch(CACHE_SCHEMA_SQL)
        .map_err(|e| Error::Sqlite(format!("schema load: {e}")))?;

    Ok(conn)
}
```

Step 3 — Create `crates/reposix-cache/src/audit.rs`:
```rust
//! Cache-event audit log — append-only INSERTs.
//!
//! Per CONTEXT.md §Atomicity: audit-row write failures log WARN via
//! `tracing::warn!` but do NOT poison the caller's flow. Callers use
//! `let _ = log_materialize(...)` to explicitly discard the result.

use chrono::Utc;
use rusqlite::{params, Connection};
use tracing::warn;

/// Insert `op='materialize'` row. Best-effort: on SQL error, WARN and return.
pub fn log_materialize(
    conn: &Connection,
    backend: &str,
    project: &str,
    issue_id: &str,
    oid_hex: &str,
    byte_len: usize,
) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, issue_id, oid, bytes) \
         VALUES (?1, 'materialize', ?2, ?3, ?4, ?5, ?6)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            issue_id,
            oid_hex,
            i64::try_from(byte_len).unwrap_or(i64::MAX),
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, issue_id, oid = oid_hex,
              "log_materialize failed: {e}");
    }
}

/// Insert `op='egress_denied'` row. Best-effort: on SQL error, WARN and return.
pub fn log_egress_denied(
    conn: &Connection,
    backend: &str,
    project: &str,
    issue_id: Option<&str>,
    reason: &str,
) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, issue_id, reason) \
         VALUES (?1, 'egress_denied', ?2, ?3, ?4, ?5)",
        params![Utc::now().to_rfc3339(), backend, project, issue_id, reason],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, reason,
              "log_egress_denied failed: {e}");
    }
}

/// Insert `op='tree_sync'` row.  Best-effort.
pub fn log_tree_sync(
    conn: &Connection,
    backend: &str,
    project: &str,
    items: usize,
) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, bytes) \
         VALUES (?1, 'tree_sync', ?2, ?3, ?4)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            i64::try_from(items).unwrap_or(i64::MAX),
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project,
              "log_tree_sync failed: {e}");
    }
}
```

Step 4 — Create `crates/reposix-cache/src/meta.rs`:
```rust
//! Non-append-only tables: `meta` (key/value) and `oid_map` (content-addressable lookup).

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension as _};

use crate::error::{Error, Result};

/// Upsert a single key/value pair into `meta`. `updated_at` is set to now.
///
/// # Errors
/// Returns [`Error::Sqlite`] for any rusqlite failure.
pub fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO meta (key, value, updated_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

/// Look up a `meta` value. Returns `None` if the key is not present.
///
/// # Errors
/// Returns [`Error::Sqlite`] for any rusqlite failure other than no-rows.
pub fn get_meta(conn: &Connection, key: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM meta WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(Error::from)
}

/// Insert (oid, issue_id) into `oid_map`. `INSERT OR REPLACE` so re-syncs
/// are clean overwrites.
///
/// # Errors
/// Returns [`Error::Sqlite`] for any rusqlite failure.
pub fn put_oid_mapping(
    conn: &Connection,
    backend: &str,
    project: &str,
    oid_hex: &str,
    issue_id: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO oid_map (oid, issue_id, backend, project) VALUES (?1, ?2, ?3, ?4)",
        params![oid_hex, issue_id, backend, project],
    )?;
    Ok(())
}

/// Look up the issue_id for a blob OID. Returns `None` if unknown.
///
/// # Errors
/// Returns [`Error::Sqlite`] for any rusqlite failure.
pub fn get_issue_for_oid(conn: &Connection, oid_hex: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT issue_id FROM oid_map WHERE oid = ?1",
        params![oid_hex],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(Error::from)
}
```

Step 5 — Update `crates/reposix-cache/src/error.rs` to replace the placeholder `Sqlite(String)` variant with `Sqlite(#[from] rusqlite::Error)` AND add the `Egress` variant explicitly:
```rust
// Replace existing Sqlite variant with:
#[error("sqlite: {0}")]
Sqlite(String),  // keep as String for flexibility — we already convert via `.map_err(|e| Sqlite(format!(...)))`

// Add new variant after `Render`:
/// Outbound HTTP origin not in REPOSIX_ALLOWED_ORIGINS.
#[error("egress denied: {0}")]
Egress(String),

/// Blob OID requested by the helper has no entry in `oid_map`.
#[error("unknown blob oid: {0}")]
UnknownOid(String),

/// The backend returned bytes whose blob OID does not match the OID
/// in the tree — eventual-consistency race on the backend side.
#[error("oid drift: requested {requested}, backend returned {actual} for issue {issue_id}")]
OidDrift { requested: String, actual: String, issue_id: String },
```

(Adjust the `From<rusqlite::Error>` impl that Plan 01 defined so it produces `Error::Sqlite(e.to_string())` — the existing Plan 01 impl already does this stringly, so likely no change needed; re-read it carefully.)

---

*Steps 6–10: [task-1-schema-B.md](./task-1-schema-B.md)*
