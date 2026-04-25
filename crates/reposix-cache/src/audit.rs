//! Cache-event audit log — append-only INSERTs.
//!
//! Per `31-CONTEXT.md` §Atomicity: audit-row write failures log WARN via
//! [`tracing::warn`] but do NOT poison the caller's flow. Callers use
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

/// Insert `op='tree_sync'` row. Best-effort: on SQL error, WARN and return.
pub fn log_tree_sync(conn: &Connection, backend: &str, project: &str, items: usize) {
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
