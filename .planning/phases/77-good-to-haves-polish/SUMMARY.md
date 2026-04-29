---
phase: 77-good-to-haves-polish
plan: 01
subsystem: quality-gates / docs-alignment
tags: [polish, good-to-haves, docs-alignment, +2-reservation, v0.12.1, last-phase]
requires: [P74-c8e4111-eager-widen, polish2-06-landing-row-bound]
provides: [GOOD-TO-HAVES-01-closed, literal-claim-heading-match]
affects:
  - docs/index.md (heading line 95)
  - quality/gates/docs-alignment/connector-matrix-on-landing.sh (regex narrow)
  - quality/catalogs/doc-alignment.json (polish2-06-landing test_body_hashes refresh)
  - .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md (entry RESOLVED)
  - CLAUDE.md (P77 H3 added)
tech_stack:
  added: []
  patterns:
    - "+2 reservation slot 2 (good-to-haves polish) drained: XS item closed in-phase"
    - "Atomic commits per item (D-02): rename then narrow then walk then flip then CLAUDE.md then SUMMARY"
    - "Walk + rebind round-trip after verifier change (same pattern as P76 entry-1 fix)"
key_files:
  created:
    - quality/reports/verdicts/p77/walk-before.txt
    - quality/reports/verdicts/p77/walk-after.txt
    - .planning/phases/77-good-to-haves-polish/SUMMARY.md
  modified:
    - docs/index.md
    - quality/gates/docs-alignment/connector-matrix-on-landing.sh
    - quality/catalogs/doc-alignment.json
    - .planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md
    - CLAUDE.md
decisions:
  - "Heading rename plus regex narrow done together to avoid an interim verifier-fail window"
  - "Rebind via explicit bind invocation, not walker self-heal (walker hashes never auto-refresh)"
  - "HANDOVER-v0.12.1.md left in place — its deletion is a session-end orchestrator action, not P77's job"
metrics:
  duration_minutes: 3.1
  tasks_completed: 7
  commits: 7
  files_modified: 5
  files_created: 3
  closures: 1
  deferrals: 0
completed: 2026-04-29
---

# Phase 77 Plan 01: Good-to-haves Polish Summary

P77 closed `GOOD-TO-HAVES-01` by draining the v0.12.1 GOOD-TO-HAVES intake — a single XS clarity item discovered during P74 — and demonstrated the +2 reservation slot 2 working as designed.

## Phase goal recap

Drain `.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md` (XS-first per D-01). The single P74 entry: rename `docs/index.md:95` heading from "What each backend can do" to "Connector capability matrix" so the catalog claim ("Connector capability matrix added to landing page") matches the live heading word-for-word, then narrow the verifier regex back to a literal `[Cc]onnector` match — reversing P74's eager-widen (commit c8e4111).

## What shipped

1. **Heading rename** (`docs/index.md:95`): `## What each backend can do` to `## Connector capability matrix`. Surrounding paragraph (lines 97-100) intentionally left referencing "backends" — only the heading line needed to change for the verifier literal claim+heading match.
2. **Verifier regex narrow** (`quality/gates/docs-alignment/connector-matrix-on-landing.sh`): `^## .*([Cc]onnector|[Bb]ackend)` to `^## .*[Cc]onnector`. FAIL message + comment block updated to reflect the single-noun match. Verifier exits 0 against the renamed heading.
3. **Walk-after capture** (`quality/reports/verdicts/p77/walk-after.txt`): post-rename + post-narrow + post-rebind walk verdict.
4. **Catalog rebind** (`quality/catalogs/doc-alignment.json`): `polish2-06-landing` `test_body_hashes` refreshed from `b6e22c8b` to `71ac092b` to match the narrowed verifier; row stays BOUND.
5. **Intake entry RESOLVED**: GOOD-TO-HAVES.md P74 entry STATUS flipped OPEN to RESOLVED with both commit SHAs (5f3a6fc heading, fb8bd28 verifier).
6. **CLAUDE.md P77 H3** under "v0.12.1 — in flight": closure summary + walk-after pointer + verbatim D-09 HANDOVER-deletion ownership note. 27 lines (≤30 budget).

## Catalog impact

- `polish2-06-landing` `test_body_hashes` refreshed (`b6e22c8b206a8cf95...` to `71ac092b65824ecde...`). `source_hash` unchanged (claim source untouched).
- `last_verdict` post-walk-post-rebind: BOUND (after a transient STALE_TEST_DRIFT in the inner walk that the rebind healed — same pattern as P76 entry-1 fix).
- `alignment_ratio`: 0.9246 to 0.9246 (unchanged).
- `coverage_ratio`: 0.2031 to 0.2031 (unchanged).
- `claims_bound`: 331 to 331 (unchanged).
- `claims_retired`: 30 (unchanged); `claims_retire_proposed`: 27 (unchanged).
- 0 new STALE_DOCS_DRIFT, 0 new STALE_TEST_DRIFT in walk-after.txt.

## Commits

| # | SHA     | Type   | Summary                                                                |
|---|---------|--------|------------------------------------------------------------------------|
| 1 | 1d523e1 | chore  | record GREEN baseline before heading rename (walk-before.txt)          |
| 2 | 5f3a6fc | polish | rename docs/index.md heading to "Connector capability matrix"          |
| 3 | fb8bd28 | polish | narrow connector-matrix-on-landing regex to literal [Cc]onnector       |
| 4 | 4ac9206 | chore  | record walk-after verdict + rebind polish2-06-landing                  |
| 5 | 93bc2e3 | polish | mark GOOD-TO-HAVES P74 entry RESOLVED                                  |
| 6 | 9ab936e | docs   | CLAUDE.md P77 H3 — closure + HANDOVER-deletion ownership note          |
| 7 | (pending) | docs | phase SUMMARY (orchestrator-landed; executor Write blocked by guard)   |

All commits atomic per D-02; commit bodies on commits 2 + 3 quote the GOOD-TO-HAVES.md entry verbatim.

## D-09 explicit note

**HANDOVER-v0.12.1.md left in place at phase close.** Its deletion is a session-end orchestrator action, not an in-phase action. P77 is the LAST phase of the v0.12.1 autonomous run, but only criterion 1 of HANDOVER's 4 self-deletion criteria is true at P77 close (P72-P77 ship GREEN); criteria 2-4 are owner-TTY (push v0.12.0 tag, bulk-confirm retires, milestone-close verdict). The top-level coordinator owns the session-end commit that removes the HANDOVER file.

## Tech stack / patterns

- No new dependencies, no cargo work (only the `reposix-quality` binary already built; bind + walk subcommands).
- Pattern: walker never auto-refreshes test/source hashes — only an explicit `bind` invocation does. This same pattern was used in P76 entry-1 (commits 0467373 + fbc3caa).
- Pattern: rename + verifier-narrow committed in immediate succession (the inner walker run between them shows the row briefly STALE_TEST_DRIFT, healed by the rebind in commit 4ac9206).

## Final autonomous-run note (last phase of v0.12.1)

P77 is the terminal planned phase of the v0.12.1 autonomous run (P72 → P73 → P74 → P75 → P76 → P77). After P77 verdict GREEN, the run is complete. The session-end summary written by the top-level orchestrator should:

1. Confirm all 6 phases verifier-GREEN.
2. Note HANDOVER-v0.12.1.md still in place — its removal is the next owner-resumed-session action.
3. Surface owner-TTY items: push v0.12.0 tag, bulk-confirm retires, ratify v0.12.1 milestone-close verdict.
4. Bump `.planning/STATE.md` cursor to "v0.12.1 in-flight (P67-P71 follow-up session pending)" per D-08.

## Verifier dispatch

Path A `gsd-verifier` from the top-level orchestrator session (Task tool invocation; not from inside `gsd-executor`). Verdict to be written to `quality/reports/verdicts/p77/VERDICT.md`.
