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

/// Insert `op='helper_connect'` row — one per `stateless-connect <service>`
/// invocation of the git remote helper. `service` is the git service
/// name (`git-upload-pack`, etc.). Best-effort: SQL errors WARN-log.
pub fn log_helper_connect(conn: &Connection, backend: &str, project: &str, service: &str) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, reason) \
         VALUES (?1, 'helper_connect', ?2, ?3, ?4)",
        params![Utc::now().to_rfc3339(), backend, project, service],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, service,
              "log_helper_connect failed: {e}");
    }
}

/// Insert `op='helper_advertise'` row — one per v2 advertisement sent
/// to git. `bytes` is the byte count of the advertisement stream.
pub fn log_helper_advertise(conn: &Connection, backend: &str, project: &str, bytes: u32) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, bytes) \
         VALUES (?1, 'helper_advertise', ?2, ?3, ?4)",
        params![Utc::now().to_rfc3339(), backend, project, i64::from(bytes)],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, bytes,
              "log_helper_advertise failed: {e}");
    }
}

/// Insert `op='helper_fetch'` row — one per completed protocol-v2 RPC
/// turn (fetch, ls-refs, object-info, etc.) proxied through the helper.
/// `reason` encodes `"<command>:<want_count>/<request_bytes>/<response_bytes>"`
/// as a compact JSON-free payload; callers needing structure should
/// build a separate decoder. Kept flat because the existing
/// `audit_events_cache` schema has no generic `meta` JSON column.
pub fn log_helper_fetch(
    conn: &Connection,
    backend: &str,
    project: &str,
    command: Option<&str>,
    want_count: u32,
    request_bytes: u32,
    response_bytes: u32,
) {
    let reason = format!(
        "{}:wants={};req={};resp={}",
        command.unwrap_or("?"),
        want_count,
        request_bytes,
        response_bytes,
    );
    // `bytes` column records the response-body size (mirrors tree_sync
    // where `bytes` records item count). For full telemetry consume
    // `reason`.
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, reason, bytes) \
         VALUES (?1, 'helper_fetch', ?2, ?3, ?4, ?5)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            reason,
            i64::from(response_bytes),
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project,
              "log_helper_fetch failed: {e}");
    }
}

/// Insert `op='helper_fetch_error'` row — one per non-zero exit from
/// `git upload-pack --stateless-rpc`.
pub fn log_helper_fetch_error(
    conn: &Connection,
    backend: &str,
    project: &str,
    exit_code: i32,
    stderr_tail: &str,
) {
    let reason = format!("exit={exit_code};tail={stderr_tail}");
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, reason) \
         VALUES (?1, 'helper_fetch_error', ?2, ?3, ?4)",
        params![Utc::now().to_rfc3339(), backend, project, reason],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, exit_code,
              "log_helper_fetch_error failed: {e}");
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
