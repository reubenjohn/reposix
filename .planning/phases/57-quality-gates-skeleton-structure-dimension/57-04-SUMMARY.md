---
phase: 57-quality-gates-skeleton-structure-dimension
plan: 04
subsystem: quality-gates
tags:
  - polish-struct
  - qg-08
  - structure-dim
dependency-graph:
  requires:
    - 57-01 (catalog scaffolded)
    - 57-02 (runner online)
    - 57-03 (verifiers + parity)
  provides:
    - .planning/ROADMAP.md scoped to active milestone (v0.12.0) + index
    - .planning/milestones/v0.11.0-phases/ROADMAP.md (NEW, verbatim move)
  affects:
    - QG-08 verifier flips RED -> PASS
    - structure-dim runner: 8 PASS + 1 WAIVED (was 7 PASS + 1 FAIL + 1 WAIVED)
tech-stack:
  added: []
  patterns:
    - one-shot Python rewriter (committed at /tmp during execution; not pushed)
    - per-milestone ROADMAP.md preamble (Extracted from + date + cause)
key-files:
  created:
    - .planning/milestones/v0.11.0-phases/ROADMAP.md
  modified:
    - .planning/ROADMAP.md
    - quality/catalogs/freshness-invariants.json
    - .planning/SESSION-END-STATE-VERDICT.md
decisions:
  - Preserve pre-existing v0.10.0/v0.9.0 per-milestone ROADMAP.md files verbatim (markers present + line counts >= 90)
  - Use Python script not sed for the 480-line move (line-bounded, auditable)
metrics:
  duration: ~10min
  completed: 2026-04-27T20:01:00Z
requirements:
  - POLISH-STRUCT
  - QG-08
---

# Phase 57 Plan 04: POLISH-STRUCT — Move historical ROADMAP sections to *-phases/ Summary

**One-liner:** Moved 3 historical milestone H2 sections (~480 lines) from `.planning/ROADMAP.md` to per-milestone `.planning/milestones/v0.X.0-phases/ROADMAP.md` files, flipping QG-08 verifier RED -> PASS and bringing the structure dimension to 8 PASS + 1 WAIVED.

## Commits

- **`cfaf7bc`** docs(p57): POLISH-STRUCT — move v0.11.0 ROADMAP to *-phases/ + scope top-level (QG-08 fix)

## Files

- `.planning/milestones/v0.11.0-phases/ROADMAP.md` (NEW, 113 lines): verbatim move of OLD top-level lines 200-307 (Phases 50-55) + 5-line preamble.
- `.planning/ROADMAP.md` (704 -> 230 lines): historical sections + their `<details>` wrappers REMOVED; new `## Previously planned milestones` index block added.
- `.planning/milestones/v0.10.0-phases/ROADMAP.md` (PRESERVED, 99 lines): pre-existing curated archive (`SHIPPED 2026-04-25` marker + min line count met) — not touched.
- `.planning/milestones/v0.9.0-phases/ROADMAP.md` (PRESERVED, 207 lines): pre-existing curated archive (`SHIPPED 2026-04-24` marker + min line count met) — not touched.
- `quality/catalogs/freshness-invariants.json`: status `FAIL` -> `PASS`, last_verified bumped to 2026-04-27T19:59:36Z.
- `.planning/SESSION-END-STATE-VERDICT.md`: shim regenerated, GREEN.

## QG-08 verifier output (after the move)

```
exit=0
asserts_passed:
  - .planning/REQUIREMENTS.md scope is clean (no v0.8/9/10/11 H2 sections)
  - .planning/ROADMAP.md scope is clean (no historical milestone H2 sections)
  - Historical milestones index paragraph is permitted in both files (informational only)
asserts_failed: []
```

## Other-RED audit results

All 6 other freshness rows + banned-words: PASS.

| row_id | status |
|---|---|
| structure/install-leads-with-pkg-mgr-docs-index | PASS |
| structure/install-leads-with-pkg-mgr-readme | PASS |
| structure/no-version-pinned-filenames | PASS |
| structure/benchmarks-in-mkdocs-nav | PASS |
| structure/no-loose-roadmap-or-requirements | PASS |
| structure/no-orphan-docs | PASS |
| structure/banned-words | PASS |
| structure/top-level-requirements-roadmap-scope | PASS (this wave's fix) |
| structure/badges-resolve | WAIVED (until 2026-07-25, P60) |

Runner: `8 PASS, 0 FAIL, 0 PARTIAL, 1 WAIVED, 0 NOT-VERIFIED -> exit=0`.

## Owner-verify checkpoint

Auto-approved per the orchestrator's prompt directive (the user authorized Waves D+E+F as a single autonomous sequence; the chunky-move was pre-reviewed against the plan spec). Verifier exit code + diff stat both confirm zero content drift.

## Deviations from Plan

None. Tasks A-D ran as specified. Tasks B + C were preserved-not-overwritten per the verify-before-edit rule (markers + line counts confirmed). Task F's "Other-RED audit" found nothing to fix — every other freshness row + banned-words was already PASS.

## Self-Check: PASSED

- `.planning/milestones/v0.11.0-phases/ROADMAP.md`: FOUND
- `.planning/ROADMAP.md`: 230 lines (`<= 300`), 1 `## v0.X.0` H2, 1 `## Backlog`
- `cfaf7bc` commit: FOUND in git log
- QG-08 verifier: exit 0
- Runner: 8 PASS + 1 WAIVED
- Pre-push hook: GREEN (push to origin/main: `bd6df6b..cfaf7bc`)

## Ready for Wave E

SIMPLIFY-03 audit closure. Wave A's boundary doc + Wave E's audit memo + `scripts/catalog.py` left in place close SIMPLIFY-03.
