← [back to index](./index.md) · phase 81 plan 01

## Task 81-01-T02b — State widening + new `precheck.rs` module

*This is part 2 of 3 for T02. Preceded by [T02a](./T02a-cache-cursor-wrappers.md) (read-first + cache wrappers). Continues in [T02c](./T02c-handle-export-rewrite-cursor-write.md) (`handle_export` rewrite + cursor-write + build/commit).*

### 2b-pre. Widen `State` and `issue_id_from_path` visibility (H3 fix)

BEFORE creating `precheck.rs`, widen these in `crates/reposix-remote/src/main.rs`:

1. Line 42: `struct State {` → `pub(crate) struct State {`.
2. Inside `State`, the four fields `rt`, `backend`, `project`, `cache` change
   from default (private) to `pub(crate)`. Other fields (`backend_name`,
   `cache_project`, `push_failed`, `last_fetch_want_count`) stay private —
   the precheck does not consume them.
3. Line 554: `fn issue_id_from_path(path: &str) -> Option<u64> {` →
   `pub(crate) fn issue_id_from_path(path: &str) -> Option<u64> {`.
   (NOTE: there is also a private `fn issue_id_from_path` in `diff.rs:74`
   used internally by that module; do NOT widen the diff.rs one — leave
   it as-is. The precheck imports the main.rs version via `use crate::issue_id_from_path;`.)

These widenings are required so the sibling `precheck.rs` module can
import `State` and `issue_id_from_path` via `use crate::{State,
issue_id_from_path};`. The path `crate::main::...` is INVALID — `main.rs`
is the binary root of the `reposix-remote` binary crate, not a `main`
sub-module.

Verify after the widenings (run AFTER 2c lands precheck.rs):

```bash
cargo check -p reposix-remote
```

If `cargo check` fails with an `unused field` warning on, e.g.,
`backend_name`, the field was already pub(crate) elsewhere or its prior
visibility was inferred — investigate before committing.

### 2b. New `precheck.rs` module — `crates/reposix-remote/src/precheck.rs`

Author the new file. Estimated 150-200 lines including the doc-comment,
the `PrecheckOutcome` enum, the `precheck_export_against_changed_set`
function, and the imports.

```rust
//! L1 conflict-detection precheck for push paths (DVCS-PERF-L1-01..03).
//!
//! See `.planning/research/v0.13.0-dvcs/architecture-sketch.md
//! § Performance subtlety` for the full L1 rationale and the L1-strict
//! delete trade-off. See
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
//! - Don't call `Cache::read_blob` here — it is async AND fetches
//!   from the backend on cache miss. Use `Cache::read_blob_cached`
//!   (sync, gix-only, returns `Ok(None)` on miss).

use std::collections::HashSet;

use anyhow::{Context, Result};
use tokio::runtime::Runtime;

use reposix_cache::Cache;
use reposix_core::{
    backend::BackendConnector, frontmatter, Record, RecordId,
};

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
/// single-backend `State` shape. The single-backend call site does
/// `precheck_export_against_changed_set(state.cache.as_ref(),
/// state.backend.as_ref(), &state.project, &state.rt, &parsed)` — about
/// 10 lines of plumbing in `handle_export`.
///
/// # Errors
/// All errors are `anyhow::Error` (the remote crate uses `anyhow`
/// throughout; there is no typed `Error` enum). Caller maps to the
/// existing `fail_push(diag, "backend-unreachable", ...)` reject path
/// by `.context("backend-unreachable: ...")` annotations on REST call
/// sites; cache-read failures bubble as `anyhow::Error` and the caller
/// surfaces them as fatal helper errors.
pub(crate) fn precheck_export_against_changed_set(
    cache: Option<&Cache>,
    backend: &dyn BackendConnector,
    project: &str,
    rt: &Runtime,
    parsed: &ParsedExport,
) -> Result<PrecheckOutcome> {
    // Step 1: read cursor.
    let since: Option<chrono::DateTime<chrono::Utc>> = cache
        .and_then(|c| c.read_last_fetched_at().ok().flatten());

    // Step 2: compute the changed-id set.
    //   cursor present  → list_changed_since(since)
    //   cursor absent   → list_records (first-push fallback)
    let changed_ids: Vec<RecordId> = match since {
        Some(since_dt) => {
            // Hot path: cursor present. ONE REST call; the empty-result
            // case (no backend changes) is the cheap success path.
            rt.block_on(backend.list_changed_since(project, since_dt))
                .context("backend-unreachable: list_changed_since")?
        }
        None => {
            // First-push fallback (RESEARCH.md § Pitfall 1; plan body S2).
            // Single line of tracing — not a hot path at scale.
            tracing::info!(
                "no last_fetched_at cursor; running full list_records (first-push fallback)"
            );
            let prior = rt
                .block_on(backend.list_records(project))
                .context("backend-unreachable: list_records (first-push)")?;
            prior.iter().map(|r| r.id).collect()
        }
    };

    // Step 3: compute push set ∩ changed set, build conflicts vec.
    let changed_set: HashSet<RecordId> = changed_ids.into_iter().collect();
    let mut conflicts: Vec<(RecordId, u64, u64, String)> = Vec::new();

    for (path, mark) in &parsed.tree {
        let Some(id_num) = issue_id_from_path(path) else {
            continue; // non-issue paths (e.g. README.md)
        };
        let id = RecordId(id_num);
        if !changed_set.contains(&id) {
            continue; // hot-path bail; no parse, no GET
        }
        let Some(cache) = cache else {
            // No cache → can't compare against prior. Skip; the existing
            // plan() path will surface any new-record CREATE issues.
            continue;
        };
        let Some(prior_oid) = cache
            .find_oid_for_record(id)
            .with_context(|| format!("find_oid_for_record({id:?})"))?
        else {
            continue; // record not in cache prior — Create path; no conflict possible
        };

        // Read prior blob from the cache's bare repo via the SYNC
        // gix-only primitive `read_blob_cached` (NEW in T02). Returns
        // `Ok(None)` on cache miss instead of fetching from backend
        // (the async `read_blob` would defeat the L1 perf goal).
        // Tainted<Vec<u8>>; use inner_ref() to extract version only —
        // never echo body bytes (T-81-02).
        let Some(prior_bytes) = cache
            .read_blob_cached(prior_oid)
            .with_context(|| format!("read_blob_cached({prior_oid})"))?
        else {
            // Cache miss — the OID-map points at this blob but the
            // object isn't materialized. Treat as no-conflict for this
            // record; `plan()` will refetch on demand later via the
            // existing `read_blob` async path during execute. Bounded
            // by the lazy-materialization design (see OP-2).
            continue;
        };
        let prior_text = String::from_utf8_lossy(prior_bytes.inner_ref());
        let Ok(_prior_record) = frontmatter::parse(&prior_text) else {
            continue; // bad cache prior frontmatter — defer to plan()
        };

        // Re-fetch backend's current state — ONE GET per record in
        // changed_set ∩ push_set, typically zero or one per push.
        let backend_now = rt
            .block_on(backend.get_record(project, id))
            .with_context(|| format!("backend-unreachable: get_record({id:?})"))?;

        // Parse new blob from the export stream.
        let Some(blob_bytes) = parsed.blobs.get(mark) else {
            continue; // unresolved mark — defer to plan() error path
        };
        let new_text = String::from_utf8_lossy(blob_bytes);
        let Ok(new_record) = frontmatter::parse(&new_text) else {
            continue; // bad new-blob frontmatter — defer to plan()
        };

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
    let prior: Vec<Record> = match cache {
        Some(c) => {
            let ids = c.list_record_ids().context("list_record_ids")?;
            let mut out = Vec::with_capacity(ids.len());
            for id in ids {
                let Some(oid) = c
                    .find_oid_for_record(id)
                    .with_context(|| format!("find_oid_for_record({id:?})"))?
                else {
                    continue;
                };
                // Use read_blob_cached here too: if the blob is not
                // materialized, the prior set excludes that record —
                // plan() will surface a Create or skip via the standard
                // path. This avoids hidden backend GETs during precheck.
                let Some(blob) = c
                    .read_blob_cached(oid)
                    .with_context(|| format!("read_blob_cached({oid})"))?
                else {
                    continue;
                };
                let text = String::from_utf8_lossy(blob.inner_ref());
                if let Ok(rec) = frontmatter::parse(&text) {
                    out.push(rec);
                }
            }
            out
        }
        None => Vec::new(), // no cache → empty prior; plan() handles all-creates
    };

    Ok(PrecheckOutcome::Proceed { prior })
}
```

#### Error flow (H4 fix)

The remote crate uses `anyhow::Result` throughout (`crates/reposix-remote/src/main.rs:18`: `use anyhow::{Context, Result}`). There is NO `crates/reposix-remote/src/error.rs` — do not fabricate `Error::BackendUnreachable` / `Error::Cache` variants. The precheck signature returns `anyhow::Result<PrecheckOutcome>`; REST call sites annotate with `.context("backend-unreachable: ...")`; the caller in `handle_export` matches on `PrecheckOutcome` for the conflict-vs-proceed branching, and any returned `anyhow::Error` propagates up via `?` to the existing `fail_push(diag, "backend-unreachable", ...)` shape that already takes a `&str` reason string.

The "no new error variants" must_have is preserved: `anyhow` stays, no typed enum is introduced.

#### Imports clarified

`ParsedExport` lives in `crate::fast_import` (NOT `crate::diff` — `diff.rs` only `use`s it). `issue_id_from_path` lives at top-level `crate::` once T02 widens the private `fn` at `main.rs:554` to `pub(crate)` (H3 fix step). The `crate::main::...` path in the prior plan revision was invalid: `main.rs` is the binary root of the `reposix-remote` binary crate, not a `main` sub-module.

*Continue to [T02c](./T02c-handle-export-rewrite-cursor-write.md) for `handle_export` rewrite + cursor-write insertion + build/commit.*
