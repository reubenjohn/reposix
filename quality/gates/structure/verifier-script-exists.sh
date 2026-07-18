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
# ── SCOPE: GRADED-OUTCOME rows only (refined 2026-07-18, P123 close) ─────────
# The violation fires ONLY for a row that asserts a GRADED RUN RESULT --
# status in {PASS, FAIL, PARTIAL} (the canonical graded-outcome triple, mirror
# of run.py's `("PASS","FAIL","PARTIAL")` real-grade set). Such a row claims a
# verifier PRODUCED that verdict, so a missing/broken verifier there IS the
# integrity hazard GTH-V15-03 names (a false-green riding on a verifier that
# cannot run).
#
# EXEMPT -- a row that asserts NO verifier-backed result, so a missing/absent
# verifier is NOT a false-green:
#   - status WAIVED or NOT-VERIFIED (and the STALE display-flavor, which
#     persists as NOT-VERIFIED) -- explicitly "waived / not run".
#   - verifier.script null/absent -- declares no verifier at all. (A graded row
#     that lacks a runnable verifier is separately flipped to NOT-VERIFIED by
#     the runner's honesty machinery at grade time; this static gate does not
#     double-police that path, and a null-script row asserts no result to back.)
# This graded-outcome scope matches GTH-V15-03's intent exactly: it catches the
# unbacked GRADED claim while exempting catalog-first placeholders that honestly
# admit their own incompleteness (WAIVED cross-platform rehearsals, NOT-VERIFIED
# animation/benchmark stubs).
#
# SKIPPED CATALOGS (independent of row status):
#   - files whose name ends in `-allowlist.json` (a different, non-row schema
#     -- e.g. docs-reproducible-allowlist.json's {ids, reasons} shape, no
#     `rows` key at all)
#   - any catalog whose wrapper `dimension == "docs-alignment"` (doc-alignment.json
#     uses a DIFFERENT per-row schema -- last_verdict/last_extracted, no
#     verifier.script field at all; quality/catalogs/README.md documents this)
#
# For every IN-SCOPE row, two violation classes (each printed on its own line,
# catalog + row id + path + a concrete fix):
#   1. MISSING-FILE   -- verifier.script does not resolve to a file on disk
#   2. NON-EXECUTABLE -- the resolved file exists but lacks the +x bit
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

# The canonical graded-outcome triple (mirror of run.py's real-grade set). A row
# whose status is one of these asserts a verifier PRODUCED that verdict, so its
# verifier.script must be real + runnable. Any other status (WAIVED, NOT-VERIFIED,
# or the STALE flavor that persists as NOT-VERIFIED) asserts no such result.
GRADED_OUTCOME = ("PASS", "FAIL", "PARTIAL")

violations = []       # list of pre-formatted "cat::row::path::reason::fix" lines
rows_seen = 0
rows_in_scope = 0     # graded-outcome rows WITH a declared verifier.script
rows_exempt = 0       # non-graded status OR null verifier.script
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
        rows_seen += 1
        rid = row.get("id", "(no id)")
        status = row.get("status")
        verifier = row.get("verifier") or {}
        script = verifier.get("script") if isinstance(verifier, dict) else None

        # EXEMPTION 1: the row asserts NO graded outcome -> makes no
        # verifier-backed claim -> a missing/absent verifier is not a false-green.
        if status not in GRADED_OUTCOME:
            rows_exempt += 1
            continue

        # EXEMPTION 2: the row declares no verifier at all (null/absent script).
        # It asserts no verifier-backed result; the runner separately flips a
        # graded row lacking a runnable verifier to NOT-VERIFIED at grade time.
        if not script:
            rows_exempt += 1
            continue

        # IN SCOPE: a graded-outcome row with a declared verifier.script -- that
        # script MUST resolve to a file AND be executable, else the graded verdict
        # rides on a verifier that cannot run (GTH-V15-03).
        rows_in_scope += 1
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
    "every GRADED-OUTCOME row (status in {PASS, FAIL, PARTIAL}) that declares a "
    "non-null verifier.script, in every quality/catalogs/*.json (excluding "
    "*-allowlist.json and docs-alignment), resolves to a file that exists on disk",
    "every such in-scope verifier.script carries the executable bit (os.access(path, os.X_OK))",
    "a row that asserts NO graded outcome is EXEMPT -- status WAIVED/NOT-VERIFIED "
    "(the STALE flavor persists as NOT-VERIFIED), or verifier.script null/absent -- "
    "because it makes no verifier-backed claim, so a missing/absent verifier there "
    "is not a false-green",
    "a violation prints the offending catalog + row id + path + a concrete fix "
    "(chmod +x, or author/correct the path)",
]

ts = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

if violations:
    for line in violations:
        print(line, file=sys.stderr)
    print(
        f"FAIL ({row_id}): {len(violations)} violation(s) across "
        f"{rows_in_scope} in-scope rows in {catalogs_checked} catalogs "
        f"({rows_seen} rows seen, {rows_exempt} exempt)",
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
    f"PASS: verifier-script-exists — {rows_in_scope} in-scope graded-outcome rows "
    f"across {catalogs_checked} catalogs ({rows_seen} rows seen, {rows_exempt} exempt: "
    f"WAIVED/NOT-VERIFIED or null verifier.script), all in-scope verifier.script "
    f"paths exist and are executable"
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
