---
phase: 123-quality-runner-catalog-integrity-hardening
plan: 01
subsystem: quality-gates
tags: [catalog-first, quality-runner, structure-dimension, ci-green-on-main, agent-ux]

# Dependency graph
requires: []
provides:
  - "4 new NOT-VERIFIED structure-dimension catalog rows (SC1-SC4) in freshness-invariants.json, each naming a not-yet-existing verifier script"
  - "code/ci-green-on-main row rewritten to the required-workflow-list contract (SC5a), status NOT-VERIFIED"
  - "agent-ux/t4-conflict-rebase-ancestry-real-backend row's contract extended with the real-stderr assertion (SC5b), status untouched"
affects: [123-02, 123-03, 123-04, 123-05, 123-06, 123-07]

# Tech tracking
tech-stack:
  added: []
  patterns: ["catalog-first: GREEN-contract rows minted before verifier implementation exists"]

key-files:
  created: []
  modified:
    - quality/catalogs/freshness-invariants.json
    - quality/catalogs/code.json
    - quality/catalogs/agent-ux.json

key-decisions:
  - "New SC1-SC4 rows carry cadences [pre-push, pre-pr] only, deliberately omitting pre-commit until 123-06 lands verifier-script-exists.sh (a P1 NOT-VERIFIED row tagged pre-commit would self-block every commit through wave 5)"
  - "code/ci-green-on-main's sources/command fields left describing the pre-P123 single-workflow probe; only comment/expected.asserts/claim_vs_assertion_audit/timeout_s/status touched per plan scope -- 123-03 updates sources/command alongside the actual script change"
  - "Verified GTH-V15-07's two open questions empirically via gh run list --workflow=release-plz.yml --branch=main --limit=8 (8/8 success, checked 2026-07-18) rather than trusting the plan's claim at face value"

requirements-completed: [DRAIN-01, DRAIN-03, DRAIN-04, DRAIN-05, DRAIN-06, DRAIN-10]

# Metrics
duration: ~20min
completed: 2026-07-18
---

# Phase 123 Plan 01: Catalog-First GREEN-Contract Authoring Summary

**Minted 4 new NOT-VERIFIED structure-dimension rows (dotenv-sourcing, persist-downgrade-refusal, persist-write-lock, verifier-script-exists) and rewrote 2 existing rows' contracts (ci-green-on-main required-workflow list, t4-real-backend real-stderr assertion) — all pre-dating any implementation code per quality/PROTOCOL.md Step 3.**

## Performance

- **Duration:** ~20 min
- **Completed:** 2026-07-18T11:03:27Z
- **Tasks:** 2/2 complete
- **Files modified:** 3

## Accomplishments

- 4 new structure-dimension rows minted, each schema-valid (`minted_at` + `claim_vs_assertion_audit` >=50 chars), NOT-VERIFIED, pointing at not-yet-written verifier scripts (SC1-SC4 / DRAIN-03/04/05/06).
- `code/ci-green-on-main` (SC5a / DRAIN-01) rewritten to the required-workflow-list contract `[ci.yml, release-plz.yml]`, with empirical evidence (real `gh run list` output) appended to `claim_vs_assertion_audit`, status flipped PASS→NOT-VERIFIED for 123-03 to re-mint.
- `agent-ux/t4-conflict-rebase-ancestry-real-backend` (SC5b / DRAIN-10) extended with a new assertion requiring the real captured git stderr instead of the misleading hardcoded git-version fallback; status left untouched (env-gated, no real creds this session).
- All 6 requirement IDs this phase addresses (DRAIN-01/03/04/05/06/10) have a traceable catalog row.

## Task Commits

1. **Task 1: Author 4 new structure-dimension catalog rows (SC1-SC4)** - `3cf15cc9` (feat)
2. **Task 2: Update code.json + agent-ux.json contracts for SC5a/SC5b** - `5a6a3362` (feat)

No separate plan-metadata commit was made yet — this SUMMARY + STATE/ROADMAP update lands in the final metadata commit below.

## Files Created/Modified

- `quality/catalogs/freshness-invariants.json` - appended 4 new NOT-VERIFIED structure rows (159 lines added, 0 removed; `git diff --stat` confirms pure addition)
- `quality/catalogs/code.json` - `code/ci-green-on-main` comment appended, `expected.asserts` rewritten (4 predicates), `verifier.timeout_s` 60→90, `status` PASS→NOT-VERIFIED, `claim_vs_assertion_audit` appended with empirical `gh run list` evidence
- `quality/catalogs/agent-ux.json` - `agent-ux/t4-conflict-rebase-ancestry-real-backend` comment appended, one new `expected.asserts` predicate added; `status` untouched

## Decisions Made

- Kept the new SC1-SC4 rows off the `pre-commit` cadence exactly as the plan's execution notes specified — verified this was correct by re-running the pre-commit hook after each commit: only 2 pre-existing rows were "in scope" for `pre-commit`, both PASS/WAIVED, exit 0 both times.
- Chose NOT to update `code/ci-green-on-main`'s `sources`/`command` fields even though they now describe the pre-P123 single-workflow probe while `expected.asserts` describes the post-P123 list contract — this mirrors the plan's precise task-2 scope (only comment/asserts/timeout_s/status/claim_vs_assertion_audit were named) and mirrors the SC1-SC4 pattern where a row's asserts can describe a target contract ahead of the sources/command update that lands with the actual script in 123-03.
- Used a Python round-trip check (`json.dumps(json.load(f), indent=2) == original`) before choosing an edit strategy per file: confirmed lossless for `freshness-invariants.json` and `code.json` (safe for a full-catalog Python rewrite), but `agent-ux.json` round-tripped DIFFERENT (some pre-existing rows use compact single-line array formatting) — so `agent-ux.json` was edited via targeted string replacement (Edit tool) instead, to avoid an unrelated 85-line reformatting diff. `git diff --stat` confirms only the 3 declared `files_modified` changed.

## Deviations from Plan

None - plan executed exactly as written. Both catalog files' edits match the plan's task actions field-for-field; no architectural changes, no new scripts, no scope creep.

## Issues Encountered

None. `run.discover_catalogs()`/`load_catalog()` loaded every catalog cleanly after each task; both post-commit pre-commit-hook runs exited 0.

## Self-Check

- `quality/catalogs/freshness-invariants.json` FOUND — contains all 4 new row ids (`structure/quality-runner-sources-dotenv`, `structure/persist-refuses-downgrade`, `structure/persist-catalog-write-locked`, `structure/verifier-script-exists`), confirmed via `grep -c` against the file post-commit.
- `quality/catalogs/code.json` FOUND — `code/ci-green-on-main` row's `status` field confirmed `NOT-VERIFIED`, `verifier.timeout_s` confirmed `90`.
- `quality/catalogs/agent-ux.json` FOUND — `agent-ux/t4-conflict-rebase-ancestry-real-backend` row's `expected.asserts` array confirmed 3 entries (was 2), `status` confirmed unchanged at `NOT-VERIFIED`.
- Commit `3cf15cc9` FOUND in `git log --oneline --all`.
- Commit `5a6a3362` FOUND in `git log --oneline --all`.

## Self-Check: PASSED

## User Setup Required

None - no external service configuration required.

## Noticed (OD-3 ownership charter)

- `code/ci-green-on-main`'s `sources`/`command` fields now lag one wave behind its own `expected.asserts` (asserts describe the `[ci.yml, release-plz.yml]` list contract; sources/command still literally name only `ci.yml`). This is intentional per the plan's precise task-2 scope, not an oversight — flagging so 123-03 remembers to update `sources`/`command` in the SAME commit that rewrites `quality/gates/code/ci-green-on-main.sh`, per the root CLAUDE.md "fix it twice" meta-rule (code changes without updating the row's descriptive fields would leave a stale citation).
- `quality/catalogs/agent-ux.json` has inconsistent JSON formatting across rows (some hand-edited rows use compact single-line arrays, most use one-item-per-line). Not a correctness issue (valid JSON either way) but it means a naive full-file `json.dump` rewrite is NOT safe for this file — any future automated catalog-editing tooling for `agent-ux.json` specifically should either preserve exact formatting via targeted string edits (as done here) or accept a one-time reformatting diff as a deliberate, reviewed change, never as a silent side effect of an unrelated row edit.
- Confirmed real `gh run list --workflow=release-plz.yml --branch=main --limit=8` output (8/8 success, 2026-07-18) rather than accepting the plan's cited evidence at face value — matches the 8/8 claim in the plan exactly, so no discrepancy found, but this is the kind of empirical claim that should always be independently re-verified rather than copy-pasted from a plan into a catalog's audit trail.

## Next Phase Readiness

- 123-02 (SC1 dotenv-sourcing verifier), 123-04 (SC2 downgrade-refusal verifier), 123-05 (SC3 write-lock verifier), 123-06 (SC4 verifier-script-exists verifier + pre-commit cadence promotion), and 123-03 (SC5a/SC5b implementation) all have a fixed, committed GREEN-contract row to build against, predating their own implementation.
- No blockers. Working tree clean; both task commits landed locally (not pushed — rides the phase-close push per this repo's cadence convention).
