---
phase: 123-quality-runner-catalog-integrity-hardening
plan: 05
subsystem: quality-gates
tags: [python, fcntl-flock, concurrency, catalog-json, quality-gates, unittest]

# Dependency graph
requires:
  - phase: 123-01
    provides: "catalog-first mint of structure/persist-catalog-write-locked (NOT-VERIFIED, P1, expected.asserts + claim_vs_assertion_audit)"
  - phase: 123-04
    provides: "quality/runners/_persist_guard.py (committed-GREEN downgrade guard) — the module this plan extends; the --persist critical section this plan wraps"
provides:
  - "_persist_guard.catalog_persist_lock(repo_root): an OS-level advisory flock over quality/reports/.persist.lock, serializing the whole per-catalog read-modify-write"
  - "run.py's --persist path holds the lock across load_catalog -> grade -> save_catalog; validate-only takes contextlib.nullcontext() and stays lock-free"
  - "a real-subprocess concurrency proof (TestPersistCatalogLock) + verifier greening structure/persist-catalog-write-locked to PASS"
affects: [quality-gates, structure-dimension, quality-runner-persist-path]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "advisory fcntl.flock(LOCK_EX) context manager wrapping a full read-modify-write to prevent a lost-update race (no timeout: kernel-released on SIGKILL)"
    - "conditional context manager (catalog_persist_lock vs contextlib.nullcontext) to make a critical section active only on the mutating path"
    - "real-subprocess concurrency test: a held-lock holder drops a sentinel, the test times its own acquire (>= 1.8s proves genuine cross-process OS exclusivity, not a mock)"

key-files:
  created:
    - quality/gates/structure/persist-catalog-write-locked.sh
  modified:
    - quality/runners/_persist_guard.py
    - quality/runners/run.py
    - quality/runners/test_run.py
    - quality/catalogs/freshness-invariants.json
    - .gitignore
    - quality/PROTOCOL.md

key-decisions:
  - "Lock scope is the FULL read-modify-write: acquired BEFORE original = load_catalog(cat_path) and released AFTER save_catalog(cat_path, data). A narrower lock (only around the write, or only around 123-04's committed_head_statuses read) leaves the lost-update window open — plan-check BLOCKER-2, proven by the failure-mode demo (6/6 no-lock runs lost an update)."
  - "One GLOBAL lock file (quality/reports/.persist.lock), acquired per-catalog inside the loop — accepts cross-catalog serialization (threat T-123-11) for correctness-first simplicity; --persist runs are infrequent + short."
  - "No timeout on flock (threat T-123-10 accept): the kernel releases an flock on process exit/crash including SIGKILL, so a crashed holder cannot wedge future runs. Documented in the module + PROTOCOL.md so a future reader does not add a timeout that reintroduces a race."
  - "Lock LOGIC lives in the _persist_guard.py sibling (run.py only gains the ~20-line conditional with:) so run.py's over-budget .py stays minimally grown, per the anti-bloat waiver guidance."
  - "Mint scoped to freshness-invariants.json (the row's own catalog, all structure-dimension/zero-cargo) because the full pre-push cadence's cargo+kcov gates exceed the 2min foreground budget; the catalog write for this row is byte-identical to a full run."

requirements-completed: [DRAIN-05]

# Metrics
duration: ~40min
completed: 2026-07-18
---

# Phase 123 Plan 05: Concurrent `--persist` Catalog-Write Lock (SC3 / DRAIN-05) Summary

**Two concurrent `run.py --persist` runners can no longer race-corrupt the shared catalog JSON — an OS-level advisory `flock` in `_persist_guard.catalog_persist_lock` wraps the whole per-catalog read-modify-write so the second writer's read cannot begin until the first's write commits; validate-only runs stay lock-free.**

## Performance

- **Duration:** ~40 min
- **Completed:** 2026-07-18
- **Tasks:** 2 (+ 1 fix-twice doc commit)
- **Files:** 6 modified + 1 created (the plan's 5 declared files + `.gitignore` for the lock artifact + `quality/PROTOCOL.md` fix-twice)

## Accomplishments

- **`_persist_guard.catalog_persist_lock(repo_root)`** — a `@contextlib.contextmanager` that `mkdir -p`s `quality/reports/`, opens `.persist.lock` append-binary, `fcntl.flock(..., LOCK_EX)` (blocking, no timeout), yields, and `LOCK_UN` + closes in `finally`. Module docstring now documents the SIGKILL-safety property so no future reader adds an unneeded timeout.
- **Wired into `run.py`'s per-catalog loop:** `persist_cm = catalog_persist_lock(REPO_ROOT) if args.persist else contextlib.nullcontext()`, and `with persist_cm:` spans `original = load_catalog(cat_path)` through `save_catalog(cat_path, data)`. The full read-modify-write (including 123-04's `committed_head_statuses` downgrade check) is inside the lock. A validate-only run takes the nullcontext branch and never opens or contends for the lock file.
- **`TestPersistCatalogLock` (4 cases, all real):** (1) a real subprocess holds the flock, the test times its own acquire at **>= 1.8s** wall-clock (proves genuine cross-process OS exclusivity, not a mock/in-process lock); (2) a validate-only `run.main()` completes without blocking while a separate process holds the lock, and never creates the lock file; (3) single-writer `--persist` minting is unchanged under the always-on lock; (4) two concurrent `run.py --persist` processes targeting the SAME catalog (one flipping only row-a via `--cadence pre-push`, the other only row-b via `pre-pr`) leave BOTH flips intact — valid/parseable JSON, no lost update.
- **Verifier `persist-catalog-write-locked.sh`** (Layer-A hermetic-unit-proof, mirrors `persist-refuses-downgrade.sh`): runs `TestPersistCatalogLock`, writes the standard JSON artifact, exit 0/1. Minted `structure/persist-catalog-write-locked` NOT-VERIFIED -> PASS through the real runner (F-K4b congruence + 123-04 guard path).
- **Reality check — failure mode proven (not just argued):** re-running case 4's two-process contention with `catalog_persist_lock` monkeypatched to a nullcontext lost an update in **6/6** runs (`{a:NV,b:PASS}` / `{a:PASS,b:NV}` — one writer's flip overwritten from a stale snapshot). WITH the lock the outcome is deterministically both-PASS. The lock is load-bearing, not decorative.
- **Full `test_run.py` suite 21/21 green** across all three run.py-touching plans (`TestPersistGate`, `TestPersistDowngradeGuard`, `TestPersistCatalogLock`, `TestEnvSelfSourcing`) — 123-04's downgrade guard still works.
- **Fix-twice:** `quality/PROTOCOL.md` now documents the concurrency lock beside the 123-04 downgrade guard (full-RMW scope, validate-only lock-free branch, deliberate no-timeout).

## Task Commits

1. **Task 1: advisory flock + run.py wiring + TestPersistCatalogLock + .gitignore** — `518c82d1` (feat)
2. **Task 2: verifier + mint `structure/persist-catalog-write-locked` GREEN** — `555d6362` (feat)
3. **Fix-twice: PROTOCOL.md concurrency-lock note** — `d2770656` (docs)

## Files Created/Modified
- `quality/runners/_persist_guard.py` — added `catalog_persist_lock`; module docstring now covers both persist-path guards
- `quality/runners/run.py` — `import contextlib`; per-catalog loop body wrapped in the conditional lock context
- `quality/runners/test_run.py` — `TestPersistCatalogLock` (4 real-concurrency cases)
- `quality/gates/structure/persist-catalog-write-locked.sh` — new Layer-A verifier
- `quality/catalogs/freshness-invariants.json` — row `structure/persist-catalog-write-locked` NOT-VERIFIED -> PASS
- `.gitignore` — ignore the generated `quality/reports/.persist.lock`
- `quality/PROTOCOL.md` — fix-twice concurrency-lock documentation

## Decisions Made
See `key-decisions` in frontmatter above.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical infra] The lock file was not gitignored**
- **Found during:** Task 1, before wiring — a real `--persist` run in the shared repo would leave an untracked `quality/reports/.persist.lock`.
- **Fix:** Added a `.gitignore` entry (the file is pure runtime coordination state, never evidence; harmless if stale).
- **Committed in:** `518c82d1`

**2. [Rule 3 - Blocking] Full `pre-push --persist` mint exceeded the 2min foreground budget**
- **Found during:** Task 2 mint — the prescribed `run.py --cadence pre-push --persist` invokes the cargo+kcov `code`/`agent-ux` gates and was SIGTERM'd at 2min without reaching persist (catalog left untouched — no corruption).
- **Fix:** Scoped the mint to `freshness-invariants.json` (the row's own catalog, zero-cargo) via a `discover_catalogs` override, using the same real runner grading/congruence/downgrade-guard path. Completed in ~15s; the catalog write for this row is byte-identical to a full run. Confirmed the diff touched ONLY this row (no collateral).
- **Artifact:** `git diff quality/catalogs/freshness-invariants.json` = 4 lines (status + last_verified).

---

**Total deviations:** 2 auto-fixed (1 missing-infra, 1 blocking-budget workaround). No architectural changes; no scope creep.

## Issues Encountered
- During test_4 authoring, the first draft's synthetic rows lacked `minted_at` — after the first writer flipped a row PASS (stamping a fresh `last_verified >= 2026-07-05`), the SECOND writer's `load_catalog` -> `_audit_field.validate_row` rejected the row for a missing `minted_at` anchor and crashed before writing, masquerading as a lock failure. Fixed by giving the fixture rows a `minted_at` + `claim_vs_assertion_audit` (as real rows carry), which also made the lost-update assertion genuinely meaningful. Not a product bug — a test-fixture realism gap.

## User Setup Required
None — POSIX `fcntl.flock` only; no external service, no credentials.

## Noticed (OD-3 #2)

- **`test_run.py` is now 35996 chars, over the 15000 `.py` ceiling** (WARN-only under the active `structure/file-size-limits` waiver; `run.py` is likewise over and pre-existing). The lock LOGIC was correctly factored into the `_persist_guard.py` sibling to keep run.py tight, but the test file grows with each TestCase class. Candidate refactor: split `test_run.py` into per-feature modules (`test_persist_gate.py` / `test_persist_downgrade.py` / `test_persist_lock.py` / `test_env_load.py`). Not eager-fixed — it is out of DRAIN-05's scope, waived until 2026-08-08, and would churn import paths across three plans' work. Recommend filing to GOOD-TO-HAVES for the milestone's absorption slot.
- **`123-04-SUMMARY.md` is absent** from the phase dir (siblings 01/02/03 have one). Sibling gap, not mine to fill — flagging for the coordinator's phase-close bookkeeping.
- **Cross-catalog serialization is coarse:** the single global `.persist.lock` serializes `--persist` runs even on DIFFERENT catalog files. Accepted (threat T-123-11) — a per-file lock is a legitimate future refinement, not required by SC3 ("a single locked persist lane" is a sanctioned design). No other unlocked shared-state write was found on the persist path; the read-modify-write is the only mutator.

## Next Phase Readiness
- `structure/persist-catalog-write-locked` (P1) grades PASS honestly against the committed catalog (verifier re-run exits 0).
- Sibling SC4 row `structure/verifier-script-exists` remains NOT-VERIFIED (its verifier lands in 123-06); a broad `--persist` still exits 1 on that, which is expected and NOT a downgrade (123-04's guard does not fire).
- No push performed (mid-phase; coordinator owns phase-close push + verifier dispatch + STATE/ROADMAP advancement).

## Self-Check: PASSED

All 7 touched files confirmed present on disk (`quality/gates/structure/persist-catalog-write-locked.sh` created; 6 modified). All 3 commit hashes (`518c82d1`, `555d6362`, `d2770656`) confirmed in `git log`. Full `test_run.py` suite 21/21 green; verifier exit 0 against committed state.

---
*Phase: 123-quality-runner-catalog-integrity-hardening*
*Completed: 2026-07-18*
