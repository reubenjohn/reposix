//! v0.11.0 §3c — token-cost audit op acceptance + JSON-in-reason round-trip.
//!
//! The token-cost ledger writes `op='token_cost'` rows with the four
//! token fields packed into the `reason` column as a tiny JSON blob.
//! These tests prove the schema CHECK accepts the op and that the
//! payload round-trips.

#![allow(clippy::missing_panics_doc)]

use reposix_cache::audit::log_token_cost;
use reposix_cache::db::open_cache_db;

#[test]
fn token_cost_op_accepted_by_check_constraint() {
    let tmp = tempfile::tempdir().unwrap();
    let conn = open_cache_db(tmp.path()).unwrap();
    log_token_cost(&conn, "sim", "demo", 1234, 5678, "fetch");
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'token_cost'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 1, "token_cost row inserted");
}

#[test]
fn token_cost_reason_payload_round_trips() {
    let tmp = tempfile::tempdir().unwrap();
    let conn = open_cache_db(tmp.path()).unwrap();
    log_token_cost(&conn, "sim", "demo", 99, 18, "push");
    let reason: String = conn
        .query_row(
            "SELECT reason FROM audit_events_cache WHERE op = 'token_cost'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(reason.contains(r#""in":99"#), "in field encoded: {reason}");
    assert!(
        reason.contains(r#""out":18"#),
        "out field encoded: {reason}"
    );
    assert!(
        reason.contains(r#""kind":"push""#),
        "kind field encoded: {reason}"
    );
}

#[test]
fn token_cost_bytes_column_records_total() {
    let tmp = tempfile::tempdir().unwrap();
    let conn = open_cache_db(tmp.path()).unwrap();
    log_token_cost(&conn, "sim", "demo", 100, 200, "fetch");
    let bytes: i64 = conn
        .query_row(
            "SELECT bytes FROM audit_events_cache WHERE op = 'token_cost'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        bytes, 300,
        "bytes column = chars_in + chars_out for quick aggregation"
    );
}

#[test]
fn token_cost_multiple_rows_sum_correctly() {
    // Mimics the reposix tokens aggregation: write three rows, sum them.
    let tmp = tempfile::tempdir().unwrap();
    let conn = open_cache_db(tmp.path()).unwrap();
    log_token_cost(&conn, "sim", "demo", 100, 200, "fetch");
    log_token_cost(&conn, "sim", "demo", 50, 50, "push");
    log_token_cost(&conn, "sim", "demo", 1000, 4000, "fetch");
    let (n, total): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(bytes), 0) FROM audit_events_cache WHERE op='token_cost'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(n, 3);
    // 100+200 + 50+50 + 1000+4000 = 5400
    assert_eq!(total, 5400);
}
