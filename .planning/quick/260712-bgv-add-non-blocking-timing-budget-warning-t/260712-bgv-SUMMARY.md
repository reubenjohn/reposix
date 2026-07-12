---
phase: quick-260712-bgv
plan: 01
subsystem: infra
tags: [git-hooks, quality-gates, observability, bash]

# Dependency graph
requires: []
provides:
  - Non-blocking SECONDS-based timing tripwire in .githooks/pre-commit (warns >3s)
  - Non-blocking SECONDS-based timing tripwire in .githooks/pre-push (warns >90s)
affects: [quality-gates-cadence-monitoring]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Bash SECONDS builtin reset immediately before a timed invocation, read after, compared with a strict > threshold — never mutates $?"

key-files:
  created: []
  modified:
    - .githooks/pre-commit
    - .githooks/pre-push

key-decisions:
  - "Placed the tripwire strictly after the RUNNER_EXIT check and before the global add-on delegation step, since pre-commit's step (3) uses `exec` which would otherwise swallow a later warning"
  - "No doc-alignment catalog row added — confirmed during planning that .githooks/ scripts are outside the tracked doc-alignment claim surface (docs/, .planning/, README.md, crates/ only)"

patterns-established:
  - "Non-blocking runtime tripwire: warn on stderr past a documented fixed-cost budget, never touch the exit code — same class as existing hook-internal comments (fixture-identity backstop, recursion guards)"

requirements-completed: [quick-260712-bgv]

# Metrics
duration: 15min
completed: 2026-07-12
---

# Quick Task 260712-bgv: Non-blocking timing-budget warning for git hooks Summary

**Added a SECONDS-based non-blocking timing tripwire to `.githooks/pre-commit` (warns >3s) and `.githooks/pre-push` (warns >90s), surfacing budget creep documented in quality/CLAUDE.md § Cadences without ever changing hook exit codes.**

## Performance

- **Duration:** ~15 min
- **Tasks:** 2 completed
- **Files modified:** 2 (.githooks/pre-commit, .githooks/pre-push)

## Accomplishments
- `.githooks/pre-commit` now times the `python3 quality/runners/run.py --cadence pre-commit` invocation via `SECONDS=0` and warns on stderr (`pre-commit: WARN — took Ns...`) if elapsed exceeds 3s (budget ~2s), placed after the RUNNER_EXIT check and before the `exec`-based personal add-on delegation.
- `.githooks/pre-push` applies the analogous tripwire at 90s (budget ~60s), placed after the RUNNER_EXIT check and before the personal add-on chain.
- Confirmed via a corrected mock (see Deviations) that the strict `>` threshold logic fires exactly at budget+slack and stays silent at/below it, for both cadences independently.
- Confirmed the real pre-commit hook still exits 0 with no warning on a fast (~0.15s) run.

## Task Commits

1. **Task 1: Wrap pre-commit and pre-push runners with a non-blocking SECONDS timing warning** - `b4e96d8` (feat)
2. **Task 2: Prove threshold logic and no-regression on a real run** - no code changes (verification-only task); proof executed against files from Task 1's commit `b4e96d8`

## Files Created/Modified
- `.githooks/pre-commit` - Added `SECONDS=0` before the runner invocation and a `(2b)` non-blocking WARN block (>3s) after the RUNNER_EXIT check, before global delegation.
- `.githooks/pre-push` - Added `SECONDS=0` before the runner invocation and a `(2b)` non-blocking WARN block (>90s) after the RUNNER_EXIT check, before global delegation.

## Decisions Made
- Followed the plan's exact placement and threshold values (pre-commit >3s / pre-push >90s) and exact WARN message wording.
- No catalog row added, per the plan's pre-confirmed `doc_alignment_conclusion` — hook-script inline comments are not tracked doc-alignment claims.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug in plan's own verify script] Task 2's literal automated threshold-check command has a false-negative design flaw**
- **Found during:** Task 2 (threshold-branch proof)
- **Issue:** The plan's `<verify>` snippet defines a single `w()` function that fires BOTH the pre-push (>90) and pre-commit (>3) branches for the same `elapsed` value. Calling `w 90` correctly stays silent for the pre-push branch (90 is not >90) but ALSO fires the pre-commit branch (90 IS >3), printing `"pre-commit: WARN 90"` — which then fails the check's own `! grep -q "WARN 90"` assertion via substring match. This is a bug in the disposable mock test snippet itself, not in the actual hook implementation (verified separately via `bash -n` and code inspection).
- **Fix:** Wrote a corrected proof script (in the session scratchpad, not committed — it's disposable verification, not a deliverable) that tests each cadence's threshold branch independently: `prepush(91)` → warns, `prepush(90)` → silent, `precommit(4)` → warns, `precommit(2)` → silent. All four assertions passed (`BRANCH-OK`).
- **Files modified:** None (scratchpad-only verification script, not part of the deliverable).
- **Verification:** `BRANCH-OK` printed; all four threshold assertions passed independently per cadence.
- **Committed in:** N/A (verification-only, no repo file changed)

---

**Total deviations:** 1 auto-fixed (1 bug found in the plan's own disposable verify snippet, not in shipped code)
**Impact on plan:** No impact on the actual deliverable (`.githooks/pre-commit`, `.githooks/pre-push`) — both files are correct and match the plan's exact specification. The flaw was isolated to an ad-hoc test mock used only to prove the branch logic, and the corrected proof confirms the real threshold logic (as implemented in the shipped hooks) is correct.

## Issues Encountered
None beyond the verify-script deviation documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- The timing tripwire is live on the next `git commit` / `git push` through the tracked `.githooks/` hooks (installed via `bash scripts/install-hooks.sh`).
- No further action needed; this is a self-contained observability addition with no downstream dependencies.

---
*Phase: quick-260712-bgv*
*Completed: 2026-07-12*

## Self-Check: PASSED

- FOUND: .githooks/pre-commit
- FOUND: .githooks/pre-push
- FOUND: b4e96d8 (Task 1 commit, in git log)
