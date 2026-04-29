#!/usr/bin/env bash
# P74 UX-BIND-03 (D-05): docs/index.md "tested against three real backends"
# claim asserts 3 sanctioned `dark_factory_real_*` test functions live in
# crates/reposix-cli/tests/agent_flow_real.rs. Verifier counts test fns;
# asserts >=3.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
TEST_FILE="${REPO_ROOT}/crates/reposix-cli/tests/agent_flow_real.rs"
if [[ ! -f "$TEST_FILE" ]]; then
  echo "FAIL: $TEST_FILE missing — sanctioned-real-backends suite gone" >&2
  exit 1
fi
COUNT=$(grep -c 'fn dark_factory_real_' "$TEST_FILE" || true)
if (( COUNT < 3 )); then
  echo "FAIL: only $COUNT dark_factory_real_* fns found in $TEST_FILE (need >=3)" >&2
  exit 1
fi
echo "PASS: $COUNT dark_factory_real_* fns present (sanctioned-real-backends suite intact)"
exit 0
