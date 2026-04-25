//! ARCH-02: `audit_events_cache` is strictly append-only.
//!
//! `BEFORE UPDATE` / `BEFORE DELETE` triggers (see
//! `fixtures/cache_schema.sql`) abort any modify-past-insert attempt.

use rusqlite::params;
use tempfile::tempdir;

#[test]
fn update_and_delete_on_audit_table_both_fail() {
    let tmp = tempdir().unwrap();
    let conn = reposix_cache::db::open_cache_db(tmp.path()).unwrap();

    // Seed one row via the helper — best-effort, but in a freshly
    // opened DB we expect the INSERT to succeed.
    reposix_cache::audit::log_tree_sync(&conn, "sim", "proj", 3);
    let seeded: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op='tree_sync'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(seeded, 1);

    // UPDATE must fail with trigger message.
    let upd = conn.execute(
        "UPDATE audit_events_cache SET ts = 'tampered' WHERE id = 1",
        [],
    );
    let err = upd.expect_err("UPDATE must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("append-only"),
        "expected trigger abort, got: {msg}"
    );

    // DELETE must fail with trigger message.
    let del = conn.execute("DELETE FROM audit_events_cache WHERE id = 1", []);
    let err = del.expect_err("DELETE must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("append-only"),
        "expected trigger abort, got: {msg}"
    );

    // Row is still there.
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_events_cache", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);

    // Extra sanity: manual INSERT with full columns still works
    // (proves triggers are UPDATE/DELETE-only).
    conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project) VALUES (?1, 'tree_sync', 'sim', 'proj')",
        params!["2026-04-24T12:00:00Z"],
    )
    .expect("direct INSERT still works");
}
