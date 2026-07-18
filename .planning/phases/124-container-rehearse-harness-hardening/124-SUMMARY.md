---
phase: 124-container-rehearse-harness-hardening
plan: 124
subsystem: quality-gates
tags: [bash, docker, container-rehearse, docs-repro, sim, sigkill, blob-limit, python3]

# Dependency graph
requires:
  - phase: 123-quality-runner-catalog-integrity-hardening
    provides: "run.py --persist committed-GREEN downgrade guard + verifier-script-exists gate (NOT-VERIFIED rows are exempt / non-blocking at pre-push, which makes catalog-first minting safe here)"
provides:
  - "quality/gates/docs-repro/container-rehearse.sh — EARNED per-step congruence via ASSERT-PASS harvesting (tautology closed), SIGKILL-proof process-group teardown + internal timeout, fail-loud pre-run stale-orphan gate, exit strictly from persisted artifact exit_code"
  - "quality/gates/docs-repro/container-congruence-earned.sh (+ .selftest.sh) — SC1 meta-check proving a no-op exit-0 script CANNOT earn congruence (P0)"
  - "quality/gates/docs-repro/container-rehearse-sigkill-safe.sh — SC2 selftest: SIGKILLed harness leaves no listener on 7878; stale-orphan precheck fail-loud (P1)"
  - "quality/gates/docs-repro/container-rehearse-exit-from-artifact.sh — SC4 selftest: forced artifact exit_code=1 with docker rc=0 → harness exits 1; .sim-*.log gitignored (P1)"
  - "quality/gates/structure/container-rehearse-binary-provenance.sh — SC3 YAML-parse gate: explicit cargo build -p reposix-cli precedes post-release gates in quality-post-release.yml (P1)"
  - "quality/gates/docs-repro/lib/sim-lifecycle.sh — shared ephemeral-sim start/teardown library extracted for SIGKILL-proof reuse"
  - "examples/05-blob-limit-recovery/run.sh — drives the REAL runtime BLOB_LIMIT_EXCEEDED_FMT stderr refusal + git sparse-checkout recovery cycle (was a pre-emptive source-constant stand-in)"
affects: [124-close, 125-real-backend-cadence, v0.15.0-milestone-close, docs-repro dimension]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ASSERT-PASS: stdout-line harvest protocol (harness ⇄ containerized example): the example prints one machine-parseable `ASSERT-PASS: <text>` line ONLY after the load-bearing step succeeds under set -euo pipefail; the harness greps container stdout and puts the captured text into asserts_passed[] — it NO LONGER copies row.expected.asserts verbatim. Non-empty-harvest guard (`if not passed: return False`) closes the zero-line tautology (asserts_congruent() itself no-ops True on an empty list)."
    - "SIGKILL-proof ephemeral-sim teardown: start the sim in its own process group (setsid), kill the GROUP on teardown, and wrap docker run in an internal `timeout` strictly shorter than the row's catalog timeout_s so the harness reaps its own children BEFORE the runner's outer subprocess.run(timeout=) SIGKILLs it (the un-firable EXIT trap alone was the b773c04 orphan bug)."
    - "Harness exit derived strictly from the persisted artifact exit_code (write artifact FIRST, re-read exit_code, exit with THAT) so a docker rc=0 can never mask an artifact exit_code=1."

key-files:
  created:
    - quality/gates/docs-repro/container-congruence-earned.sh
    - quality/gates/docs-repro/container-congruence-earned.selftest.sh
    - quality/gates/docs-repro/container-rehearse-sigkill-safe.sh
    - quality/gates/docs-repro/container-rehearse-exit-from-artifact.sh
    - quality/gates/structure/container-rehearse-binary-provenance.sh
    - quality/gates/docs-repro/lib/sim-lifecycle.sh
  modified:
    - quality/gates/docs-repro/container-rehearse.sh
    - quality/catalogs/docs-reproducible.json
    - quality/catalogs/freshness-invariants.json
    - examples/01-shell-loop/run.sh
    - examples/02-python-agent/run.py
    - examples/04-conflict-resolve/run.sh
    - examples/05-blob-limit-recovery/run.sh
    - examples/05-blob-limit-recovery/expected-output.md
    - examples/05-blob-limit-recovery/RUN.md
    - .github/workflows/quality-post-release.yml
    - .gitignore
    - quality/CLAUDE.md
    - CLAUDE.md

key-decisions:
  - "Container examples use a stdout-LINE ASSERT-PASS protocol (not tutorial-replay.sh's in-process PASSED[] array): the example runs in an ISOLATED docker container the harness can only observe via captured stdout, so the host in-process array is unavailable — the line protocol is the container-case analogue, riding AFTER the real check, never replacing it."
  - "example-05's expected.asserts were REWRITTEN (not deleted) to the real-runtime-error contract and the row flipped to NOT-VERIFIED — a stale PASS describing the old pre-emptive-only behavior would have been a false-green; the 2026-07-13 honesty reword is now explicitly SUPERSEDED in the row comment."
  - "The 5 P124 catalog rows remain status: NOT-VERIFIED on disk (catalog-first). A validate-only pre-push GATE run computes the flip in-memory but does not persist (P123 D-P96-01); only a --persist MINT run writes PASS back. The 4 mechanical rows grade PASS when run for real (proven at phase-close); example-05 (post-release cadence) re-grades in the post-release workflow. Flipping them is the verifier/close-step's job, not this executor's."

patterns-established:
  - "Every new container/harness gate degrades to NOT-VERIFIED (never FAIL) where docker/lsof/kcov substrate is absent (OP-2: skip is never pass); the docker-free static leg (grep-absence / YAML-parse / gitignore) still runs and can PASS on its own."

requirements-completed: [DRAIN-13, DRAIN-14, DRAIN-22, DRAIN-23, DRAIN-24]

# Metrics
duration: ~2h 45m
completed: 2026-07-18
---

# Phase 124: Container-rehearse harness hardening Summary

**`container-rehearse.sh` now EARNS each container row's congruence by harvesting per-step `ASSERT-PASS:` lines (a no-op `exit 0` script can no longer pass — the F-K4b tautology is closed), tears the ephemeral sim down SIGKILL-proof via a process-group kill + an internal `timeout` shorter than the row's `timeout_s` (fail-loud on a stale orphan on 7878), derives its exit strictly from the persisted artifact `exit_code`, gains a guaranteed `target/debug/reposix` build step on the post-release runner, and example-05 drives the REAL runtime `BLOB_LIMIT_EXCEEDED_FMT` refusal + `git sparse-checkout` recovery — all graded by 5 machine-checkable catalog rows.**

## Performance

- **Duration:** ~2h 45m (plan doc `ffcf865d` 09:26 PDT → pushed tip `790aa73c` 12:10 PDT)
- **Completed:** 2026-07-18T12:10:59-07:00
- **Tasks:** 5 waves (W0 catalog-first, W1a/W1b, W2, W3, W4) + hygiene + review-fix
- **Files modified:** 25 files changed, 1828(+) / 135(-) across `ac632c5d^..790aa73c`

## Goal-backward: each Success Criterion → evidence

Every SC's evidence is (impl commit SHA) + (catalog row / gate) + (reality check run at phase-close). All four mechanical gates were RUN for real at close (docker + kcov + lsof present on this host); example-05 is a `post-release`-cadence container row verified on-host in W1b.

| SC | Requirement | Impl commit(s) | Catalog row / gate | Reality-check verdict (run at close) |
|----|-------------|----------------|--------------------|--------------------------------------|
| **SC1** | DRAIN-22 (earned congruence) | `a54ba881` (W1a harvest) | `docs-repro/container-congruence-earned` (P0) + `container-rehearse.sh` | selftest **ALL PASS** (T1–T5); gate **rc=0**. T4 proves a no-op `exit 0` fixture does NOT earn congruence; T2/T5 prove a re-introduced verbatim-copy path or a missing empty-harvest guard FAILS the gate — the tautology is closed. |
| **SC1** | DRAIN-22 (example-05 real runtime) | `7c590ea2` (W1b) | `docs-repro/example-05-blob-limit-recovery` (P1, `post-release`, asserts rewritten to real-runtime contract) | `run.sh` rewritten to drive the real `BLOB_LIMIT_EXCEEDED_FMT` (`[RPX-0503]`) stderr refusal then recover via `git sparse-checkout set`; verified on-host in W1b (cargo build + ephemeral sim + captured transcript). Row is NOT-VERIFIED on disk; re-grades in the post-release container job. |
| **SC2** | DRAIN-23 (SIGKILL-proof teardown) | `3a946acd` (W2) | `docs-repro/container-rehearse-sigkill-safe` (P1) | gate **rc=0**; after the selftest's mid-run SIGKILL, `lsof -ti:7878` shows **7878 free** and **no orphan `reposix sim`** survives. Process-group kill + internal `timeout` + fail-loud pre-run orphan gate all present. |
| **SC3** | DRAIN-24 (binary provenance) | `e91a9b5c` (W3) | `structure/container-rehearse-binary-provenance` (P1) | gate **rc=0** — the YAML-parse gate confirms an explicit `cargo build -p reposix-cli` step (with an inline provenance comment) precedes the post-release gates in `quality-post-release.yml`. |
| **SC4** | DRAIN-13 + DRAIN-14 (exit-from-artifact + gitignore) | `d83bbe32` (W4) | `docs-repro/container-rehearse-exit-from-artifact` (P1) | gate **rc=0** — forced artifact `exit_code=1` while docker rc would be 0 makes the harness exit 1; `.sim-*.log` under `quality/reports/verifications/docs-repro/` is git-ignored. |

**Catalog-first ordering held:** W0 commit `ac632c5d` minted the 4 new NOT-VERIFIED rows + rewrote example-05's asserts BEFORE any implementation (`a54ba881` W1a is the first impl commit). Each new row carries `minted_at` + a ≥50-char `claim_vs_assertion_audit`.

## Accomplishments

- **Closed the F-K4b container-congruence tautology (SC1/DRAIN-22).** `container-rehearse.sh` deletes the verbatim `expected.asserts → asserts_passed` copy path and instead harvests `^ASSERT-PASS: ` lines from container stdout; examples 01/02/04 emit one such line per expected.assert *after* the load-bearing step; a non-empty-harvest guard rejects a zero-line no-op. Proven by the `container-congruence-earned` P0 meta-check (5-subtest selftest).
- **example-05 now drives the REAL error (SC1/DRAIN-22).** Reversed the 2026-07-13 "never reads helper stderr" caveat: `run.sh` triggers a real `command=fetch` blob-materialization over `REPOSIX_BLOB_LIMIT`, captures the `[RPX-0503]` `git sparse-checkout` stderr refusal, then narrows via `git sparse-checkout set` and retries clean. `expected-output.md` + `RUN.md` rewritten to the real observe-then-recover flow.
- **SIGKILL-proof teardown + fail-loud orphan gate (SC2/DRAIN-23).** Extracted `lib/sim-lifecycle.sh`; sim starts in its own process group, teardown kills the GROUP, `docker run` is wrapped in an internal `timeout` < the row's `timeout_s`, and a pre-run port-7878-occupied check fails loud instead of silently reusing a stale orphan.
- **Guaranteed binary provenance on the post-release runner (SC3/DRAIN-24).** Added an explicit `cargo build -p reposix-cli` step (inline provenance comment) before the post-release gates in `quality-post-release.yml`, closing the unconfirmed-implicit-cache gap that would silently degrade container rows to NOT-VERIFIED on a cold runner.
- **Artifact-authoritative exit + gitignore (SC4/DRAIN-13+14).** Harness writes the artifact first, re-reads `exit_code`, and exits with it; readiness gate now requires real sim reachability, not merely a bound port; `.sim-*.log` gitignored.

## Task Commits

1. **W0 · catalog-first** — `ac632c5d` (test) — 4 new NOT-VERIFIED rows (3 docs-repro + 1 structure) + example-05 asserts rewritten to the real-runtime contract
2. **W1a · earned congruence** — `a54ba881` (feat) — harness ASSERT-PASS harvesting + examples 01/02/04 emission + `container-congruence-earned.sh`(+selftest)
3. **W1b · example-05 real runtime** — `7c590ea2` (feat) — real `BLOB_LIMIT_EXCEEDED_FMT` + sparse-checkout recovery in `run.sh`; docs rewritten
4. **W2 · SIGKILL-proof teardown** — `3a946acd` (fix) — process-group teardown + internal timeout + fail-loud orphan gate + `container-rehearse-sigkill-safe.sh`; `b5755484` (docs) — logged the pre-existing shell-coverage counter-drift FAIL as recurrence, not a W2 regression
5. **W3 · binary provenance** — `e91a9b5c` (ci) — explicit reposix build step in `quality-post-release.yml` + `container-rehearse-binary-provenance.sh`
6. **W4 · exit-from-artifact** — `d83bbe32` (fix) — harness exit from persisted `exit_code` + `.sim-*.log` gitignore + `container-rehearse-exit-from-artifact.sh`
7. **Phase-close hygiene** — `5ad18f20` (chore) — intake migration, cadence-budget + shell-coverage-honesty CLAUDE.md docs, gate-confirm
8. **Review-fix** — `790aa73c` (fix) — pinned the harness harvest-guard in the congruence gate [M1/M2], de-flaked the SIGKILL control (outer 4s→8s + bounded 10s post-SIGKILL bind retry) + `ss` fallback [L1], catalog wording [L2]

**Plan doc:** `ffcf865d` (docs: P124 plan). **Pushed tip:** `790aa73c` == origin/main; CI run `29657431393` (`ci.yml`) = success.

## Decisions Made

See frontmatter `key-decisions`. In brief: the container case uses a stdout-LINE ASSERT-PASS protocol (not the host in-process array) because the example is only observable via captured container stdout; example-05's asserts were rewritten-not-deleted with the row flipped to NOT-VERIFIED to avoid a stale false-green; and the 5 rows are intentionally left NOT-VERIFIED on disk (catalog-first — the close-step's `--persist` MINT flips them).

## Deviations from Plan

Plan executed wave-for-wave as written. The following were handled per the ownership/deviation rules:

**1. [Filed — pre-existing, not a P124 regression] `code/shell-coverage` FAIL (transcript.sh counter drift)**
- **Found during:** W1a + W2 (running pre-push)
- **Issue:** `scripts/shell_coverage.py` static counter=34 vs kcov=27 on `transcript.sh` = 25.9% anti-gaming drift — flips the P2 counter-validation assert, NOT the aggregate floor. Pre-existing (P123-verifier already tracked it), surfaced again because pre-push walks the whole corpus.
- **Disposition:** consolidated into `.planning/milestones/v0.15.0-phases/surprises-intake/part-07.md` (the `2026-07-18 | P124 W1a + W2` entry) as corroborating data points; `deferred-items.md` records it as drained. Not a W2 regression (`b5755484` documents this).

**2. [Rule 2 — fix-twice, side-investigation resolved] pre-push budget + shell-coverage honesty docs**
- L1129: the pre-push budget had outgrown its ≈55s figure to ~90–120s (kcov-corpus-dominant, NOT diff-size-scaled) — corrected in `quality/CLAUDE.md` § Runtime with re-measured 122s + per-gate breakdown.
- L1166: clarified the two honesty layers (kcov runtime instrumentation vs `scripts/shell_coverage.py` static anti-gaming counter) so a >15% drift on a small script reads as expected divergence, not a gamed denominator — documented in `quality/CLAUDE.md` § Two honesty layers.

**3. [Review-fix noticings closed — `790aa73c`]**
- The congruence gate had a real falsifiability hole: the verifier did not pin the `not-harvested → exit=1` empty-harvest guard, so a harness that dropped the guard could have passed. Closed and selftest-locked (T2/T5).
- example-05's "never observes a runtime blob-limit stderr" caveat was false on modern git (ubuntu:24.04 ships git 2.43, the protocol-v2 stateless-connect fetch path fires the real error) — corrected in the row comment + docs.

---

**Total deviations:** 1 filed (pre-existing shell-coverage drift), 2 fix-twice doc corrections, 2 review-fix noticings closed. No scope creep; no architectural changes.

## Issues Encountered

None beyond the deviations above. All four mechanical gates grade PASS when run for real; 7878 left clean after the SIGKILL selftest.

## Known Stubs / NOT-VERIFIED on disk

The 5 P124 catalog rows sit at `status: NOT-VERIFIED` on disk — this is correct catalog-first state, not a stub. The 4 mechanical rows (`container-congruence-earned` P0, `container-rehearse-sigkill-safe` P1, `container-rehearse-exit-from-artifact` P1, `container-rehearse-binary-provenance` P1) grade PASS when run (proven at close); `example-05-blob-limit-recovery` (P1, `post-release`) re-grades in the post-release container job. A `--persist` MINT run at phase-close/verifier flips them to PASS — out of this executor's charter.

## Intake filings

- **SURPRISES** `surprises-intake/part-07.md` — transcript.sh shell-coverage counter drift (pre-existing, corroborated).
- **GTH-V15-84** (MED) — `container-rehearse.sh` + `container-rehearse-sigkill-safe.sh` exceed the 10k `.sh` file-size ceiling; covered by the global file-size waiver → 2026-08-08.
- **GTH-V15-85** (LOW) — residual cruft cleanup.

## Self-Check: PASSED

- FOUND: quality/gates/docs-repro/container-congruence-earned.sh (+ .selftest.sh)
- FOUND: quality/gates/docs-repro/container-rehearse-sigkill-safe.sh
- FOUND: quality/gates/docs-repro/container-rehearse-exit-from-artifact.sh
- FOUND: quality/gates/structure/container-rehearse-binary-provenance.sh
- FOUND: quality/gates/docs-repro/lib/sim-lifecycle.sh
- FOUND: commits ac632c5d, a54ba881, 7c590ea2, 3a946acd, b5755484, e91a9b5c, d83bbe32, 5ad18f20, 790aa73c
- Gates run at close: SC1 selftest ALL PASS + gate rc=0; SC2 rc=0 (7878 free after); SC3 rc=0; SC4 rc=0
- Push landed: origin/main == 790aa73c; CI ci.yml run 29657431393 = success

## Next Phase Readiness

- All four ROADMAP SCs verified against reality; DRAIN-13/14/22/23/24 each map to a committed catalog row minted BEFORE its implementation.
- The independent phase-close verifier should grade the 5 rows (running them flips the 4 mechanical rows PASS; example-05 re-grades post-release) and dispatch its verdict; STATE.md advance + the phase-close push ride the P125 boundary per the close plan.
- No blockers for P125 (real-backend cadence & mirror-drift resilience).

---
*Phase: 124-container-rehearse-harness-hardening*
*Completed: 2026-07-18*
