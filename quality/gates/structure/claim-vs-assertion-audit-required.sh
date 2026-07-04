#!/usr/bin/env bash
# quality/gates/structure/claim-vs-assertion-audit-required.sh — RBF-FW-11 verifier
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

python3 -m unittest quality.runners.test_audit_field > /dev/null
echo "PASS: claim_vs_assertion_audit field validation (15/15 unit tests, including kind:shell-subprocess transcript_path sub-rule)"
