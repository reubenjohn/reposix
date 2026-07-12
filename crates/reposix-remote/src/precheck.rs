//! L1 conflict-detection precheck for push paths (DVCS-PERF-L1-01..03).
//!
//! See `.planning/research/v0.13.0-dvcs/architecture-sketch.md
//! § Performance subtlety` for the full L1 rationale and the L1-strict
//! delete trade-off (D-01). See
//! `.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model.md
//! § L2/L3 cache-desync hardening` for the deferred hardening path.
//!
//! Summary: one `list_changed_since` REST call replaces the
//! unconditional `list_records` walk; the cache is trusted as the prior
//! set; backend-deleted records surface as REST 404 at PATCH time and
//! are recovered via `reposix sync --reconcile` (D-01 RATIFIED). Both
//! `handle_export` (P81) and the future bus handler (P82+) call this
//! same function (DVCS-PERF-L1-03).
//!
//! Anti-patterns:
//! - Don't fetch `list_records` "just to be safe" — defeats L1.
//! - Don't conflate `last_fetched_at` (INBOUND) with
//!   `refs/mirrors/<sot>-synced-at` (OUTBOUND, P80).
//! - Don't echo Tainted prior-blob bytes into log lines (T-81-02).
//! - Don't call [`reposix_cache::Cache::read_blob`] here — it is async
//!   AND fetches from the backend on cache miss. Use
//!   [`reposix_cache::Cache::read_blob_cached`] (sync, gix-only,
//!   returns `Ok(None)` on miss).
//!
//! ## Bus-vs-single-backend precheck asymmetry (P82+)
//!
//! [`precheck_sot_drift_any`] is a COARSER sibling intended for the
//! bus handler's PRECHECK B — it runs BEFORE reading stdin (push
//! set unknown), so it asks "did anything change?" and bails on any
//! drift. The finer [`precheck_export_against_changed_set`] runs
//! AFTER stdin is read (single-backend path today; P83's bus handler
//! will run BOTH — coarser before stdin, finer after). The
//! architecture-sketch's step 3 prose ratifies this asymmetry per Q3.1.

use std::collections::HashSet;

use anyhow::{Context, Result};
use tokio::runtime::Runtime;

use reposix_cache::Cache;
use reposix_core::{backend::BackendConnector, frontmatter, Record, RecordId};

use crate::fast_import::ParsedExport;
use reposix_core::path::record_id_from_path;

/// Outcome of the L1 precheck. The caller (today: `handle_export`,
/// future: bus handler) consumes this to either reject the push with
/// a conflict diagnostic or proceed to `plan()` against the cache-derived
/// prior.
pub(crate) enum PrecheckOutcome {
    /// Conflicts detected — the caller emits the existing
    /// `error refs/heads/main fetch first` reject path with these
    /// tuples. Tuple shape:
    /// `(id, local_version, backend_version, backend_updated_at_rfc3339)`.
    Conflicts(Vec<(RecordId, u64, u64, String)>),
    /// No conflicts — the caller proceeds to `plan(&prior, &parsed)`.
    /// The `prior` vector is materialized from the cache (D-03).
    Proceed { prior: Vec<Record> },
}

/// L1 precheck — the single conflict-detection mechanism for both
/// single-backend and bus push paths (DVCS-PERF-L1-01..03).
///
/// Walks the export stream's tree against the cache's prior view +
/// the backend's changed-set delta. Returns either the conflict tuples
/// (caller rejects) or the prior `Vec<Record>` for `plan()` (caller
/// proceeds).
///
/// # Parameters (M1: narrowed dependencies for P82 reuse)
///
/// `cache`, `backend`, `project`, `rt` are taken explicitly rather than
/// via `&mut State` so the future bus handler's `BusState { sot, mirror }`
/// can call this function directly without conforming to the
/// single-backend `State` shape.
///
/// # Errors
/// All errors are [`anyhow::Error`] — the remote crate uses `anyhow`
/// throughout; there is no typed `Error` enum. REST call sites annotate
/// with `.context("backend-unreachable: ...")` so the caller can map
/// the error message to the existing `fail_push(diag, ...)` shape that
/// already takes a `&str` reason string. Cache-read failures surface
/// as fatal helper errors via `?`-propagation.
// The function is ~130 lines because the L1 algorithm has five
// numbered steps that read top-to-bottom; splitting them across helpers
// would make the algorithm harder to audit against the
// architecture-sketch's prose. Documented exception to the
// `clippy::too_many_lines` ceiling.
#[allow(clippy::too_many_lines)]
pub(crate) fn precheck_export_against_changed_set(
    cache: Option<&Cache>,
    backend: &dyn BackendConnector,
    project: &str,
    rt: &Runtime,
    parsed: &ParsedExport,
) -> Result<PrecheckOutcome> {
    // Step 1: read cursor.
    let since: Option<chrono::DateTime<chrono::Utc>> =
        cache.and_then(|c| c.read_last_fetched_at().ok().flatten());

    // Step 2: compute the changed-id set.
    //   cursor present  → list_changed_since(since)
    //   cursor absent   → list_records (first-push fallback per
    //   architecture-sketch.md § Performance subtlety + RESEARCH.md
    //   § Pitfall 1; the cost is unchanged for the rare first-push
    //   case, every subsequent push is fast.)
    let (changed_ids, first_push_prior): (Vec<RecordId>, Option<Vec<Record>>) =
        if let Some(since_dt) = since {
            // Hot path: cursor present. ONE REST call; the empty-result
            // case (no backend changes) is the cheap success path.
            let ids = rt
                .block_on(backend.list_changed_since(project, since_dt))
                .context("backend-unreachable: list_changed_since")?;
            (ids, None)
        } else {
            // First-push fallback. Single line of tracing — not a hot
            // path at scale.
            tracing::info!(
                "no last_fetched_at cursor; running full list_records (first-push fallback)"
            );
            let prior = rt
                .block_on(backend.list_records(project))
                .context("backend-unreachable: list_records (first-push)")?;
            let ids = prior.iter().map(|r| r.id).collect();
            (ids, Some(prior))
        };

    // Step 3: for every pushed record, verify the agent's local base
    // version against the AUTHORITATIVE backend version. Build conflicts vec.
    //
    // LOST-UPDATE GUARD (shared-cache cursor staleness): the `changed_set`
    // derived from `list_changed_since(last_fetched_at)` is ADVISORY here,
    // NOT the conflict gate. `last_fetched_at` is a single wall-clock cursor
    // stored in the bare cache, which is keyed by `(backend, project)` and
    // therefore SHARED across every `reposix init`/`attach` checkout of the
    // same SoT. When sibling clone A pushes, the SoT-write branch advances
    // that shared cursor to `now`; a concurrent clone B then reads the moved
    // cursor, so `list_changed_since(now)` returns an EMPTY changed-set for
    // A's write (A's `updated_at` is at-or-before `now`) — the record that
    // DID move is invisible in the delta. Gating the version check on
    // `changed_set` membership (the pre-guard behavior) let B's stale-base
    // PATCH land and silently clobber A's edit (SILENT LOST UPDATE, HIGH,
    // empirically reproduced against a live sim). We therefore issue the
    // authoritative `get_record` for EVERY pushed record the cache knows
    // about (Updates), regardless of `changed_set` — the backend is the sole
    // arbiter of "did the SoT move under me." A cache-prior comparison would
    // NOT close the window: `execute_action` does not refresh the shared
    // cache prior on push, so both clones' cache priors are equally stale.
    //
    // The conflict itself is content-aware: a stale base `version` ALONE is a
    // no-op (a `git pull --no-rebase` leaves the merged blob at a stale
    // server-controlled version while its writable content already matches the
    // backend — QL-001); only a stale base WITH divergent writable content is
    // the lost-update shape that rejects.
    //
    // Cost: one GET per pushed Update, bounded by the push size (not project
    // size). `changed_set` is retained as a forensic signal — a conflict on a
    // record ABSENT from the delta is exactly the shared-cursor staleness
    // fingerprint and is WARN-logged for operators.
    //
    // Two paths converge here:
    //
    // - Hot path (cursor present): use the cache OID-map to gate Create vs
    //   Update cheaply, then re-fetch backend's current state via one GET per
    //   pushed Update. The cache trust contract is L1-strict per D-01.
    //
    // - First-push fallback (cursor absent): we already fetched
    //   `list_records` above, so we have authoritative `Vec<Record>`
    //   without needing per-record GETs. Use that vector directly for
    //   the version compare — same shape as pre-P81 conflict detection,
    //   only fires on the rare first push. Avoids ZeroN+ extra GETs and
    //   the cache-miss skip path that defeated push-time conflict
    //   detection on first-push stale-prior pushes.
    let changed_set: HashSet<RecordId> = changed_ids.into_iter().collect();
    let prior_by_id_first_push: std::collections::HashMap<RecordId, &Record> = first_push_prior
        .as_ref()
        .map(|p| p.iter().map(|r| (r.id, r)).collect())
        .unwrap_or_default();
    let mut conflicts: Vec<(RecordId, u64, u64, String)> = Vec::new();

    for (path, mark) in &parsed.tree {
        let Some(id_num) = record_id_from_path(path) else {
            continue; // non-record paths (e.g. README.md, .reposix/*)
        };
        let id = RecordId(id_num);
        // NOTE: we deliberately do NOT `continue` on `!changed_set.contains(&id)`
        // here. The shared wall-clock cursor can be advanced past a concurrent
        // sibling-clone write, emptying the delta for a record that DID move
        // (see the Step-3 lost-update-guard rationale above). Every pushed
        // Update is version-checked against the backend below.

        // Parse new blob from the export stream once per record.
        let Some(blob_bytes) = parsed.blobs.get(mark) else {
            continue; // unresolved mark — defer to plan() error path
        };
        let new_text = String::from_utf8_lossy(blob_bytes);
        let Ok(new_record) = frontmatter::parse(&new_text) else {
            continue; // bad new-blob frontmatter — defer to plan()
        };

        // First-push fallback path: prior_by_id_first_push is the
        // authoritative backend snapshot; compare directly. Same
        // semantics as pre-P81 conflict detection.
        if !prior_by_id_first_push.is_empty() {
            let Some(prior_record) = prior_by_id_first_push.get(&id) else {
                continue; // record absent from backend prior — Create path
            };
            if new_record.version != prior_record.version {
                conflicts.push((
                    id,
                    new_record.version,
                    prior_record.version,
                    prior_record.updated_at.to_rfc3339(),
                ));
            }
            continue;
        }

        // Hot path (cursor present): only conflict-check records the
        // cache claims to know about. New records (Create path) skip
        // the check — no base to conflict with. We use the cache's
        // OID-map (cheap, sync, no backend egress) to gate the GET.
        let Some(cache) = cache else {
            // No cache → can't compare against prior. Skip; the existing
            // plan() path will surface any new-record CREATE issues.
            continue;
        };
        let Some(_prior_oid) = cache
            .find_oid_for_record(id)
            .with_context(|| format!("find_oid_for_record({id:?})"))?
        else {
            continue; // record not in cache prior — Create path; no conflict possible
        };

        // Re-fetch backend's current state — ONE GET per record in
        // changed_set ∩ push_set, typically zero or one per push.
        // We DON'T need the cache's prior blob bytes here: the conflict
        // signal is "agent's local-base version (from new blob's
        // frontmatter) vs. backend's CURRENT version (from get_record)."
        // If the agent's blob says v3 and the backend says v5, the
        // agent's view is stale — reject. The cache's OID is gating
        // the get_record call (so we skip Create-path records cheaply),
        // but the version comparison itself does not consult the cache
        // prior blob (which may not be materialized in a partial-clone
        // cache — `Cache::build_from` only writes the oid_map, not the
        // blob bytes; calling `read_blob` would trigger a hidden GET
        // per record and defeat the L1 perf goal — see PLAN-CHECK.md
        // H1 alternative).
        let backend_now = rt
            .block_on(backend.get_record(project, id))
            .with_context(|| format!("backend-unreachable: get_record({id:?})"))?;

        // A stale base `version` alone is NOT a conflict. A routine
        // `git pull --no-rebase` leaves the merged working blob at the
        // PRE-fetch server-controlled `version`/`updated_at` while its WRITABLE
        // content already matches the backend (QL-001 Assertion-2 / no-op-push
        // idempotency). Rejecting that no-op would turn every routine pull/push
        // cycle into a spurious `fetch first`. The lost-update shape is
        // narrower: a stale base AND writable content that DIVERGES from the
        // backend's CURRENT state — that is the write that would clobber a
        // sibling's edit. We compare via the SAME writable projection the
        // planner uses (`crate::diff`), so precheck's no-op notion is
        // byte-identical to plan()'s and the two can never disagree.
        let stale_base = new_record.version != backend_now.version;
        let content_diverges = || -> bool {
            match (
                crate::diff::render_writable_for_compare(&new_record),
                crate::diff::render_writable_for_compare(&backend_now),
            ) {
                (Ok(pushed), Ok(current)) => {
                    crate::diff::normalize_for_compare(&pushed)
                        != crate::diff::normalize_for_compare(&current)
                }
                // Fail CLOSED: a stale base we cannot prove is a content no-op
                // is treated as a conflict (reject → the agent pulls-rebases;
                // never a silent clobber).
                _ => true,
            }
        };

        if stale_base && content_diverges() {
            // Forensic signal: a conflict on a record ABSENT from the
            // `list_changed_since` delta is the shared-cache cursor-staleness
            // fingerprint — the lost-update guard just prevented a silent
            // clobber that the pre-guard (delta-gated) code would have allowed.
            if !changed_set.contains(&id) {
                tracing::warn!(
                    "lost-update guard: record {} conflicts (local base v{} vs backend v{}, \
                     divergent writable content) but was ABSENT from the list_changed_since \
                     delta — shared-cache last_fetched_at cursor was advanced past a \
                     concurrent write. Rejecting the stale-base push (fetch first).",
                    id.0,
                    new_record.version,
                    backend_now.version,
                );
            }
            conflicts.push((
                id,
                new_record.version,
                backend_now.version,
                backend_now.updated_at.to_rfc3339(),
            ));
        }
    }

    // Step 4: short-circuit on conflicts.
    if !conflicts.is_empty() {
        conflicts.sort_by_key(|c| c.0 .0);
        return Ok(PrecheckOutcome::Conflicts(conflicts));
    }

    // Step 5: materialize prior Vec<Record> from cache (D-03).
    // plan()'s signature is `prior: &[Record]` — unchanged in P81.
    //
    // First-push fallback: when the cursor was absent we already
    // fetched `list_records` above; reuse that vector verbatim so
    // we don't double-spend (and so plan()'s prior reflects the
    // backend's truth on the very first push, before any cache
    // materialization could have happened).
    let prior: Vec<Record> = if let Some(p) = first_push_prior {
        p
    } else {
        match cache {
            Some(c) => {
                let ids = c.list_record_ids().context("list_record_ids")?;
                let mut out = Vec::with_capacity(ids.len());
                let mut materialized_count: usize = 0;
                for id in &ids {
                    let Some(oid) = c
                        .find_oid_for_record(*id)
                        .with_context(|| format!("find_oid_for_record({id:?})"))?
                    else {
                        continue;
                    };
                    // Use read_blob_cached: returns Ok(None) on cache
                    // miss instead of fetching from the backend. The
                    // partial-clone contract means oid_map can be
                    // populated by `Cache::build_from` BEFORE any
                    // `read_blob` materialization writes the bytes.
                    // Records whose blobs aren't materialized are
                    // EXCLUDED from prior here; if the count is short
                    // of the oid_map's row count, we fall through to
                    // a list_records walk below (ONE backend call) so
                    // plan()'s delete-detection and update-equivalence
                    // checks have authoritative prior data.
                    let Some(blob) = c
                        .read_blob_cached(oid)
                        .with_context(|| format!("read_blob_cached({oid})"))?
                    else {
                        continue;
                    };
                    let text = String::from_utf8_lossy(blob.inner_ref());
                    if let Ok(rec) = frontmatter::parse(&text) {
                        out.push(rec);
                        materialized_count += 1;
                    }
                }
                if materialized_count < ids.len() {
                    // Cache has oid_map entries but blobs aren't fully
                    // materialized (typical for a fresh cache built
                    // via `build_from`, which writes the OID-map but
                    // leaves blobs lazy per the partial-clone invariant
                    // documented at `crates/reposix-cache/src/builder.rs:65-78`).
                    // Fall through to a single `list_records` walk for
                    // THIS push only — bounded by ONE call regardless
                    // of project size. Subsequent pushes hit the L1
                    // fast path once blobs materialize via the agent's
                    // working-tree reads. The agent-ux perf test in
                    // T04 calls `Cache::sync` AND warms blobs (via
                    // `read_blob` per record) so this fallback does
                    // not fire on the steady-state hot path.
                    tracing::info!(
                        "cache prior lazy-materialization gap ({materialized}/{total}); running list_records to populate plan() prior",
                        materialized = materialized_count,
                        total = ids.len(),
                    );
                    rt.block_on(backend.list_records(project)).context(
                        "backend-unreachable: list_records (lazy-materialization fallback)",
                    )?
                } else {
                    out
                }
            }
            None => Vec::new(), // no cache → empty prior; plan() handles all-creates
        }
    };

    Ok(PrecheckOutcome::Proceed { prior })
}

/// Coarser SoT-drift outcome — bus handler's PRECHECK B reports
/// whether ANY backend record has changed since the cache cursor,
/// without intersecting against a push set (the bus path runs this
/// BEFORE reading stdin, so the push set is unknown). The finer
/// intersect-with-push-set check lives in
/// [`precheck_export_against_changed_set`] and runs in P83 AFTER
/// `parse_export_stream` consumes stdin.
#[derive(Debug, Clone)]
pub(crate) enum SotDriftOutcome {
    /// Backend has at least one record changed since `last_fetched_at`.
    /// `changed_count` is reported for diagnostic / logging only —
    /// the bus handler emits the rejection unconditionally on `Drifted`.
    Drifted { changed_count: usize },
    /// Backend stable since `last_fetched_at` (or no cursor — first-push
    /// fallback per [`precheck_export_against_changed_set`]'s policy).
    Stable,
}

/// PRECHECK B (coarser sibling of [`precheck_export_against_changed_set`]).
///
/// The bus handler runs this BEFORE reading stdin, so the push set is
/// unknown. This wrapper asks "did anything change since
/// `last_fetched_at`?" and bails on any drift; the architecture-sketch's
/// step 3 prose ratifies this coarser semantic for the bus path. The
/// finer intersect-with-push-set check (which the bus handler will
/// also run in P83 AFTER stdin is read) lives in
/// [`precheck_export_against_changed_set`].
///
/// First-push policy: when the cursor is absent, returns
/// [`SotDriftOutcome::Stable`] — same shape as
/// [`precheck_export_against_changed_set`]'s no-cursor path. The
/// inner correctness check at SoT-write time (P83) is the safety
/// net for first pushes.
///
/// # Errors
/// REST failure annotates with `.context("backend-unreachable: ...")`
/// so the bus handler maps it to the existing `fail_push(diag,
/// "backend-unreachable", ...)` shape.
pub(crate) fn precheck_sot_drift_any(
    cache: Option<&Cache>,
    backend: &dyn BackendConnector,
    project: &str,
    rt: &Runtime,
) -> Result<SotDriftOutcome> {
    // Step 1: read cursor. No cursor → first-push policy = Stable.
    let Some(since) = cache.and_then(|c| c.read_last_fetched_at().ok().flatten()) else {
        return Ok(SotDriftOutcome::Stable);
    };

    // Step 2: list_changed_since on SoT. Empty → Stable; non-empty →
    // Drifted. Bus handler emits `error refs/heads/main fetch first`
    // on Drifted.
    let changed = rt
        .block_on(backend.list_changed_since(project, since))
        .context("backend-unreachable: list_changed_since (PRECHECK B)")?;

    if changed.is_empty() {
        Ok(SotDriftOutcome::Stable)
    } else {
        Ok(SotDriftOutcome::Drifted {
            changed_count: changed.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// First-push policy: no cursor → Stable. Mirrors the no-cursor
    /// fallback in `precheck_export_against_changed_set` so the bus
    /// handler's PRECHECK B doesn't misfire on a fresh attach.
    ///
    /// We pass `cache: None` to short-circuit the cursor-read path
    /// entirely — the wrapper must NOT call `list_changed_since` when
    /// the cursor is absent. Asserting on the outcome verifies the
    /// first-push semantic without spinning up a backend.
    #[test]
    fn precheck_sot_drift_any_returns_stable_when_no_cursor() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        // SimBackend is the cheapest BackendConnector to instantiate.
        // We pass loopback :0 because the test passes `cache: None` —
        // the wrapper short-circuits on the cursor-read path and never
        // makes an HTTP call. Even if it did, REPOSIX_ALLOWED_ORIGINS
        // is loopback-by-default so the call would fail closed; the
        // assertion (Stable from the no-cursor branch) is unaffected.
        let backend = reposix_core::backend::sim::SimBackend::new("http://127.0.0.1:0".to_owned())
            .expect("build sim backend");

        let outcome = precheck_sot_drift_any(None, &backend, "demo", &rt)
            .expect("no-cursor case should return Stable without erroring");
        match outcome {
            SotDriftOutcome::Stable => {}
            SotDriftOutcome::Drifted { changed_count } => {
                panic!("expected Stable; got Drifted({changed_count})")
            }
        }
    }

    // ---- Lost-update shared-cursor regression (HIGH, data-loss) ----

    use std::collections::{BTreeMap, HashMap};
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use reposix_cache::Cache;
    use reposix_core::backend::{BackendFeature, DeleteReason};
    use reposix_core::{
        Error as CoreError, RecordStatus, Result as CoreResult, Untainted,
    };

    /// A backend whose sole record (issue 1) already sits at `version = 2`
    /// with an `updated_at` fixed in the PAST. This models the state AFTER a
    /// sibling clone A has pushed its edit: the `SoT` moved to v2, but the shared
    /// cache cursor (advanced to `now` by A's push) is now AHEAD of issue 1's
    /// `updated_at`, so the default `list_changed_since(now)` filter
    /// (`updated_at` > since) returns an EMPTY delta — issue 1's move is invisible.
    struct AdvancedCursorMock {
        record: Record,
    }

    impl AdvancedCursorMock {
        fn new() -> Self {
            // updated_at deliberately in the past relative to the cursor that
            // `Cache::build_from` writes (Utc::now()).
            let past = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
            let record = Record {
                id: RecordId(1),
                title: "issue 1 (backend at v2 after sibling push)".to_owned(),
                status: RecordStatus::Open,
                assignee: None,
                labels: vec![],
                created_at: past,
                updated_at: past,
                version: 2,
                body: "backend body\n".to_owned(),
                parent_id: None,
                extensions: BTreeMap::new(),
            };
            Self { record }
        }
    }

    #[async_trait]
    impl BackendConnector for AdvancedCursorMock {
        fn name(&self) -> &'static str {
            "advanced-cursor-mock"
        }
        fn supports(&self, _feature: BackendFeature) -> bool {
            false
        }
        async fn list_records(&self, _project: &str) -> CoreResult<Vec<Record>> {
            Ok(vec![self.record.clone()])
        }
        // NOTE: `list_changed_since` is intentionally NOT overridden — the
        // default impl filters `list_records` by `updated_at > since`. With the
        // cursor at `now` and issue 1's `updated_at` in 2020, the delta is
        // EMPTY. That empty delta is the exact shared-cursor-staleness window
        // the lost-update guard must NOT trust as the conflict gate.
        async fn get_record(&self, _project: &str, id: RecordId) -> CoreResult<Record> {
            if id == self.record.id {
                Ok(self.record.clone())
            } else {
                Err(CoreError::NotFound {
                    project: "demo".into(),
                    id: id.0.to_string(),
                })
            }
        }
        async fn create_record(&self, _: &str, _: Untainted<Record>) -> CoreResult<Record> {
            Err(CoreError::Other("unused in advanced-cursor-mock".into()))
        }
        async fn update_record(
            &self,
            _: &str,
            _: RecordId,
            _: Untainted<Record>,
            _: Option<u64>,
        ) -> CoreResult<Record> {
            Err(CoreError::Other("unused in advanced-cursor-mock".into()))
        }
        async fn delete_or_close(&self, _: &str, _: RecordId, _: DeleteReason) -> CoreResult<()> {
            Err(CoreError::Other("unused in advanced-cursor-mock".into()))
        }
    }

    /// Render issue 1 at a STALE base `version` into an on-disk frontmatter
    /// blob — this is clone B's pushed edit (it branched from v0/v1, before A
    /// moved the `SoT` to v2).
    fn stale_base_blob(version: u64) -> Vec<u8> {
        let past = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();
        let rec = Record {
            id: RecordId(1),
            title: "issue 1 (clone B stale edit)".to_owned(),
            status: RecordStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: past,
            updated_at: past,
            version,
            body: "B-CHANGED-BODY\n".to_owned(),
            parent_id: None,
            extensions: BTreeMap::new(),
        };
        frontmatter::render(&rec)
            .expect("render stale-base blob")
            .into_bytes()
    }

    /// REGRESSION (HIGH, data-loss): SILENT LOST UPDATE via the shared-cache
    /// `last_fetched_at` cursor. Two `reposix init` clones of one `SoT` share a
    /// single bare cache (and thus one wall-clock cursor). After clone A pushes
    /// (`SoT` → v2, shared cursor → now), clone B pushes a stale-base edit.
    /// B's L1 precheck runs `list_changed_since(now)` → EMPTY delta (A's write
    /// predates `now`), so the pre-guard code — which gated the version check on
    /// delta membership — never version-checked issue 1 and let B's PATCH
    /// silently clobber A's edit.
    ///
    /// The fix makes the authoritative `get_record` version check fire for
    /// EVERY pushed Update regardless of the delta. This test drives the REAL
    /// `precheck_export_against_changed_set` against a REAL `Cache` (`oid_map`
    /// populated + cursor advanced by `build_from`) and asserts the stale-base
    /// push is REJECTED as a conflict (local base v0 vs backend v2).
    ///
    /// Fails WITHOUT the fix (precheck returns `Proceed` → B's PATCH would
    /// land = lost update); passes WITH it (`Conflicts([(1, 0, 2, _)])`).
    #[test]
    // test-name-honesty: ok — asserts precheck REJECTS the stale-base push under an
    // advanced shared cursor (Conflicts with local v0 vs backend v2), the exact lost-update guard
    fn stale_base_push_rejected_when_shared_cursor_advanced_past_concurrent_write() {
        // Per-test cache isolation: unique REPOSIX_CACHE_DIR so `Cache::open`
        // resolves into a throwaway dir. nextest runs each test in its own
        // process, so setting the env here does not race sibling tests.
        let cache_dir = tempfile::tempdir().expect("cache tempdir");
        std::env::set_var("REPOSIX_CACHE_DIR", cache_dir.path());

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        let backend: Arc<dyn BackendConnector> = Arc::new(AdvancedCursorMock::new());
        let cache = Cache::open(backend.clone(), "sim", "demo").expect("open cache");

        // build_from: populates the oid_map for issue 1 (so the hot-path
        // Create-vs-Update gate finds a prior OID) AND writes last_fetched_at
        // = now (so the precheck takes the cursor-present hot path and
        // `list_changed_since(now)` returns the EMPTY delta).
        rt.block_on(cache.build_from()).expect("build_from seeds cache");
        assert!(
            cache
                .find_oid_for_record(RecordId(1))
                .expect("oid lookup")
                .is_some(),
            "precondition: issue 1 must be in the cache oid_map after build_from"
        );
        assert!(
            cache
                .read_last_fetched_at()
                .expect("cursor read")
                .is_some(),
            "precondition: build_from must advance last_fetched_at (the shared cursor)"
        );

        // Prove the staleness window is real: with the advanced cursor, the
        // backend delta is EMPTY even though the SoT is at v2.
        let since = cache.read_last_fetched_at().unwrap().unwrap();
        let delta = rt
            .block_on(backend.list_changed_since("demo", since))
            .expect("list_changed_since");
        assert!(
            delta.is_empty(),
            "precondition: advanced shared cursor must empty the delta (got {delta:?}) — \
             this is the exact window the pre-guard code trusted"
        );

        // Clone B pushes issue 1 with a STALE base version (0) — it branched
        // before A moved the SoT to v2.
        let mut blobs: HashMap<u64, Vec<u8>> = HashMap::new();
        blobs.insert(100, stale_base_blob(0));
        let mut tree: BTreeMap<String, u64> = BTreeMap::new();
        tree.insert("issues/1.md".to_owned(), 100);
        let parsed = ParsedExport {
            commit_message: "B stale edit\n".to_owned(),
            blobs,
            tree,
            deletes: vec![],
            saw_commit: true,
        };

        let outcome =
            precheck_export_against_changed_set(Some(&cache), backend.as_ref(), "demo", &rt, &parsed)
                .expect("precheck must not error");

        match outcome {
            PrecheckOutcome::Conflicts(conflicts) => {
                assert_eq!(conflicts.len(), 1, "exactly one conflict expected");
                let (id, local_v, backend_v, _ts) = &conflicts[0];
                assert_eq!(*id, RecordId(1), "conflict must name issue 1");
                assert_eq!(*local_v, 0, "local base version is the stale v0");
                assert_eq!(*backend_v, 2, "backend is at v2 (post sibling-A push)");
            }
            PrecheckOutcome::Proceed { .. } => {
                panic!(
                    "LOST UPDATE: precheck returned Proceed for a stale-base (v0) push while the \
                     backend is at v2 — B's PATCH would land and silently clobber A's edit. The \
                     shared-cursor lost-update guard failed to fire."
                );
            }
        }

        drop(cache_dir);
    }
}
