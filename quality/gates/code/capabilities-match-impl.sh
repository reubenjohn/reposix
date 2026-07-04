#!/usr/bin/env bash
# quality/gates/code/capabilities-match-impl.sh -- code/capabilities-match-impl verifier.
#
# Runs the capabilities_match_create_impl tests already present in
# reposix-github, reposix-jira, reposix-confluence (commit ca7cb61) as a
# SINGLE cargo invocation (CLAUDE.md "Build memory budget"). Asserts exit
# 0 AND that >=3 tests actually matched -- guards against a silent
# 0-matches false-pass if a test name or --lib filter drifts.
#
# Honors --row-id <id> (defaults to code/capabilities-match-impl).
# Implements catalog row code/capabilities-match-impl.
set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
readonly ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/code"
readonly ARTIFACT="${ARTIFACT_DIR}/capabilities-match-impl.json"
readonly MIN_TESTS=3

row_id="code/capabilities-match-impl"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  row_id="$2"
fi

cd "$REPO_ROOT"
mkdir -p "$ARTIFACT_DIR"

stdout=$(cargo test -p reposix-github -p reposix-jira -p reposix-confluence capabilities_match 2>&1) \
  && cargo_exit=0 || cargo_exit=$?
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Sum "N passed" across every per-crate libtest/integration-test summary line
# (`test result: ok. N passed; 0 failed; ...`). A crate with zero matching
# tests still prints a summary line with N=0 -- summing catches a
# silently-empty filter as well as an outright cargo failure.
ran_total=$(printf '%s\n' "$stdout" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
failed_total=$(printf '%s\n' "$stdout" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')

exit_code=0
asserts_passed=()
asserts_failed=()

if [[ "$cargo_exit" -ne 0 ]]; then
  exit_code=1
  asserts_failed+=("cargo test exited ${cargo_exit} (nonzero)")
elif [[ "$failed_total" -gt 0 ]]; then
  exit_code=1
  asserts_failed+=("${failed_total} capabilities_match test(s) FAILED")
else
  asserts_passed+=("cargo test -p reposix-github -p reposix-jira -p reposix-confluence capabilities_match exits 0")
fi

if [[ "$ran_total" -lt "$MIN_TESTS" ]]; then
  exit_code=1
  asserts_failed+=("only ${ran_total} test(s) matched/ran; expected >= ${MIN_TESTS} (silent 0-matches guard -- test name or --lib filter may have drifted)")
else
  asserts_passed+=("${ran_total} capabilities_match test(s) ran (>= ${MIN_TESTS} required)")
fi

py_json_array() {
  if [[ "$#" -eq 0 ]]; then
    echo "[]"
  else
    python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" "$@"
  fi
}
asserts_passed_json=$(py_json_array "${asserts_passed[@]}")
asserts_failed_json=$(py_json_array "${asserts_failed[@]}")
stdout_json=$(printf '%s' "$stdout" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))")

cat > "$ARTIFACT" <<EOF
{
  "ts": "$ts",
  "row_id": "$row_id",
  "exit_code": $exit_code,
  "cargo_exit_code": $cargo_exit,
  "tests_ran": $ran_total,
  "tests_failed": $failed_total,
  "stdout": $stdout_json,
  "asserts_passed": $asserts_passed_json,
  "asserts_failed": $asserts_failed_json
}
EOF

if [[ "$exit_code" -ne 0 ]]; then
  echo "$stdout" >&2
fi
exit "$exit_code"
