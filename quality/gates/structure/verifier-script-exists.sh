#!/usr/bin/env bash
# quality/gates/structure/verifier-script-exists.sh
#
# Verifier for catalog row structure/verifier-script-exists (P123/SC4,
# DRAIN-06, cites GTH-V15-03 -- P104 discovery, 2026-07-12). Closes a
# framework-integrity hole: a catalog row could be minted `status: PASS`
# with a `verifier.script` path that does not exist on disk (or exists but
# lacks the executable bit), and nothing structurally prevented that
# unbacked PASS from riding to green.
#
# Scans every quality/catalogs/*.json row's `verifier.script` field, EXCEPT:
#   - files whose name ends in `-allowlist.json` (a different, non-row schema
#     -- e.g. docs-reproducible-allowlist.json's {ids, reasons} shape, no
#     `rows` key at all)
#   - any catalog whose wrapper `dimension == "docs-alignment"` (doc-alignment.json
#     uses a DIFFERENT per-row schema -- last_verdict/last_extracted, no
#     verifier.script field at all; quality/catalogs/README.md documents this)
#
# For every remaining row, three violation classes (each printed on its own
# line, catalog + row id + path + a concrete fix):
#   1. MISSING-FIELD  -- verifier.script is null/absent
#   2. MISSING-FILE   -- verifier.script does not resolve to a file on disk
#   3. NON-EXECUTABLE -- the resolved file exists but lacks the +x bit
#
# Real JSON parsing (python3 -c), not grep -- catalog rows are structured
# data and a regex scan over JSON risks both false positives (a "script"
# substring inside an unrelated string field) and false negatives (a
# multi-line JSON value a naive regex doesn't span).
#
# Writes the standard per-row JSON artifact to
# quality/reports/verifications/structure/verifier-script-exists.json.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

ARTIFACT="quality/reports/verifications/structure/verifier-script-exists.json"
mkdir -p "$(dirname "$ARTIFACT")"

PYCODE=$(cat <<'PYEOF'
import datetime
import glob
import json
import os
import sys

repo_root = os.getcwd()
row_id = "structure/verifier-script-exists"
artifact_path = sys.argv[1]

violations = []       # list of pre-formatted "cat::row::path::reason::fix" lines
rows_checked = 0
catalogs_checked = 0

for cat_path in sorted(glob.glob("quality/catalogs/*.json")):
    base = os.path.basename(cat_path)
    if base.endswith("-allowlist.json"):
        continue
    with open(cat_path, encoding="utf-8") as f:
        data = json.load(f)
    if data.get("dimension") == "docs-alignment":
        continue
    catalogs_checked += 1
    for row in data.get("rows", []):
        rows_checked += 1
        rid = row.get("id", "(no id)")
        verifier = row.get("verifier") or {}
        script = verifier.get("script") if isinstance(verifier, dict) else None

        if not script:
            violations.append(
                f"{cat_path}::{rid}::(no verifier.script field)::"
                "this row has no verifier.script — every non-docs-alignment row requires one"
            )
            continue

        resolved = os.path.join(repo_root, script)
        if not os.path.isfile(resolved):
            violations.append(
                f"{cat_path}::{rid}::{script}::file does not exist::"
                "author the missing verifier, or correct verifier.script if the file moved"
            )
            continue

        if not os.access(resolved, os.X_OK):
            violations.append(
                f"{cat_path}::{rid}::{script}::not executable::chmod +x {script}"
            )

asserts = [
    "every row with a non-null verifier.script in every quality/catalogs/*.json "
    "(excluding *-allowlist.json and docs-alignment) resolves to a file that exists on disk",
    "every resolved verifier.script carries the executable bit (os.access(path, os.X_OK))",
    "a row whose verifier.script is null/absent (outside docs-alignment) is ALSO a violation",
    "a violation prints the offending catalog + row id + path + a concrete fix "
    "(chmod +x, or author/correct the path)",
]

ts = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

if violations:
    for line in violations:
        print(line, file=sys.stderr)
    print(
        f"FAIL ({row_id}): {len(violations)} violation(s) across "
        f"{rows_checked} rows in {catalogs_checked} catalogs",
        file=sys.stderr,
    )
    artifact = {
        "ts": ts,
        "row_id": row_id,
        "exit_code": 1,
        "status": "FAIL",
        "asserts_passed": [],
        "asserts_failed": violations,
    }
    with open(artifact_path, "w", encoding="utf-8") as f:
        json.dump(artifact, f, indent=2)
        f.write("\n")
    sys.exit(1)

print(
    f"PASS: verifier-script-exists — {rows_checked} rows across "
    f"{catalogs_checked} catalogs, all verifier.script paths exist and are executable"
)
artifact = {
    "ts": ts,
    "row_id": row_id,
    "exit_code": 0,
    "status": "PASS",
    "asserts_passed": asserts,
    "asserts_failed": [],
}
with open(artifact_path, "w", encoding="utf-8") as f:
    json.dump(artifact, f, indent=2)
    f.write("\n")
sys.exit(0)
PYEOF
)

python3 -c "$PYCODE" "$ARTIFACT"
