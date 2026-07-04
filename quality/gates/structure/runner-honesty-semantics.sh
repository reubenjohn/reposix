#!/usr/bin/env bash
# quality/gates/structure/runner-honesty-semantics.sh — P90 90-02 verifier.
#
# Backs the six runner/validator honesty rows minted by 90-01:
#   structure/minted-at-write-once            (D90-03 / cross-AI H2)
#   structure/coverage-kind-required          (RBF-FW-06 / D90-05)
#   structure/verifier-missing-demotes        (RBF-FW-07a / cross-AI H4)
#   structure/skip-fail-closed-with-history   (RBF-FW-07b / M8, AMENDED D90-04)
#   structure/shell-subprocess-transcript-runtime (RBF-FW-08 / M6)
#   structure/asserts-congruence-grade-time   (F-K4b / ROADMAP SC2)
#
# All six behaviors are covered by the two python unittest suites below; the
# row's verifier.args entry (the row slug) is accepted for traceability but the
# suite is run whole (a single behavior cannot be green while the suite is red).
# The runner synthesizes the artifact JSON on exit 0 (mechanical-kind
# convention, per the sibling claim-vs-assertion-audit-required.sh).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

ROW_SLUG="${1:-<all>}"

python3 -m unittest quality.runners.test_audit_field quality.runners.test_realbackend > /dev/null

echo "PASS: runner honesty semantics (row=${ROW_SLUG}): test_audit_field + test_realbackend suites green -- minted_at anchor, coverage_kind load-block, verifier-missing demote, skip fail-closed-with-history, shell-subprocess transcript-runtime gate, F-K4b per-expected-assert congruence (incl. p86 F6 regression)"
