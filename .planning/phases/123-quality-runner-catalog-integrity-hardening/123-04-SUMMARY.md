---
phase: 123-quality-runner-catalog-integrity-hardening
plan: 04
subsystem: quality-gates
tags: [python, subprocess, git-show, catalog-json, quality-gates, unittest]

# Dependency graph
requires:
  - phase: 123-01
    provides: "catalog-first mint of structure/persist-refuses-downgrade (NOT-VERIFIED, P1, expected.asserts + claim_vs_assertion_audit)"
  - phase: 123-02
    provides: "run.py's persist branch + arg parser this plan extends (shares run.py + freshness-invariants.json with 123-02, hence the wave-3 sequential bump)"
provides:
  - "quality/runners/_persist_guard.py: committed_head_statuses(repo_root, cat_path) -> dict[str,str]|None (reads the LAST COMMITTED catalog via `git show HEAD:<path>`)"
  - "_persist_guard.refuse_downgrade_without_flag(committed, new_rows) -> list[(row_id, old_status, new_status)] (the explicit-regression detector)"
  - "run.py --allow-downgrade CLI flag (store_true, default False) restoring the pre-guard unconditional-write behavior with a loud per-row notice"
affects: [quality-gates, structure-dimension, quality-runner-persist-path]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "git show HEAD:<path> subprocess read as the committed-baseline oracle (non-zero exit / no-such-path treated as 'no baseline', not an error)"
    - "regression-vs-TTL semantics: the guard fires ONLY on an EXPLICIT {PASS,WAIVED}->{FAIL,PARTIAL} transition; a demotion to NOT-VERIFIED (freshness-TTL, missing-verifier, env-skip, exit-75) is never a downgrade and is always allowed unconditionally, preventing a deadlock against the phase's own freshness-invariant mints"
    - "throwaway /tmp git-init fixture per test case (never the shared repo) to produce a real git-committed baseline for the `git show HEAD:` oracle"

key-files:
  created:
    - quality/gates/structure/persist-refuses-downgrade.sh
  modified:
    - quality/runners/_persist_guard.py
    - quality/runners/run.py
    - quality/runners/test_run.py
    - quality/catalogs/freshness-invariants.json
    - quality/PROTOCOL.md

key-decisions:
  - "Guard fires ONLY on the explicit {FAIL,PARTIAL} transition from a committed {PASS,WAIVED} baseline — a transition to NOT-VERIFIED is NEVER a violation regardless of cause, because NOT-VERIFIED is this project's designed-in 'row went stale / couldn't be graded' channel, not a regression. Distinguishing 'legitimate TTL NOT-VERIFIED' from 'other-cause NOT-VERIFIED' via transient flags (`_stale` etc.) was explicitly rejected — those flags are popped before persistence and unavailable at the guard's call site; status value alone is sufficient and correct."
  - "A brand-new row absent from the git-HEAD committed catalog (e.g. straight out of 123-01's catalog-first commit) has no baseline and mints freely — `committed_head_statuses` returns None on a non-zero `git show` exit, and `refuse_downgrade_without_flag` short-circuits to `[]` when `committed is None`."
  - "`--allow-downgrade` restores the write but still prints a loud 'ALLOWED downgrade' notice per row — the override is never silent, matching the refusal's teaching style."
  - "A blocked downgrade sets `exit_code = max(exit_code, 1)` — a refused write always surfaces as a failing run, never silently swallowed into an otherwise-green exit."
  - "Guard logic lives in the `_persist_guard.py` sibling module (mirroring `_env_load.py`'s anti-bloat precedent from 123-02) so `run.py` only gains a small wiring block rather than growing the guard logic inline."

requirements-completed: [DRAIN-04]

# Metrics
duration: ~22min
completed: 2026-07-18
---

# Phase 123 Plan 04: `--persist` Committed-GREEN Downgrade Guard (SC2 / DRAIN-04) Summary

**`--persist` now refuses to silently rewrite a committed-GREEN (`PASS`/`WAIVED`) catalog row to an explicit regression (`FAIL`/`PARTIAL`) without an explicit `--allow-downgrade` opt-in — closing the exact near-miss where a rotation's `--persist` run downgraded `vision-litmus` PASS→FAIL on an env-skip false negative and was caught only by a manual `git restore` before staging.**

## Performance

- **Duration:** ~22 min
- **Completed:** 2026-07-18
- **Tasks:** 2 (+ 1 fix-twice doc commit)
- **Files:** 4 modified + 1 created

## Accomplishments

- **`_persist_guard.committed_head_statuses(repo_root, cat_path)`** — reads the LAST COMMITTED version of a catalog via `git show HEAD:<relative_path>`; a non-zero exit (path absent from HEAD — a brand-new catalog file) returns `None`, treated as "no baseline" rather than an error.
- **`_persist_guard.refuse_downgrade_without_flag(committed, new_rows)`** — for each fresh row, flags a violation only when the row id existed in the committed baseline at `PASS`/`WAIVED` AND the fresh grade is `FAIL`/`PARTIAL` (an EXPLICIT regression, not a mere status-unknown transition). Returns the empty list when `committed is None`.
- **Wired into `run.py`'s persist branch**, immediately before each `save_catalog()` call: computes `committed`/`violations`, and when `violations` is non-empty and `--allow-downgrade` is absent, refuses to persist that catalog file, prints one `REFUSED to persist {row_id}: committed status was {old}, this run graded {new}. Pass --allow-downgrade to override: ...` line per violation, and forces the run's exit code non-zero. With `--allow-downgrade` set, the write proceeds but still prints a loud `ALLOWED downgrade (--allow-downgrade): {row_id} {old} -> {new}` notice per row — never silent either way.
- **New `--allow-downgrade` CLI flag** (`store_true`, default `False`) added to `_build_arg_parser()`, mirroring the existing `--persist` flag's help-text style.
- **`TestPersistDowngradeGuard`** (9 unit tests: the 6 named behavior cases from the plan — blocked-without-flag, allowed-with-flag, WAIVED-counts-as-green, brand-new-row-never-blocked, no-change-not-blocked, NOT-VERIFIED-transition-never-blocked — plus the P2/exit-code-forcing case, the `committed is None` short-circuit, and the flag's default-off introspection test) — each builds a throwaway `/tmp` git-init fixture (never the shared repo) to produce a real `git show HEAD:` baseline, then exercises `_persist_guard`'s functions directly.
- **Verifier `quality/gates/structure/persist-refuses-downgrade.sh`** — Layer-A hermetic-unit-proof (mirrors `catalog-immutable-on-read.sh`): runs `TestPersistDowngradeGuard`, writes the standard JSON artifact, exit 0/1. Minted the row `structure/persist-refuses-downgrade` NOT-VERIFIED → PASS via the real runner (`run.py --cadence pre-push --persist`, scoped write — diff touched exactly this row's status + `last_verified`).
- **Full `test_run.py` regression check**: the pre-existing `TestPersistGate` suite (brand-new-row-with-no-baseline mint path — the exact shape of 123-01's own catalog-first rows on their first real mint) still passes unchanged under the new guard.
- **Fix-twice**: `quality/PROTOCOL.md` documents the downgrade guard (git-HEAD baseline mechanism, the teaching refusal + `--allow-downgrade` recovery command, and the NOT-VERIFIED-is-never-a-downgrade deadlock-prevention rule) beside the existing D-P96-01 GRADE/PERSIST split description.

## Task Commits

1. **Task 1: implement the downgrade guard (`_persist_guard.py` + `run.py` wiring + `TestPersistDowngradeGuard`)** — `584b6691` (feat)
2. **Task 2: verifier + mint `structure/persist-refuses-downgrade` PASS** — `f19ad6f5` (feat)
3. **Fix-twice: PROTOCOL.md downgrade-guard documentation** — `dffd6966` (docs)

## Files Created/Modified
- `quality/runners/_persist_guard.py` — new sibling module: `committed_head_statuses` + `refuse_downgrade_without_flag`
- `quality/runners/run.py` — `--allow-downgrade` flag; persist-branch wiring calling the guard before each `save_catalog()`
- `quality/runners/test_run.py` — `TestPersistDowngradeGuard` (9 cases)
- `quality/gates/structure/persist-refuses-downgrade.sh` — new Layer-A verifier
- `quality/catalogs/freshness-invariants.json` — row `structure/persist-refuses-downgrade` NOT-VERIFIED → PASS
- `quality/PROTOCOL.md` — fix-twice downgrade-guard documentation

## Decisions Made
See `key-decisions` in frontmatter above.

## Deviations from Plan
None — plan executed exactly as written; all 6 behavior cases plus the 3 supplementary cases (P2 exit-forcing, `None`-baseline short-circuit, flag-default introspection) landed as specified.

## Issues Encountered
None blocking. The intermediate `run.py --cadence pre-push --persist` mint run's overall exit 1 (expected — sibling SC3/SC4 verifiers from later waves still grade NOT-VERIFIED at this point in the phase) was anticipated by the plan and did not require investigation.

## User Setup Required
None — pure Python stdlib (`subprocess`, `json`) + `git show`; no external service, no credentials.

## Noticed (OD-3)

- **`123-04-SUMMARY.md` itself was missing** from the phase directory until this backfill (123-07 close-wave Task 2) — siblings 123-01/02/03/05/06 each had one; this plan's own close-out step was skipped at execution time. Backfilled now from the real committed diffs (`584b6691`, `f19ad6f5`, `dffd6966`) rather than from memory.
- **`test_run.py` growth**: this plan's 9 new tests, stacked on top of 123-02's `TestEnvSelfSourcing` and followed by 123-05's `TestPersistCatalogLock`, are the proximate cause of `test_run.py` crossing 2.5x its `.py` file-size ceiling — filed as `GTH-V15-83` in the 123-07 close wave (not this plan's scope to fix retroactively).

## Next Phase Readiness
- `structure/persist-refuses-downgrade` (P1) grades PASS honestly against the committed catalog.
- 123-05 (SC3/DRAIN-05, the concurrency lock) directly extends `_persist_guard.py` with `catalog_persist_lock`, wrapping the SAME read-modify-write this plan's guard sits inside — confirmed compatible (123-05's `TestPersistCatalogLock` suite runs green alongside this plan's tests in the full `test_run.py` suite).
- No push performed (mid-phase; coordinator owns phase-close push + verifier dispatch + STATE/ROADMAP advancement).

## Self-Check: PASSED

All 5 touched files confirmed present on disk (`quality/gates/structure/persist-refuses-downgrade.sh` created; 4 modified). All 3 commit hashes (`584b6691`, `f19ad6f5`, `dffd6966`) confirmed in `git log`.

---
*Phase: 123-quality-runner-catalog-integrity-hardening*
*Completed: 2026-07-18*
