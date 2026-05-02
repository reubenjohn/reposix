← [back to index](./index.md)

# Task 02-T01 — Author `no-loose-top-level-planning-audits.sh`

<read_first>
- `quality/gates/docs-alignment/jira-adapter-shipped.sh` (entire file — 23 lines; the TINY shape).
- `quality/gates/structure/freshness-invariants.py:274-300` (the existing Python implementation; translates 1:1 to shell `find ... | grep -v archive`).
- `quality/catalogs/freshness-invariants.json:324-360` (the row's `expected.asserts` text — verbatim assertion to implement).
</read_first>

<action>
Create `quality/gates/structure/no-loose-top-level-planning-audits.sh`:

```bash
#!/usr/bin/env bash
# P78-02 HYGIENE-02 — verifier for catalog row
# `structure/no-loose-top-level-planning-audits`. TINY shape mirrors
# quality/gates/docs-alignment/jira-adapter-shipped.sh.
#
# Asserts no *MILESTONE-AUDIT*.md or SESSION-END-STATE* file lives at
# .planning/ top level (excluding .planning/archive/). Per the catalog
# row's `expected.asserts`:
#   find .planning -maxdepth 1 -type f \( -name '*MILESTONE-AUDIT*.md'
#     -o -name 'SESSION-END-STATE*' \) | grep -v archive returns empty
#
# Owner-hint on RED: relocate the loose audit doc under
# .planning/milestones/audits/ or .planning/archive/.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"
OFFENDERS=$(find .planning -maxdepth 1 -type f \( -name '*MILESTONE-AUDIT*.md' -o -name 'SESSION-END-STATE*' \) 2>/dev/null | grep -v archive || true)
if [[ -n "${OFFENDERS}" ]]; then
  echo "FAIL: loose milestone-audit / session-end-state files at .planning/ top level:" >&2
  echo "${OFFENDERS}" >&2
  echo "owner_hint: relocate under .planning/milestones/audits/ or .planning/archive/" >&2
  exit 1
fi
echo "PASS: no loose *MILESTONE-AUDIT*.md / SESSION-END-STATE* files at .planning/ top level"
exit 0
```

Make executable: `chmod +x quality/gates/structure/no-loose-top-level-planning-audits.sh`.

Smoke-test from the repo root: `bash quality/gates/structure/no-loose-top-level-planning-audits.sh` MUST exit 0 and print `PASS: ...`.

Edit `quality/gates/structure/freshness-invariants.py` line ~274 — add a
one-line comment immediately above
`def verify_no_loose_top_level_planning_audits`:

```python
# P78-02 path-forward: quality/gates/structure/no-loose-top-level-planning-audits.sh
# is now the catalog verifier; this Python branch retained as regression net.
def verify_no_loose_top_level_planning_audits(row: dict, repo_root: Path) -> int:
```
</action>

<acceptance_criteria>
- File `quality/gates/structure/no-loose-top-level-planning-audits.sh` exists, is executable (`-rwxr-xr-x`), and is between 5 and 30 lines.
- `bash quality/gates/structure/no-loose-top-level-planning-audits.sh` exits 0 + prints `PASS: ...`.
- The script emits FAIL on a synthetic loose audit (smoke: `touch .planning/MILESTONE-AUDIT-FAKE.md && bash <script>; echo $? ; rm .planning/MILESTONE-AUDIT-FAKE.md` should print exit code 1).
- `freshness-invariants.py` has the path-forward comment above the Python branch.
</acceptance_criteria>
