---
phase: 75-bind-verb-hash-fix
plan: 01
status: COMPLETE
requirement_closed: BIND-VERB-FIX-01
milestone: v0.12.1
mode: --auto (sequential gsd-executor on main, depth-1)
duration_min: ~25
verifier_verdict_path: quality/reports/verdicts/p75/VERDICT.md
---

# Phase 75 Plan 01: Bind-verb hash-overwrite fix — Summary

One-liner: `verbs::bind` no longer clobbers `row.source_hash` on
`Source::Single → Source::Multi` promotion; the walker's first-source
compare invariant is now preserved by construction, and the linkedin
row from P74 healed STALE_DOCS_DRIFT → BOUND on a fresh re-bind.

## Commits (newest first)

| SHA       | Type    | What                                                            |
|-----------|---------|-----------------------------------------------------------------|
| c13e8e5   | docs    | CLAUDE.md P75 H3 + v0.13.0 `MULTI-SOURCE-WATCH-01` carry-forward |
| 9e07028   | docs    | linkedin row heal + live-walk smoke at `quality/reports/verdicts/p75/walk-after-fix.txt` |
| 69a30b0   | fix     | `verbs::bind` preserves `source_hash` on Multi paths (the GREEN fix) |
| 5f419a1   | test    | three walker regression tests (RED): Multi-stable, Multi-first-drift, Single-rebind-heal |

Five-commit canonical order (RED → GREEN → SMOKE → DOCS → SUMMARY); this
SUMMARY is the fifth.

## What was found / what was fixed

**Root cause:** `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind`
unconditionally executed `row.source_hash = Some(src_hash);` after every
re-bind. `src_hash` is the freshly-computed hash of the *invoked* source.
On a `Source::Single → Source::Multi` promotion, the bind verb appends
the new source as `sources[1]` (preserves the existing first source),
but `row.source_hash` got overwritten with `hash(sources[1])`. The
walker (`verbs::walk`, line ~836) reads `row.source_hash` against
`source.as_slice()[0]` — the *first* source. After promotion that
compare is hash(B) ≠ hash(A) for stable A, firing false
`STALE_DOCS_DRIFT` on every cluster sweep that added a Multi citation.

**Fix (path (a) per CONTEXT.md D-01):** refresh `source_hash` only when
the result is `Source::Single`; preserve it on `Multi` paths. Single
re-binds with the same citation still refresh (this IS the heal path
for drifted-prose Single rows). Path (b) — walker hashes every source
from a Multi via parallel-array `source_hashes` — is filed as v0.13.0
carry-forward `MULTI-SOURCE-WATCH-01` (see deferred section below).

**P74 broadening finding (Test C):** P74's SURPRISES-INTAKE.md noted
that `Source::Single` rows (like the linkedin row) also stuck in
`STALE_DOCS_DRIFT`. Test C (`walk_single_source_rebind_heals_after_drift`)
confirmed that the existing Single re-bind path was already correct —
the test PASSED pre-fix. The linkedin row stayed STALE because **walks
never refresh stored hashes; only binds do** (per
`crates/reposix-quality/src/commands/doc_alignment.rs::verbs::walk`
docstring line 802–804). The procedural finding (auto-rebind on
walk-after-edit doesn't exist) is real, but it's not a second bug —
it's the documented walker contract. P75 healed the linkedin row by
running an explicit `bind` at task 4.

## What tests caught

- **Test A** `walk_multi_source_stable_no_false_drift` — FAILED pre-fix
  (the canonical regression). PASSED post-fix.
- **Test B** `walk_multi_source_first_drift_fires_stale` — PASSED both
  pre- and post-fix; positive case for the path-(a) compare site
  (NOT a regression, just lock-in coverage that the fix doesn't break
  the legitimate STALE detection on first-source drift).
- **Test C** `walk_single_source_rebind_heals_after_drift` — PASSED
  both pre- and post-fix; documents that the heal path was already
  correct for Single rows (P74 finding diagnostic).

`cargo test -p reposix-quality` GREEN: all suites pass; no pre-existing
test regressed under the bind-verb narrowing.

## Live evidence

`quality/reports/verdicts/p75/walk-after-fix.txt` captures the live walk
against the populated 388-row catalog post-fix:

- linkedin row (`docs/social/linkedin/token-reduction-92pct`):
  `STALE_DOCS_DRIFT` (source_hash=`1a19b86e…`) → `BOUND`
  (source_hash=`7a1d7a4e…`).
- **Net new STALE_DOCS_DRIFT transitions caused by P75: 0.**
- The two STALE_DOCS_DRIFT rows still in the catalog
  (`polish-03-mermaid-render`, `cli-subcommand-surface`) are
  pre-existing per P72 SURPRISES-INTAKE.md and explicitly carved out
  of the P75 verdict invariant (P76 triages).

## Catalog deltas

| Metric                 | Pre-P75       | Post-P75      | Δ        |
|------------------------|---------------|---------------|----------|
| `claims_bound`         | 328           | 329           | +1       |
| `claims_missing_test`  | 0             | 0             | 0        |
| `claims_retire_proposed`| 27           | 27            | 0        |
| `claims_retired`       | 30            | 30            | 0        |
| `alignment_ratio`      | 0.9162        | 0.9190        | +0.0028  |

The single +1 in `claims_bound` is the linkedin row tipping from
STALE_DOCS_DRIFT to BOUND. No other rows changed — P75 is a surgical
fix, not a sweep.

## What was deferred

- **`MULTI-SOURCE-WATCH-01`** (v0.13.0) — walker iterates every source
  citation in a `Source::Multi` row via parallel-array `source_hashes`.
  Path (b) requires schema migration + 388-row catalog backfill;
  out of scope for a single-phase fix. Filed at
  `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md`.

## CLAUDE.md update

CLAUDE.md:390–407 — `### P75 — bind-verb hash-overwrite fix` (18 lines,
≤20 per CONTEXT.md D-08). Names the bind-verb invariant
(`source_hash == hash(first source)`), the path-(a) walker tradeoff
(first-source-only watch), and the v0.13.0 carry-forward pointer.

## SURPRISES-INTAKE / GOOD-TO-HAVES appends

**SURPRISES-INTAKE:** none — the fix landed cleanly. No bind/walker
bugs surfaced beyond the documented scope. The P74 "didn't heal"
broadening was confirmed-not-a-bug (procedural; walker contract is
intentional), so no NEW intake entry is warranted.

**GOOD-TO-HAVES:** none — no polish opportunities observed during this
phase. (The bind verb's verb-pattern getting busy is already filed for
P77 per CONTEXT.md "deferred" section; no need to re-file.)

## Self-Check: PASSED

- `crates/reposix-quality/src/commands/doc_alignment.rs` — modified (commit `69a30b0`).
- `crates/reposix-quality/tests/walk.rs` — modified (commit `5f419a1`).
- `quality/catalogs/doc-alignment.json` — modified (commit `9e07028`).
- `CLAUDE.md` — modified (commit `c13e8e5`).
- `quality/reports/verdicts/p75/walk-after-fix.txt` — created (commit `9e07028`).
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` — created (commit `c13e8e5`).
- All four claimed commits exist in `git log`.
- `cargo test -p reposix-quality` exit 0; all 6/6 walk tests pass.
- linkedin row `last_verdict == "BOUND"`, `source_hash == 7a1d7a4e…`.

## Verifier verdict

To be filed at `quality/reports/verdicts/p75/VERDICT.md` by the
top-level orchestrator's Path A `gsd-verifier` dispatch. P75 closes
when the verdict is GREEN.
