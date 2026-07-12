#!/usr/bin/env bash
# quality/gates/code/lost-update-shared-cursor.sh -- code/lost-update-shared-cursor-rejected verifier.
#
# P106 (v0.14.0 wave-2) lost-update guard. Runs the precheck regression that
# proves a stale-base push under an ADVANCED shared last_fetched_at cursor is
# REJECTED (fetch first) rather than silently clobbering a sibling clone's edit
# (SILENT LOST UPDATE, HIGH, data-loss -- SURPRISES-INTAKE.md 2026-07-12 08:10).
#
# Single cargo invocation (CLAUDE.md "Build memory budget"). Asserts exit 0 AND
# that exactly 1 test matched -- guards against a silent 0-matches false-pass if
# the test name drifts.
#
# Honors --row-id <id> (defaults to code/lost-update-shared-cursor-rejected).
# Implements catalog row code/lost-update-shared-cursor-rejected.
set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
readonly ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/code"
readonly ARTIFACT="${ARTIFACT_DIR}/lost-update-shared-cursor-rejected.json"
readonly TEST_NAME="stale_base_push_rejected_when_shared_cursor_advanced_past_concurrent_write"
readonly MIN_TESTS=1

row_id="code/lost-update-shared-cursor-rejected"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  row_id="$2"
fi

cd "$REPO_ROOT"
mkdir -p "$ARTIFACT_DIR"

stdout=$(cargo test -p reposix-remote --lib "$TEST_NAME" 2>&1) \
  && cargo_exit=0 || cargo_exit=$?
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Sum "N passed"/"N failed" across the libtest summary line(s).
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
  asserts_failed+=("${failed_total} lost-update regression test(s) FAILED")
else
  asserts_passed+=("cargo test -p reposix-remote --lib ${TEST_NAME} exits 0")
fi

if [[ "$ran_total" -lt "$MIN_TESTS" ]]; then
  exit_code=1
  asserts_failed+=("only ${ran_total} test(s) matched/ran; expected >= ${MIN_TESTS} (silent 0-matches guard -- test name may have drifted)")
else
  asserts_passed+=("${ran_total} lost-update regression test(s) ran (>= ${MIN_TESTS} required)")
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
