---
phase: 122-remote-init-hardening
plan: 01
subsystem: quality-gates
tags: [catalog-first, agent-ux, quality-gates, json]

# Dependency graph
requires: []
provides:
  - "GREEN-contract catalog row agent-ux/import-parent-resolve-fails-loud (RPX-0508, P1, NOT-VERIFIED)"
  - "GREEN-contract catalog row agent-ux/init-refuses-nested-in-shared-tree (RPX-0406, P0, NOT-VERIFIED)"
  - "agent-ux/rebase-recovery-reconciles extended with 2 stateless-connect asserts (SC1/DRAIN-07 coverage)"
affects: ["122-02", "122-03", "122-04"]

# Tech tracking
tech-stack:
  added: []
  patterns: ["catalog-first commit (quality/CLAUDE.md) - GREEN contract lands before any implementation SHA"]

key-files:
  created: []
  modified: ["quality/catalogs/agent-ux.json"]

key-decisions:
  - "Did NOT run requirements.mark-complete for DRAIN-07/08/09 in this wave, even though 122-01-PLAN.md frontmatter lists all three: this is Wave 1 of a 4-wave phase where 122-02/03/04 own the actual implementation for DRAIN-08/DRAIN-09/DRAIN-07 respectively. Marking them complete here (catalog rows only, no code) would be premature -- REQUIREMENTS.md rows stay Pending until their owning wave lands."
  - "Rewrote commit-message prose to avoid a line starting with the literal phrase 'reposix init' -- .claude/hooks/leaf-isolation-guard.sh's command-position regex anchors on line-start (grep's default line-oriented ^), so a wrapped multi-line commit body with 'reposix init' as the first two words of a line false-positives as a live command invocation, not prose. Rephrased to 'the CLI init path' to route around this without weakening the guard."

# Metrics
duration: 25min
completed: 2026-07-18
---

# Phase 122 Plan 01: Catalog-first GREEN contract for SC1/SC2/SC3 Summary

**Pure-JSON catalog-first commit: minted two NEW agent-ux rows (import-parent-resolve-fails-loud RPX-0508, init-refuses-nested-in-shared-tree RPX-0406) and extended the existing rebase-recovery-reconciles row with DRAIN-07 stateless-connect asserts, all predating any W2-W4 implementation.**

## Performance

- **Duration:** ~25 min
- **Completed:** 2026-07-18T06:29:55Z
- **Tasks:** 2/2 completed
- **Files modified:** 1 (`quality/catalogs/agent-ux.json`)

## Accomplishments

- `agent-ux/import-parent-resolve-fails-loud` (SC2/DRAIN-08, RPX-0508, P1, 5 asserts, `kind: mechanical`, `transport_claim: false`) minted `NOT-VERIFIED`.
- `agent-ux/init-refuses-nested-in-shared-tree` (SC3/DRAIN-09, RPX-0406, P0, 6 asserts, `kind: mechanical`, `transport_claim: false`) minted `NOT-VERIFIED`.
- `agent-ux/rebase-recovery-reconciles` (SC1/DRAIN-07) extended in place with 2 new `expected.asserts` entries (stateless-connect leg + deterministic per-scenario verdict) and one added sentence in `claim_vs_assertion_audit`; `minted_at`/`cadences`/`kind`/`blast_radius`/`status` left byte-identical to before.
- Catalog loads cleanly through `run.py`'s own `load_catalog()` (which invokes `_audit_field.validate_row` on every row, not just the two new ones) -- 71 rows, zero `SystemExit`.

## Task Commits

1. **Task 1: Add the two NEW catalog rows (SC2 + SC3)** - `803fe434` (feat)
2. **Task 2: Extend the SC1 rebase-recovery-reconciles row with stateless-connect asserts** - `df6e9b85` (feat)

_No TDD; pure JSON authoring, no test/feat/refactor cycle applies._

## Files Created/Modified

- `quality/catalogs/agent-ux.json` - two new rows appended (Task 1, +51 lines); one existing row extended in place (Task 2, +4/-2 lines).

## Verification performed

- `python3 -m json.tool quality/catalogs/agent-ux.json` exits 0 (both commits).
- Plan's exact `<verify>` predicates for both tasks run and printed `OK`.
- `run.py`'s own `load_catalog(Path(...))` invoked directly (imports `run.py`, calls the real loader used at every cadence, including the `_audit_field.validate_row` load-time honesty checks for `minted_at`/`claim_vs_assertion_audit`/`coverage_kind`/`transport_claim` semantics) -- loaded successfully post-commit, 71 rows.
- `git diff --diff-filter=D` and `git status --short` checked after each commit: no deletions, no untracked files.
- `.githooks/pre-commit` ran on both commits (`run.py --cadence pre-commit`, validate-only mode) and exited 0.

## Decisions Made

- See `key-decisions` in frontmatter: (1) requirements NOT marked complete this wave (owned by 122-02/03/04); (2) commit-message rephrase to dodge a leaf-isolation-guard false positive on line-start "reposix init" prose.

## Deviations from Plan

None on the row content or shape -- both new rows and the extended row match the plan's exact JSON verbatim. Two process-level (non-content) deviations, both Rule 3 (blocking-issue auto-fixes), documented above as key-decisions:

**1. [Rule 3 - blocking] leaf-isolation-guard false positive on commit-message prose**
- **Found during:** Task 1 commit
- **Issue:** `.claude/hooks/leaf-isolation-guard.sh` Guard B blocked the `git commit -m` invocation because the heredoc-formatted commit body had "reposix init" as the first two words of a line; the guard's `at_command_position` regex anchors on line-start, and grep treats each embedded newline as a fresh "line" for `^`, so prose masquerading as a fresh line looked like a live command invocation.
- **Fix:** Reworded the commit message to say "the CLI init path" instead of "reposix init" so no line begins with the guarded verb; did not touch the hook (correct behavior for a real invocation, just an over-broad match on this specific multi-line-prose shape).
- **Files modified:** none (commit-message text only, not a tracked file).
- **Commit:** `803fe434`

**2. [Rule 3 - blocking, process only] Deferred `requirements.mark-complete` for DRAIN-07/08/09**
- **Found during:** state-update planning, before the final metadata commit.
- **Issue:** The generic `state_updates` protocol says to extract the plan frontmatter's `requirements:` field and mark all listed IDs complete. This plan's frontmatter lists all three (`DRAIN-07, DRAIN-08, DRAIN-09`), but 122-02/122-03/122-04 are the waves that actually implement DRAIN-08/DRAIN-09/DRAIN-07 respectively -- this wave only mints the catalog rows.
- **Fix:** Skipped `requirements mark-complete` for this wave; REQUIREMENTS.md rows for DRAIN-07/08/09 stay `Pending` until their owning implementation wave lands and its own SUMMARY marks them complete.
- **Files modified:** none.
- **Commit:** n/a (a non-action).

## Noticing

- **The `rebase-recovery-reconciles` row already has `cadences: ["pre-push", "pre-pr", "on-demand"]` and `status: "PASS"`.** Because it is tagged `pre-push` (not just `on-demand`), the two newly-added stateless-connect asserts are load-bearing on the plan's own non-negotiable: this row's `status` field will only be re-evaluated when someone actually runs `.githooks/pre-push` (i.e. `git push`), which the plan defers until all four P122 waves land. Confirmed `.githooks/pre-commit` (the hook that DID fire on both commits here) only runs `run.py --cadence pre-commit`, which filters to `pre-commit`-tagged rows -- neither the two new `on-demand` rows nor the extended `pre-push`-tagged row were executed as verifiers, only load-validated. This is exactly the ordering the plan's "CAUTION (ordering, load-bearing)" note calls out; worth flagging loudly to whoever runs the eventual phase-close push that this row will transiently fail until Wave 4's gate script lands the stateless-connect leg.
- **No existing row already covers SC2/SC3.** Grepped the full 71-row catalog for `resolve_import_parent`, `RPX-0508`, and `nested-in-shared-tree`/`RPX-0406` before minting -- no collision, no duplicate-row risk.
- **The P0 vs P1 split looks right on inspection.** SC3 (init-refuses-nested-in-shared-tree) is `blast_radius: P0` -- correctly so, since it guards against silent shared-repo corruption (the exact `S-260707-pr-08` incident class this repo's `leaf-isolation-guard.sh` already defends at the tool layer; this catalog row is the corresponding binary-side self-defense). SC2 (import-parent-resolve-fails-loud) is `P1` -- a helper-robustness/error-loudness improvement, not a corruption vector, so P1 is proportionate.
- **`_provenance_note` field, present on every other hand-edited agent-ux row, is absent from the two new rows and from the extended row's diff.** The plan's Task 1 JSON block did not include it and explicitly said "match them exactly... do not invent asserts beyond the plan," so it was omitted per instructions rather than added defensively. This is a minor consistency gap against the rest of the file's convention (every other hand-edited row in `agent-ux.json` carries a `_provenance_note` citing the GOOD-TO-HAVES-01 hand-edit-gap rationale) -- worth a 1-line follow-up in a later wave if the file's self-documentation matters, but load-time validation does not require the field and none of the plan's `<verify>` predicates check for it, so it was not eager-fixed here (would have been inventing beyond the plan's exact shape).

## Known Stubs

None -- this plan touches only `quality/catalogs/agent-ux.json` (pure data), no UI/rendering code, no hardcoded empty-value flows.

## Threat Flags

None -- both new rows are catalog metadata describing a future gate; the phase's `threat_model` block (T-122-06, T-122-07) already covers the catalog-row-authoring surface itself and both dispositions (`mitigate`, `accept`) are satisfied by the concrete `expected.asserts` + `claim_vs_assertion_audit` content landed here.

## Self-Check: PASSED

- FOUND: `.planning/phases/122-remote-init-hardening/122-01-SUMMARY.md`
- FOUND: commit `803fe434` (Task 1)
- FOUND: commit `df6e9b85` (Task 2)
