#!/usr/bin/env bash
# quality/gates/agent-ux/milestone-adversarial-pass.sh — RBF-FW-12 (D90-09).
# Implements catalog row agent-ux/milestone-adversarial-pass.
#
# Thin wrapper: runs the adversarial-gate regression cases in
# quality/runners/test_verdict.py (artifact-absent-forces-red,
# empty-rows-failed-follows-compute_color, failed-row-forces-red, plus the
# darken-only-never-lightens case). This mirrors the existing sibling
# pattern (e.g. test-name-vs-asserts.sh runs a scan directly; this one runs
# a unittest suite directly) rather than re-implementing the assertions in
# bash — the suite IS the assertion.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

if ! python3 -m unittest quality.runners.test_verdict -v 2>&1 | tail -40; then
  echo "FAIL: quality.runners.test_verdict adversarial-gate cases did not pass" >&2
  echo "owner_hint: milestone_adversarial_gate in quality/runners/verdict.py must block GREEN when the adversarial artifact is absent or reports >=1 failed row, and must never lighten an already-red verdict" >&2
  exit 1
fi

echo "PASS: milestone-adversarial-pass regression cases green"
