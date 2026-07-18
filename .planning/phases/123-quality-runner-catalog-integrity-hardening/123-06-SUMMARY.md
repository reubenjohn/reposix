---
phase: 123-quality-runner-catalog-integrity-hardening
plan: 06
subsystem: quality-gates
tags: [python3, bash, json, structure-dimension, catalog-integrity]

# Dependency graph
requires:
  - phase: 123-01
    provides: "the GREEN-contract catalog row structure/verifier-script-exists (NOT-VERIFIED, cadences [pre-push, pre-pr])"
provides:
  - "quality/gates/structure/verifier-script-exists.sh — scans every quality/catalogs/*.json row's verifier.script for existence + executable bit"
  - "quality/gates/structure/verifier-script-exists.selftest.sh — throwaway /tmp fixture proving all 3 violation classes + the pass path"
  - "32 pre-existing missing-chmod-+x violations fixed across 13 real verifier scripts"
  - "SURPRISES-INTAKE entry documenting 5 pre-existing, already-tracked-elsewhere stub-verifier violations this gate cannot fix in-scope"
affects: [123-close, v0.15.0-milestone-close, structure-dimension gates]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "python3 -c real-JSON-parsing gate (not grep) for catalog-wide structural audits, mirroring file-size-limits.sh's git-ls-files control flow"
    - "emit_artifact-style standard JSON artifact write (ts/row_id/exit_code/status/asserts_passed/asserts_failed) matching catalog-immutable-on-read.sh precedent"

key-files:
  created:
    - quality/gates/structure/verifier-script-exists.sh
    - quality/gates/structure/verifier-script-exists.selftest.sh
  modified:
    - quality/catalogs/freshness-invariants.json
    - quality/CLAUDE.md
    - .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md
    - quality/gates/agent-ux/cadence-pre-release-real-backend.sh (chmod +x only)
    - quality/gates/agent-ux/fleet-safety-tat-identity-reject.sh (chmod +x only)
    - quality/gates/agent-ux/fleet-safety-leaf-isolation-enforce.sh (chmod +x only)
    - quality/gates/agent-ux/fleet-safety-shared-config-write-guard.sh (chmod +x only)
    - quality/gates/docs-repro/snippet-extract.py (chmod +x only)
    - quality/gates/structure/freshness-invariants.py (chmod +x only)
    - quality/gates/perf/bench_token_economy.py (chmod +x only)
    - quality/gates/perf/headline-numbers-cross-check.py (chmod +x only)
    - quality/gates/release/installer-asset-bytes.py (chmod +x only)
    - quality/gates/release/brew-formula-current.py (chmod +x only)
    - quality/gates/release/crates-io-max-version.py (chmod +x only)
    - quality/gates/release/gh-assets-present.py (chmod +x only)
    - quality/gates/release/cargo-binstall-resolves.py (chmod +x only)

key-decisions:
  - "Did NOT promote structure/verifier-script-exists's cadences to include pre-commit: the gate is not clean against the live catalog (5 pre-existing, already-tracked-elsewhere violations remain), and a dirty pre-commit-tagged P1 row would self-block every future commit repo-wide. Cadences stay [pre-push, pre-pr] as minted in 123-01."
  - "The gate scans every non-docs-alignment row's verifier.script unconditionally, with no status==PASS scoping — matches the row's committed claim_vs_assertion_audit exactly; a status-scoped narrowing would be an unauthorized silent design change, so it was NOT applied even though it would have let the row go green today."
  - "Did not fabricate stub verifier scripts for the 5 structural violations (2 WAIVED cross-platform rehearsal rows, docs-build/animation-renders, 2 docs-repro/benchmark-claim rows) just to satisfy the mechanical existence check — all 5 are honest, deliberate, catalog-first placeholders already tracked on other timelines (P90/P97/117-07-W5/GOOD-TO-HAVES-04); faking them would be exactly the 'silently waiving to force a false PASS' anti-pattern the plan forbids."

requirements-completed: [DRAIN-06]

duration: ~35min
completed: 2026-07-18
---

# Phase 123 Plan 06: Verifier-script-existence gate (SC4/DRAIN-06) Summary

**New `structure/verifier-script-exists.sh` gate mechanically proves every catalog row's verifier.script exists+executable; fixed 32 pre-existing missing-chmod-+x violations directly, filed 5 pre-existing catalog-first stub-verifier rows to SURPRISES-INTAKE rather than fake them, and left the row honestly grading FAIL (not promoted to pre-commit) until those 5 are resolved.**

## Performance

- **Duration:** ~35 min
- **Completed:** 2026-07-18T12:57:38Z
- **Tasks:** 2/2 completed
- **Files modified:** 17 (2 created, 3 substantively modified, 12 chmod-only)

## Accomplishments

- Authored `quality/gates/structure/verifier-script-exists.sh`: real `python3 -c` JSON parsing (not grep) over every `quality/catalogs/*.json` row, excluding `*-allowlist.json` files and the `docs-alignment` dimension (per the row's own Interfaces spec). Three violation classes (missing-field, missing-file, non-executable), each printed with catalog + row id + path + a concrete fix. Writes the standard per-row JSON artifact directly (mirrors `catalog-immutable-on-read.sh`'s `emit_artifact` shape).
- Ran the gate for real against the live repo catalogs (172 rows / 11 catalogs): found 37 pre-existing violations. Fixed 32 directly (`chmod +x` on 13 real verifier scripts — zero behavior risk since `run.py` always invokes `.py`/`.sh` verifiers via an explicit `python3`/`bash` interpreter prefix, never relying on the executable bit for its own dispatch).
- Authored `quality/gates/structure/verifier-script-exists.selftest.sh`: throwaway `/tmp` git-repo fixtures (never the shared repo), proving each of the 3 violation classes surfaces individually (not just a generic count) plus the all-good pass path. 9/9 assertions pass.
- Minted the row via `run.py --cadence pre-push --persist`. It grades FAIL, honestly reflecting the 5 remaining structural violations (all pre-existing, already-tracked-elsewhere stub rows — none of them was ever PASS, so none was ever an "unbacked PASS," but the gate's committed contract scans unconditionally regardless of a row's own status).
- Filed a SURPRISES-INTAKE entry (severity MEDIUM) citing each of the 5 rows, their existing tracking (P90 cross-platform waiver renewal, P97 launch-readiness, 117-07 W5 animation artifact, GOOD-TO-HAVES-04), and two sketched resolution paths for the row owner (build the deferred verifiers, or deliberately narrow the gate's scope to `status == PASS` rows).
- Fix-twice: added a "Verifier-script existence" subsection to `quality/CLAUDE.md` (structure dimension) documenting the gate, its unconditional-scan design choice, current FAIL state + why, and the selftest.

## Task Commits

1. **Task 1: Implement the gate** — `c3782526` (feat) — new gate script + 13 chmod +x fixes for the 32 mechanical violations
2. **Task 2: Selftest + mint** — `e95a13bd` (test) — selftest + honest FAIL mint + row comment update + fix-twice CLAUDE.md doc + SURPRISES-INTAKE filing

## Files Created/Modified

- `quality/gates/structure/verifier-script-exists.sh` - the new gate (real JSON parse, 3 violation classes, standard artifact write)
- `quality/gates/structure/verifier-script-exists.selftest.sh` - throwaway-/tmp selftest, 9/9 assertions
- `quality/catalogs/freshness-invariants.json` - row `structure/verifier-script-exists`: `status` NOT-VERIFIED→FAIL, `last_verified` set, `comment` rewritten to explain the outcome; `cadences` unchanged at `[pre-push, pre-pr]`
- `quality/CLAUDE.md` - new "Verifier-script existence" subsection under Structure-dimension gates
- `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` - new 2026-07-18 06:00 entry, severity MEDIUM, STATUS OPEN
- 13 verifier scripts across `agent-ux/`, `docs-repro/`, `structure/`, `perf/`, `release/` - `chmod +x` only, no content change

## Decisions Made

- Left the row's `cadences` at `[pre-push, pre-pr]` rather than adding `pre-commit` as Task 2 literally instructed, because the dispatcher's explicit load-bearing instruction required proving the gate clean against the full real catalog before promotion, and it is not clean (5 pre-existing violations remain, all filed). Promoting a dirty P1 row to pre-commit would self-block every future commit repo-wide.
- Did not scope the gate to `status == PASS` rows only, even though that framing would exempt all 5 remaining violations and let the row go green today — the row's committed `claim_vs_assertion_audit` (authored in 123-01, pre-existing the implementation) describes an unconditional scan across every row regardless of status, and narrowing that unilaterally would be a silent, unauthorized contract change. Flagged as sketched-resolution-path (b) in the filed SURPRISES-INTAKE entry for the row owner to decide.
- Did not author placeholder/stub verifier scripts for the 5 structural violations just to satisfy the mechanical check — all 5 are legitimate, already-documented, catalog-first placeholders (2 WAIVED, 3 NOT-VERIFIED) whose owner_hint/waiver text explicitly says the verifier is intentionally absent; faking them would misrepresent that honest incompleteness.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 32 pre-existing missing-executable-bit violations across 13 verifier scripts**
- **Found during:** Task 1 (running the new gate for real against the live catalog)
- **Issue:** 13 real verifier scripts (agent-ux fleet-safety/cadence scripts, docs-repro/snippet-extract.py, structure/freshness-invariants.py, 2 perf scripts, 5 release scripts) lacked the `+x` bit, tripping 32 catalog-row violations (several rows share the same underlying script).
- **Fix:** `chmod +x` on all 13 files. Zero behavior risk: `run.py` always invokes `.py` verifiers via `[sys.executable, script, *args]` and `.sh` verifiers via `["bash", script, *args]` — it never execs the script path directly, so the missing bit never affected any actual verification run; this closes a latent hygiene gap the new gate is designed to catch.
- **Files modified:** the 13 scripts listed above (mode-only changes).
- **Verification:** re-ran the gate; violation count dropped 37 → 5.
- **Committed in:** `c3782526` (Task 1 commit)

### Deferred Issues (filed, not fixed)

**2. [Structural — filed per Task 1's explicit branch] 5 pre-existing stub-verifier violations**
- **Found during:** Task 1, persisting through Task 2's mint
- **Issue:** `cross-platform/windows-2022-rehearsal`, `cross-platform/macos-14-rehearsal` (both WAIVED, P90-renewed cost-deferred stubs), `docs-build/animation-renders` (NOT-VERIFIED, 117-07 W5 catalog-first placeholder), `docs-repro/benchmark-claim/8ms-cached-read` and `.../89.1-percent-token-reduction` (both NOT-VERIFIED, `verifier.script: null` by design, GOOD-TO-HAVES-04-routed) all lack real verifiers today, by deliberate, already-documented design.
- **Why not fixed:** each requires non-trivial, already-separately-scoped engineering (real GH Actions windows/macos rehearsal containers, a playwright animation-capture pipeline, headline-number extraction scripts) — well beyond the plan's <1h eager-fix budget, and building throwaway stand-ins just to pass the mechanical check would misrepresent honest incompleteness.
- **Disposition:** filed to `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (2026-07-18 06:00 entry, severity MEDIUM, STATUS OPEN), with two sketched resolution paths for the row owner.
- **Committed in:** `e95a13bd` (Task 2 commit, alongside the honest FAIL mint)

---

**Total deviations:** 1 auto-fixed (32 individual violations, one root-cause class), 1 filed (5 violations, one class, 2 sub-causes)
**Impact on plan:** The auto-fix was zero-risk and strictly beneficial (closes real hygiene drift). The filed item leaves `structure/verifier-script-exists` grading FAIL and not yet pre-commit-tagged — this is the gate doing its job (surfacing real, pre-existing, already-tracked debt), not a regression introduced by this plan.

## Issues Encountered

None beyond the deviations documented above.

## Self-Check: PASSED

- FOUND: quality/gates/structure/verifier-script-exists.sh
- FOUND: quality/gates/structure/verifier-script-exists.selftest.sh
- FOUND: commit c3782526
- FOUND: commit e95a13bd
- Gate run against real repo: exit 1, 5 named violations (matches filed SURPRISES-INTAKE entry)
- Selftest run: 9/9 assertions passed, exit 0

## Next Phase Readiness

- SC4/DRAIN-06's mechanical deliverable (the gate + selftest) is complete and committed. The catalog row itself is RED (FAIL, P1) pending an owner decision on the two sketched resolution paths — phase-close grading should treat this as a known, filed, pre-existing finding rather than a new defect, per the SURPRISES-INTAKE entry and the row's own updated `comment`.
- No blockers for the phase's other plans (this plan did not touch `run.py` and has no logical dependency on SC1-3).

---
*Phase: 123-quality-runner-catalog-integrity-hardening*
*Completed: 2026-07-18*
