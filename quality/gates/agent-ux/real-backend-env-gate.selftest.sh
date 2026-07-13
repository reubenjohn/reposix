#!/usr/bin/env bash
# quality/gates/agent-ux/real-backend-env-gate.selftest.sh
#
# Hermetic self-test (quick 260712-phc) for the two new
# pre-release-real-backend verifiers: B4
# (t4-conflict-rebase-ancestry-real-backend.sh) and B5
# (github-front-door-real-backend.sh). For EACH script, run it standalone
# with EVERY real-backend cred/allowlist var unset and assert:
#   (a) exit code == 75 (NOT-VERIFIED, fail-closed per OD-2)
#   (b) the row's committed artifact JSON exists, parses, and carries
#       status == "NOT-VERIFIED" and exit_code == 75
#
# This path exits BEFORE any cargo/init/network call -- fully hermetic (no
# real backend touched, no cargo invocation, no /tmp reposix setup). Mirrors
# the structure/file-size-limits.selftest.sh idiom: a pass/fail counter, a
# RESULT line, exit 0 all-pass / exit 1 on any regression.
#
# Run: bash quality/gates/agent-ux/real-backend-env-gate.selftest.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

pass=0
fail=0

check_env_gate() {  # check_env_gate <label> <script-rel-path> <artifact-rel-path>
  local label="$1" script="$2" artifact="$3"
  local rc=0 errf
  errf="$(mktemp "${TMPDIR:-/tmp}/real-backend-env-gate-selftest.XXXXXX")"

  env -u GITHUB_TOKEN -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL \
      -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_ALLOWED_ORIGINS \
      -u JIRA_EMAIL -u JIRA_API_TOKEN -u REPOSIX_JIRA_INSTANCE \
    bash "${REPO_ROOT}/${script}" >/dev/null 2>"$errf" || rc=$?

  if [[ "$rc" -eq 75 ]]; then
    echo "  PASS: ${label} exits 75 with all real-backend creds/allowlist unset"
    pass=$((pass + 1))
  else
    echo "  FAIL: ${label} expected exit 75 (NOT-VERIFIED), got ${rc}"
    sed 's/^/    stderr: /' "$errf"
    fail=$((fail + 1))
  fi
  rm -f "$errf"

  local art="${REPO_ROOT}/${artifact}"
  if [[ -f "$art" ]] && python3 -c '
import json, sys
d = json.load(open(sys.argv[1], encoding="utf-8"))
sys.exit(0 if d.get("status") == "NOT-VERIFIED" and d.get("exit_code") == 75 else 1)
' "$art" 2>/dev/null; then
    echo "  PASS: ${label} artifact is a well-formed NOT-VERIFIED (exit_code=75) at ${artifact}"
    pass=$((pass + 1))
  else
    echo "  FAIL: ${label} artifact missing or malformed at ${artifact}"
    fail=$((fail + 1))
  fi
}

echo "== B4: t4-conflict-rebase-ancestry-real-backend =="
check_env_gate "B4" \
  "quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh" \
  "quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry-real-backend.json"

echo "== B5: github-front-door-real-backend =="
check_env_gate "B5" \
  "quality/gates/agent-ux/github-front-door-real-backend.sh" \
  "quality/reports/verifications/agent-ux/github-front-door-real-backend.json"

echo
echo "RESULT: ${pass} passed, ${fail} failed"
[[ "$fail" -eq 0 ]] || exit 1
