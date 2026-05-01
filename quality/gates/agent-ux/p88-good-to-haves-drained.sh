#!/usr/bin/env bash
# P88 good-to-haves drain verifier — asserts the v0.13.0
# GOOD-TO-HAVES.md is fully drained (zero entries without terminal STATUS;
# every entry carries `STATUS: <RESOLVED|DEFERRED|WONTFIX> [...]`).
# TINY shape mirrors p87-surprises-absorption.sh.
#
# Asserts:
#  1. .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md exists.
#  2. Every `## GOOD-TO-HAVES-NN` entry has a matching terminal STATUS line
#     (RESOLVED|DEFERRED|WONTFIX). Entries without a STATUS line, or with
#     STATUS=TBD, fail.
#  3. >=1 entry with terminal STATUS — v0.13.0 surfaced exactly 1
#     (GOOD-TO-HAVES-01); P88 must touch it.
#
# Owner-hint on RED: re-run P88 drain — flip remaining TBD/missing entries
# to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

GTH=".planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md"

if [[ ! -f "${GTH}" ]]; then
  echo "FAIL: ${GTH} not found" >&2
  exit 1
fi

# Count headings of the form `## GOOD-TO-HAVES-NN` (NN = 2 digits).
ENTRY_COUNT=$(grep -cE '^## GOOD-TO-HAVES-[0-9]{2}' "${GTH}" || true)
if [[ "${ENTRY_COUNT}" -lt 1 ]]; then
  echo "FAIL: ${GTH} has 0 entries; expected >=1 (v0.13.0 surfaced GOOD-TO-HAVES-01)" >&2
  exit 1
fi

# Count terminal STATUS lines outside any markdown fence.
# Recognized terminal forms (case-insensitive on the keyword):
#   STATUS: RESOLVED ...
#   STATUS: DEFERRED ...
#   STATUS: WONTFIX ...
# (also tolerant of bold-wrapped form `**STATUS:** ...` for parity with
# SURPRISES-INTAKE.md schema)
TERMINAL_COUNT=$(awk '
  /^```/ { in_fence = !in_fence; next }
  !in_fence && /^[*[:space:]]*STATUS\**:?\**[[:space:]]+(RESOLVED|DEFERRED|WONTFIX)/ { count++ }
  !in_fence && /^[*[:space:]]*\*\*STATUS:\*\*[[:space:]]+(RESOLVED|DEFERRED|WONTFIX)/ { count++ }
  END { print count + 0 }
' "${GTH}")

# TBD = not drained yet.
TBD_COUNT=$(awk '
  /^```/ { in_fence = !in_fence; next }
  !in_fence && /^[*[:space:]]*STATUS\**:?\**[[:space:]]+TBD/ { count++ }
  !in_fence && /^[*[:space:]]*\*\*STATUS:\*\*[[:space:]]+TBD/ { count++ }
  END { print count + 0 }
' "${GTH}")
if [[ "${TBD_COUNT}" -gt 0 ]]; then
  echo "FAIL: ${GTH} has ${TBD_COUNT} entry/entries with STATUS: TBD" >&2
  echo "owner_hint: flip TBD entries to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA" >&2
  exit 1
fi

if [[ "${TERMINAL_COUNT}" -lt "${ENTRY_COUNT}" ]]; then
  echo "FAIL: ${GTH} has ${ENTRY_COUNT} entry/entries but only ${TERMINAL_COUNT} terminal STATUS lines" >&2
  echo "owner_hint: every entry needs a terminal STATUS line (RESOLVED|DEFERRED|WONTFIX)" >&2
  exit 1
fi

echo "PASS: GOOD-TO-HAVES drained (${ENTRY_COUNT} entry/entries; ${TERMINAL_COUNT} terminal STATUS; 0 TBD)"
exit 0
