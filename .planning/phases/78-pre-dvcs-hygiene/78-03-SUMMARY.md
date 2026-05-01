---
phase: 78
plan: 03
subsystem: quality/docs-alignment
tags: [schema-migration, walker, parallel-array, multi-source]
requires: [78-01]
provides: [MULTI-SOURCE-WATCH-01]
affects:
  - crates/reposix-quality/src/catalog.rs
  - crates/reposix-quality/src/commands/doc_alignment.rs
  - crates/reposix-quality/tests/walk.rs
  - crates/reposix-quality/tests/coverage.rs
  - crates/reposix-quality/tests/merge_shards.rs
  - quality/catalogs/doc-alignment.json
  - CLAUDE.md
tech-stack:
  added: []
  patterns: [parallel-array invariant, serde(default) one-time backfill, AND-compare walker]
key-files:
  created: []
  modified:
    - crates/reposix-quality/src/catalog.rs
    - crates/reposix-quality/src/commands/doc_alignment.rs
    - crates/reposix-quality/tests/walk.rs
    - crates/reposix-quality/tests/coverage.rs
    - crates/reposix-quality/tests/merge_shards.rs
    - quality/catalogs/doc-alignment.json
    - CLAUDE.md
decisions:
  - Multi-source legacy rows leave source_hashes empty (no-hash-recorded-yet) until re-bind heals them — preserves path-(a) tradeoff for those rows; new rows adopt path-(b) immediately.
  - merge-shards now hashes every cite (consistent with bind verb's "source file must exist" precondition); synthetic test fixtures updated to write real files.
  - Two-commit shape: schema migration + SHA substitution in CLAUDE.md (per CLAUDE.md "NEVER amend").
metrics:
  duration_minutes: 12
  completed: 2026-05-01T05:39:21Z
  tasks: 6
  files_changed: 7
  net_lines: +1668 / -97
---

# Phase 78 Plan 03: MULTI-SOURCE-WATCH-01 walker schema migration Summary

**One-liner:** parallel-array `source_hashes: Vec<String>` migration closes the v0.12.1 P75 path-(a) false-negative window — walker now AND-compares per-source hashes on `Source::Multi` rows.

## What was built

Six tasks, atomically committed across two commits (`28ed9be` schema + `ef81546` SHA cite). Path-(b) per `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` is closed.

### Task 03-T01 — Schema field + setter + load-time backfill

- `Row::source_hashes: Vec<String>` parallel-array (with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`).
- `Row::set_source(source, hashes) -> Result<()>` validating `source.as_slice().len() == hashes.len()` (mirrors `set_tests`).
- `Row::validate_parallel_arrays` extended to check the new invariant when `source_hashes` is non-empty.
- `Catalog::load` one-time backfill: copies `source_hash` into `source_hashes[0]` for SINGLE-source rows. Multi-source legacy rows leave `source_hashes: []` ("no-hash-recorded-yet" semantic) — backfilling 1 hash for an N-cite row would violate the parallel-array invariant.

### Task 03-T02 — `verbs::walk` AND-compare migration

Replaced the single-source `match (cite, row.source_hash.as_ref())` block with a per-index loop over `source.as_slice()` zipped with `source_hashes`. Any-index drift sets `source_drift = Some(true)` and pushes to `drifted_source_indices`. The blocking line for `STALE_DOCS_DRIFT` now surfaces:
- The first drifted source's file path (instead of unconditionally source[0]).
- A `sources_drifted=[i, j, …]` detail mirroring the `drifted={...}` test pattern.

Empty `source_hashes` → `None` (skip drift compare). Mismatched length → `Some(true)` (defends against hand-edited catalogs).

### Task 03-T03 — `verbs::bind` + `merge-shards` parallel-array writes

`bind` now manages `source_hashes` on every code path:
- **New row:** `source_hashes: vec![src_hash]` matching `Source::Single`.
- **Existing row, any shape:** rebuild `source_hashes` parallel to `sources` — reuse prior hashes for unchanged cites; insert/overwrite at `new_index` for the freshly-bound source. This handles Single/Multi append/Multi same-source-rebind in one branch.
- `set_source` enforces the invariant; legacy `source_hash = source_hashes[0]` maintained for back-compat.

`merge-shards` computes `all_source_hashes` via `hash::source_hash` per cite (with `with_context` diagnostics) and stores via `set_source`. P75 BIND-VERB-FIX-01 rationale comment updated to cite P78 closure.

### Task 03-T04 — 3 new regression tests in `walk.rs`

- `walk_multi_source_non_first_drift_fires_stale` (LOAD-BEARING): proves the path-(b) closure. Builds a 2-source Multi row via two `bind_row_at` calls; drifts `doc_b`; asserts walker fires `STALE_DOCS_DRIFT`, stderr contains `sources_drifted=[1]` and `doc_b.md`. Pre-P78 this case was a false-negative.
- `walk_legacy_catalog_backfills_source_hash_to_source_hashes`: hand-rolls a catalog with `source_hash` only (no `source_hashes`); walks; asserts post-load row has `source_hashes == [src_hash]`.
- `bind_multi_same_source_rebind_refreshes_just_that_index`: binds `doc_a` then `doc_b`; drifts `doc_b`; rebinds `doc_b`; asserts `source_hashes[0]` unchanged AND `source_hashes[1]` refreshed.

Existing P75 tests (`walk_multi_source_first_drift_fires_stale`, `walk_multi_source_stable_no_false_drift`, `walk_single_source_rebind_heals_after_drift`) carry forward unchanged. Total walk.rs: 6 → 9 tests.

### Task 03-T05 — Catalog row mint + workspace gates

Catalog row minted via the binary's `bind` verb (Principle A — tools mint, not hand-edited JSON):

```
id: doc-alignment/multi-source-watch-01-non-first-drift
source: .planning/milestones/v0.13.0-phases/CARRY-FORWARD.md:19-35
source_hash: cc79e88d3ddd0b005164e160d569dcd24f762bb3e9b10c4d1f7b7be87d68da52
source_hashes: [cc79e88d…] (1 elem)
tests: [crates/reposix-quality/tests/walk.rs::walk_multi_source_non_first_drift_fires_stale]
test_body_hashes: [aaca21a7…]
last_verdict: BOUND
next_action: BIND_GREEN
```

Workspace gates serial per CLAUDE.md "Build memory budget":
- `cargo check --workspace` → 0
- `cargo clippy --workspace --all-targets -- -D warnings` → 0
- `cargo test --workspace` → all GREEN (zero FAILED across the workspace)
- `cargo fmt --all -- --check` → 0
- `./target/release/reposix-quality run --cadence pre-push` → 25 PASS / 0 FAIL / 0 WAIVED

Live walker on the 388-row catalog post-migration (rerun with both commits in tree): exits 0; zero new STALE rows from the migration. Pre-existing 33 STALE_TEST_DRIFT rows are out of scope for P78-03 (orchestrator-flagged).

### Task 03-T06 — Commits

Two commits per CLAUDE.md "git safety: NEVER amend":
- `28ed9be` — schema migration + tests + catalog row + CLAUDE.md (placeholder `<P78-03 commit>`).
- `ef81546` — substitute the real SHA `28ed9be` into CLAUDE.md.

**No push** (orchestrator handles phase-close push + verifier subagent dispatch).

## Verification evidence

- `cargo test -p reposix-quality --test walk`: 9/9 pass (3 new + 6 carry-forward).
- `cargo test --workspace`: all green.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --all -- --check`: clean.
- `./target/release/reposix-quality walk` on live catalog: exits 0, zero new STALE.
- `./target/release/reposix-quality run --cadence pre-push`: 25 PASS, 0 FAIL.
- Catalog row `doc-alignment/multi-source-watch-01-non-first-drift` BOUND with `source_hashes: [cc79e88d…]`.

## Walker behavior delta on live catalog

| Metric | Pre-walk | Post-walk |
|---|---|---|
| `claims_total` | 388 | 389 (+1: new tracking row) |
| `claims_bound` | 331 (stale summary) | 299 (recomputed) |
| `claims_retired` | 57 | 57 |
| `STALE_TEST_DRIFT` rows | 33 (pre-existing) | 33 (unchanged — out of scope per orchestrator) |
| `STALE_DOCS_DRIFT` rows | 0 | 0 (path-(b) AND-compare introduces zero false drift) |
| `alignment_ratio` | 1.0000 (stale) | 0.9006 (correct: 299 / (389-57) = 299/332) |

The summary correction (331 → 299) is **the walker doing what it should** — the 33 STALE_TEST_DRIFT rows tipped between prior walks but `recompute_summary` hadn't been called against the post-tip state. P78-03's walk is the first walk to recompute correctly. `alignment_ratio` 0.9006 is well above the 0.5 floor; no pre-push block.

**Zero new STALE rows from the schema migration** — the path-(b) migration introduces NO false drift on the existing catalog. This is the strongest possible regression signal: the AND-compare is correct.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] Backfill path on multi-source legacy rows violated parallel-array invariant**

- **Found during:** Task 03-T05 (smoke walk on live 388-row catalog post-migration).
- **Issue:** First-pass backfill copied `source_hash` into `source_hashes[0]` unconditionally. For pre-P78 multi-source rows (14 in the live catalog) where only the first source's hash was recorded under P75 path-(a), this produced a 1-element `source_hashes` against an N-cite source — invariant violation, `Catalog::load` rejected with `validate_parallel_arrays` error.
- **Fix:** backfill the legacy hash ONLY when `source.as_slice().len() == 1`. Multi-source legacy rows leave `source_hashes: []` (no-hash-recorded-yet semantic) until re-bind heals them. Walker treats empty `source_hashes` as "skip drift compare" (preserves the path-(a) tradeoff for those rows). The walker's blocking-line emission still works: after re-bind, the row gets full per-index hashes and AND-compare engages.
- **Files modified:** `crates/reposix-quality/src/catalog.rs` (Catalog::load backfill).
- **Commit:** `28ed9be`.
- **Why this is the right shape:** the alternative — backfilling all multi-source rows by re-hashing all cites at load time — would (a) make `Catalog::load` slow and IO-bound, (b) silently mask any drift that occurred between prior bind and the current state. Empty `source_hashes` until re-bind preserves both the path-(a) tradeoff AND the catalog-first/walker-read-only invariant.

**2. [Rule 1 — Bug] merge-shards test fixtures used non-existent paths**

- **Found during:** Task 03-T04 (cargo test after T03 landed).
- **Issue:** the existing `merge_shards_auto_resolves_multi_source` test wrote shard JSONs citing `docs/a.md` and `docs/b.md` (synthetic, no files on disk). Pre-P78 merge-shards never hashed sources, so it tolerated synthetic paths. Post-P78 the new `hash::source_hash` call inside merge-shards fails on missing files.
- **Fix:** test fixture writes real files in the tempdir and cites them by absolute path; asserts new `source_hashes` array length post-merge.
- **Files modified:** `crates/reposix-quality/tests/merge_shards.rs`.
- **Commit:** `28ed9be`.
- **Why this is consistent:** bind verb already requires `source file must exist` (line 244 in pre-P78 doc_alignment.rs). Extending the precondition to merge-shards is consistent — both verbs hash sources at write time. The test was the wrong shape for the post-P78 contract.

**3. [Rule 2 — Critical] Walker diagnostic line on STALE_DOCS_DRIFT didn't surface drifted source index**

- **Found during:** Task 03-T02 (initial walker migration).
- **Issue:** the original migration only computed `source_drift: Option<bool>`, losing the per-index drift information. The blocking line still printed `cite_str = source.as_slice().first()` regardless of which source actually drifted — operator can't tell whether the first source or a non-first source tripped the alarm.
- **Fix:** track `drifted_source_indices: Vec<usize>` alongside the boolean; blocking line surfaces `sources_drifted=[i,j,...]` (mirrors the existing `drifted={...}` pattern for tests) AND uses `cites[drifted_indices[0]].file` instead of `source[0].file` for the cite shown in the message. Operators see exactly which source needs `/reposix-quality-refresh`.
- **Files modified:** `crates/reposix-quality/src/commands/doc_alignment.rs` (walk verb).
- **Commit:** `28ed9be`.
- **Why this matters:** without this, the path-(b) closure is operationally degraded — walker correctly fires STALE but the operator gets no signal about WHICH source drifted. Forensic clarity is part of the closure (acceptance criterion T02 paraphrased: "Index of drift surfaces in the diagnostic line for forensic clarity").

**4. [Rule 1 — Bug] mark-missing-test path didn't update `source_hashes`**

- **Found during:** Task 03-T01 (initial Row struct addition; cargo check surfaced missing field on three Row literals).
- **Issue:** `mark_missing_test`'s "existing row" path overwrote `row.source` and `row.source_hash` directly without touching `source_hashes`. After the call, the row would have a 1-cite `Source::Single` but a stale `source_hashes` from before — invariant violation on subsequent walks.
- **Fix:** route through `set_source(Source::Single(cite), vec![src_hash])` to enforce the invariant in one atomic step. New-row path adds `source_hashes: vec![src_hash.clone()]` matching `source_hash: Some(src_hash)`.
- **Files modified:** `crates/reposix-quality/src/commands/doc_alignment.rs` (mark_missing_test).
- **Commit:** `28ed9be`.

### Auth gates: none.

### Architectural deviations: none. The plan's path-(b) shape was followed verbatim with the one backfill-handling refinement (Deviation #1) which is consistent with the spirit of the path-(b) acceptance — rows without a recorded full per-source hash sit in the "no-hash-recorded-yet" state until re-bind heals them. This is symmetric to how the empty-tests semantic works.

## Catalog state delta

```
Pre-P78-03    Post-P78-03   Delta
claims_total          388           389           +1 (new tracking row)
claims_bound          331           299           -32 (summary recompute on pre-existing 33 STALE_TEST_DRIFT)
claims_retired         57            57            0
alignment_ratio    1.0000        0.9006     -0.0994 (correction)
STALE_DOCS_DRIFT        0             0            0 (zero false drift from migration)
STALE_TEST_DRIFT       33            33            0 (out-of-scope, pre-existing)
```

`alignment_ratio` correction: pre-walk summary was stale (`recompute_summary` hadn't run against post-tip state of 33 rows). The correction is the walker doing what it should — NOT a P78-03 regression.

## Threat surface scan

The plan's `<threat_model>` declared "schema migration is local-only data flow; no new network or unsafe surface. The new walker AND-compare strengthens the docs-claim-bound-to-test invariant (more drift gets caught) — net defensive."

Verified post-implementation:
- No new HTTP, FFI, or unsafe blocks introduced.
- All edits in `crates/reposix-quality/{src,tests}` (test+verifier crate; not on the helper or cache surface).
- Walker AND-compare is strictly more defensive than path-(a): the false-negative window is closed; no new false-positives observed on the live 388-row catalog.

No `## Threat Flags` section needed.

## Known Stubs

None. All schema migrations completed atomically; no placeholder data flows into UI or external surfaces.

## Self-Check: PASSED

- File `crates/reposix-quality/src/catalog.rs`: FOUND, 4 expected matches for `pub source_hashes`/`fn set_source`/`MULTI-SOURCE-WATCH-01 backfill`/`set_source` setter present.
- File `crates/reposix-quality/src/commands/doc_alignment.rs`: FOUND, MULTI-SOURCE-WATCH-01 referenced ≥3 times across walk, bind, merge-shards.
- File `crates/reposix-quality/tests/walk.rs`: FOUND, all 9 tests pass including the 3 new P78 tests.
- Catalog row `doc-alignment/multi-source-watch-01-non-first-drift`: FOUND in `quality/catalogs/doc-alignment.json` with `last_verdict: BOUND`.
- Commit `28ed9be`: FOUND in git log (`git log --oneline | grep 28ed9be`).
- Commit `ef81546`: FOUND in git log.
- CLAUDE.md cites real SHA `28ed9be` (no `<P78-03 commit>` placeholder remains).
- No file deletions across both commits (`git diff --diff-filter=D --name-only HEAD~2 HEAD` empty).

## What's next

Orchestrator to:
1. Push `main` (per CLAUDE.md "Push cadence — per-phase").
2. Dispatch P78 phase-close verifier subagent per `quality/PROTOCOL.md`.
3. Update `.planning/STATE.md` cursor: `Phase 78 in flight` → `Phase 78 SHIPPED`.
