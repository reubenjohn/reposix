//! `SQLite` storage for the simulator — issues table + audit schema load.
//!
//! The simulator uses a single connection (see [`crate::state::AppState`]).
//! Persistent stores open in WAL mode with a 5 s busy timeout; `:memory:`
//! skips WAL since it is meaningless there. The append-only `audit_events`
//! schema is loaded via [`reposix_core::audit::load_schema`], which installs
//! the `BEFORE UPDATE` / `BEFORE DELETE` triggers that enforce SG-06.

use std::path::Path;

use reposix_core::audit;
use rusqlite::Connection;

use crate::error::ApiError;

/// DDL for the `issues` table. Composite primary key `(project, id)` so every
/// project has its own id namespace starting at 1. `labels` is stored as a
/// JSON array string; parsed via `serde_json::from_str::<Vec<String>>` in the
/// handlers.
pub const ISSUES_SQL: &str = "\
CREATE TABLE IF NOT EXISTS issues (
    project    TEXT    NOT NULL,
    id         INTEGER NOT NULL,
    title      TEXT    NOT NULL,
    status     TEXT    NOT NULL,
    assignee   TEXT,
    labels     TEXT    NOT NULL DEFAULT '[]',
    created_at TEXT    NOT NULL,
    updated_at TEXT    NOT NULL,
    version    INTEGER NOT NULL DEFAULT 1,
    body       TEXT    NOT NULL DEFAULT '',
    PRIMARY KEY(project, id)
);
";

/// Open (or create) the simulator `SQLite` database at `path`.
///
/// - If `ephemeral` is true OR `path == :memory:`, opens an in-memory
///   connection and skips `PRAGMA journal_mode=WAL`.
/// - Otherwise opens the file, enables WAL mode, sets
///   `PRAGMA synchronous=NORMAL`, and sets `busy_timeout=5000`.
/// - Always creates the `issues` table (idempotent) and loads the audit
///   schema via [`reposix_core::audit::load_schema`].
///
/// # Errors
/// Returns [`ApiError::Db`] if the file cannot be opened or any DDL fails.
/// Returns [`ApiError::Internal`] wrapping the core error from
/// [`audit::load_schema`] if the audit DDL fails.
pub fn open_db(path: &Path, ephemeral: bool) -> Result<Connection, ApiError> {
    let conn = if ephemeral || path == Path::new(":memory:") {
        Connection::open_in_memory()?
    } else {
        Connection::open(path)?
    };

    // WAL + durability pragmas only for file-backed DBs. `:memory:` rejects
    // WAL with a cryptic error — skip silently and carry on.
    let is_memory = ephemeral || path == Path::new(":memory:");
    if !is_memory {
        // pragma_update returns Err if the pragma is unrecognised; fine —
        // propagate, since a file-backed DB that refuses WAL is a sign of
        // serious trouble.
        let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "busy_timeout", 5000_i64)?;
    }

    conn.execute_batch(ISSUES_SQL)?;
    audit::load_schema(&conn).map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::{open_db, Path};

    #[test]
    fn open_db_in_memory_succeeds() {
        let conn = open_db(Path::new(":memory:"), true).expect("open");
        // issues table exists.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='issues'",
                [],
                |r| r.get(0),
            )
            .expect("issues table present");
        assert_eq!(count, 1);
    }

    #[test]
    fn open_db_is_idempotent_on_repeated_schema_load() {
        // open_db runs execute_batch for issues + load_schema for audit; the
        // fixture wraps DROP+CREATE in a transaction, so calling twice is a
        // no-op.
        let conn = open_db(Path::new(":memory:"), true).expect("first");
        // Run the audit load manually again to confirm idempotency.
        reposix_core::audit::load_schema(&conn).expect("second load_schema");
        // Issues table ddl is IF NOT EXISTS so re-running is fine.
        conn.execute_batch(super::ISSUES_SQL).expect("re-run ddl");
    }

    #[test]
    fn open_db_installs_audit_triggers() {
        let conn = open_db(Path::new(":memory:"), true).expect("open");
        let triggers: Vec<String> = {
            let mut stmt = conn
                .prepare(
                    "SELECT name FROM sqlite_master WHERE type='trigger' \
                     AND tbl_name='audit_events' ORDER BY name",
                )
                .expect("prepare");
            stmt.query_map([], |r| r.get::<_, String>(0))
                .expect("query_map")
                .map(std::result::Result::unwrap)
                .collect()
        };
        assert_eq!(triggers, vec!["audit_no_delete", "audit_no_update"]);
    }
}
