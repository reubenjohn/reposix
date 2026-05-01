#!/usr/bin/env bash
# v0.13.0 tag-script presence verifier (TINY).
#
# Asserts:
#  1. .planning/milestones/v0.13.0-phases/tag-v0.13.0.sh exists.
#  2. The script is executable.
#  3. The script contains >=6 guards (matched as `# Guard N:` comments)
#     mirroring the v0.12.0/v0.11.x precedent.
#  4. The script contains a `git tag -s` invocation (signed-tag attempt).
#
# Owner-hint on RED: re-author tag-v0.13.0.sh with the >=6 guard pattern.
# Reference: .planning/milestones/v0.12.0-phases/tag-v0.12.0.sh.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

TS=".planning/milestones/v0.13.0-phases/tag-v0.13.0.sh"
if [[ ! -f "${TS}" ]]; then
  echo "FAIL: ${TS} not found" >&2
  exit 1
fi

if [[ ! -x "${TS}" ]]; then
  echo "FAIL: ${TS} not executable (chmod +x ${TS})" >&2
  exit 1
fi

GUARD_COUNT=$(grep -cE '^# Guard [0-9]+:' "${TS}" || true)
if [[ "${GUARD_COUNT}" -lt 6 ]]; then
  echo "FAIL: ${TS} has only ${GUARD_COUNT} '# Guard N:' comments; expected >=6" >&2
  exit 1
fi

if ! grep -qE 'git tag -s' "${TS}"; then
  echo "FAIL: ${TS} missing 'git tag -s' (signed-tag attempt)" >&2
  exit 1
fi

echo "PASS: ${TS} present (executable; ${GUARD_COUNT} guards; signed-tag invocation present)"
exit 0
