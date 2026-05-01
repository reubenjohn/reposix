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

use std::collections::HashSet;

use anyhow::{Context, Result};
use tokio::runtime::Runtime;

use reposix_cache::Cache;
use reposix_core::{backend::BackendConnector, frontmatter, Record, RecordId};

use crate::fast_import::ParsedExport;
use crate::issue_id_from_path;

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

    // Step 3: compute push set ∩ changed set, build conflicts vec.
    //
    // Two paths converge here:
    //
    // - Hot path (cursor present): use cache prior to extract local-base
    //   version + re-fetch backend's current state via one GET per
    //   intersect record. The cache trust contract is L1-strict per D-01.
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
        let Some(id_num) = issue_id_from_path(path) else {
            continue; // non-issue paths (e.g. README.md)
        };
        let id = RecordId(id_num);
        if !changed_set.contains(&id) {
            continue; // hot-path bail; no parse, no GET
        }
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

        if new_record.version != backend_now.version {
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
