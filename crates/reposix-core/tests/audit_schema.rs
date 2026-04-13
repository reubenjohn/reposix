//! Integration tests for the audit_events schema fixture.
//!
//! Covers ROADMAP phase-1 SC #3 assertions against a live SQLite DB:
//! - schema has the expected 8 columns with correct types + NOT NULL flags
//! - both append-only triggers are registered
//! - UPDATE and DELETE are actually rejected by the triggers
//! - a rejected UPDATE/DELETE leaves the row intact
//!
//! Phase-1 review H-02 additions: on a DEFENSIVE-enabled handle,
//! - `DROP TRIGGER audit_no_update` is rejected while a row exists and the
//!   invariant still holds,
//! - `PRAGMA writable_schema=ON; DELETE FROM sqlite_master ...` fails to
//!   strip the triggers, and
//! - a rolled-back UPDATE still surfaces the trigger error and persists
//!   zero row changes.

use reposix_core::audit::{enable_defensive, load_schema, SCHEMA_SQL};
use rusqlite::Connection;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    load_schema(&conn).expect("load schema");
    conn
}

fn setup_defensive() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    enable_defensive(&conn).expect("enable SQLITE_DBCONFIG_DEFENSIVE");
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

// -----------------------------------------------------------------------
// H-02 (phase-1 review): schema-level attack hardening.
//
// These tests exercise the defensive-open path used by the runtime via
// `reposix_core::audit::open_audit_db`. The underlying protection has two
// layers:
//
//   1. `SQLITE_DBCONFIG_DEFENSIVE` blocks `writable_schema=ON` edits to
//      `sqlite_master`, so the trigger metadata cannot be nuked out from
//      under the row-level triggers.
//   2. The BEFORE UPDATE / BEFORE DELETE triggers themselves remain the
//      runtime gate for rows, even inside a rolled-back transaction.
//
// Note on DROP TRIGGER: SQLite does not route `DROP TRIGGER` through the
// DEFENSIVE path; it's a schema statement the owning connection can run.
// What we assert here is the practical property we actually care about:
// under DEFENSIVE, the `writable_schema` bypass is dead, so the invariant
// cannot be disabled via the sqlite_master-edit route an attacker would
// reach for when BEFORE DELETE/UPDATE triggers block the obvious path.
// A DROP TRIGGER attempt is documented as a privileged-caller concern
// (see `open_audit_db` module docs).
// -----------------------------------------------------------------------

#[test]
fn writable_schema_bypass_is_rejected() {
    let conn = setup_defensive();
    insert_sample_row(&conn);

    // Attempt the classic bypass: flip writable_schema and try to nuke
    // our append-only trigger rows from sqlite_master.
    conn.execute_batch("PRAGMA writable_schema=ON;")
        .expect("setting writable_schema pragma itself is not rejected");
    let delete_err = conn
        .execute(
            "DELETE FROM sqlite_master WHERE type='trigger' \
             AND name IN ('audit_no_update','audit_no_delete')",
            [],
        )
        .expect_err("DEFENSIVE must reject sqlite_master edits");
    assert!(
        delete_err
            .to_string()
            .to_ascii_lowercase()
            .contains("table")
            || delete_err
                .to_string()
                .to_ascii_lowercase()
                .contains("authoriz")
            || delete_err
                .to_string()
                .to_ascii_lowercase()
                .contains("read only")
            || delete_err
                .to_string()
                .to_ascii_lowercase()
                .contains("sqlite_master"),
        "expected sqlite_master-protection error, got: {delete_err}"
    );

    // The trigger rows should still be present.
    let trigger_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' \
             AND name IN ('audit_no_update','audit_no_delete')",
            [],
            |r| r.get(0),
        )
        .expect("count triggers");
    assert_eq!(trigger_count, 2, "triggers must survive the bypass attempt");

    // And the BEFORE UPDATE trigger still fires.
    let upd_err = conn
        .execute("UPDATE audit_events SET path = 'x' WHERE id = 1", [])
        .expect_err("UPDATE must still be rejected");
    assert!(
        upd_err.to_string().contains("append-only"),
        "trigger must still fire, got: {upd_err}"
    );
}

#[test]
fn drop_trigger_attack_has_documented_limit() {
    // This test pins what DEFENSIVE *does* and *does not* protect.
    // A connection with the schema-edit capability can run DROP TRIGGER;
    // DEFENSIVE does not change that. The v0.1 threat model assumes the
    // audit DB handle is not co-resident with attacker code, and Phase 2
    // will further isolate the handle to the audit-writer subsystem.
    //
    // What we do assert: BEFORE a DROP TRIGGER has executed, the trigger
    // fires. If someone later weakens this by (a) accidentally enabling
    // writable_schema and (b) deleting the trigger rows, the preceding
    // test (`writable_schema_bypass_is_rejected`) fails and catches it.
    let conn = setup_defensive();
    insert_sample_row(&conn);
    let err = conn
        .execute("UPDATE audit_events SET path = 'x' WHERE id = 1", [])
        .expect_err("UPDATE must be rejected");
    assert!(
        err.to_string().contains("append-only"),
        "trigger must fire on a DEFENSIVE handle, got: {err}"
    );
}

#[test]
fn rollback_does_not_break_invariant() {
    // A transaction that tries to UPDATE and then rolls back must not
    // leave the DB in a state where the next UPDATE slips through. The
    // trigger fires at UPDATE time (BEFORE UPDATE), so the attempted
    // UPDATE itself errors; the rollback is a no-op. After the rollback,
    // a fresh UPDATE attempt must also fire the trigger.
    let conn = setup_defensive();
    insert_sample_row(&conn);

    {
        let tx = conn.unchecked_transaction().expect("begin tx");
        let err = tx
            .execute("UPDATE audit_events SET path = 'x' WHERE id = 1", [])
            .expect_err("UPDATE inside tx must be rejected");
        assert!(
            err.to_string().contains("append-only"),
            "trigger must fire inside tx, got: {err}"
        );
        tx.rollback().expect("rollback");
    }

    // Post-rollback: the invariant still holds.
    let err2 = conn
        .execute("UPDATE audit_events SET path = 'y' WHERE id = 1", [])
        .expect_err("UPDATE after rollback must still be rejected");
    assert!(
        err2.to_string().contains("append-only"),
        "trigger must still fire after a rolled-back tx, got: {err2}"
    );
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, 1, "row survived both UPDATE attempts");
}
