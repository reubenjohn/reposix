//! Append-only audit-log schema fixture (SG-06 + FC-06).
//!
//! This module publishes the DDL Phase 2 loads at simulator startup. The
//! schema itself — specifically, the `BEFORE UPDATE` / `BEFORE DELETE`
//! triggers on `audit_events` — enforces append-only semantics. Runtime
//! Rust code never needs to "remember" not to delete; the DB refuses.
//!
//! T-01-13 (exfiltration via raw body content) is `accept`ed at this layer:
//! the schema defines `request_body TEXT` but Phase 2's insert path is
//! responsible for hashing / redacting sensitive content before insert.

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

#[cfg(test)]
mod tests {
    use super::{load_schema, SCHEMA_SQL};

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
}
