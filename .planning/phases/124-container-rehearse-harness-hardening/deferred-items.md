# P124 deferred items (out-of-scope discoveries)

Logged per gsd-executor SCOPE BOUNDARY (only auto-fix issues DIRECTLY caused by
the current task; pre-existing/unrelated failures are logged, not fixed).

## Wave 1a (SC1 / DRAIN-22)

### D-124-W1a-1 — `code/shell-coverage` grades FAIL locally (P2, pre-existing, environmental)

- **Observed:** `python3 quality/runners/run.py --cadence pre-push` grades
  `code/shell-coverage` **FAIL (P2, 64.81s)** locally.
- **Root cause:** NOT the aggregate (17.78% ≥ 13.0% floor — that assert PASSES).
  The FAIL is the anti-gaming counter-validation assert flipping via F-K4b:
  `quality/gates/agent-ux/lib/transcript.sh` drifts `counter=34 vs kcov=27 = 25.9%`
  (>15%) in this box's local kcov, so the expected assert "coverable-line counter
  validated within 15% of kcov on all executed scripts" is uncovered.
- **Why out of scope:** committed status at HEAD is **PASS** (last verified in CI
  2026-07-16); `transcript.sh` and `scripts/shell_coverage.py` are untouched by
  Wave 1a. It is case (c) in the row's own `owner_hint` (kcov line-counter drift,
  retune `coverable_line_count`) and a documented local-vs-CI kcov discrepancy
  (quality/CLAUDE.md § Shell-coverage ratchet). P2 → does not block the pre-push
  P0/P1 exit code. `run.py` here is validate-only (no `--persist`) so the FAIL is
  never written back to the committed catalog.
- **Not fixed** (SCOPE BOUNDARY). If it recurs in CI (not just locally), retune
  `scripts/shell_coverage.py`'s coverable-line counter for `transcript.sh`.

## Wave 2 (SC2 / DRAIN-23)

### D-124-W2-1 — `code/shell-coverage` FAIL recurs identically (same D-124-W1a-1 root cause)

- **Observed:** Wave 2 `run.py --cadence pre-push` grades `code/shell-coverage`
  **FAIL (P2, 62.10s)** — the SAME anti-gaming counter-validation flip: `quality/gates/
  agent-ux/lib/transcript.sh` `counter=34 vs kcov=27 = 25.9%` (>15%). Aggregate is
  17.52% ≥ 13.0% floor (that assert still PASSES).
- **Not a Wave 2 regression.** Wave 2 touched only `quality/gates/docs-repro/*`
  (harness + `lib/sim-lifecycle.sh` + `container-rehearse-sigkill-safe.sh`); none is in
  the `shell-coverage-tests/*.sh` harness set, so none is counter-validated — the new
  scripts only enter the aggregate denominator (which stays above floor). `transcript.sh`
  is unchanged (last touched 2026-07-13, pre-P124). Same case (c) local-vs-CI kcov drift
  as D-124-W1a-1; committed HEAD status is PASS; P2 does not gate the pre-push P0/P1 exit;
  validate-only run so never persisted. Same fix path if it recurs in CI.
