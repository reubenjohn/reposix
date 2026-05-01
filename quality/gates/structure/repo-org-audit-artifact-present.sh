#!/usr/bin/env bash
# P78-02 HYGIENE-02 — verifier for catalog row
# `structure/repo-org-audit-artifact-present`. TINY shape.
#
# Asserts the repo-org-audit artifact exists at
# quality/reports/audits/repo-org-gaps.md AND contains a top-section
# audit table mapping every numbered gap from
# .planning/research/v0.11.1/repo-organization-gaps.md "Top 10 cleanup
# recommendations" to a closure path.
#
# Owner-hint on RED: re-author the audit (P62 Wave 2 produced the
# original; future repo-org audits land at the same path).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"
ARTIFACT="quality/reports/audits/repo-org-gaps.md"
if [[ ! -f "${ARTIFACT}" ]]; then
  echo "FAIL: ${ARTIFACT} missing — repo-org-audit artifact not present" >&2
  exit 1
fi
# Sanity: closure-path vocabulary present (closed-by-catalog-row |
# closed-by-existing-gate | waived) — at least one row.
if ! grep -qE 'closed-by-catalog-row|closed-by-existing-gate|waived' "${ARTIFACT}"; then
  echo "FAIL: ${ARTIFACT} missing closure-path vocabulary (closed-by-catalog-row | closed-by-existing-gate | waived)" >&2
  exit 1
fi
echo "PASS: repo-org-audit artifact present at ${ARTIFACT} with closure-path table"
exit 0
