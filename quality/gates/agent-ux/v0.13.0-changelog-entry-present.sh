#!/usr/bin/env bash
# v0.13.0 CHANGELOG entry presence verifier (TINY).
#
# Asserts:
#  1. CHANGELOG.md exists.
#  2. CHANGELOG.md contains a heading line matching `^## \[v0.13.0\]`.
#  3. The v0.13.0 section is non-empty (>=10 non-blank lines between the
#     v0.13.0 heading and the next `^## \[` heading).
#
# Owner-hint on RED: re-run P88 CHANGELOG drafting — append the
# `## [v0.13.0] -- DVCS over REST -- YYYY-MM-DD` section above the
# previous entry.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

CL="CHANGELOG.md"
if [[ ! -f "${CL}" ]]; then
  echo "FAIL: ${CL} not found" >&2
  exit 1
fi

if ! grep -qE '^## \[v0\.13\.0\]' "${CL}"; then
  echo "FAIL: ${CL} missing '## [v0.13.0]' heading" >&2
  exit 1
fi

# Extract the v0.13.0 section: from the heading to the next `^## \[` (or EOF).
SECTION=$(awk '
  /^## \[v0\.13\.0\]/ { capturing=1; next }
  capturing && /^## \[/ { exit }
  capturing { print }
' "${CL}")

NONBLANK=$(printf '%s\n' "${SECTION}" | grep -cE '\S' || true)
if [[ "${NONBLANK}" -lt 10 ]]; then
  echo "FAIL: ${CL} v0.13.0 section is too short (${NONBLANK} non-blank lines; expected >=10)" >&2
  exit 1
fi

echo "PASS: ${CL} contains v0.13.0 section (${NONBLANK} non-blank lines)"
exit 0
