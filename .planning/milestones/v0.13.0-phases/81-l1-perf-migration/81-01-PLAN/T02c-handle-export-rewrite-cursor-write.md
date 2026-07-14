← [back to index](./index.md) · phase 81 plan 01

## Task 81-01-T02c — `handle_export` rewrite + cursor-write insertion + build/commit

*This is part 3 of 3 for T02. Preceded by [T02a](./T02a-cache-cursor-wrappers.md) (cache wrappers) and [T02b](./T02b-state-widening-precheck-module.md) (State widening + `precheck.rs`).*

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
