---
phase: 91-attach-sync-real-backend-wiring
plan: 01
subsystem: quality-gates
tags: [catalog-first, agent-ux, ql-001, honesty-schema, shell-subprocess]

# Dependency graph
requires:
  - phase: 90-quality-honesty-followups
    provides: minted_at/coverage_kind/claim_vs_assertion_audit honesty schema + _audit_field.validate_row load gate
provides:
  - "agent-ux/ql-001-canonical-path-shape catalog row (NOT-VERIFIED) — the GREEN contract 91-02 must satisfy"
  - "agent-ux/attach-sync-real-backend catalog row (NOT-VERIFIED, coverage_kind: real-backend) — the GREEN contract 91-03 must satisfy"
  - "quality/gates/agent-ux/ql-001-canonical-path.sh verifier skeleton (exit-75 stub with real grep asserts pre-sketched, gated off)"
affects: [91-02-record-path-fix, 91-03-attach-sync-dispatch, 91-06-phase-close-verifier]

# Tech tracking
tech-stack:
  added: []
  patterns: ["catalog-first minting (rows exist before implementation)", "coverage_kind tri-state via transport_claim (explicit opt-in/opt-out vs regex default)"]

key-files:
  created:
    - quality/gates/agent-ux/ql-001-canonical-path.sh
  modified:
    - quality/catalogs/agent-ux.json

key-decisions:
  - "Row #1 (ql-001-canonical-path-shape) sets transport_claim: false (explicit opt-out) rather than relying on comment-wording avoidance, per interface note — this is the robust way to suppress the transport/perf coverage_kind gate for a sim/box-local proof."
  - "Row #2 (attach-sync-real-backend) declares coverage_kind: real-backend explicitly; its id contains 'real-backend' which the transport regex matches anyway, so the explicit declaration is load-bearing, not redundant."
  - "Transcript contract for row #2 satisfied via BOTH an expected.artifact.transcript_path field AND a 'transcript artifact emitted' asserts-string mention, matching the milestone-close-vision-litmus row's belt-and-braces style."

requirements-completed: [QL-001, RBF-A-01, RBF-A-02, RBF-A-04]

# Metrics
duration: 25min
completed: 2026-07-04
---

# Phase 91 Plan 01: Catalog-first minting for record-path canonicalization + real-backend attach/sync Summary

**Minted 2 NOT-VERIFIED catalog rows (ql-001-canonical-path-shape, attach-sync-real-backend) and an exit-75 verifier skeleton, with zero Rust/implementation edits — the GREEN contract now exists for 91-02/91-03 to satisfy.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-07-04T19:00Z (approx)
- **Completed:** 2026-07-04
- **Tasks:** 3/3 completed
- **Files modified:** 2 (1 modified, 1 created)

## Accomplishments
- Minted `agent-ux/ql-001-canonical-path-shape` (mechanical, `transport_claim: false`, `minted_at` set) as the box-independent proof contract for the D91-01 record-path fix landing in 91-02.
- Minted `agent-ux/attach-sync-real-backend` (shell-subprocess, `coverage_kind: real-backend`, `waiver: null`, `pre-release-real-backend` cadence only) as the real-backend attach/sync proof contract for 91-03.
- Wrote `quality/gates/agent-ux/ql-001-canonical-path.sh`: writes a NOT-VERIFIED artifact and exits 75 today; the three real grep/name asserts (zero-padded record-path construction outside core, single shared path-id helper, QL-157 duplicate deletion) are pre-written in a `run_real_asserts` function, ready for 91-02 to wire in by deleting the short-circuit block.

## Task Commits

1. **Task 1: Mint agent-ux/ql-001-canonical-path-shape** — part of `710211d`
2. **Task 2: Mint agent-ux/attach-sync-real-backend** — part of `710211d`
3. **Task 3: ql-001-canonical-path.sh verifier skeleton** — part of `710211d`

All three tasks landed as a single commit (`710211d`) since the two catalog-row edits and the verifier script are one atomic catalog-first mint; splitting them into separate commits would have left an intermediate state where the catalog referenced a not-yet-existing verifier script.

**Plan metadata:** included in `710211d` (no separate metadata commit — single-commit plan).

## Files Created/Modified
- `quality/catalogs/agent-ux.json` — +86 lines: 2 new rows inserted after `agent-ux/real-git-push-e2e`, before `agent-ux/test-name-vs-asserts`.
- `quality/gates/agent-ux/ql-001-canonical-path.sh` — new, executable, exit-75 skeleton.

## Decisions Made
- Followed the interface note literally: row #1 uses `transport_claim: false` rather than word-avoidance in `comment`, since the explicit tri-state override is the authoritative suppression mechanism in `_audit_field.is_transport_or_perf_row` (checked before the regex).
- Row #2's id (`attach-sync-real-backend`) itself matches the transport regex (`real[- ]backend`), confirming the plan's `<what_to_notice>` N-2 assumption was inverted for the id (not just comment) — declaring `coverage_kind: real-backend` is therefore load-bearing, not optional decoration.
- Inserted both new rows adjacent to the existing `real-git-push-e2e`/`test-name-vs-asserts` P90-era rows (chronological/thematic grouping) rather than at file end, matching task 1's explicit placement instruction.

## Deviations from Plan

None in the row/script content — plan executed exactly as written. One process deviation, documented for transparency:

**1. [Process — not a Rule 1-4 deviation] Reverted incidental runner-mutation side effects before commit**
- **Found during:** Task 1 verification (`python3 quality/runners/run.py --cadence on-demand`)
- **Issue:** Running the runner (as the plan's own `<verify>` step mandates) mutates `quality/catalogs/agent-ux.json` in place — 4 unrelated pre-existing rows (`agent-ux/p87-surprises-absorption`, `agent-ux/p88-good-to-haves-drained`, `agent-ux/v0.13.0-tag-script-present`, `agent-ux/v0.13.0-retrospective-distilled`) flip from a stale committed `PASS` (last_verified 2026-05-01) to a freshly-graded `FAIL` (last_verified now), because these milestone-close artifacts (SURPRISES-INTAKE.md drain, GOOD-TO-HAVES.md drain, tag-v0.13.0.sh, RETROSPECTIVE.md v0.13.0 section) are genuinely not yet in a passing state — confirmed reproducible even against the pre-existing HEAD commit with no P91 edits applied (`git stash` + re-run showed the identical 4 FAIL rows).
- **Fix:** Restored those 4 rows to their prior committed values before staging/committing, keeping this commit scoped to the 2 new P91 rows only (out-of-scope boundary per CLAUDE.md "Scope Boundary" + this plan's own `<out_of_scope>`).
- **Files affected:** `quality/catalogs/agent-ux.json` (reverted incidental hunks only; the 2 new-row hunks were untouched).
- **Verification:** Post-revert `git diff --stat` showed 86 insertions / 0 deletions (additions-only); re-ran the runner post-stage and confirmed `git diff --cached` stayed clean (the runner mutates the working tree, not the index).
- **Not committed:** No commit was made for this revert since it's a no-op restoration, not new work.

**Total deviations:** 0 Rule-1-4 auto-fixes. 1 process note (scope-boundary revert of an incidental runner side-effect).
**Impact on plan:** None on plan content. The revert kept the P91-01 commit honestly scoped; the underlying P88/OP-8/OP-9 milestone-close-artifact gap is real and is filed below, not fixed here (multi-file, cross-phase, well outside a catalog-first minting plan's 1-hour eager-fix threshold).

## Issues Encountered
None blocking. See NOTICED section below for a substantive discovery that required filing rather than fixing.

## User Setup Required
None — no external service configuration required.

## Next Phase Readiness
- 91-02 can now cite `agent-ux/ql-001-canonical-path-shape` and flip `quality/gates/agent-ux/ql-001-canonical-path.sh` from its exit-75 skeleton to the real asserts (already sketched in `run_real_asserts()`) once the D91-01 path fix lands.
- 91-03 can now cite `agent-ux/attach-sync-real-backend` when it creates `quality/gates/agent-ux/attach-sync-real-backend.sh` (the path currently dangles per plan's noted N-2 — harmless, since `pre-release-real-backend` cadence never runs at load/pre-push/on-demand).
- No blockers for Wave 1 continuation.

## NOTICED

- **[Significant — filed, not fixed]** `python3 quality/runners/run.py --cadence on-demand` reveals 4 P88-era milestone-close-artifact rows are currently genuinely FAILing against the live working tree, despite the committed catalog claiming stale `PASS` since 2026-05-01: `agent-ux/p87-surprises-absorption` (SURPRISES-INTAKE.md drain), `agent-ux/p88-good-to-haves-drained` (GOOD-TO-HAVES.md drain), `agent-ux/v0.13.0-tag-script-present` (tag-v0.13.0.sh missing/malformed), `agent-ux/v0.13.0-retrospective-distilled` (RETROSPECTIVE.md v0.13.0 section incomplete/missing). This is exactly the class of gap CLAUDE.md OP-8/OP-9 (+2 phase absorption practice) exists to catch — these are the "last two phases" milestone-close artifacts for v0.13.0, and per the roadmap those slots are presumably still ahead (P91 is mid-milestone). Confirmed this is pre-existing (reproduces identically against bare HEAD with no P91-01 edits applied via `git stash`), so it is out of scope for this catalog-first minting plan and was reverted out of the diff rather than "fixed" (fixing would mean actually drafting the RETROSPECTIVE.md section / draining the intake files / authoring the tag script — all multi-hour, cross-cutting work belonging to the milestone's own P88-successor absorption slots, not a 3-task catalog-mint plan). **Not filing a new SURPRISES-INTAKE entry for this** since it's the milestone's OWN absorption-slot mechanism surfacing correctly — flagging here so the phase coordinator/orchestrator sees it before P91 closes, in case it changes the milestone-close sequencing plan.
- Per the plan's own `<what_to_notice>`: confirmed by eye that row #2's `comment` field additionally uses "real-backend" phrasing (already covered — `coverage_kind: real-backend` is set), and no sibling agent-ux row was found with `last_verified >= cutoff` but missing `minted_at` (the ones inspected around the insertion point — `real-git-push-e2e`, `test-name-vs-asserts`, `absorption-honesty-template-present`, `milestone-adversarial-pass` — all correctly carry `minted_at` or predate the cutoff).
- The `run.py` runner mutates `quality/catalogs/agent-ux.json` in place as a side effect of grading (rewrites `status`/`last_verified` for every row it executes, even under `--cadence on-demand`). This is useful to know for future catalog-first plans: run the mandated verify command, then diff before staging to confirm no unrelated rows got swept into the commit — worth a one-line callout in `quality/PROTOCOL.md` if not already there (did not check; filing as a possible doc gap rather than confirming/fixing, out of this plan's scope).

## Self-Check: PASSED

- FOUND: quality/gates/agent-ux/ql-001-canonical-path.sh (executable, exit 75 confirmed)
- FOUND: 710211d in `git log --oneline --all`
- FOUND: both new row ids present in `quality/catalogs/agent-ux.json` (`agent-ux/ql-001-canonical-path-shape`, `agent-ux/attach-sync-real-backend`)
- Catalog load: `python3 quality/runners/run.py --cadence on-demand` — no `SystemExit`/traceback; `agent-ux.json (41 rows; 9 in scope)`; `ql-001-canonical-path-shape` graded `NOT-VERIFIED (verifier exited 75 (NOT-VERIFIED convention; not a missing-script error))`.

---
*Phase: 91-attach-sync-real-backend-wiring*
*Completed: 2026-07-04*
