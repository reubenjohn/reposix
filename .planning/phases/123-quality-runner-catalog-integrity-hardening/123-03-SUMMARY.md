---
phase: 123-quality-runner-catalog-integrity-hardening
plan: 03
subsystem: quality-gates
tags: [gh-cli, github-actions, bash, catalog-json, git-stderr, quality-gates]

# Dependency graph
requires:
  - phase: 123-01
    provides: "catalog-first rewrite of code/ci-green-on-main (required-workflow-list contract, status NOT-VERIFIED, timeout_s 90) and the new real-stderr assertion on agent-ux/t4-conflict-rebase-ancestry-real-backend"
provides:
  - "code/ci-green-on-main.sh watches [ci.yml, release-plz.yml] instead of a single hardcoded workflow, with NOT-VERIFIED-beats-FAIL aggregation"
  - "lib/t4-real-backend-flow.sh surfaces real captured git checkout stderr instead of a hardcoded, always-false git-version fallback"
  - "a hermetic /tmp selftest proving the SC5b fix without real Confluence credentials"
affects: [quality-gates, code-dimension, agent-ux-dimension]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "required-workflow LIST aggregation for post-push CI-green probes (NOT-VERIFIED outranks FAIL)"
    - "env -u GH_TOKEN -u GITHUB_TOKEN before gh CLI calls in a verifier, so a stale .env REST bearer token can never override the operator's gh auth session"
    - "extracted _t4_checkout_or_fail helper + hermetic /tmp selftest for a sourced-only lib fn"

key-files:
  created:
    - quality/gates/agent-ux/lib/t4-real-backend-flow.selftest.sh
  modified:
    - quality/gates/code/ci-green-on-main.sh
    - quality/gates/agent-ux/lib/t4-real-backend-flow.sh
    - quality/catalogs/code.json
    - quality/CLAUDE.md

key-decisions:
  - "WORKFLOWS=(ci.yml release-plz.yml) is the required-workflow list -- the only two confirmed to fire unconditionally on push:branches:[main] with no path filter; audit.yml/docs.yml/quality-post-release.yml deliberately excluded"
  - "Aggregation priority: NOT-VERIFIED (any watched workflow unknowable) always outranks FAIL (a red workflow) when rolling up per-workflow verdicts"
  - "Rule 1 fix: gh calls run under env -u GH_TOKEN -u GITHUB_TOKEN so this row's own documented trust boundary ('authenticated via the operator's own gh auth') is actually enforced, not incidentally true"
  - "Scoped the SC5b fix to the real-backend arm only (lib/t4-real-backend-flow.sh) -- the sim-arm sibling t4-conflict-rebase-ancestry.sh is a different bug class and stays untouched"

requirements-completed: [DRAIN-01, DRAIN-10]

# Metrics
duration: ~25min
completed: 2026-07-18
---

# Phase 123 Plan 03: Quality Runner Catalog Integrity Hardening (SC5a/SC5b) Summary

**`code/ci-green-on-main` now watches a required-workflow list (ci.yml + release-plz.yml) with honest NOT-VERIFIED/FAIL aggregation, and the t4 real-backend checkout helper surfaces real git stderr instead of a hardcoded, always-false git-version claim.**

## Performance

- **Duration:** ~25 min
- **Completed:** 2026-07-18
- **Tasks:** 2 (+ 1 fix-twice doc commit, + 1 file-size trim commit)
- **Files modified:** 5 (4 declared + `quality/CLAUDE.md` per the plan's fix-twice directive)

## Accomplishments

- **SC5a (DRAIN-01):** `quality/gates/code/ci-green-on-main.sh` loops over `WORKFLOWS=("ci.yml" "release-plz.yml")` instead of a single hardcoded `WORKFLOW="ci.yml"`. Aggregation: any unknowable workflow (gh missing/unauth, no run found, in-progress) -> overall NOT-VERIFIED naming which; else any red workflow -> overall FAIL naming which and its conclusion; else PASS. NOT-VERIFIED outranks FAIL when both occur. The artifact JSON now carries a `workflows` object with each watched workflow's raw verdict.
- **The "no run for this workflow" edge case** is handled as an honest NOT-VERIFIED (never a silent PASS, never a hang) — verified with a mocked `gh` returning `[]` for `release-plz.yml`.
- **Closed Wave-1 noticing #1:** the row's `sources`/`command` fields in `quality/catalogs/code.json` were lagging its `expected.asserts` by one wave — updated in the same commit to describe the loop/list-watching script, and the stale "until plan 123-03 lands" comment sentence was corrected to reflect landing.
- **Rule 1 fix (found running the row through `run.py`):** `run.py` self-sources `./.env` (P123/DRAIN-03), which sets `GITHUB_TOKEN` — a REST bearer PAT meant for other real-backend rows. `gh` prioritizes that env var over the operator's `gh auth login` session, and the `.env` token is invalid (`HTTP 401: Bad credentials`), which flipped the row to NOT-VERIFIED under `run.py` even though the bare script showed real PASS. Fixed by running `gh run list` under `env -u GH_TOKEN -u GITHUB_TOKEN` in the verifier, matching the row's own documented trust boundary. Re-minted PASS afterward.
- **SC5b (DRAIN-10):** extracted `_t4_checkout_or_fail <tree> <label>` in `quality/gates/agent-ux/lib/t4-real-backend-flow.sh`, replacing the two `git checkout -B main ... || hard_fail_exit "..." "requires git >= 2.34 stateless-connect fetch"` call sites. The helper captures ONLY stderr (`2>&1 >/dev/null`) and passes it as the failure detail — real, actionable text (e.g. the Confluence oid-drift class) instead of a claim the caller already disproved (git version is gated upstream in the real-backend caller script).
- New hermetic selftest `t4-real-backend-flow.selftest.sh`: a throwaway `/tmp` `git init` repo (no `reposix init`/sim-seed — leaf-isolation-guard is a no-op) with no `refs/reposix/origin/main` ref, a stub `hard_fail_exit` recording args, asserting (a) it was invoked, (b) the detail matches `error:|fatal:|pathspec`, (c) the detail does NOT contain `"requires git >= 2.34"`. All 3 assertions pass.
- Confirmed via `git diff --stat` that the sim-arm sibling `quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh` is untouched.
- **Fix-twice:** added a short pointer in `quality/CLAUDE.md`'s code-dimension section documenting the required-workflow-list convention and its aggregation rule, so a future contributor adding a third workflow knows to re-verify its trigger shape first.

## Task Commits

1. **Task 1: SC5a — parameterize ci-green-on-main.sh into a required-workflow list** - `5ad70863` (feat)
2. **Trim:** file-size ceiling cleanup on the same file - `e5ad4241` (style)
3. **Task 2: SC5b — t4 real-backend-flow surfaces real checkout stderr** - `adb51c0c` (fix)
4. **Fix-twice:** `quality/CLAUDE.md` convention note - `8a6dde2a` (docs)

## Files Created/Modified
- `quality/gates/code/ci-green-on-main.sh` - required-workflow list + `env -u GH_TOKEN -u GITHUB_TOKEN` isolation
- `quality/catalogs/code.json` - `code/ci-green-on-main` row `sources`/`command` updated to match; status now PASS
- `quality/gates/agent-ux/lib/t4-real-backend-flow.sh` - new `_t4_checkout_or_fail` helper; two call sites replaced
- `quality/gates/agent-ux/lib/t4-real-backend-flow.selftest.sh` - new hermetic selftest (3/3 assertions pass)
- `quality/CLAUDE.md` - fix-twice convention pointer

## Decisions Made
See `key-decisions` in frontmatter above.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `gh run list` calls in `ci-green-on-main.sh` silently trusted an ambient `GITHUB_TOKEN`, contradicting the row's own documented trust boundary**
- **Found during:** Task 1, minting via `python3 quality/runners/run.py --cadence post-push --persist`
- **Issue:** `run.py` self-sources `./.env` (a prior-phase feature, P123/DRAIN-03), setting `GITHUB_TOKEN` in the subprocess environment. `gh` CLI prioritizes `GH_TOKEN`/`GITHUB_TOKEN` over its stored keyring auth. The `.env` token is currently invalid (`HTTP 401: Bad credentials`), so running the verifier through `run.py` produced NOT-VERIFIED even though a bare invocation showed real PASS. This is a correctness gap independent of whether the token is valid today: the row's threat model claims `gh run list` output is "authenticated GitHub API via the operator's own `gh auth`", which the script did not actually enforce.
- **Fix:** Wrapped the `gh run list` invocation in `env -u GH_TOKEN -u GITHUB_TOKEN`, mirroring the isolation precedent in `quality/gates/agent-ux/real-backend-env-gate.selftest.sh`.
- **Files modified:** `quality/gates/code/ci-green-on-main.sh`
- **Verification:** Re-ran `bash quality/gates/code/ci-green-on-main.sh` and `GITHUB_TOKEN=<bad-value> bash quality/gates/code/ci-green-on-main.sh` — both PASS. Re-minted via `run.py --cadence post-push --persist` -> row status PASS.
- **Committed in:** `5ad70863` (Task 1 commit)

**2. [Rule 2 - Missing polish] File-size ceiling exceeded on first draft**
- **Found during:** Task 1 pre-commit hook (WARN, non-blocking under the active `structure/file-size-limits` waiver)
- **Issue:** The expanded header comment pushed `ci-green-on-main.sh` to 10141 chars against the 10000 `.sh` ceiling.
- **Fix:** Tightened the WHY/AGGREGATION prose and the Rule-1-fix inline comment without dropping rationale; re-verified PASS afterward.
- **Files modified:** `quality/gates/code/ci-green-on-main.sh`
- **Committed in:** `e5ad4241`

---

**Total deviations:** 2 auto-fixed (1 bug/trust-boundary gap, 1 file-size polish)
**Impact on plan:** Both fixes necessary for the row to actually grade PASS honestly under the real minting path; no scope creep beyond the plan's two success criteria.

## Issues Encountered
None beyond the deviations above.

## User Setup Required
None - no external service configuration required. The SC5b row (`agent-ux/t4-conflict-rebase-ancestry-real-backend`) remains env-gated NOT-VERIFIED pending real Confluence credentials, per design (not a gap introduced by this plan).

## Noticed (OD-3 #2)

- The `.env`'s `GITHUB_TOKEN` currently returns `HTTP 401: Bad credentials` against the GitHub API. This is out of my scope to regenerate, but it means ANY row that relies on `run.py`'s self-sourced `GITHUB_TOKEN` for a real `gh`/REST call (e.g. `github-front-door-real-backend.sh`, `perf/latency-bench/github.sh`) would currently fail/skip for the same reason. Worth a fresh credential rotation before those rows are next exercised. Not filed as a SURPRISES-INTAKE entry since it's a transient credential-freshness issue, not a code/process gap — flagging here for visibility.
- `quality/gates/agent-ux/real-git-push-e2e.sh:143` carries a similarly-shaped hardcoded git-version fallback string (`"requires git >= 2.34 stateless-connect fetch to have populated refs/reposix/origin/main"`). Out of scope for DRAIN-10 (which named only `lib/t4-real-backend-flow.sh`'s two call sites) — not touched. Whether it shares the same "caller already gated git >= 2.34" property would need its own investigation before applying the same fix there.

## Next Phase Readiness
- `code/ci-green-on-main` (P0) now grades PASS honestly against the real main tip, closing DRAIN-01/SC5a.
- `agent-ux/t4-conflict-rebase-ancestry-real-backend`'s stderr-honesty assertion (added in 123-01) is now structurally satisfied and proven by a hermetic selftest; the row itself stays NOT-VERIFIED until real Confluence creds are supplied at a pre-release-real-backend cadence run — no blocker for this plan.
- Sibling waves (123-02, 04, 05, 06) touch disjoint files per the coordinator's routing note; no merge-conflict risk observed.

## Self-Check: PASSED

All 5 modified/created files confirmed present on disk; all 4 commit hashes
(`5ad70863`, `e5ad4241`, `adb51c0c`, `8a6dde2a`) confirmed in `git log --oneline --all`.

---
*Phase: 123-quality-runner-catalog-integrity-hardening*
*Completed: 2026-07-18*
