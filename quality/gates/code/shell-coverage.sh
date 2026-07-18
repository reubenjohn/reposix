#!/usr/bin/env bash
# quality/gates/code/shell-coverage.sh
# Binds catalog row: code/shell-coverage  (kind: mechanical)
#
# Aggregate line-coverage gate for reposix shell scripts. Every in-scope shell
# script counts in the denominator (never-run ones at 0%); we mandate an
# *average* above the ratchet floor in quality/shell-coverage-floor.txt, not
# per-file bars. The metric is kcov line coverage (executes each target via its
# shebang so kcov instruments the script, not the bash ELF).
#
# Behavior:
#   - kcov missing  -> clear FAIL with the apt-get install hint, exit 1.
#   - else          -> invoke scripts/shell_coverage.py run, which drives every
#                      harness under quality/gates/code/shell-coverage-tests/*.sh
#                      through kcov, merges, grades aggregate vs floor, and
#                      writes the JSON artifact. Exit code is propagated.
#
# Exit 0 = aggregate >= floor (and counter validated); exit 1 = below floor or
# kcov missing.
#
# TWO honesty layers cross-check here and CAN legitimately diverge: (1) kcov's
# runtime line instrumentation is the coverage metric itself; (2) scripts/
# shell_coverage.py's static bash-aware `coverable_line_count` is an independent
# anti-gaming counter, cross-checked to stay within 15% of kcov's own coverable
# total (catches a denominator gamed so kcov sees artificially few coverable
# lines). They estimate the SAME "coverable lines" by different heuristics (the
# static parser's heredoc / case-arm / multi-line-continuation skip rules vs
# kcov's interpreter-driven instrumentation), so on a SMALL script a handful of
# structurally-ambiguous lines is a large PERCENT on a tiny absolute gap — e.g.
# transcript.sh at counter=34 vs kcov=27 is a 7-line gap reading as 25.9% — which
# WARNs and flips only the P2 counter-validation assert, never the aggregate-floor
# pass/fail. A >15% drift on a small script is expected, not evidence of gaming.
set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

if ! command -v kcov >/dev/null 2>&1; then
  echo "FAIL: kcov not installed — run: sudo apt-get install -y kcov" >&2
  exit 1
fi

readonly ARTIFACT="quality/reports/verifications/code/shell-coverage.json"
readonly COBERTURA="quality/reports/verifications/code/shell-coverage.cobertura.xml"
mkdir -p "$(dirname "$ARTIFACT")"

# --cobertura-out writes the merged kcov cobertura.xml for the CI Codecov
# upload (shell flag). Both generated paths are gitignored (per-run artifacts).
rc=0
python3 scripts/shell_coverage.py run \
  --floor-file quality/shell-coverage-floor.txt \
  --json "$ARTIFACT" \
  --cobertura-out "$COBERTURA" || rc=$?

if [[ $rc -eq 0 ]]; then
  echo "PASS: aggregate shell line-coverage >= floor (artifact: $ARTIFACT)"
else
  echo "FAIL: shell-coverage gate failed (see $ARTIFACT and output above)" >&2
fi
exit "$rc"
