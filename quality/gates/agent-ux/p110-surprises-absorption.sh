#!/usr/bin/env bash
# P110 surprises absorption verifier — asserts the v0.14.0
# SURPRISES-INTAKE.md is fully drained (zero OPEN entries, every entry
# carries terminal STATUS: RESOLVED | DEFERRED | WONTFIX). Adapts the
# P87 (v0.13.0) precedent verbatim, swapping the milestone + phase paths
# and raising the terminal-count floor to the v0.14.0 surfaced count.
#
# Asserts:
#  1. .planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md exists.
#  2. Zero `**STATUS:** OPEN` lines remain (fence-aware — the schema
#     example at the top of the file lives inside a ```markdown fence
#     and must NOT be counted as a real entry).
#  3. >=10 entries with terminal STATUS (RESOLVED|DEFERRED|WONTFIX) —
#     P102-P109 surfaced 16 entries; P110 must touch all of them.
#  4. .planning/phases/110-op-8-slot-1-surprises-drain/honesty-spot-check.md
#     exists (verifier-readable evidence the F-K5 honesty check ran).
#
# Owner-hint on RED: re-run P110 drain — flip remaining OPEN entries to
# RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

INTAKE=".planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md"
PARTS_DIR=".planning/milestones/v0.14.0-phases/surprises-intake"
SCAN_FILES=("${INTAKE}")
# OP-8 file-size drain (2026-07-12): entries relocated verbatim into part files; count across both.
if compgen -G "${PARTS_DIR}/part-*.md" > /dev/null 2>&1; then
  SCAN_FILES+=("${PARTS_DIR}"/part-*.md)
fi
HONESTY=".planning/phases/110-op-8-slot-1-surprises-drain/honesty-spot-check.md"
ARTIFACT="quality/reports/verifications/agent-ux/p110-surprises-absorption.json"

fail() {
  local msg="$1"
  mkdir -p "$(dirname "${ARTIFACT}")"
  printf '{\n  "row_id": "agent-ux/p110-surprises-absorption",\n  "status": "FAIL",\n  "open_count": %s,\n  "terminal_count": %s,\n  "reason": "%s"\n}\n' \
    "${OPEN_COUNT:-null}" "${TERMINAL_COUNT:-null}" "${msg}" > "${ARTIFACT}"
  echo "FAIL: ${msg}" >&2
  exit 1
}

if [[ ! -f "${INTAKE}" ]]; then
  echo "FAIL: ${INTAKE} not found" >&2
  exit 1
fi

# Count STATUS lines OUTSIDE markdown-fenced code blocks (the schema
# template at the top of the file lives inside a ```markdown ... ```
# fence and must NOT be counted as a real entry).
OPEN_COUNT=$(awk '
  FNR==1 { in_fence=0 }
  /^```/ { in_fence = !in_fence; next }
  !in_fence && /^\*\*STATUS:\*\* OPEN/ { count++ }
  END { print count + 0 }
' "${SCAN_FILES[@]}")
if [[ "${OPEN_COUNT}" != "0" ]]; then
  fail "${INTAKE} still has ${OPEN_COUNT} entry/entries with STATUS: OPEN — flip to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA"
fi

TERMINAL_COUNT=$(awk '
  FNR==1 { in_fence=0 }
  /^```/ { in_fence = !in_fence; next }
  !in_fence && /^\*\*STATUS:\*\* (RESOLVED|DEFERRED|WONTFIX)/ { count++ }
  END { print count + 0 }
' "${SCAN_FILES[@]}")
if [[ "${TERMINAL_COUNT}" -lt 10 ]]; then
  fail "${INTAKE} has only ${TERMINAL_COUNT} terminal STATUS entries; expected >=10"
fi

if [[ ! -f "${HONESTY}" ]]; then
  fail "${HONESTY} not found — F-K5 honesty spot-check evidence missing"
fi

mkdir -p "$(dirname "${ARTIFACT}")"
printf '{\n  "row_id": "agent-ux/p110-surprises-absorption",\n  "status": "PASS",\n  "open_count": %s,\n  "terminal_count": %s,\n  "honesty_spot_check": "%s"\n}\n' \
  "${OPEN_COUNT}" "${TERMINAL_COUNT}" "${HONESTY}" > "${ARTIFACT}"

echo "PASS: SURPRISES-INTAKE drained (0 OPEN, ${TERMINAL_COUNT} terminal); honesty spot-check artifact present"
exit 0
