← [back to index](./index.md)

## Task 83-01-T02 — Lift `handle_export` write loop into `write_loop::apply_writes`

<read_first>
- `crates/reposix-remote/src/main.rs` lines 343-606 (full
  `handle_export` body — the source of the lift). Pay particular
  attention to: lines 360-375 (parse_export_stream), lines 377-411
  (precheck), lines 413-456 (conflicts branch), lines 458-482
  (plan), lines 484-500 (touched_ids/summary), lines 502-512
  (execute_action loop), lines 513-604 (success branch — audit
  rows, last_fetched_at, mirror refs, log_token_cost, ack).
- `crates/reposix-remote/src/precheck.rs` lines 86-311
  (`precheck_export_against_changed_set` body) — the narrow-deps
  shape the lift mirrors.
- `crates/reposix-remote/src/diff.rs` (`plan` + `PlannedAction` +
  `PlanError` exports — confirm signatures).
- `crates/reposix-remote/src/main.rs` lines 619-700+
  (`execute_action` body — confirm it takes `&mut State`; the lift
  must construct an `&mut State`-equivalent or pass through the
  same shape).
- `crates/reposix-remote/src/main.rs` lines 24-36 (existing module
  declarations — `mod write_loop;` joins alphabetical, between
  `mod stateless_connect;` and... wait, between which? Confirm
  alphabetical placement).
- `crates/reposix-remote/Cargo.toml` `[[test]]` targets — confirm
  `mirror_refs`, `push_conflict`, `bulk_delete_cap`, `perf_l1`,
  `stateless_connect`, `stateless_connect_e2e` exist as the
  regression check (D-05).
</read_first>

<action>
**HARD-BLOCK before writing the new file:** `execute_action` (line
619 of `main.rs`) takes `&mut State`. The lifted `apply_writes`
function takes `(cache, backend, backend_name, project, rt, ...)`
— NOT `&mut State`. So `apply_writes` cannot call `execute_action`
directly without either:

(a) Reverting `execute_action` to take its own narrow-deps shape;
(b) Constructing a temporary `&mut State`-shaped value inside
`apply_writes` (awkward — `State` is a multi-field struct);
(c) Keeping `execute_action`'s `&mut State` signature and calling
it from a wrapper that the caller (single-backend `handle_export`
or bus `bus_handler`) provides.

The cleanest approach is **(a) refactor `execute_action` to a
narrow-deps shape FIRST**, then `apply_writes` calls it directly.
Per RESEARCH.md S1 the lift is mechanical preservation; refactoring
`execute_action` is a precondition.

**`execute_action` new signature:**

```rust
// Source: crates/reposix-remote/src/main.rs (replaces line 619)
fn execute_action(
    backend: &dyn BackendConnector,
    project: &str,
    rt: &Runtime,
    cache: Option<&Cache>,
    action: PlannedAction,
) -> Result<()> {
    // body unchanged: state.backend.as_ref() → backend; state.rt → rt;
    // state.project → project; state.cache.as_ref() → cache.
}
```

The single existing call site (line 505 in `handle_export`'s
execute loop) updates to pass `state.backend.as_ref(), &state.project,
&state.rt, state.cache.as_ref(), action`. After T02's lift, the
new caller (`apply_writes`) calls `execute_action` with the
function's own bound parameters.

`pub(crate) fn execute_action` widening: the function stays
private to `main.rs` if `apply_writes` is `pub(crate)` in the
sibling `write_loop` module — but `write_loop` calls
`crate::execute_action`, which means the function MUST be widened
to `pub(crate)`. Mark the visibility change in the commit message;
no behavior change.

Now author the new file.

### 2a. New module — `crates/reposix-remote/src/write_loop.rs`

Estimated 250-300 lines including module-doc, the `WriteOutcome`
enum, the `apply_writes` function body, and `# Errors` doc
sections.

```rust
//! Shared SoT-write loop — lifted from `handle_export` lines
//! 360-606 with a narrow-deps signature `(cache, backend,
//! backend_name, project, rt, proto, parsed)` per P81's
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
//! - `refs/mirrors/<sot>-head`: advanced to the new SoT SHA
//!   (when `refresh_for_mirror_head` returned Some; gated on
//!   `files_touched > 0` to avoid the build_from cost on no-op
//!   pushes — same semantic as `handle_export` lines 558-573).
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
//!   bytes-in + ack-out + mirror-push-stderr-tail-bytes).
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
use crate::precheck::{
    self, precheck_export_against_changed_set, PrecheckOutcome,
};
use crate::protocol::Protocol;

/// Result of the shared SoT-write loop. Communicated back to the
/// caller (`handle_export` / `bus_handler::handle_bus_export`)
/// who maps it to `state.push_failed` and any post-write actions
/// (synced-at ref + `mirror_sync_written` audit row + token-cost
/// audit row + `ok refs/heads/main` ack OR — for bus — the
/// mirror-push subprocess + branching).
#[derive(Debug)]
pub(crate) enum WriteOutcome {
    /// All SoT writes succeeded. `audit_events_cache` got a
    /// `helper_push_accepted` row; `last_fetched_at` advanced;
    /// `refs/mirrors/<sot>-head` advanced (when `sot_sha.is_some()`).
    SotOk {
        /// New SoT SHA — input to `refs/mirrors/<sot>-synced-at`
        /// (caller writes) AND `mirror_sync_written` audit row
        /// `oid_hex` (caller writes). `None` when `files_touched
        /// == 0` (no-op push; cache's `refresh_for_mirror_head`
        /// skipped to avoid the build_from cost).
        sot_sha: Option<gix::ObjectId>,
        /// Number of records create/update/delete'd. Used by
        /// caller's `log_token_cost` (the `chars_in` is the
        /// fast-import payload size, but `files_touched` is the
        /// audit-row signal).
        files_touched: u32,
        /// Comma-separated id list (deterministic order).
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
    /// 3, SoT may be in a partial state (some PATCHes succeeded,
    /// some did not); recovery is next-push reads new SoT via
    /// PRECHECK B.
    SotPartialFail,
    /// L1 precheck itself errored (REST unreachable). `fail_push`
    /// emitted `error refs/heads/main backend-unreachable` line.
    /// Caller sets `state.push_failed = true`.
    PrecheckBackendUnreachable,
}

/// Apply REST writes to the SoT. Lifted from `handle_export` lines
/// 360-606 verbatim with the [S1 mechanical replacements](self).
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
/// the function never `?`-bubbles a SoT REST error to the caller;
/// it consumes the error and emits the appropriate protocol line.
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
    // BODY: lifted verbatim from handle_export lines 377-585 with
    // S1 mechanical replacements:
    //   - state.cache.as_ref()  → cache
    //   - state.backend.as_ref() → backend
    //   - state.backend_name     → backend_name
    //   - state.project          → project
    //   - state.rt               → rt
    //   - state.push_failed = true; return Ok(())  → return Ok(WriteOutcome::<variant>)
    //
    // REMOVED from the lifted body (deferred to caller per D-01):
    //   - cache.write_mirror_synced_at(...)
    //   - cache.log_mirror_sync_written(...)
    //   - cache.log_token_cost(...)
    //   - proto.send_line("ok refs/heads/main") + send_blank() + flush()
    //
    // RETAINED in the lifted body:
    //   - cache.log_helper_push_accepted(...)        (audit row, OP-3)
    //   - cache.write_last_fetched_at(...)           (L1 cursor)
    //   - cache.refresh_for_mirror_head() + write_mirror_head(...) (head ref)

    // ---- BEGIN LIFTED BODY ----

    let (prior, mut conflicts) = match precheck_export_against_changed_set(
        cache, backend, project, rt, &parsed,
    ) {
        Ok(PrecheckOutcome::Conflicts(c)) => (Vec::new(), c),
        Ok(PrecheckOutcome::Proceed { prior }) => (prior, Vec::new()),
        Err(e) => {
            crate::diag(&format!("L1 precheck failed: {e:#}"));
            proto.send_line("error refs/heads/main backend-unreachable")?;
            proto.send_blank()?;
            proto.flush()?;
            return Ok(WriteOutcome::PrecheckBackendUnreachable);
        }
    };

    if !conflicts.is_empty() {
        // ... lifted from handle_export lines 414-456 verbatim ...
        // (sort conflicts; emit diag; log_helper_push_rejected_conflict;
        //  emit reject-hint citing mirror-lag refs when populated;
        //  send `error refs/heads/main fetch first`; return)
        conflicts.sort_by_key(|c| c.0 .0);
        let (first_id, local_v, backend_v, backend_ts) = &conflicts[0];
        crate::diag(&format!(
            "issue {} modified on backend at {} since last fetch (local base version: {}, backend version: {}). Run: git pull --rebase",
            first_id.0, backend_ts, local_v, backend_v,
        ));
        if let Some(c) = cache {
            c.log_helper_push_rejected_conflict(
                &first_id.0.to_string(), *local_v, *backend_v,
            );
            // mirror-lag-ref hint per DVCS-MIRROR-REFS-03
            if let Ok(Some(synced_at)) = c.read_mirror_synced_at(backend_name) {
                let ago = chrono::Utc::now().signed_duration_since(synced_at);
                let mins = ago.num_minutes().max(0);
                crate::diag(&format!(
                    "hint: your origin (GH mirror) was last synced from {sot} at {ts} ({mins} minutes ago); see refs/mirrors/{sot}-synced-at",
                    sot = backend_name,
                    ts = synced_at.to_rfc3339(),
                ));
                crate::diag(&format!(
                    "hint: run `reposix sync` to update local cache from {sot} directly, then `git rebase`",
                    sot = backend_name,
                ));
            }
        }
        proto.send_line("error refs/heads/main fetch first")?;
        proto.send_blank()?;
        proto.flush()?;
        return Ok(WriteOutcome::Conflict);
    }

    let actions = match plan(&prior, &parsed) {
        Ok(a) => a,
        Err(PlanError::BulkDeleteRefused { count, limit, tag, .. }) => {
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

    // touched_ids/summary — lifted verbatim from lines 484-500
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
    let summary = touched_ids.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    let files_touched = u32::try_from(touched_ids.len()).unwrap_or(u32::MAX);

    // execute_action loop — lifted verbatim from 502-512 with the
    // narrow-deps execute_action call (T02 precondition: `execute_action`
    // takes `(backend, project, rt, cache, action)`).
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

    // SoT-success branch — lifted from 519-585 with the synced-at /
    // mirror_sync_written / token_cost / ok-line writes REMOVED
    // (deferred to caller per D-01).
    let sot_sha_opt = if let Some(c) = cache {
        c.log_helper_push_accepted(files_touched, &summary);
        if let Err(e) = c.write_last_fetched_at(Utc::now()) {
            tracing::warn!("write_last_fetched_at failed: {e:#}");
        }
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
```

*Continued in [T02-step-2.md](./T02-step-2.md) — `### 2b` through `### 2f`, `<verify>`, `<done>`.*
