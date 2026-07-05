#!/usr/bin/env bash
# quality/gates/agent-ux/audit-absence-is-red.sh — agent-ux verifier for
# catalog row `agent-ux/audit-absence-is-red` (P92 SC4, RBF-B-03).
#
# INVARIANT: quality/PROTOCOL.md's "Verifier subagent prompt template"
# explicitly names the rule that an ABSENT audit_events / audit_events_cache
# row for an executed push is graded RED (FAIL), never waved off as
# "out of scope for this layer" or a soft/PARTIAL miss. Mirrors
# agent-ux/absorption-honesty-template-present's grep-clause pattern, but
# WITHOUT full-file content-hash binding (PROTOCOL.md is a living,
# frequently-edited file — hash-binding the whole file would make every
# unrelated PROTOCOL.md edit a false-RED here; the verbatim-clause grep is
# the right-sized check for a single-paragraph rule addition).
#
# Implements catalog row agent-ux/audit-absence-is-red.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

PROTOCOL="quality/PROTOCOL.md"

if [[ ! -f "${PROTOCOL}" ]]; then
  echo "FAIL: ${PROTOCOL} not found" >&2
  exit 1
fi

missing=0

check_clause() {
  local label="$1"
  local pattern="$2"
  if ! grep -qF "${pattern}" "${PROTOCOL}"; then
    echo "FAIL: clause ${label} not found verbatim in ${PROTOCOL} (looked for: ${pattern})" >&2
    missing=1
  fi
}

check_clause "audit-row absence is RED" \
  "audit-row absence is RED, not out of scope for this layer"
check_clause "OP-3 non-optional" \
  "OP-3 (CLAUDE.md) makes the dual-table audit log non-optional"
check_clause "row id anchor" \
  "agent-ux/audit-absence-is-red"

if [[ "${missing}" -eq 1 ]]; then
  echo "owner_hint: restore the missing audit-absence-is-red clause verbatim in ${PROTOCOL}'s Verifier subagent prompt template (P92 SC4)" >&2
  exit 1
fi

echo "PASS: ${PROTOCOL} verifier prompt template names 'audit-row absence is RED, not out of scope for this layer'"
exit 0
