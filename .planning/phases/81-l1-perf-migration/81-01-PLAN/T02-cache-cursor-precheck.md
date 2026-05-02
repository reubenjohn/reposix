← [back to index](./index.md) · phase 81 plan 01

## Task 81-01-T02 — Cache cursor wrappers + helper precheck rewrite + new `precheck.rs` module

<read_first>
- `crates/reposix-cache/src/meta.rs` (entire file — 67 lines; the
  `set_meta` / `get_meta` API the new wrappers call).
- `crates/reposix-cache/src/cache.rs` lines 1-100 (`Cache::open` +
  field declarations — confirm `db: Mutex<Connection>` field
  availability and the existing log-helper-* family pattern).
- `crates/reposix-cache/src/cache.rs` lines 232-310 (`log_helper_*`
  family — style precedent for new method placement).
- `crates/reposix-cache/src/cache.rs` lines 345-400 (`list_record_ids`
  + `find_oid_for_record` — used by the new precheck path).
- `crates/reposix-cache/src/builder.rs` lines 226-249 (`Cache::sync`
  cursor read+seed-fallback shape — T02's `read_last_fetched_at`
  parses RFC3339 verbatim from this pattern).
- `crates/reposix-cache/src/lib.rs` (entire file — to confirm the
  cache crate's pub-mod list; `read_blob` returns `Tainted<Vec<u8>>`).
- `crates/reposix-remote/src/main.rs` (entire `handle_export` function
  — currently lines 280-549 post-P80; the rewrite scope is lines
  334-382 + the cursor-write insertion point near line 491).
  **Re-confirm the line numbers via `grep -n 'fn handle_export\|state.backend.list_records\|log_helper_push_accepted\|refresh_for_mirror_head' crates/reposix-remote/src/main.rs`** before editing — P80's edits shifted the region.
- `crates/reposix-remote/src/main.rs` lines 24-32 (`mod backend_dispatch;`
  + `use crate::backend_dispatch::...;` — the new `mod precheck;`
  declaration sits alphabetically between these).
- `crates/reposix-remote/src/main.rs` lines 40-71 (`State` struct —
  confirm `state.cache: Option<Cache>`, `state.backend_name: String`,
  `state.rt: tokio::runtime::Runtime`, `state.backend: Box<dyn BackendConnector>`).
- `crates/reposix-remote/src/diff.rs` lines 99-202 (`plan` function —
  signature `prior: &[Record]` UNCHANGED in P81 per D-03).
- `crates/reposix-remote/src/fast_import.rs` (find `ParsedExport`
  struct — confirm field shape: `commit_message`, `blobs: HashMap<u32, Vec<u8>>`,
  `tree: BTreeMap<String, u32>`, `deletes: Vec<String>`).
- `crates/reposix-core/src/backend.rs` lines 235-264 (`BackendConnector`
  trait definition — confirm `list_records` and `list_changed_since`
  signatures).
- `crates/reposix-core/src/lib.rs` (find `Tainted<T>` re-export — the
  `Tainted::inner_ref()` accessor T02 uses).
- gix 0.83 docs / `crates/reposix-cache/src/cache.rs::read_blob` —
  confirm `read_blob` returns `Tainted<Vec<u8>>`.
</read_first>

<action>
Three concerns in this task; keep ordering: cache wrappers (cache crate)
→ new `precheck.rs` module (remote crate) → `handle_export` rewrite
(remote crate) → cursor-write insertion (remote crate) → cargo check +
nextest + commit.

### 2a. Cache wrappers — `crates/reposix-cache/src/cache.rs`

Append the two wrapper methods to the existing `impl Cache` block. Place
them AFTER the existing `log_*` family (line ~470 post-P79's
`log_attach_walk`). Two methods:

```rust
    /// Read the cache's `meta.last_fetched_at` cursor — the timestamp
    /// of the most recent successful `Cache::build_from` or
    /// `Cache::sync` call. Used by the helper's L1 precheck on push
    /// entry (`crates/reposix-remote/src/precheck.rs`).
    ///
    /// Returns:
    /// - `Ok(Some(ts))` — the cursor is populated; the helper passes
    ///   `ts` to `BackendConnector::list_changed_since`.
    /// - `Ok(None)` — the cursor is absent (fresh cache, never built;
    ///   OR the stored string failed to parse defensively). The
    ///   helper falls through to a `list_records` walk for THIS push
    ///   only (first-push fallback per
    ///   `architecture-sketch.md § Performance subtlety` and
    ///   RESEARCH.md § Pitfall 1).
    ///
    /// # Errors
    /// - [`Error::Sqlite`] for any rusqlite I/O failure.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn read_last_fetched_at(&self) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        let conn = self.db.lock().expect("cache.db mutex poisoned");
        let raw: Option<String> = crate::meta::get_meta(&conn, "last_fetched_at")?;
        let Some(s) = raw else {
            return Ok(None);
        };
        match chrono::DateTime::parse_from_rfc3339(&s) {
            Ok(dt) => Ok(Some(dt.with_timezone(&chrono::Utc))),
            Err(e) => {
                // Defensive: malformed RFC3339 in the cursor row should
                // not poison the precheck path. WARN-log and fall back
                // to first-push semantics. This is the same shape as
                // the parse-error guard in builder.rs:233-236, except
                // we degrade to None instead of erroring — the helper's
                // first-push fallback is the intended recovery path.
                tracing::warn!(
                    "cache.last_fetched_at malformed: {s:?}: {e}; falling back to first-push semantics"
                );
                Ok(None)
            }
        }
    }

    /// Write the cache's `meta.last_fetched_at` cursor. Called by the
    /// helper after a successful push so the next push's precheck has
    /// a recent cursor.
    ///
    /// Best-effort caller pattern: callers should `tracing::warn!` on
    /// failure and continue. The push still acks `ok` to git. Cursor
    /// drift is recoverable via `reposix sync --reconcile` (the L1
    /// escape hatch).
    ///
    /// # Errors
    /// - [`Error::Sqlite`] for any rusqlite I/O failure.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn write_last_fetched_at(&self, ts: chrono::DateTime<chrono::Utc>) -> Result<()> {
        let conn = self.db.lock().expect("cache.db mutex poisoned");
        crate::meta::set_meta(&conn, "last_fetched_at", &ts.to_rfc3339())
    }
```

Append the two unit tests inside the existing
`#[cfg(test)] mod tests` block at the bottom of `cache.rs` (or, if
that block is too crowded, add a new `#[cfg(test)] mod last_fetched_at_tests`
section — match the existing test-organization style):

```rust
    #[test]
    fn read_last_fetched_at_round_trips() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // Use a deterministic backend; sim is non-network for the test.
        let cache = open_test_cache(tmp.path(), "sim", "demo");
        // Use second precision so to_rfc3339 + parse_from_rfc3339 round-trip exactly.
        let t1: chrono::DateTime<chrono::Utc> =
            "2026-05-01T12:34:56Z".parse().expect("parse t1");
        cache
            .write_last_fetched_at(t1)
            .expect("write_last_fetched_at");
        let read_back = cache
            .read_last_fetched_at()
            .expect("read_last_fetched_at")
            .expect("cursor present after write");
        assert_eq!(read_back, t1);
    }

    #[test]
    fn read_last_fetched_at_returns_none_when_absent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cache = open_test_cache(tmp.path(), "sim", "demo");
        let result = cache
            .read_last_fetched_at()
            .expect("read should succeed even when cursor absent");
        assert!(result.is_none(), "expected None for fresh cache; got {result:?}");
    }
```

The `open_test_cache` helper MUST be reused if it already exists in
the existing test module; if not, add a fresh one mirroring the P80
test pattern (`Cache::open(path, "sim", "demo")` — the exact signature
matches `crates/reposix-cache/src/cache.rs:54`). The test pattern
relies on the fact that `Cache::open` does NOT auto-call `build_from`
— the cursor remains absent until something writes it. Confirm this
in T02 read_first.

Build serially:

```bash
cargo check -p reposix-cache
cargo clippy -p reposix-cache -- -D warnings
cargo nextest run -p reposix-cache last_fetched_at
```

If `cargo clippy` fires `clippy::pedantic` lints (e.g., `must_use`
attribute on `read_last_fetched_at` — the return type carries semantic
meaning), fix inline; do NOT add `#[allow(...)]` without rationale.
Each new public fn must have a `# Errors` doc.

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

### 2c. `handle_export` rewrite — `crates/reposix-remote/src/main.rs`

Locate the rewrite scope. The current code at lines 334–382 (post-P80,
re-confirm via grep) is:

```rust
    let prior = match state
        .rt
        .block_on(state.backend.list_records(&state.project))
    { /* … 14 lines … */ };

    // ARCH-08: conflict detection. Build prior_by_id index, then walk
    // the new tree. … 33 lines …
    let prior_by_id: … = prior.iter().map(|i| (i.id, i)).collect();
    let mut conflicts: … = Vec::new();
    for (path, mark) in &parsed.tree {
        // … per-record conflict logic …
    }
```

Replace with the precheck call (M1: narrow dependencies — `~10 lines of plumbing`):

```rust
    // M1: pass narrow dependencies so the future bus handler in P82 can
    // call the same precheck without conforming to the single-backend
    // State shape. State.backend is `Arc<dyn BackendConnector>` per
    // main.rs:44, so deref to `&dyn BackendConnector`.
    let (prior, conflicts) = match precheck::precheck_export_against_changed_set(
        state.cache.as_ref(),
        state.backend.as_ref(),
        &state.project,
        &state.rt,
        &parsed,
    )? {
        precheck::PrecheckOutcome::Conflicts(c) => (Vec::new(), c),
        precheck::PrecheckOutcome::Proceed { prior } => (prior, Vec::new()),
    };
```

The `state.cache.as_ref()` call returns `Option<&Cache>` (matches the precheck's `cache: Option<&Cache>` parameter). `state.backend.as_ref()` returns `&dyn BackendConnector` (Arc deref). `&state.project` and `&state.rt` are direct field borrows. NOTE: these field access patterns require the H3 visibility widening (`pub(crate) struct State` + `pub(crate)` on the four fields) — without it, `precheck.rs` can name `State` but cannot access its fields.

The existing reject branch at lines 384–427 stays VERBATIM (consumes
the `conflicts` vec the same way). The existing `plan(&prior, &parsed)`
call at line 429 stays VERBATIM (consumes the cache-derived `prior`).

Add `mod precheck;` to the top-of-file mod declarations (alphabetical
placement — between `mod backend_dispatch;` at line 24 and the next
mod):

```rust
mod backend_dispatch;
mod fast_import;     // existing — verify position
mod precheck;        // NEW
mod stateless_connect; // existing — verify position
```

### 2d. Cursor-write insertion point — same file

Locate the success branch's cache-write block (post-P80, around lines
489–528). The current code is:

```rust
    } else {
        if let Some(cache) = state.cache.as_ref() {
            cache.log_helper_push_accepted(files_touched, &summary);

            // Mirror-lag refs (DVCS-MIRROR-REFS-02). [P80 block — UNCHANGED]
            let sot_sha_opt = match state.rt.block_on(cache.refresh_for_mirror_head()) { /* … */ };
            // … P80 ref + audit writes …
```

Insert the cursor-write call BETWEEN `log_helper_push_accepted` and the
P80 mirror-refs block (line 491 → 493):

```rust
    } else {
        if let Some(cache) = state.cache.as_ref() {
            cache.log_helper_push_accepted(files_touched, &summary);

            // L1 INBOUND-SoT cursor (DVCS-PERF-L1-01). Best-effort —
            // a write failure WARN-logs and does not poison the push
            // ack. Self-healing on next successful push (the existing
            // log_helper_push_accepted row is always written above,
            // even if the cursor write fails). NOTE: this cursor is
            // distinct from the OUTBOUND mirror-lag cursor written by
            // the P80 block below — same direction of travel on a
            // successful push but different storage layers (meta table
            // vs gix refs). See `crates/reposix-remote/src/precheck.rs`
            // module-doc for the full distinction.
            if let Err(e) = cache.write_last_fetched_at(chrono::Utc::now()) {
                tracing::warn!("write_last_fetched_at failed: {e:#}");
            }

            // Mirror-lag refs (DVCS-MIRROR-REFS-02). [P80 — UNCHANGED]
            let sot_sha_opt = match state.rt.block_on(cache.refresh_for_mirror_head()) {
                /* … */
            };
            /* … rest of P80 block … */
```

Build serially (per-crate per CLAUDE.md "Build memory budget"):

```bash
cargo check -p reposix-remote
cargo clippy -p reposix-remote -- -D warnings
cargo nextest run -p reposix-remote precheck
cargo nextest run -p reposix-remote      # full crate test run; ensure no existing-test regression
```

Stage and commit:

```bash
git add crates/reposix-cache/src/cache.rs \
        crates/reposix-remote/src/main.rs \
        crates/reposix-remote/src/precheck.rs
git commit -m "$(cat <<'EOF'
feat(cache,remote): L1 precheck — read_last_fetched_at + precheck.rs + handle_export rewrite (DVCS-PERF-L1-01, DVCS-PERF-L1-03)

- crates/reposix-cache/src/cache.rs — Cache::read_last_fetched_at + Cache::write_last_fetched_at (thin wrappers over meta::get_meta/set_meta with key 'last_fetched_at') + Cache::read_blob_cached (NEW sync gix-only primitive; returns Ok(None) on cache miss instead of fetching from backend; H1 fix — precheck path uses this NOT the async read_blob to preserve L1 perf goal)
- 4 unit tests added: read_last_fetched_at_round_trips, read_last_fetched_at_returns_none_when_absent, read_blob_cached_returns_some_when_blob_in_repo, read_blob_cached_returns_none_when_blob_absent
- crates/reposix-remote/src/main.rs — struct State widened to pub(crate) (H3 fix) with pub(crate) on rt/backend/project/cache fields; fn issue_id_from_path widened to pub(crate). Precheck.rs imports via `use crate::{State, issue_id_from_path};` (NOT crate::main::... — main.rs is binary root, not sub-module)
- crates/reposix-remote/src/precheck.rs (new) — single L1 precheck function consumed by both handle_export (this phase) and the future bus handler (P82+); enum PrecheckOutcome { Conflicts(...), Proceed { prior } }
- L1-strict delete trade-off RATIFIED inline (D-01): cache trusted as prior; backend-deleted records surface as REST 404 on PATCH; user recovery via `reposix sync --reconcile` (T03). L2/L3 hardening deferred to v0.14.0 per architecture-sketch § Performance subtlety + v0.14.0 vision-and-mental-model § L2/L3 cache-desync hardening.
- crates/reposix-remote/src/main.rs::handle_export — replaced lines 334-382 (unconditional list_records walk + per-record conflict loop) with single precheck() call matched on PrecheckOutcome
- Cursor write inserted into success branch (between log_helper_push_accepted and P80 mirror-refs block); best-effort with WARN-log on failure
- First-push fallback (cursor None → list_records walk for THIS push only; subsequent pushes hit L1 fast path)
- mod precheck declaration added alphabetically
- No new error variants (anyhow::Result throughout per H4 fix; remote crate uses `use anyhow::{Context, Result}` per main.rs:18; there is NO crates/reposix-remote/src/error.rs)
- precheck signature accepts narrow dependencies (cache, backend, project, rt, parsed) per M1 fix — unlocks P82 bus-handler reuse without State coupling

Phase 81 / Plan 01 / Task 02 / DVCS-PERF-L1-01, DVCS-PERF-L1-03.
EOF
)"
```
</action>

<verify>
  <automated>cargo check -p reposix-cache && cargo check -p reposix-remote && cargo clippy -p reposix-cache -- -D warnings && cargo clippy -p reposix-remote -- -D warnings && cargo nextest run -p reposix-cache last_fetched_at && cargo nextest run -p reposix-cache read_blob_cached && cargo nextest run -p reposix-remote && grep -q "pub(crate) struct State" crates/reposix-remote/src/main.rs && grep -q "pub(crate) fn issue_id_from_path" crates/reposix-remote/src/main.rs && grep -q "use crate::{State, issue_id_from_path}" crates/reposix-remote/src/precheck.rs && grep -q "pub fn read_blob_cached" crates/reposix-cache/src/cache.rs</automated>
</verify>

<done>
- `crates/reposix-cache/src/cache.rs` includes `Cache::read_last_fetched_at`
  + `Cache::write_last_fetched_at` + `Cache::read_blob_cached` (each
  with `# Errors` doc).
- 2 unit tests pass (`cargo nextest run -p reposix-cache last_fetched_at`)
  + 2 unit tests pass (`cargo nextest run -p reposix-cache read_blob_cached`).
- `Cache::read_blob_cached` is sync, gix-only, returns `Ok(None)` on
  cache miss (does NOT touch the backend) — H1 fix; the precheck calls
  this NOT the async `read_blob`.
- `crates/reposix-remote/src/main.rs` `struct State` is widened to
  `pub(crate) struct State` with `pub(crate)` on `rt`, `backend`,
  `project`, `cache` fields; `fn issue_id_from_path` widened to
  `pub(crate)` (H3 fix). `grep -q "pub(crate) struct State"` and
  `grep -q "pub(crate) fn issue_id_from_path"` against
  `crates/reposix-remote/src/main.rs` both succeed.
- `crates/reposix-remote/src/precheck.rs` imports via
  `use crate::{State, issue_id_from_path};` (NOT `crate::main::...`) and
  `cargo check -p reposix-remote` passes (the State import resolves).
- `crates/reposix-remote/src/precheck.rs` exists, ≤ 200 lines.
- Module-doc cites both `architecture-sketch.md § Performance subtlety`
  AND v0.14.0 vision-and-mental-model § L2/L3 (D-01 verbatim).
- `crates/reposix-remote/src/main.rs` declares `mod precheck;`.
- `handle_export` (current line range ~334-382 post-P80; re-confirm
  via grep at execution time) no longer calls
  `state.backend.list_records(&state.project)` on the cursor-present
  hot path. Single call to `precheck::precheck_export_against_changed_set`
  replaces the previous unconditional walk + per-record conflict loop.
- The conflict-reject branch (lines 384-427) consumes the conflicts vec
  from `PrecheckOutcome::Conflicts` UNCHANGED.
- `plan(&prior, &parsed)` is called with the prior `Vec<Record>` from
  `PrecheckOutcome::Proceed { prior }` — D-03 holds (plan signature
  unchanged).
- The cursor write `cache.write_last_fetched_at(chrono::Utc::now())`
  fires AFTER `log_helper_push_accepted` and BEFORE the P80 mirror-refs
  block. Best-effort: failure WARN-logs.
- `cargo check -p reposix-remote` exits 0.
- `cargo clippy -p reposix-remote -- -D warnings` exits 0.
- `cargo nextest run -p reposix-remote` exits 0; existing conflict-detection
  tests in `crates/reposix-remote/src/diff.rs` continue to pass (D-03
  preserves the `plan()` signature).
- Each new pub fn has a `# Errors` doc section.
- Cargo serialized: T02 cargo invocations run only after T01's commit
  has landed; per-crate fallback used.
</done>

---

