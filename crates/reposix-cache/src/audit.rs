//! Cache-event audit log — append-only INSERTs.
//!
//! Per `31-CONTEXT.md` §Atomicity: audit-row write failures log WARN via
//! [`tracing::warn`] but do NOT poison the caller's flow. Callers use
//! `let _ = log_materialize(...)` to explicitly discard the result.
//!
//! # Dual-schema audit design (POLISH2-22, friction row 12)
//!
//! This crate's `audit_events_cache` table is one of **two** audit
//! schemas reposix ships — see [`reposix_core::audit`] for the
//! backend-side `audit_events` table. The split is intentional, and
//! both schemas are append-only at the SQL level (`BEFORE UPDATE` /
//! `BEFORE DELETE` triggers raise rather than allow row mutations).
//!
//! - `audit_events_cache` (this crate) captures **cache-internal
//!   operations**: blob materialization, gc eviction, helper RPC turns
//!   (`stateless-connect` fetch / push), integrity events, sync-tag
//!   writes.
//! - `audit_events` (in `reposix-core`) captures **backend-side
//!   mutating writes**: every `create_record` / `update_record` /
//!   `delete_or_close` issued by the sim/confluence/jira adapters.
//!
//! A complete forensic query (e.g., "what did we write to JIRA in the
//! last 24h?") reads both tables — the helper-level
//! `helper_push_accepted` row lives here; the per-issue
//! `update_record` rows live in `audit_events`. The split keeps this
//! crate strictly cache-side and lets backend adapters fail closed
//! without coupling to the cache's `SQLite` connection lifecycle.
//! Physical unification is deferred to v0.12.0+ (CC-3).

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

/// Insert `op='blob_limit_exceeded'` row — one per `command=fetch` request
/// that would have wanted more blobs than `REPOSIX_BLOB_LIMIT` allows.
/// `bytes` records the would-be want count; `reason` records `limit=<M>`.
/// Best-effort: SQL errors WARN-log (the helper has already written the
/// agent-facing stderr message and is about to exit non-zero).
pub fn log_blob_limit_exceeded(
    conn: &Connection,
    backend: &str,
    project: &str,
    want_count: u32,
    limit: u32,
) {
    let reason = format!("limit={limit}");
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, bytes, reason) \
         VALUES (?1, 'blob_limit_exceeded', ?2, ?3, ?4, ?5)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            i64::from(want_count),
            reason,
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, want_count, limit,
              "log_blob_limit_exceeded failed: {e}");
    }
}

/// Insert `op='helper_push_started'` row — one per `handle_export`
/// invocation. `ref_name` is the git ref being pushed (e.g.
/// `refs/heads/main`). Best-effort.
pub fn log_helper_push_started(conn: &Connection, backend: &str, project: &str, ref_name: &str) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, reason) \
         VALUES (?1, 'helper_push_started', ?2, ?3, ?4)",
        params![Utc::now().to_rfc3339(), backend, project, ref_name],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, ref_name,
              "log_helper_push_started failed: {e}");
    }
}

/// Insert `op='helper_push_accepted'` row — one per successful push.
/// `files_touched` is the count of changed paths (creates+updates+deletes).
/// `summary` is a comma-separated id list (deterministic order).
/// Best-effort.
pub fn log_helper_push_accepted(
    conn: &Connection,
    backend: &str,
    project: &str,
    files_touched: u32,
    summary: &str,
) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, bytes, reason) \
         VALUES (?1, 'helper_push_accepted', ?2, ?3, ?4, ?5)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            i64::from(files_touched),
            summary,
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, files_touched,
              "log_helper_push_accepted failed: {e}");
    }
}

/// Insert `op='helper_push_rejected_conflict'` row — one per push refused
/// because at least one issue's local base version differs from the
/// backend's current version. `issue_id` is the first id that triggered
/// the reject (deterministic — smallest id wins). `reason` records
/// `local=<v>;backend=<v>`. Best-effort.
pub fn log_helper_push_rejected_conflict(
    conn: &Connection,
    backend: &str,
    project: &str,
    issue_id: &str,
    local_version: u64,
    backend_version: u64,
) {
    let reason = format!("local={local_version};backend={backend_version}");
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, issue_id, reason) \
         VALUES (?1, 'helper_push_rejected_conflict', ?2, ?3, ?4, ?5)",
        params![Utc::now().to_rfc3339(), backend, project, issue_id, reason],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, issue_id,
              "log_helper_push_rejected_conflict failed: {e}");
    }
}

/// Insert `op='helper_push_sanitized_field'` row — one per Update action
/// where `sanitize()` would have overwritten a server-controlled field
/// (`id`/`created_at`/`updated_at`/`version`). Best-effort signal for
/// post-hoc introspection. `field` names the overwritten frontmatter
/// field.
pub fn log_helper_push_sanitized_field(
    conn: &Connection,
    backend: &str,
    project: &str,
    issue_id: &str,
    field: &str,
) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, issue_id, reason) \
         VALUES (?1, 'helper_push_sanitized_field', ?2, ?3, ?4, ?5)",
        params![Utc::now().to_rfc3339(), backend, project, issue_id, field],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, issue_id, field,
              "log_helper_push_sanitized_field failed: {e}");
    }
}

/// Insert `op='helper_backend_instantiated'` row — one per
/// `git-remote-reposix` invocation, written from the URL-scheme
/// dispatcher when the helper resolves the remote URL to a concrete
/// backend (sim/github/confluence/jira). `reason` carries the live
/// backend project string so forensics can reconstruct
/// `(backend, project)` even when the cache uses a sanitized
/// directory name (e.g. GitHub `owner/repo` → `owner-repo`).
/// Best-effort: SQL errors WARN-log.
pub fn log_helper_backend_instantiated(
    conn: &Connection,
    backend: &str,
    project: &str,
    project_for_backend: &str,
) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, reason) \
         VALUES (?1, 'helper_backend_instantiated', ?2, ?3, ?4)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            project_for_backend,
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project,
              "log_helper_backend_instantiated failed: {e}");
    }
}

/// Insert `op='sync_tag_written'` row — one per `Cache::tag_sync` call.
/// `ref_name` is the full ref written (`refs/reposix/sync/<ISO8601-no-colons>`);
/// `commit_oid` is the synthesis-commit OID the tag points at.
/// Best-effort: SQL errors WARN-log.
pub fn log_sync_tag_written(
    conn: &Connection,
    backend: &str,
    project: &str,
    ref_name: &str,
    commit_oid: &str,
) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, oid, reason) \
         VALUES (?1, 'sync_tag_written', ?2, ?3, ?4, ?5)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            commit_oid,
            ref_name,
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, ref_name, oid = commit_oid,
              "log_sync_tag_written failed: {e}");
    }
}

/// Insert `op='cache_gc'` row — one per blob evicted by [`crate::Cache::gc`].
///
/// `oid` is the evicted blob's hex OID, `bytes` records the bytes
/// reclaimed (size of the loose-object file on disk), and `reason`
/// encodes the strategy + selection rule:
///   - `evicted:strategy=lru` — least-recently-accessed blob
///   - `evicted:strategy=ttl;age_days=N` — TTL strategy
///   - `evicted:strategy=all` — wholesale eviction
///   - `dry_run:strategy=...` — `--dry-run` flag set, nothing on disk changed
///
/// Best-effort: SQL errors WARN-log. Per CLAUDE.md threat model (v0.11.0 §3j)
/// gc only ever evicts loose blob objects; tree/commit objects, refs, and
/// sync tags are never touched.
pub fn log_cache_gc(
    conn: &Connection,
    backend: &str,
    project: &str,
    oid_hex: &str,
    bytes_reclaimed: u64,
    reason: &str,
) {
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, oid, bytes, reason) \
         VALUES (?1, 'cache_gc', ?2, ?3, ?4, ?5, ?6)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            oid_hex,
            i64::try_from(bytes_reclaimed).unwrap_or(i64::MAX),
            reason,
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, oid = oid_hex, bytes_reclaimed, reason,
              "log_cache_gc failed: {e}");
    }
}

/// Insert `op='token_cost'` row — one per helper RPC turn (`fetch` or
/// `push`). The `bytes` column carries the total chars (in+out) for
/// quick aggregation; the `reason` column carries a small JSON blob
/// `{"in":N,"out":M,"kind":"fetch|push"}` for full granular replay.
///
/// The token estimate is `chars / 4` — a conservative English-text
/// heuristic. See `.planning/research/v0.11.0-vision-and-innovations.md` §3c.
///
/// Best-effort: SQL errors WARN-log.
pub fn log_token_cost(
    conn: &Connection,
    backend: &str,
    project: &str,
    chars_in: u64,
    chars_out: u64,
    kind: &str,
) {
    // JSON-in-reason avoids adding a new column to the schema. The four
    // numeric fields callers care about (chars_in, chars_out, est_in,
    // est_out) all derive from `chars_in` and `chars_out` via `/ 4`, so
    // we only persist the inputs and let the consumer derive estimates.
    let reason = format!(r#"{{"in":{chars_in},"out":{chars_out},"kind":"{kind}"}}"#);
    let total = chars_in.saturating_add(chars_out);
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, bytes, reason) \
         VALUES (?1, 'token_cost', ?2, ?3, ?4, ?5)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            i64::try_from(total).unwrap_or(i64::MAX),
            reason,
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, chars_in, chars_out, kind,
              "log_token_cost failed: {e}");
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

/// Insert `op='delta_sync'` row INSIDE a `SQLite` transaction. Used by
/// [`crate::Cache::sync`] so the audit row commits atomically with the
/// `meta.last_fetched_at` update and the changed-issue `oid_map` rows.
///
/// Unlike the other audit helpers in this module, this one returns
/// `rusqlite::Result<()>` (not best-effort): a failed audit insert
/// MUST roll the whole transaction back, otherwise we'd risk a torn
/// state where the cursor advanced but no audit row was written.
///
/// `since_iso` is the RFC3339 string of the `last_fetched_at` that was
/// passed to the backend (or `None` for a seed-equivalent sync — but
/// the seed path uses `tree_sync` instead, so in practice this is
/// always `Some` from the `Cache::sync` caller).
///
/// `items_returned` is the count of IDs the backend declared changed —
/// stored in the `bytes` column to mirror the `tree_sync` convention
/// (`bytes` records item count, not literal byte length).
///
/// # Errors
/// Returns the underlying `rusqlite::Error` from `Transaction::execute`.
pub fn log_delta_sync_tx(
    tx: &rusqlite::Transaction<'_>,
    backend: &str,
    project: &str,
    since_iso: Option<&str>,
    items_returned: usize,
) -> rusqlite::Result<()> {
    let reason = match since_iso {
        Some(s) => format!("since={s}"),
        None => "since=NULL".to_owned(),
    };
    tx.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, bytes, reason) \
         VALUES (?1, 'delta_sync', ?2, ?3, ?4, ?5)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            i64::try_from(items_returned).unwrap_or(i64::MAX),
            reason,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_cache_db;
    use tempfile::tempdir;

    #[test]
    fn log_delta_sync_tx_inserts_row() {
        let tmp = tempdir().unwrap();
        let mut conn = open_cache_db(tmp.path()).unwrap();
        let tx = conn.transaction().unwrap();
        log_delta_sync_tx(&tx, "sim", "demo", Some("2026-04-24T00:00:00Z"), 3).unwrap();
        tx.commit().unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'delta_sync'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
        let (bytes, reason): (i64, String) = conn
            .query_row(
                "SELECT bytes, reason FROM audit_events_cache WHERE op = 'delta_sync'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(bytes, 3);
        assert_eq!(reason, "since=2026-04-24T00:00:00Z");
    }

    #[test]
    fn log_delta_sync_tx_roll_back_does_not_leak_row() {
        // Atomicity proof at the unit level: dropping the tx without
        // commit must roll back the audit insert.
        let tmp = tempdir().unwrap();
        let mut conn = open_cache_db(tmp.path()).unwrap();
        let tx = conn.transaction().unwrap();
        log_delta_sync_tx(&tx, "sim", "demo", Some("2026-04-24T00:00:00Z"), 1).unwrap();
        // Intentionally drop without commit.
        drop(tx);
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'delta_sync'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 0, "rollback must not leave the delta_sync row");
    }

    #[test]
    fn log_blob_limit_exceeded_inserts_row() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        log_blob_limit_exceeded(&conn, "sim", "demo", 250, 200);
        let (op, bytes, reason): (String, i64, String) = conn
            .query_row(
                "SELECT op, bytes, reason FROM audit_events_cache WHERE op = 'blob_limit_exceeded'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(op, "blob_limit_exceeded");
        assert_eq!(bytes, 250);
        assert_eq!(reason, "limit=200");
    }

    #[test]
    fn log_helper_push_started_inserts_row() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        log_helper_push_started(&conn, "sim", "demo", "refs/heads/main");
        let (op, reason): (String, String) = conn
            .query_row(
                "SELECT op, reason FROM audit_events_cache WHERE op = 'helper_push_started'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(op, "helper_push_started");
        assert_eq!(reason, "refs/heads/main");
    }

    #[test]
    fn log_helper_push_accepted_records_summary() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        log_helper_push_accepted(&conn, "sim", "demo", 3, "1,2,5");
        let (bytes, reason): (i64, String) = conn
            .query_row(
                "SELECT bytes, reason FROM audit_events_cache WHERE op = 'helper_push_accepted'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(bytes, 3);
        assert_eq!(reason, "1,2,5");
    }

    #[test]
    fn log_helper_push_rejected_conflict_records_versions() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        log_helper_push_rejected_conflict(&conn, "sim", "demo", "42", 1, 2);
        let (issue_id, reason): (String, String) = conn
            .query_row(
                "SELECT issue_id, reason FROM audit_events_cache WHERE op = 'helper_push_rejected_conflict'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(issue_id, "42");
        assert_eq!(reason, "local=1;backend=2");
    }

    #[test]
    fn log_helper_push_sanitized_field_records_field_name() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        log_helper_push_sanitized_field(&conn, "sim", "demo", "42", "version");
        let (issue_id, reason): (String, String) = conn
            .query_row(
                "SELECT issue_id, reason FROM audit_events_cache WHERE op = 'helper_push_sanitized_field'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(issue_id, "42");
        assert_eq!(reason, "version");
    }

    #[test]
    fn log_delta_sync_tx_handles_null_since() {
        let tmp = tempdir().unwrap();
        let mut conn = open_cache_db(tmp.path()).unwrap();
        let tx = conn.transaction().unwrap();
        log_delta_sync_tx(&tx, "sim", "demo", None, 0).unwrap();
        tx.commit().unwrap();
        let reason: String = conn
            .query_row(
                "SELECT reason FROM audit_events_cache WHERE op = 'delta_sync'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(reason, "since=NULL");
    }

    #[test]
    fn log_helper_connect_inserts_row() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        log_helper_connect(&conn, "sim", "demo", "git-upload-pack");
        let (op, reason): (String, String) = conn
            .query_row(
                "SELECT op, reason FROM audit_events_cache WHERE op = 'helper_connect'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(op, "helper_connect");
        assert_eq!(reason, "git-upload-pack");
    }

    #[test]
    fn log_helper_advertise_records_bytes() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        log_helper_advertise(&conn, "sim", "demo", 512);
        let (op, bytes): (String, i64) = conn
            .query_row(
                "SELECT op, bytes FROM audit_events_cache WHERE op = 'helper_advertise'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(op, "helper_advertise");
        assert_eq!(bytes, 512);
    }

    #[test]
    fn log_helper_fetch_encodes_reason_and_bytes() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        log_helper_fetch(&conn, "sim", "demo", Some("fetch"), 2, 128, 4096);
        let (op, reason, bytes): (String, String, i64) = conn
            .query_row(
                "SELECT op, reason, bytes FROM audit_events_cache WHERE op = 'helper_fetch'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(op, "helper_fetch");
        assert_eq!(reason, "fetch:wants=2;req=128;resp=4096");
        assert_eq!(bytes, 4096);
    }

    #[test]
    fn log_helper_fetch_error_records_exit_and_tail() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        log_helper_fetch_error(&conn, "sim", "demo", 128, "fatal: not a git repository");
        let (op, reason): (String, String) = conn
            .query_row(
                "SELECT op, reason FROM audit_events_cache WHERE op = 'helper_fetch_error'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(op, "helper_fetch_error");
        assert_eq!(reason, "exit=128;tail=fatal: not a git repository");
    }

    #[test]
    fn log_helper_backend_instantiated_records_project_for_backend() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        log_helper_backend_instantiated(&conn, "sim", "demo", "demo");
        let (op, reason): (String, String) = conn
            .query_row(
                "SELECT op, reason FROM audit_events_cache WHERE op = 'helper_backend_instantiated'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(op, "helper_backend_instantiated");
        assert_eq!(reason, "demo");
    }
}
