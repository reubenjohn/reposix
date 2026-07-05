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
set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

if ! command -v kcov >/dev/null 2>&1; then
  echo "FAIL: kcov not installed — run: sudo apt-get install -y kcov" >&2
  exit 1
fi

readonly ARTIFACT="quality/reports/verifications/code/shell-coverage.json"
mkdir -p "$(dirname "$ARTIFACT")"

rc=0
python3 scripts/shell_coverage.py run \
  --floor-file quality/shell-coverage-floor.txt \
  --json "$ARTIFACT" || rc=$?

if [[ $rc -eq 0 ]]; then
  echo "PASS: aggregate shell line-coverage >= floor (artifact: $ARTIFACT)"
else
  echo "FAIL: shell-coverage gate failed (see $ARTIFACT and output above)" >&2
fi
exit "$rc"
