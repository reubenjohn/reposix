#!/usr/bin/env bash
# P78-02 HYGIENE-02 — verifier for catalog row
# `structure/no-loose-top-level-planning-audits`. TINY shape mirrors
# quality/gates/docs-alignment/jira-adapter-shipped.sh.
#
# Asserts no *MILESTONE-AUDIT*.md or SESSION-END-STATE* file lives at
# .planning/ top level (excluding .planning/archive/). Per the catalog
# row's `expected.asserts`:
#   find .planning -maxdepth 1 -type f \( -name '*MILESTONE-AUDIT*.md'
#     -o -name 'SESSION-END-STATE*' \) | grep -v archive returns empty
#
# Owner-hint on RED: relocate the loose audit doc under
# .planning/milestones/audits/ or .planning/archive/.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"
OFFENDERS=$(find .planning -maxdepth 1 -type f \( -name '*MILESTONE-AUDIT*.md' -o -name 'SESSION-END-STATE*' \) 2>/dev/null | grep -v archive || true)
if [[ -n "${OFFENDERS}" ]]; then
  echo "FAIL: loose milestone-audit / session-end-state files at .planning/ top level:" >&2
  echo "${OFFENDERS}" >&2
  echo "owner_hint: relocate under .planning/milestones/audits/ or .planning/archive/" >&2
  exit 1
fi
echo "PASS: no loose *MILESTONE-AUDIT*.md / SESSION-END-STATE* files at .planning/ top level"
exit 0
