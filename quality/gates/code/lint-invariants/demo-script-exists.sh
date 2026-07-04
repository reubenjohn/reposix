#!/usr/bin/env bash
# quality/gates/code/lint-invariants/demo-script-exists.sh
# Binds catalog row: docs-development-contributing-md/demo-script-exists
#
# Asserts `quality/gates/agent-ux/dark-factory.sh` exists and is executable.
# Cheapest verifier in the lint-invariants sub-area: one filesystem stat. No
# cargo. D-CONV-3 (2026-07-04): retargeted from the scripts/dark-factory-test.sh
# shim (deleted) to the canonical path it always delegated to.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
readonly DEMO="${REPO_ROOT}/quality/gates/agent-ux/dark-factory.sh"

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
