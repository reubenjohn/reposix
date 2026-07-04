#!/usr/bin/env bash
# quality/gates/agent-ux/absorption-honesty-template-present.sh — RBF-FW-10
# (F-K5, D90-08). Implements catalog row
# agent-ux/absorption-honesty-template-present.
#
# Asserts, in order:
#  1. quality/dispatch/absorption-honesty-spot-check.md exists.
#  2. its content sha256 matches EXPECTED_SHA below (content-hash binding —
#     a gutted-but-present file must FAIL, not just an existence check).
#  3. all four F-K5 clauses are grep-present verbatim:
#     (a) sample EVERY no-intake phase
#     (b) spot-check author != milestone orchestrator
#     (c) rubric "walk one critical example end-to-end mentally — does it work?"
#     (d) content-hash binding (verifier hash-binds, not mere existence)
#
# Hash-binding mechanism (per the plan's "Hash-binding mechanism" note):
# EXPECTED_SHA is the single source of truth. Editing the template requires
# updating this constant in the SAME commit — that IS the binding contract.
# Recompute with: sha256sum quality/dispatch/absorption-honesty-spot-check.md
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

TEMPLATE="quality/dispatch/absorption-honesty-spot-check.md"
EXPECTED_SHA="15daa649bf81793229ea738fdb669bca864e7203fa3ec44eefe29f2ca8feec67"

if [[ ! -f "${TEMPLATE}" ]]; then
  echo "FAIL: ${TEMPLATE} not found" >&2
  echo "owner_hint: the F-K5 absorption-honesty template must exist (D90-08); see quality/PROTOCOL.md § absorption honesty spot-check" >&2
  exit 1
fi

ACTUAL_SHA="$(sha256sum "${TEMPLATE}" | awk '{print $1}')"
if [[ "${ACTUAL_SHA}" != "${EXPECTED_SHA}" ]]; then
  echo "FAIL: ${TEMPLATE} content sha256 mismatch (content-hash binding failed)" >&2
  echo "  expected: ${EXPECTED_SHA}" >&2
  echo "  actual:   ${ACTUAL_SHA}" >&2
  echo "owner_hint: if this edit was intentional, update EXPECTED_SHA in this script in the SAME commit (that update IS the binding contract)" >&2
  exit 1
fi

missing_clauses=0

check_clause() {
  local label="$1"
  local pattern="$2"
  if ! grep -qF "${pattern}" "${TEMPLATE}"; then
    echo "FAIL: clause ${label} not found verbatim in ${TEMPLATE} (looked for: ${pattern})" >&2
    missing_clauses=1
  fi
}

check_clause "(a) sample EVERY no-intake phase" \
  "Sample MUST include every phase that closed without filing intake."
check_clause "(b) author != milestone orchestrator" \
  "Spot-check author ≠ milestone orchestrator."
check_clause "(c) rubric question" \
  "walk one critical example end-to-end mentally"
check_clause "(d) content-hash binding" \
  "Verifier hash-binds the spot-check content"

if [[ "${missing_clauses}" -eq 1 ]]; then
  echo "owner_hint: restore the missing F-K5 clause verbatim; see REMEDIATION-PLAN.md:124 for the canonical wording" >&2
  exit 1
fi

echo "PASS: ${TEMPLATE} exists, content-hash matches, all four F-K5 clauses present"
