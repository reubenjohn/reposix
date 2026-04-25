//! Non-append-only tables: `meta` (key/value) and `oid_map`
//! (content-addressable lookup).

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

/// Insert `(oid, issue_id)` into `oid_map`. `INSERT OR REPLACE` so
/// re-syncs are clean overwrites.
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

/// Look up the `issue_id` for a blob OID. Returns `None` if unknown.
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
