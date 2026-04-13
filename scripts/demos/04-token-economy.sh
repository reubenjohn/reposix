#!/usr/bin/env bash
# scripts/demos/04-token-economy.sh — Tier 1 buyer-audience demo.
#
# AUDIENCE: buyer
# RUNTIME_SEC: 10
# REQUIRES: python3
# ASSERTS: "reduction" "DEMO COMPLETE"
#
# Narrative:
#   [1/3] the central claim, in two sentences.
#   [2/3] run bench_token_economy.py; it writes benchmarks/RESULTS.md
#         and prints the same table to stdout.
#   [3/3] extract the headline "Reduction: ... %" line and bold-print it.

set -euo pipefail

# Self-wrap in `timeout 90` so a stuck step can't hang smoke.sh.
if [[ -z "${REPOSIX_DEMO_INNER:-}" ]]; then
    exec timeout 90 env REPOSIX_DEMO_INNER=1 bash "$0" "$@"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/_lib.sh"

require python3

# Locate the benchmark script; look upward from $SCRIPT_DIR.
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BENCH_SCRIPT="${REPO_ROOT}/scripts/bench_token_economy.py"
if [[ ! -f "$BENCH_SCRIPT" ]]; then
    echo "ERROR: cannot find $BENCH_SCRIPT" >&2
    exit 2
fi

section "[1/3] the central claim"
cat <<'EOF'
For the same task (read 3 issues, edit one, push the change), an
MCP-mediated agent ingests the tool catalog plus every schema before
making its first call. A reposix-based agent just reads the bytes of
its shell session. The cost difference is the whole value proposition.
EOF

section "[2/3] run the benchmark (token-count measurement, not inference cost)"
BENCH_OUT=$(python3 "$BENCH_SCRIPT")
echo "$BENCH_OUT"

section "[3/3] the headline"
HEADLINE=$(echo "$BENCH_OUT" | grep -E '^\*\*Reduction:' | head -1)
if [[ -z "$HEADLINE" ]]; then
    echo "ERROR: could not find '**Reduction:' line in benchmark output" >&2
    exit 1
fi
# Bold-print via ANSI; terminal capture strips escape codes but grep
# still matches the text.
printf '\n\033[1m%s\033[0m\n\n' "$HEADLINE"

echo "== DEMO COMPLETE =="
