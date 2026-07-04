#!/usr/bin/env bash
# quality/gates/security/connector-audit-wired.sh -- security/connector-audit-wired verifier.
#
# OP-3 (CLAUDE.md "Audit log is non-optional"): production helper backend
# dispatch MUST wire audit_events for write-capable real connectors
# (Confluence/JIRA). Two checks, ONE cargo invocation:
#   1. cargo test -p reposix-remote connector_audit_wired -- exercises
#      build_confluence / build_jira (commit a0c84a3) and asserts
#      backend.has_audit() + a real audit_events row.
#   2. A static grep confirming the PRODUCTION (non-test) build_confluence
#      / build_jira functions actually call .with_audit(...) -- so a
#      regression that deletes the with_audit() call but leaves the tests
#      untouched (e.g. tests construct their own audited backend) cannot
#      slip through as a false PASS.
#
# Honors --row-id <id> (defaults to security/connector-audit-wired).
# Implements catalog row security/connector-audit-wired.
set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
readonly ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/security"
readonly ARTIFACT="${ARTIFACT_DIR}/connector-audit-wired.json"
readonly DISPATCH_SRC="${REPO_ROOT}/crates/reposix-remote/src/backend_dispatch.rs"
readonly MIN_TESTS=2

row_id="security/connector-audit-wired"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  row_id="$2"
fi

cd "$REPO_ROOT"
mkdir -p "$ARTIFACT_DIR"

stdout=$(cargo test -p reposix-remote connector_audit_wired 2>&1) && cargo_exit=0 || cargo_exit=$?
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)

ran_total=$(printf '%s\n' "$stdout" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')
failed_total=$(printf '%s\n' "$stdout" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' | awk '{s+=$1} END {print s+0}')

# Static grep: isolate everything BEFORE the `#[cfg(test)]` module marker
# (production code) and confirm build_confluence + build_jira each call
# .with_audit(...) in that region. Using `awk` to cut at the test-module
# boundary avoids false-passing on a with_audit() call that only exists
# inside a test helper.
production_src=$(awk '/#\[cfg\(test\)\]/{exit} {print}' "$DISPATCH_SRC")
grep_confluence_ok=0
grep_jira_ok=0
if printf '%s\n' "$production_src" | grep -Pzo '(?s)fn build_confluence.*?\.with_audit\(' > /dev/null 2>&1; then
  grep_confluence_ok=1
fi
if printf '%s\n' "$production_src" | grep -Pzo '(?s)fn build_jira.*?\.with_audit\(' > /dev/null 2>&1; then
  grep_jira_ok=1
fi

exit_code=0
asserts_passed=()
asserts_failed=()

if [[ "$cargo_exit" -ne 0 ]]; then
  exit_code=1
  asserts_failed+=("cargo test -p reposix-remote connector_audit_wired exited ${cargo_exit} (nonzero)")
elif [[ "$failed_total" -gt 0 ]]; then
  exit_code=1
  asserts_failed+=("${failed_total} connector_audit_wired test(s) FAILED")
else
  asserts_passed+=("cargo test -p reposix-remote connector_audit_wired exits 0")
fi

if [[ "$ran_total" -lt "$MIN_TESTS" ]]; then
  exit_code=1
  asserts_failed+=("only ${ran_total} test(s) matched/ran; expected >= ${MIN_TESTS} (connector_audit_wired_confluence + connector_audit_wired_jira)")
else
  asserts_passed+=("${ran_total} connector_audit_wired test(s) ran (>= ${MIN_TESTS} required)")
fi

if [[ "$grep_confluence_ok" -eq 1 && "$grep_jira_ok" -eq 1 ]]; then
  asserts_passed+=("production build_confluence AND build_jira (outside #[cfg(test)]) both call .with_audit(...) in ${DISPATCH_SRC}")
else
  exit_code=1
  asserts_failed+=("production dispatch missing .with_audit(...): build_confluence_ok=${grep_confluence_ok} build_jira_ok=${grep_jira_ok} in ${DISPATCH_SRC}")
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
  "build_confluence_wired": $([ "$grep_confluence_ok" -eq 1 ] && echo true || echo false),
  "build_jira_wired": $([ "$grep_jira_ok" -eq 1 ] && echo true || echo false),
  "stdout": $stdout_json,
  "asserts_passed": $asserts_passed_json,
  "asserts_failed": $asserts_failed_json
}
EOF

if [[ "$exit_code" -ne 0 ]]; then
  echo "$stdout" >&2
fi
exit "$exit_code"
