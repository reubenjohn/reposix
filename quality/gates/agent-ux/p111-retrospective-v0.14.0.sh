#!/usr/bin/env bash
# P111 RETROSPECTIVE v0.14.0 distillation verifier (TINY, OP-9 ritual).
#
# Milestone-close hygiene item 6: RETROSPECTIVE.md must gain a v0.14.0
# section using the OP-9 template (all 5 subheadings) BEFORE archive, and
# it MUST name the GTH-09 -> v0.15.0 deferral explicitly (the deferral is
# only honest if it is written down with its carry-forward target).
#
# Asserts:
#  1. .planning/RETROSPECTIVE.md exists.
#  2. It contains a `## Milestone: v0.14.0` heading.
#  3. The v0.14.0 section contains all 5 OP-9 subheadings as markdown
#     headings: What Was Built / What Worked / What Was Inefficient /
#     Patterns Established / Key Lessons.
#  4. The section names BOTH `GTH-09` and `v0.15.0` (the deferral +
#     its carry-forward target).
#
# Owner-hint on RED: re-run the OP-9 distillation against the template
# (root CLAUDE.md OP-9). Ensure all 5 subheadings appear in the v0.14.0
# section and the GTH-09 -> v0.15.0 deferral is named.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

R=".planning/RETROSPECTIVE.md"
if [[ ! -f "${R}" ]]; then
  echo "FAIL: ${R} not found" >&2
  exit 1
fi

if ! grep -qE '^## Milestone: v0\.14\.0' "${R}"; then
  echo "FAIL: ${R} missing '## Milestone: v0.14.0' heading" >&2
  exit 1
fi

# Extract the v0.14.0 section: from heading to next `^## Milestone:` (or EOF).
SECTION=$(awk '
  /^## Milestone: v0\.14\.0/ { capturing=1; next }
  capturing && /^## Milestone:/ { exit }
  capturing { print }
' "${R}")

MISSING=()
for HEADING in "What Was Built" "What Worked" "What Was Inefficient" "Patterns Established" "Key Lessons"; do
  if ! printf '%s\n' "${SECTION}" | grep -qE "^#{2,4}[[:space:]]+${HEADING}\$"; then
    MISSING+=("${HEADING}")
  fi
done

if [[ "${#MISSING[@]}" -gt 0 ]]; then
  echo "FAIL: ${R} v0.14.0 section missing OP-9 template subheading(s):" >&2
  for M in "${MISSING[@]}"; do
    echo "  - ${M}" >&2
  done
  exit 1
fi

if ! printf '%s\n' "${SECTION}" | grep -qF 'GTH-09'; then
  echo "FAIL: ${R} v0.14.0 section does not name GTH-09 (the deferred good-to-have)" >&2
  exit 1
fi

if ! printf '%s\n' "${SECTION}" | grep -qF 'v0.15.0'; then
  echo "FAIL: ${R} v0.14.0 section does not name the v0.15.0 carry-forward target for GTH-09" >&2
  exit 1
fi

echo "PASS: ${R} v0.14.0 section present with all 5 OP-9 subheadings + GTH-09->v0.15.0 deferral named"
exit 0
