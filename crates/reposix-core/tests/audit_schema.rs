//! Integration tests for the audit_events schema fixture.
//!
//! Covers ROADMAP phase-1 SC #3 assertions against a live SQLite DB:
//! - schema has the expected 8 columns with correct types + NOT NULL flags
//! - both append-only triggers are registered
//! - UPDATE and DELETE are actually rejected by the triggers
//! - a rejected UPDATE/DELETE leaves the row intact

use reposix_core::audit::{load_schema, SCHEMA_SQL};
use rusqlite::Connection;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    load_schema(&conn).expect("load schema");
    conn
}

fn insert_sample_row(conn: &Connection) {
    conn.execute(
        "INSERT INTO audit_events (ts, method, path) VALUES (?1, ?2, ?3)",
        rusqlite::params!["2026-04-13T00:00:00Z", "GET", "/"],
    )
    .expect("insert must succeed on fresh schema");
}

#[test]
fn audit_schema_has_expected_columns() {
    let conn = setup();
    // SELECT name, type, "notnull" from pragma_table_info to verify columns.
    let mut stmt = conn
        .prepare(
            "SELECT name, type, \"notnull\" FROM pragma_table_info('audit_events') ORDER BY cid",
        )
        .expect("prepare pragma_table_info");
    let rows: Vec<(String, String, i64)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .expect("query")
        .map(std::result::Result::unwrap)
        .collect();
    let expected: &[(&str, &str, i64)] = &[
        ("id", "INTEGER", 0),
        ("ts", "TEXT", 1),
        ("agent_id", "TEXT", 0),
        ("method", "TEXT", 1),
        ("path", "TEXT", 1),
        ("status", "INTEGER", 0),
        ("request_body", "TEXT", 0),
        ("response_summary", "TEXT", 0),
    ];
    assert_eq!(rows.len(), expected.len(), "got rows: {rows:?}");
    for ((got_name, got_type, got_notnull), (exp_name, exp_type, exp_notnull)) in
        rows.iter().zip(expected.iter())
    {
        assert_eq!(got_name, exp_name, "column name mismatch");
        assert_eq!(
            got_type.to_ascii_uppercase(),
            *exp_type,
            "type mismatch for {got_name}"
        );
        assert_eq!(got_notnull, exp_notnull, "notnull mismatch for {got_name}");
    }
}

#[test]
fn audit_schema_lists_both_triggers() {
    let conn = setup();
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = 'trigger' \
             AND tbl_name = 'audit_events' ORDER BY name",
        )
        .expect("prepare trigger list");
    let triggers: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .expect("query")
        .map(std::result::Result::unwrap)
        .collect();
    assert_eq!(
        triggers,
        vec!["audit_no_delete".to_string(), "audit_no_update".to_string()],
        "expected both triggers"
    );
}

#[test]
fn audit_update_is_rejected_by_trigger() {
    let conn = setup();
    insert_sample_row(&conn);
    let err = conn
        .execute("UPDATE audit_events SET path = 'x' WHERE id = 1", [])
        .expect_err("UPDATE must be rejected");
    assert!(
        err.to_string().contains("append-only"),
        "error must surface trigger message, got: {err}"
    );
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, 1, "row survived the rejected UPDATE");
}

#[test]
fn audit_delete_is_rejected_by_trigger() {
    let conn = setup();
    insert_sample_row(&conn);
    let err = conn
        .execute("DELETE FROM audit_events WHERE id = 1", [])
        .expect_err("DELETE must be rejected");
    assert!(
        err.to_string().contains("append-only"),
        "error must surface trigger message, got: {err}"
    );
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, 1, "row survived the rejected DELETE");
}

#[test]
fn schema_sql_is_stable_bytes() {
    // Canary: if someone edits the fixture, this forces a diff review.
    // We don't pin the exact hash — keeping the fixture human-readable is
    // the point — but we assert bounds so an accidental truncation fails CI.
    assert!(SCHEMA_SQL.len() > 200, "schema suspiciously short");
    assert!(SCHEMA_SQL.len() < 2000, "schema suspiciously long");
}
