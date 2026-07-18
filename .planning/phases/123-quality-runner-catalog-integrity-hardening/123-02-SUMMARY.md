---
phase: 123-quality-runner-catalog-integrity-hardening
plan: 02
subsystem: testing
tags: [quality-gates, dotenv, real-backend, env-sourcing, python, egress-boundary]

# Dependency graph
requires:
  - phase: 123-01
    provides: "catalog-first GREEN-contract row structure/quality-runner-sources-dotenv (NOT-VERIFIED, minted 2026-07-18)"
provides:
  - "quality/runners/_env_load.py — load_dotenv_if_present(repo_root): present-only, non-clobbering (existing env wins), KEY-names-only stderr diagnostic"
  - "run.py main() self-sources ./.env before any catalog/real-backend gating"
  - "quality/gates/structure/quality-runner-sources-dotenv.sh verifier; row structure/quality-runner-sources-dotenv now PASS"
  - "docs fix-twice: .planning/CLAUDE.md + docs/reference/testing-targets.md note the runner self-sources .env"
affects: [123-04, 123-05, 123-06, milestone-close-real-backend-cadence]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Sibling-module anti-bloat: net-new runner logic lands in a _*.py sibling (mirrors _freshness/_realbackend/_audit_field/_shell_verdict), keeping run.py off further growth past its ceiling"
    - "os.environ.setdefault precedence: existing (operator/CI) env always wins over ./.env per key"
    - "Secret-hygiene diagnostic: emit KEY names only, never values (env_keys convention, quality/CLAUDE.md)"

key-files:
  created:
    - quality/runners/_env_load.py
    - quality/gates/structure/quality-runner-sources-dotenv.sh
  modified:
    - quality/runners/run.py
    - quality/runners/test_run.py
    - quality/catalogs/freshness-invariants.json

key-decisions:
  - "EXISTING-ENV-WINS per-key (os.environ.setdefault), a strict superset of preflight's file-level guard"
  - "import _env_load + _env_load.load_dotenv_if_present(REPO_ROOT) (module-import form) to satisfy the plan's machine-checkable key_link pattern _env_load\\.load_dotenv_if_present and the dominant 3/4 sibling convention"
  - "Did NOT run gsd-sdk state.advance-plan (documented-unsafe on this repo's narrative STATE.md; SURPRISES-INTAKE 2026-07-18 11:09) — STATE.md left to the coordinator, mirroring 123-01"

patterns-established:
  - "Runner .env self-sourcing: conditional, present-only, non-clobbering, no value leak"

requirements-completed: [DRAIN-03]

# Metrics
duration: ~30min
completed: 2026-07-18
---

# Phase 123 Plan 02: run.py `.env` self-sourcing (SC1 / DRAIN-03) Summary

**`quality/runners/run.py` now conditionally self-sources `./.env` (present-only, non-clobbering, KEY-names-only diagnostic) via a new `_env_load.py` sibling, closing the false-green-preflight gap where a `pre-release-real-backend` cadence silently skipped every real-backend row to NOT-VERIFIED unless the caller had pre-sourced `.env`.**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-07-18T11:00Z (approx)
- **Completed:** 2026-07-18T11:31Z
- **Tasks:** 2 (Task 1 TDD RED→GREEN, Task 2 docs fix-twice)
- **Files modified:** 7 (2 created, 5 modified)

## Accomplishments

- New `quality/runners/_env_load.py::load_dotenv_if_present(repo_root)`: parses `./.env` (skips blanks/`#`-comments/malformed lines, strips matched surrounding quotes), applies via `os.environ.setdefault` so an already-exported var always wins, and emits ONE stderr line naming loaded KEY names only.
- Wired as the FIRST statement in `run.py::main()`, before any catalog load or real-backend gating — a harmless no-op for cadences that never consult real-backend creds and for CI (no `.env`).
- New verifier `quality/gates/structure/quality-runner-sources-dotenv.sh` (Layer A hermetic unit proof); catalog row `structure/quality-runner-sources-dotenv` minted **NOT-VERIFIED → PASS**.
- Fix-twice: both `.planning/CLAUDE.md` (9th-probe bullet) and `docs/reference/testing-targets.md` now document the runner self-sourcing (manual `set -a; . ./.env; set +a` no longer required) and reaffirm the unchanged OP-1 egress gate.

## Task Commits

1. **Task 1 (RED): failing TestEnvSelfSourcing** - `7a002495` (test)
2. **Task 1 (GREEN): _env_load impl + run.py wiring + verifier + row mint** - `a99d87cb` (feat)
3. **Task 2: docs fix-twice** - `b9f9b445` (docs)

**Plan metadata:** this SUMMARY + ROADMAP update land in the final metadata commit below.

_TDD gate compliance: `test(...)` (RED) precedes `feat(...)` (GREEN); no REFACTOR needed._

## Files Created/Modified

- `quality/runners/_env_load.py` (created) - conditional, present-only, non-clobbering `.env` sourcing; KEY-names-only diagnostic.
- `quality/gates/structure/quality-runner-sources-dotenv.sh` (created) - verifier; runs the hermetic unittest, emits 3 `asserts_passed` that token-map the row's 3 `expected.asserts` (F-K4b congruence).
- `quality/runners/run.py` (modified) - `import _env_load` + `_env_load.load_dotenv_if_present(REPO_ROOT)` as the first line of `main()`.
- `quality/runners/test_run.py` (modified) - added `TestEnvSelfSourcing` (4 hermetic tempfile cases).
- `quality/catalogs/freshness-invariants.json` (modified) - row `structure/quality-runner-sources-dotenv` status NOT-VERIFIED→PASS, last_verified null→timestamp (the ONLY status change; zero collateral catalogs).

## Decisions Made

- **Env precedence = existing-env-wins, per key** (`os.environ.setdefault`). An operator- or CI-exported credential is never overridden by a stale `.env`. This is a strict per-key superset of `scripts/preflight-real-backends.sh`'s file-level `[ -z "${CRED:-}" ]` guard, and matches the row's expected asserts + the non-clobber unit test.
- **Module-import call form** (`import _env_load` + `_env_load.load_dotenv_if_present(...)`) rather than the plan-prose `from _env_load import ...`, to satisfy the plan's machine-checkable `key_link` pattern (`_env_load\.load_dotenv_if_present`) AND the dominant sibling convention (3/4 siblings import as `import X`). Behaviour identical.
- **STATE.md left untouched** — `gsd-sdk query state.advance-plan` is documented-unsafe on this repo's narrative STATE.md (SURPRISES-INTAKE 2026-07-18 11:09: it parse-fails yet corrupts frontmatter). Mirrored 123-01: no STATE.md commit; coordinator owns the phase cursor.

## Deviations from Plan

Plan executed as written, with one intentional interface choice (documented above):

### Adjustments

**1. [Plan-contract reconciliation] Module-import call form over the prose import form**
- **Found during:** Task 1 (wiring `_env_load` into run.py)
- **Issue:** The plan action prose said `from _env_load import load_dotenv_if_present`, but the plan's own machine-checkable `key_links.pattern` (`_env_load\.load_dotenv_if_present`) requires a qualified `_env_load.` call site.
- **Fix:** Used `import _env_load` + `_env_load.load_dotenv_if_present(REPO_ROOT)` — satisfies the stronger machine contract and matches 3/4 existing sibling imports. Behaviour identical.
- **Files modified:** quality/runners/run.py
- **Verification:** live `run.py --cadence pre-commit` exit 0 against the real `.env`; row minted PASS.
- **Committed in:** a99d87cb

---

**Total deviations:** 1 interface reconciliation (no scope creep, no architectural change).
**Impact on plan:** None negative — closes the SC1 gap exactly as specified.

## Issues Encountered

None during planned work. The pre-push `--persist` mint exited 1 (the 4 sibling SC-rows without verifiers grade NOT-VERIFIED) — this was pre-declared EXPECTED by the plan; my row minted PASS and no other catalog changed.

## Threat Flags

None. The single trust boundary (`.env` file → process environment) is exactly the one in the plan's `<threat_model>` (T-123-03 mitigated by KEY-names-only logging, proven by Test 4; T-123-04 accepted — `.env` is gitignored/operator-authored). No new network endpoint, auth path, or schema surface introduced.

## Self-Check

- `quality/runners/_env_load.py` FOUND.
- `quality/gates/structure/quality-runner-sources-dotenv.sh` FOUND (executable, exit 0 standalone).
- `quality/catalogs/freshness-invariants.json` — row `structure/quality-runner-sources-dotenv` status confirmed `PASS`, last_verified `2026-07-18T11:25:23Z`.
- Commit `7a002495` (test) FOUND in git log.
- Commit `a99d87cb` (feat) FOUND in git log.
- Commit `b9f9b445` (docs) FOUND in git log.
- Reality proof: `TestEnvSelfSourcing` 4/4 pass; full `test_run` 8/8 pass; with-`.env` fixture loads the var (visible in os.environ, quotes stripped) while stderr names the KEY but never the fake value; without-`.env` = silent no-op (no exception, env unchanged).

## Self-Check: PASSED

## User Setup Required

None - no external service configuration required. (Operators who want the milestone-close real-backend cadence to exercise live backends now only need creds in `./.env` + a non-default `REPOSIX_ALLOWED_ORIGINS`; no manual shell pre-sourcing.)

## Noticed (OD-3 ownership charter)

- **run.py is 25242 chars, over the `.py` file-size ceiling (15000), WAIVED until 2026-08-08.** Pre-existing (not introduced here) — my change added only ~8 lines to run.py by design (the ~50-line impl lives in the `_env_load.py` sibling, exactly the anti-bloat rationale the plan cites). Flagging because the waiver lapses 2026-08-08; run.py will need a real extraction pass before then.
- **The `state.advance-plan` SDK hazard (SURPRISES-INTAKE 2026-07-18 11:09) is still OPEN**, and its fix-twice doc-update (documenting the hazard in `.planning/CLAUDE.md`/`ORCHESTRATION.md`) is NOT yet landed. I hit and avoided the same trap; not implementing the doc-update here (out of SC1 scope; already filed) but re-surfacing so the drain/close wave lands it.
- **Adding a new never-exercised `.sh` verifier drags the kcov `code/shell-coverage` aggregate** (new script counted at 0%). It did NOT affect THIS mint run (the verifier was untracked, so `git ls-files` excluded it), but the close-wave pre-push WILL include it once committed. Floor is a low 13% with ample headroom (this mint's full pre-push run showed 0 FAIL), so no block expected — flagging so the close wave isn't surprised if the aggregate ticks down a fraction.
- **Every commit now emits `run.py: sourced N var(s) from ./.env: ...` on stderr** (a real `.env` is present locally, and the pre-commit hook runs run.py). Names-only, non-blocking, sanctioned by the `env_keys` convention — intentional transparency, but worth knowing it appears in every local commit's pre-commit output.

## Next Phase Readiness

- SC1/DRAIN-03 closed and green. 123-04 (SC2), 123-05 (SC3), 123-06 (SC4) each still have their catalog-first row (NOT-VERIFIED) awaiting their verifiers — unblocked by this plan.
- No blockers. Working tree clean; all commits local (not pushed — rides the phase-close push per this repo's cadence).

---
*Phase: 123-quality-runner-catalog-integrity-hardening*
*Completed: 2026-07-18*
