//! Shared `SoT`-write loop — lifted from `handle_export` lines 377-585
//! with a narrow-deps signature `(cache, backend, backend_name, project,
//! rt, proto, parsed)` per P81's
//! [`crate::precheck::precheck_export_against_changed_set`] precedent.
//!
//! Both `handle_export` (single-backend) and
//! [`crate::bus_handler::handle_bus_export`] (bus) call
//! [`apply_writes`] after they've parsed the fast-import stream
//! from stdin. The function returns a [`WriteOutcome`] enum the
//! caller maps to `state.push_failed` + the protocol-error line —
//! `apply_writes` itself NEVER touches the caller's `State`.
//!
//! ## What this function writes (D-01)
//!
//! On `SotOk` outcome:
//! - `audit_events_cache` rows: `helper_push_accepted` (the OP-3
//!   helper-RPC turn).
//! - `audit_events` rows: per-record (one per `execute_action`
//!   success — the backend adapter's responsibility).
//! - `last_fetched_at` cursor: advanced to `now`.
//! - `refs/mirrors/<sot>-head`: advanced to the new `SoT` SHA
//!   (when `refresh_for_mirror_head` returned Some; gated on
//!   `files_touched > 0` to avoid the `build_from` cost on no-op
//!   pushes — same semantic as pre-lift `handle_export`).
//!
//! ## What this function does NOT write (D-01 — caller's job)
//!
//! - `refs/mirrors/<sot>-synced-at`: written by single-backend
//!   caller unconditionally on `SotOk`; written by bus caller
//!   only after `push_mirror` returns `MirrorResult::Ok`.
//! - `audit_events_cache::mirror_sync_written` row: written by
//!   the caller alongside the synced-at write.
//! - `audit_events_cache::token_cost` row: written by the
//!   caller (single-backend with bytes-in + ack-out; bus with
//!   bytes-in + ack-out).
//! - `proto.send_line("ok refs/heads/main")`: caller's job (the
//!   bus caller emits this after the mirror-push branch resolves).
//!
//! ## Reject paths
//!
//! On non-`SotOk` outcomes (`Conflict`, `PlanRejected`,
//! `SotPartialFail`, `PrecheckBackendUnreachable`), `apply_writes`
//! has ALREADY emitted the protocol error line + reject hint
//! diagnostics + the appropriate audit row (e.g.
//! `helper_push_rejected_conflict` for `Conflict`). The caller's
//! only job is `state.push_failed = true; return Ok(())`.

use anyhow::Result;
use chrono::Utc;
use tokio::runtime::Runtime;

use reposix_cache::Cache;
use reposix_core::backend::BackendConnector;

use crate::diff::{plan, PlanError, PlannedAction};
use crate::execute_action;
use crate::fast_import::ParsedExport;
use crate::precheck::{precheck_export_against_changed_set, PrecheckOutcome};
use crate::protocol::Protocol;

/// Result of the shared `SoT`-write loop. Communicated back to the
/// caller (`handle_export` / `bus_handler::handle_bus_export`)
/// who maps it to `state.push_failed` and any post-write actions
/// (synced-at ref + `mirror_sync_written` audit row + token-cost
/// audit row + `ok refs/heads/main` ack OR — for bus — the
/// mirror-push subprocess + branching).
#[derive(Debug)]
pub(crate) enum WriteOutcome {
    /// All `SoT` writes succeeded. `audit_events_cache` got a
    /// `helper_push_accepted` row; `last_fetched_at` advanced;
    /// `refs/mirrors/<sot>-head` advanced (when `sot_sha.is_some()`).
    SotOk {
        /// New `SoT` SHA — input to `refs/mirrors/<sot>-synced-at`
        /// (caller writes) AND `mirror_sync_written` audit row
        /// `oid_hex` (caller writes). `None` when `files_touched
        /// == 0` (no-op push; cache's `refresh_for_mirror_head`
        /// skipped to avoid the `build_from` cost).
        sot_sha: Option<gix::ObjectId>,
        /// Number of records create/update/delete'd. Used by
        /// caller's `log_token_cost` (the `chars_in` is the
        /// fast-import payload size, but `files_touched` is the
        /// audit-row signal).
        #[allow(dead_code)]
        files_touched: u32,
        /// Comma-separated id list (deterministic order).
        #[allow(dead_code)]
        summary: String,
    },
    /// L1 precheck found a conflict. Reject lines + hint already
    /// emitted on stdout/stderr; `helper_push_rejected_conflict`
    /// audit row already written. Caller sets `state.push_failed
    /// = true; return Ok(())`.
    Conflict,
    /// `diff::plan` rejected (bulk-delete cap or invalid blob).
    /// Reject lines already emitted. Caller sets `state.push_failed
    /// = true`.
    PlanRejected,
    /// At least one `execute_action` returned `Err`. Protocol
    /// error `error refs/heads/main some-actions-failed` already
    /// emitted; per-action stderr `error: <e>` already emitted.
    /// Caller sets `state.push_failed = true`. Note: per Pitfall
    /// 3 / D-09, `SoT` may be in a partial state (some `PATCHes`
    /// succeeded, some did not); recovery is next-push reads new
    /// `SoT` via PRECHECK B.
    SotPartialFail,
    /// L1 precheck itself errored (REST unreachable). Protocol
    /// error `error refs/heads/main backend-unreachable` already
    /// emitted. Caller sets `state.push_failed = true`.
    PrecheckBackendUnreachable,
}

/// Apply REST writes to the `SoT`. Lifted from `handle_export` lines
/// 377-585 with the S1 mechanical replacements:
/// - `state.cache.as_ref()`     → `cache`
/// - `state.backend.as_ref()`   → `backend`
/// - `state.backend_name`       → `backend_name`
/// - `state.project`            → `project`
/// - `state.rt`                 → `rt`
/// - `state.push_failed = true; return Ok(())` → `return Ok(WriteOutcome::<variant>)`
///
/// Caller MUST have:
/// - parsed the fast-import stream from stdin into `parsed`,
/// - lazy-opened the cache (best-effort; `cache: None` is
///   acceptable — audit rows + `last_fetched_at` writes drop
///   silently),
/// - emitted the `helper_push_started` audit row.
///
/// On `SotOk`: caller writes `synced-at` ref +
/// `mirror_sync_written` audit row + `log_token_cost` +
/// `ok refs/heads/main`.
/// On reject outcomes: caller sets `state.push_failed = true; return
/// Ok(())`.
///
/// # Errors
/// Returns `Err` only on `proto` write failures (stdout/stderr I/O).
/// All other errors map to a [`WriteOutcome`] reject variant —
/// the function never `?`-bubbles a `SoT` REST error to the caller;
/// it consumes the error and emits the appropriate protocol line.
#[allow(clippy::too_many_lines)] // narrow lift; readability beats split fns here
#[allow(clippy::too_many_arguments)] // narrow-deps shape mirrors precheck.rs
pub(crate) fn apply_writes<R, W>(
    cache: Option<&Cache>,
    backend: &dyn BackendConnector,
    backend_name: &str,
    project: &str,
    rt: &Runtime,
    proto: &mut Protocol<R, W>,
    parsed: &ParsedExport,
) -> Result<WriteOutcome>
where
    R: std::io::Read,
    W: std::io::Write,
{
    // ---- BEGIN LIFTED BODY ----

    // L1 precheck (DVCS-PERF-L1-01..03). Same call pattern as
    // pre-lift handle_export, with `cache`/`backend`/`project`/`rt`
    // already in narrow-deps shape (P81 substrate).
    let (prior, mut conflicts) =
        match precheck_export_against_changed_set(cache, backend, project, rt, parsed) {
            Ok(PrecheckOutcome::Conflicts(c)) => (Vec::new(), c),
            Ok(PrecheckOutcome::Proceed { prior }) => (prior, Vec::new()),
            Err(e) => {
                // Map the precheck error to the existing protocol-error shape.
                // The precheck annotates REST call sites with
                // `.context("backend-unreachable: ...")`, so the rendered
                // message preserves the same diagnostic the prior code
                // emitted via `fail_push`.
                crate::diag(&format!("error: L1 precheck failed: {e:#}"));
                proto.send_line("error refs/heads/main backend-unreachable")?;
                proto.send_blank()?;
                proto.flush()?;
                return Ok(WriteOutcome::PrecheckBackendUnreachable);
            }
        };

    if !conflicts.is_empty() {
        conflicts.sort_by_key(|c| c.0 .0);
        let (first_id, local_v, backend_v, backend_ts) = &conflicts[0];
        crate::diag(&format!(
            "issue {} modified on backend at {} since last fetch (local base version: {}, backend version: {}). Run: git pull --rebase",
            first_id.0, backend_ts, local_v, backend_v,
        ));
        if let Some(c) = cache {
            c.log_helper_push_rejected_conflict(&first_id.0.to_string(), *local_v, *backend_v);

            // Mirror-lag-ref reject hint (DVCS-MIRROR-REFS-03). When
            // refs are populated (post-first-push), name the staleness
            // gap; when absent (first-push case), omit the hint cleanly
            // per RESEARCH.md pitfall 7.
            if let Ok(Some(synced_at)) = c.read_mirror_synced_at(backend_name) {
                let ago = chrono::Utc::now().signed_duration_since(synced_at);
                let mins = ago.num_minutes().max(0);
                crate::diag(&format!(
                    "hint: your origin (GH mirror) was last synced from {sot} at {ts} ({mins} minutes ago); see refs/mirrors/{sot}-synced-at",
                    sot = backend_name,
                    ts = synced_at.to_rfc3339(),
                ));
                crate::diag(&format!(
                    "hint: run `reposix sync` to update local cache from {backend_name} directly, then `git rebase`",
                ));
            }
        }
        proto.send_line("error refs/heads/main fetch first")?;
        proto.send_blank()?;
        proto.flush()?;
        return Ok(WriteOutcome::Conflict);
    }

    let actions = match plan(&prior, parsed) {
        Ok(a) => a,
        Err(PlanError::BulkDeleteRefused {
            count, limit, tag, ..
        }) => {
            crate::diag(&format!(
                "error: refusing to push (would delete {count} issues; cap is {limit}; commit message tag '{tag}' overrides)"
            ));
            proto.send_line("error refs/heads/main bulk-delete")?;
            proto.send_blank()?;
            proto.flush()?;
            return Ok(WriteOutcome::PlanRejected);
        }
        Err(PlanError::InvalidBlob { path, source }) => {
            crate::diag(&format!(
                "error: invalid issue at {path}: {source}; refusing push"
            ));
            proto.send_line(&format!("error refs/heads/main invalid-blob:{path}"))?;
            proto.send_blank()?;
            proto.flush()?;
            return Ok(WriteOutcome::PlanRejected);
        }
    };

    // Capture summary for the audit row before consuming `actions`.
    let mut touched_ids: Vec<u64> = Vec::new();
    for action in &actions {
        match action {
            PlannedAction::Create(issue) => touched_ids.push(issue.id.0),
            PlannedAction::Update { id, .. } | PlannedAction::Delete { id, .. } => {
                touched_ids.push(id.0);
            }
        }
    }
    touched_ids.sort_unstable();
    let summary = touched_ids
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let files_touched = u32::try_from(touched_ids.len()).unwrap_or(u32::MAX);

    // Execute. Order = creates → updates → deletes (per diff::plan).
    let mut any_failure = false;
    for action in actions {
        match execute_action(backend, project, rt, cache, action) {
            Ok(()) => {}
            Err(e) => {
                crate::diag(&format!("error: {e:#}"));
                any_failure = true;
            }
        }
    }
    if any_failure {
        proto.send_line("error refs/heads/main some-actions-failed")?;
        proto.send_blank()?;
        proto.flush()?;
        return Ok(WriteOutcome::SotPartialFail);
    }

    // SoT-success branch — lifted with synced-at / mirror_sync_written
    // / token_cost / ok-line writes REMOVED (deferred to caller per D-01).
    let sot_sha_opt = if let Some(c) = cache {
        c.log_helper_push_accepted(files_touched, &summary);

        // L1 INBOUND-SoT cursor (DVCS-PERF-L1-01). Best-effort —
        // a write failure WARN-logs and does not poison the push
        // ack.
        if let Err(e) = c.write_last_fetched_at(Utc::now()) {
            tracing::warn!("write_last_fetched_at failed: {e:#}");
        }

        // Mirror-head ref (DVCS-MIRROR-REFS-02). Best-effort. P81
        // L1 perf-fix: skip refresh_for_mirror_head when files_touched
        // is 0 (no-op push). Self-healing on next non-trivial push.
        if files_touched > 0 {
            match rt.block_on(c.refresh_for_mirror_head()) {
                Ok(oid) => {
                    if let Err(e) = c.write_mirror_head(backend_name, oid) {
                        tracing::warn!("write_mirror_head failed: {e:#}");
                    }
                    Some(oid)
                }
                Err(e) => {
                    tracing::warn!("mirror-head SHA derivation failed: {e:#}");
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(WriteOutcome::SotOk {
        sot_sha: sot_sha_opt,
        files_touched,
        summary,
    })

    // ---- END LIFTED BODY ----
}
