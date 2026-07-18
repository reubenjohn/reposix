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
