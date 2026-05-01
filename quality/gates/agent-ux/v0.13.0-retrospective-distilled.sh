#!/usr/bin/env bash
# v0.13.0 RETROSPECTIVE distillation verifier (TINY).
#
# Asserts:
#  1. .planning/RETROSPECTIVE.md exists.
#  2. RETROSPECTIVE.md contains a v0.13.0 milestone heading
#     (`^## Milestone: v0.13.0`).
#  3. The v0.13.0 section contains all 5 OP-9 template subheadings:
#     What Was Built / What Worked / What Was Inefficient /
#     Patterns Established / Key Lessons.
#
# Owner-hint on RED: re-run P88 RETROSPECTIVE distillation against the
# OP-9 template (CLAUDE.md OP-9 milestone-close ritual).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

R=".planning/RETROSPECTIVE.md"
if [[ ! -f "${R}" ]]; then
  echo "FAIL: ${R} not found" >&2
  exit 1
fi

if ! grep -qE '^## Milestone: v0\.13\.0' "${R}"; then
  echo "FAIL: ${R} missing '## Milestone: v0.13.0' heading" >&2
  exit 1
fi

# Extract the v0.13.0 section: from heading to next `^## Milestone:`
# (or EOF). Same parsing pattern as the CHANGELOG verifier.
SECTION=$(awk '
  /^## Milestone: v0\.13\.0/ { capturing=1; next }
  capturing && /^## Milestone:/ { exit }
  capturing { print }
' "${R}")

MISSING=()
# Match each as an actual markdown heading (### or ####), not just an
# inline mention. The OP-9 template uses `### <heading>` (per
# v0.12.1 + v0.8.0 sections in RETROSPECTIVE.md).
for HEADING in "What Was Built" "What Worked" "What Was Inefficient" "Patterns Established" "Key Lessons"; do
  if ! printf '%s\n' "${SECTION}" | grep -qE "^#{2,4}[[:space:]]+${HEADING}\$"; then
    MISSING+=("${HEADING}")
  fi
done

if [[ "${#MISSING[@]}" -gt 0 ]]; then
  echo "FAIL: ${R} v0.13.0 section missing OP-9 template subheading(s):" >&2
  for M in "${MISSING[@]}"; do
    echo "  - ${M}" >&2
  done
  exit 1
fi

echo "PASS: ${R} v0.13.0 section present with all 5 OP-9 template subheadings"
exit 0
