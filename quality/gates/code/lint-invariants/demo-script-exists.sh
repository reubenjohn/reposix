#!/usr/bin/env bash
# quality/gates/code/lint-invariants/demo-script-exists.sh
# Binds catalog row: docs-development-contributing-md/demo-script-exists
#
# Asserts `scripts/dark-factory-test.sh` exists and is executable. Cheapest
# verifier in the lint-invariants sub-area: one filesystem stat. No cargo.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
readonly DEMO="${REPO_ROOT}/scripts/dark-factory-test.sh"

if [ ! -e "$DEMO" ]; then
  echo "FAIL: demo script does not exist: $DEMO" >&2
  exit 1
fi
if [ ! -x "$DEMO" ]; then
  echo "FAIL: demo script exists but is not executable: $DEMO" >&2
  ls -l "$DEMO" >&2
  exit 1
fi

echo "PASS: $DEMO exists and is executable"
exit 0
