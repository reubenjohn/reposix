//! Append-only audit-log schema fixture (SG-06 + FC-06).
//!
//! This module publishes the DDL Phase 2 loads at simulator startup. The
//! schema itself — specifically, the `BEFORE UPDATE` / `BEFORE DELETE`
//! triggers on `audit_events` — enforces append-only semantics. Runtime
//! Rust code never needs to "remember" not to delete; the DB refuses.
//!
//! # Schema-attack hardening (H-02, phase-1 review)
//!
//! Row-level UPDATE/DELETE triggers are only half the SG-06 story. An
//! attacker with the same DB handle could otherwise disable the triggers
//! with `DROP TRIGGER audit_no_delete`, remove the whole table with
//! `DROP TABLE audit_events`, or flip `PRAGMA writable_schema=ON` and edit
//! `sqlite_master` directly. This module's [`open_audit_db`] opens the
//! connection with `SQLite`'s `SQLITE_DBCONFIG_DEFENSIVE` flag, which makes
//! `writable_schema=ON` a no-op and (crucially for us) prevents
//! `sqlite_master` edits that could strip the triggers. Integration tests
//! in `tests/audit_schema.rs` prove the three schema-attack vectors are
//! rejected or rendered harmless on a defensively-opened handle.
//!
//! Callers in Phase 2 MUST open the runtime audit DB via [`open_audit_db`]
//! (not via raw `rusqlite::Connection::open`) so the DEFENSIVE flag is set
//! before any schema statement executes.
//!
//! T-01-13 (exfiltration via raw body content) is `accept`ed at this layer:
//! the schema defines `request_body TEXT` but Phase 2's insert path is
//! responsible for hashing / redacting sensitive content before insert.

use std::path::Path;

use rusqlite::config::DbConfig;

use crate::{Error, Result};

/// Canonical DDL for the `audit_events` table and its append-only triggers.
pub const SCHEMA_SQL: &str = include_str!("../fixtures/audit.sql");

/// Load the schema into an open `SQLite` connection. Idempotent: every
/// statement uses `IF NOT EXISTS`, so calling this twice on the same
/// connection is a no-op.
///
/// # Errors
/// Returns [`Error::Other`] wrapping the underlying `rusqlite::Error` if
/// the batch execute fails (typically a bad connection or concurrent-schema
/// race).
pub fn load_schema(conn: &rusqlite::Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_SQL)
        .map_err(|e| Error::Other(format!("load_schema: {e}")))
}

/// Enable `SQLITE_DBCONFIG_DEFENSIVE` on `conn`.
///
/// The DEFENSIVE flag (`SQLite` 3.26+) refuses edits to `sqlite_master` via
/// `PRAGMA writable_schema=ON`. On an in-process attacker path, this means
/// `DROP TRIGGER` / `DROP TABLE` remain syntactically legal (they take
/// the *authenticated* schema-edit path) but attempts to bypass them by
/// writing to `sqlite_master` directly are rejected.
///
/// # Errors
/// Returns [`Error::Other`] wrapping the underlying `rusqlite::Error` if
/// the flag cannot be set (typically: `SQLite` built without defensive
/// support — `SQLITE_DBCONFIG_DEFENSIVE` is in the stable API since 3.26
/// and the `rusqlite` bundled build is well past that).
pub fn enable_defensive(conn: &rusqlite::Connection) -> Result<()> {
    conn.set_db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)
        .map(|_| ())
        .map_err(|e| Error::Other(format!("set DEFENSIVE: {e}")))
}

/// Open the audit `SQLite` DB at `path` with the append-only invariant
/// hardened against schema-level attacks.
///
/// Opens the file with default read-write flags, enables
/// `SQLITE_DBCONFIG_DEFENSIVE` (see [`enable_defensive`]), then loads the
/// canonical schema via [`load_schema`]. The returned connection is ready
/// for inserts; further schema edits from this handle are either rejected
/// by DEFENSIVE (`writable_schema` path) or will still be caught by the
/// row-level triggers (UPDATE/DELETE path).
///
/// # Errors
/// Returns [`Error::Other`] if the file cannot be opened, the defensive
/// flag cannot be set, or the schema batch fails.
pub fn open_audit_db(path: &Path) -> Result<rusqlite::Connection> {
    let conn = rusqlite::Connection::open(path)
        .map_err(|e| Error::Other(format!("open_audit_db({}): {e}", path.display())))?;
    enable_defensive(&conn)?;
    load_schema(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::{enable_defensive, load_schema, open_audit_db, SCHEMA_SQL};

    #[test]
    fn schema_sql_is_non_empty_and_contains_triggers() {
        assert!(!SCHEMA_SQL.is_empty());
        assert!(SCHEMA_SQL.contains("CREATE TRIGGER"));
        assert!(SCHEMA_SQL.contains("audit_no_update"));
        assert!(SCHEMA_SQL.contains("audit_no_delete"));
        assert!(SCHEMA_SQL.contains("BEFORE UPDATE"));
        assert!(SCHEMA_SQL.contains("BEFORE DELETE"));
    }

    #[test]
    fn load_schema_on_in_memory_db_succeeds() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        load_schema(&conn).unwrap();
    }

    #[test]
    fn load_schema_is_idempotent() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        load_schema(&conn).unwrap();
        load_schema(&conn).unwrap();
    }

    #[test]
    fn enable_defensive_succeeds_on_in_memory_db() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        enable_defensive(&conn).unwrap();
    }

    #[test]
    fn open_audit_db_creates_file_with_schema() {
        let tmp = tempfile_path();
        let conn = open_audit_db(&tmp).unwrap();
        // Sanity: trigger list contains our two triggers.
        let triggers: Vec<String> = {
            let mut stmt = conn
                .prepare(
                    "SELECT name FROM sqlite_master WHERE type='trigger' \
                     AND tbl_name='audit_events' ORDER BY name",
                )
                .unwrap();
            stmt.query_map([], |r| r.get::<_, String>(0))
                .unwrap()
                .map(std::result::Result::unwrap)
                .collect()
        };
        assert_eq!(triggers, vec!["audit_no_delete", "audit_no_update"]);
        drop(conn);
        let _ = std::fs::remove_file(&tmp);
    }

    fn tempfile_path() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        p.push(format!("reposix-audit-test-{nanos}.db"));
        p
    }
}
