//! `cache.db` — audit + meta + `oid_map` storage for the reposix-cache crate.
//!
//! Opens `<cache-dir>/cache.db` with:
//! - File created at mode `0o600` (mitigates T-31-02-01 local-user read).
//! - `SQLite` DEFENSIVE flag to block `writable_schema` attacks (H-02).
//! - WAL journal mode for concurrent-reader friendliness.
//! - Schema loaded from `fixtures/cache_schema.sql`.
//!
//! We deliberately do NOT use EXCLUSIVE locking here (unlike
//! `crates/reposix-cache/src/cli_compat.rs` which preserves the
//! pre-v0.9.0 refresh contract): the cache DB is read from multiple
//! concurrent code paths (Phase 32's `stateless-connect` handler reads
//! `oid_map` while `build_from` writes). Concurrency safety is
//! Phase 33's job.

use std::os::unix::fs::OpenOptionsExt as _;
use std::path::Path;

use rusqlite::Connection;

use crate::error::{Error, Result};

/// Embedded schema DDL used by [`open_cache_db`].
const CACHE_SCHEMA_SQL: &str = include_str!("../fixtures/cache_schema.sql");

/// Open `<cache_dir>/cache.db` with `0o600` permissions, DEFENSIVE flag,
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

#[cfg(test)]
mod tests {
    use super::open_cache_db;
    use tempfile::tempdir;

    #[test]
    fn open_creates_cache_db_file() {
        let tmp = tempdir().unwrap();
        let _conn = open_cache_db(tmp.path()).expect("open");
        assert!(tmp.path().join("cache.db").exists());
    }

    #[test]
    fn open_is_idempotent() {
        let tmp = tempdir().unwrap();
        drop(open_cache_db(tmp.path()).expect("first open"));
        let _ = open_cache_db(tmp.path()).expect("second open");
    }

    #[test]
    fn cache_db_has_expected_tables() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).expect("open");
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(std::result::Result::unwrap)
            .collect();
        // sqlite_sequence is auto-created because audit_events_cache uses AUTOINCREMENT.
        assert!(tables.contains(&"audit_events_cache".to_owned()));
        assert!(tables.contains(&"meta".to_owned()));
        assert!(tables.contains(&"oid_map".to_owned()));
    }

    #[test]
    fn cache_db_has_append_only_triggers() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).expect("open");
        let triggers: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='trigger' \
                 AND tbl_name='audit_events_cache' ORDER BY name",
            )
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(std::result::Result::unwrap)
            .collect();
        assert_eq!(
            triggers,
            vec!["audit_cache_no_delete", "audit_cache_no_update"]
        );
    }
}
