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
# EXEMPT -- a row that asserts NO verifier-backed GREEN, so a missing/absent
# verifier is NOT a false-green:
#   - status WAIVED or NOT-VERIFIED (and the STALE display-flavor, which
#     persists as NOT-VERIFIED) -- explicitly "waived / not run".
#   - verifier.script null/absent AND status FAIL or PARTIAL -- the row declares
#     no verifier AND asserts no green, so there is no unbacked-PASS hazard.
#
# FLAGGED even with a null verifier.script (the residual hole this close plugs):
#   - status PASS + verifier.script null/absent -- an UNBACKED PASS: a green with
#     NO verifier that could ever have produced it, exactly the GTH-V15-03 hazard.
#     Flagged DIRECTLY here, cadence-agnostically. The runner's NOT-VERIFIED flip
#     is NOT a reliable backstop -- run.py grades only rows in-scope for the
#     RUNNING cadence, so a PASS+null-script row scoped to (e.g.) `weekly` is never
#     re-graded at pre/post-push and rides green; the flip is a SECONDARY defense.
# This scope matches GTH-V15-03 exactly: it catches the unbacked GRADED claim
# (incl. a PASS with no verifier) while exempting catalog-first placeholders that
# honestly admit their incompleteness (WAIVED rehearsals, NOT-VERIFIED stubs, and
# FAIL/PARTIAL null-script rows that assert no green).
#
# SKIPPED CATALOGS (independent of row status):
#   - files whose name ends in `-allowlist.json` (a different, non-row schema
#     -- e.g. docs-reproducible-allowlist.json's {ids, reasons} shape, no
#     `rows` key at all)
#   - any catalog whose wrapper `dimension == "docs-alignment"` (doc-alignment.json
#     uses a DIFFERENT per-row schema -- last_verdict/last_extracted, no
#     verifier.script field at all; quality/catalogs/README.md documents this)
#
# For every IN-SCOPE row, three violation classes (each printed on its own line,
# catalog + row id + path + a concrete fix):
#   1. MISSING-FILE   -- verifier.script does not resolve to a file on disk
#   2. NON-EXECUTABLE -- the resolved file exists but lacks the +x bit
#   3. UNBACKED-PASS  -- status PASS but verifier.script is null/absent (a graded
#                        green with no verifier that could have produced it)
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
rows_in_scope = 0     # graded-outcome rows that are ENFORCED (declared script,
                      # or PASS+null-script -- the unbacked-PASS violation)
rows_exempt = 0       # non-graded status, OR FAIL/PARTIAL with a null verifier.script
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

        # NULL-SCRIPT split (the row declares no verifier at all):
        #   - PASS + null script is an UNBACKED PASS (a green with no verifier that
        #     could have produced it) -> VIOLATION, flagged directly here. The
        #     runner's NOT-VERIFIED flip is cadence-scoped, so it is not a reliable
        #     backstop (see header); this cadence-agnostic gate must catch it.
        #   - FAIL/PARTIAL + null script asserts NO green -> EXEMPT.
        if not script:
            if status == "PASS":
                rows_in_scope += 1
                violations.append(
                    f"{cat_path}::{rid}::(null)::"
                    "status:PASS but declares no verifier.script -- an unbacked PASS::"
                    "set a real verifier.script that produces this PASS, or change "
                    "status to WAIVED/NOT-VERIFIED with a justification"
                )
            else:
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
    "a PASS row with a null/absent verifier.script is FLAGGED as an unbacked PASS "
    "(GTH-V15-03) -- there is no verifier that could have produced that green, and "
    "the runner's cadence-scoped NOT-VERIFIED flip is not a reliable backstop",
    "a row that asserts NO green is EXEMPT -- status WAIVED/NOT-VERIFIED (the STALE "
    "flavor persists as NOT-VERIFIED), or a FAIL/PARTIAL row with a null verifier.script "
    "(it asserts no green, so a missing verifier there is not a false-green)",
    "a violation prints the offending catalog + row id + path + a concrete fix "
    "(chmod +x, author/correct the path, or set a real verifier.script / re-status)",
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
    f"WAIVED/NOT-VERIFIED, or FAIL/PARTIAL with a null verifier.script), all in-scope "
    f"rows carry a real +x verifier and no PASS is unbacked"
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
