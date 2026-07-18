#!/usr/bin/env bash
# quality/gates/agent-ux/cadence-pre-release-real-backend.sh — RBF-FW-01 verifier
# Grades catalog row: agent-ux/cadence-pre-release-real-backend (P89 89-03).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

# NOTE: run.py uses script-style absolute imports (`from _freshness import ...`),
# so `from quality.runners import run` breaks in package context. sys.path-prime
# the runners dir instead (same shim test_realbackend.py uses).

# Assert 1: VALID_CADENCES contains the new cadence
python3 -c "import sys; sys.path.insert(0, 'quality/runners'); import run; assert 'pre-release-real-backend' in run.VALID_CADENCES, 'cadence missing from VALID_CADENCES'"

# Assert 2: _realbackend.is_skipped returns True for empty env on a tagged row
python3 -c "import sys; sys.path.insert(0, 'quality/runners'); import _realbackend; assert _realbackend.is_skipped({'cadences': ['pre-release-real-backend']}, {}), 'is_skipped should return True for empty env'"

# Assert 3: unit + integration tests pass
python3 -m unittest quality.runners.test_realbackend > /dev/null

# Assert 4: exit-75 -> NOT-VERIFIED mapping wired
python3 -c "import sys; sys.path.insert(0, 'quality/runners'); import _realbackend; assert _realbackend.map_exit_code_to_status(75) == 'NOT-VERIFIED', 'exit-75 must map to NOT-VERIFIED'"

echo "PASS: pre-release-real-backend cadence wired (4/4 asserts)"
