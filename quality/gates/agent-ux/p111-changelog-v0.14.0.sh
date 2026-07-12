#!/usr/bin/env bash
# P111 CHANGELOG [v0.14.0] section presence verifier (TINY).
#
# Milestone-close hygiene item 8: CHANGELOG.md must carry a substantive
# `## [v0.14.0]` section marked PENDING (the tag is not cut by the
# executor — the owner cuts it, so the section ships in a PENDING state,
# mirroring the v0.13.0 precedent).
#
# Asserts:
#  1. CHANGELOG.md exists.
#  2. It contains a `## [v0.14.0]` heading.
#  3. The v0.14.0 section is substantive (>=10 non-blank lines between
#     the heading and the next `## [` heading).
#  4. The section carries a PENDING release-status marker (case-insensitive
#     'PENDING' somewhere inside the section) — the tag is owner-cut.
#
# Owner-hint on RED: append a `## [v0.14.0] -- <theme> -- PENDING` section
# above the `## [v0.13.0]` entry; include a `> **Release status: PENDING ...`
# line and the milestone's Added/Changed/Fixed bullets.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

CL="CHANGELOG.md"
if [[ ! -f "${CL}" ]]; then
  echo "FAIL: ${CL} not found" >&2
  exit 1
fi

if ! grep -qE '^## \[v0\.14\.0\]' "${CL}"; then
  echo "FAIL: ${CL} missing '## [v0.14.0]' heading" >&2
  exit 1
fi

# Extract the v0.14.0 section: from the heading to the next `^## \[` (or EOF).
SECTION=$(awk '
  /^## \[v0\.14\.0\]/ { capturing=1; next }
  capturing && /^## \[/ { exit }
  capturing { print }
' "${CL}")

NONBLANK=$(printf '%s\n' "${SECTION}" | grep -cE '\S' || true)
if [[ "${NONBLANK}" -lt 10 ]]; then
  echo "FAIL: ${CL} v0.14.0 section is too short (${NONBLANK} non-blank lines; expected >=10)" >&2
  exit 1
fi

if ! printf '%s\n' "${SECTION}" | grep -qiF 'PENDING'; then
  echo "FAIL: ${CL} v0.14.0 section lacks a PENDING release-status marker (tag is owner-cut; ship PENDING)" >&2
  exit 1
fi

echo "PASS: ${CL} contains a substantive PENDING v0.14.0 section (${NONBLANK} non-blank lines)"
exit 0
