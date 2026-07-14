← [back to index](./index.md)

# Task 02-T03 — Author `repo-org-audit-artifact-present.sh`

<read_first>
- `quality/gates/structure/freshness-invariants.py:520-` (Python impl — find lines via `grep -n "verify_repo_org_audit_artifact_present" -A 40`).
- `quality/catalogs/freshness-invariants.json:398-435` (row's `expected.asserts`).
- `quality/reports/audits/repo-org-gaps.md` (confirm the artifact actually exists at the catalog-cited path).
</read_first>

<action>
Create `quality/gates/structure/repo-org-audit-artifact-present.sh`:

```bash
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
# Sanity: artifact has a closure-path table. Loose check — table header line
# present anywhere in the file. The catalog's verbatim assertion mentions
# closed-by-catalog-row | closed-by-existing-gate | waived as the closure
# path categories; require at least one row using that vocabulary.
if ! grep -qE 'closed-by-catalog-row|closed-by-existing-gate|waived' "${ARTIFACT}"; then
  echo "FAIL: ${ARTIFACT} missing closure-path vocabulary (closed-by-catalog-row | closed-by-existing-gate | waived)" >&2
  exit 1
fi
echo "PASS: repo-org-audit artifact present at ${ARTIFACT} with closure-path table"
exit 0
```

Make executable. Smoke-test exits 0 (artifact exists per the original P62
Wave 2 work).

Edit `freshness-invariants.py` ~line 520 — path-forward one-line comment.

If the artifact does NOT exist (smoke-test fails), DO NOT proceed with the
flip. This row's WAIVED status persists for a reason — the absent artifact
is the failure the row was designed to catch. STOP, append to
SURPRISES-INTAKE.md (severity HIGH: "P78-02 cannot flip
repo-org-audit-artifact-present to PASS — artifact missing at <path>"), and
either (a) re-author the artifact in P78 if < 1hr / no new dep, or (b) defer
to P87 surprises absorption with a clear note.
</action>

<acceptance_criteria>
- File exists, executable, between 5 and 30 lines.
- `bash quality/gates/structure/repo-org-audit-artifact-present.sh` exits 0 + prints `PASS: ...`.
- Synthetic FAIL smoke (rename the artifact then restore): `mv quality/reports/audits/repo-org-gaps.md /tmp/ && bash <script>; rc=$?; mv /tmp/repo-org-gaps.md quality/reports/audits/; [[ $rc == 1 ]]` succeeds.
- `freshness-invariants.py` has path-forward comment.
</acceptance_criteria>
