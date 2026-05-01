#!/usr/bin/env bash
# P87 surprises absorption verifier — asserts the v0.13.0
# SURPRISES-INTAKE.md is fully drained (zero OPEN entries, every entry
# carries terminal STATUS: RESOLVED | DEFERRED | WONTFIX). TINY shape
# mirrors quality/gates/structure/no-loose-top-level-planning-audits.sh.
#
# Asserts:
#  1. .planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md exists.
#  2. Zero `**STATUS:** OPEN` lines remain.
#  3. >=5 entries with terminal STATUS (RESOLVED|DEFERRED|WONTFIX) —
#     P78-P86 surfaced 5 entries, P87 must touch all of them.
#  4. .planning/phases/87-surprises-absorption/honesty-spot-check.md
#     exists (verifier-readable evidence the honesty check ran).
#
# Owner-hint on RED: re-run P87 drain — flip remaining OPEN entries to
# RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

INTAKE=".planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md"
HONESTY=".planning/phases/87-surprises-absorption/honesty-spot-check.md"

if [[ ! -f "${INTAKE}" ]]; then
  echo "FAIL: ${INTAKE} not found" >&2
  exit 1
fi

# Count STATUS lines OUTSIDE markdown-fenced code blocks (the schema
# template at the top of the file lives inside a ```markdown ... ```
# fence and must NOT be counted as a real entry).
OPEN_COUNT=$(awk '
  /^```/ { in_fence = !in_fence; next }
  !in_fence && /^\*\*STATUS:\*\* OPEN/ { count++ }
  END { print count + 0 }
' "${INTAKE}")
if [[ "${OPEN_COUNT}" != "0" ]]; then
  echo "FAIL: ${INTAKE} still has ${OPEN_COUNT} entry/entries with STATUS: OPEN" >&2
  echo "owner_hint: flip remaining OPEN entries to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA" >&2
  exit 1
fi

TERMINAL_COUNT=$(awk '
  /^```/ { in_fence = !in_fence; next }
  !in_fence && /^\*\*STATUS:\*\* (RESOLVED|DEFERRED|WONTFIX)/ { count++ }
  END { print count + 0 }
' "${INTAKE}")
if [[ "${TERMINAL_COUNT}" -lt 5 ]]; then
  echo "FAIL: ${INTAKE} has only ${TERMINAL_COUNT} terminal STATUS entries; expected >=5" >&2
  exit 1
fi

if [[ ! -f "${HONESTY}" ]]; then
  echo "FAIL: ${HONESTY} not found — verifier honesty spot-check evidence missing" >&2
  exit 1
fi

echo "PASS: SURPRISES-INTAKE drained (0 OPEN, ${TERMINAL_COUNT} terminal); honesty spot-check artifact present"
exit 0
