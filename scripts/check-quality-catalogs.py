#!/usr/bin/env python3
# KEEP-AS-CANONICAL (D-CONV-3, 2026-07-04): no canonical home; meta-helper /
# lightweight generic schema validator for quality/catalogs/*.json.
#
# scripts/check_quality_catalogs.py (underscore twin) was DELETED in the same
# commit as this rewrite -- it hardcoded per-catalog row counts + required-id
# sets (release-assets=15 rows, code=6 rows, orphan-scripts=16 rows, ...)
# that go stale every time a catalog grows, and both scripts had drifted onto
# the retired scalar `cadence` key (SURPRISES-14). This generic validator
# checks the UNIFIED schema shape itself (quality/catalogs/README.md) rather
# than any one catalog's current row inventory, so it doesn't need editing
# every time a row is added/removed. Cross-catalog id uniqueness (the one
# check neither twin had) is added here as the residual value this script
# earns its keep with.
"""Lightweight schema validator for quality/catalogs/*.json.

Checks structural invariants the runner depends on (per the unified schema
table in quality/catalogs/README.md):
- Wrapper keys ($schema, comment, dimension, rows) present.
- Each row has the required unified-schema fields.
- cadences/kind/status/blast_radius vocabularies are valid.
- Waiver shape (when present) has until/reason/dimension_owner/tracked_in.
- Every row id is globally unique across all checked catalogs.

The `docs-alignment` dimension catalog (doc-alignment.json) is skipped for
per-row checks -- it uses a distinct per-dimension schema (source_hash /
test_body_hashes / last_verdict, no cadences/kind/verifier) documented
separately in quality/catalogs/README.md § "docs-alignment dimension".
Mirrors the same carve-out as quality/runners/_audit_field.py.

Stdlib only. No third-party deps. Exits 0 on success, 1 on first failure.

Usage:
  python3 scripts/check-quality-catalogs.py             # check all catalogs
  python3 scripts/check-quality-catalogs.py <file>      # check one
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CATALOG_DIR = REPO_ROOT / "quality" / "catalogs"

VALID_DIMENSIONS = {
    "code", "docs-build", "docs-repro", "release",
    "structure", "agent-ux", "perf", "security",
    "cross-platform", "docs-alignment", "meta", "subjective",
}
VALID_CADENCES = {
    "pre-commit", "pre-push", "pre-pr", "weekly", "pre-release",
    "post-release", "on-demand", "pre-release-real-backend",
}
VALID_KINDS = {
    "mechanical", "container", "asset-exists", "subagent-graded", "manual",
    "shell-subprocess",
}
VALID_STATUSES = {"PASS", "FAIL", "PARTIAL", "NOT-VERIFIED", "WAIVED"}
VALID_BLAST = {"P0", "P1", "P2"}
WAIVER_KEYS = {"until", "reason", "dimension_owner", "tracked_in"}
ROW_REQUIRED = [
    "id", "dimension", "cadences", "kind", "sources", "command", "expected",
    "verifier", "artifact", "status", "last_verified", "freshness_ttl",
    "blast_radius", "owner_hint", "waiver",
]


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def check_row(catalog_path: Path, idx: int, row: dict, seen_ids: dict[str, str]) -> None:
    where = f"{catalog_path.name}[{idx}] id={row.get('id', '<missing>')}"
    for key in ROW_REQUIRED:
        if key not in row:
            fail(f"{where}: missing field {key!r}")
    if row["dimension"] not in VALID_DIMENSIONS:
        fail(f"{where}: dimension {row['dimension']!r} not in {sorted(VALID_DIMENSIONS)}")
    cadences = row["cadences"]
    if not isinstance(cadences, list) or not cadences:
        fail(f"{where}: cadences must be a non-empty list, got {cadences!r}")
    bad = set(cadences) - VALID_CADENCES
    if bad:
        fail(f"{where}: cadences {sorted(bad)} not in {sorted(VALID_CADENCES)}")
    if row["kind"] not in VALID_KINDS:
        fail(f"{where}: kind {row['kind']!r} not in {sorted(VALID_KINDS)}")
    if row["status"] not in VALID_STATUSES:
        fail(f"{where}: status {row['status']!r} not in {sorted(VALID_STATUSES)}")
    if row["blast_radius"] not in VALID_BLAST:
        fail(f"{where}: blast_radius {row['blast_radius']!r} not in {sorted(VALID_BLAST)}")
    ver = row["verifier"]
    if not isinstance(ver, dict) or "script" not in ver or "timeout_s" not in ver:
        fail(f"{where}: verifier missing script/timeout_s")
    if not isinstance(row["expected"], dict) or "asserts" not in row["expected"]:
        fail(f"{where}: expected.asserts missing")
    if row["waiver"] is not None:
        missing = WAIVER_KEYS - set(row["waiver"].keys())
        if missing:
            fail(f"{where}: waiver missing {sorted(missing)}")

    row_id = row.get("id")
    if row_id in seen_ids:
        fail(
            f"{where}: duplicate id {row_id!r} -- also present in "
            f"{seen_ids[row_id]} (row ids MUST be globally unique across "
            f"every catalog, not just within one file)"
        )
    if row_id is not None:
        seen_ids[row_id] = catalog_path.name


def check_catalog(path: Path, seen_ids: dict[str, str]) -> int:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        fail(f"{path}: invalid JSON: {e}")
        return 1  # unreachable
    for key in ("$schema", "comment", "dimension", "rows"):
        if key not in data:
            fail(f"{path}: wrapper missing {key!r}")
    if not isinstance(data["rows"], list):
        fail(f"{path}: rows must be a list")
    if data["dimension"] not in VALID_DIMENSIONS:
        fail(f"{path}: dimension {data['dimension']!r} not valid")

    if data["dimension"] == "docs-alignment":
        # Distinct per-dimension schema (README.md § "docs-alignment
        # dimension") -- rows have no cadences/kind/verifier. Wrapper shape
        # already checked above; skip the unified per-row checks.
        print(f"OK: {path.name} dimension=docs-alignment rows={len(data['rows'])} (schema exempt)")
        return 0

    for idx, row in enumerate(data["rows"]):
        check_row(path, idx, row, seen_ids)
    print(f"OK: {path.name} dimension={data['dimension']} rows={len(data['rows'])}")
    return 0


def main() -> int:
    if len(sys.argv) > 1:
        targets = [Path(p) for p in sys.argv[1:]]
    else:
        # Skip allow-list sidecars (different schema; not catalogs).
        targets = sorted(
            p for p in CATALOG_DIR.glob("*.json") if not p.stem.endswith("-allowlist")
        )
    if not targets:
        print(f"no catalogs found under {CATALOG_DIR}", file=sys.stderr)
        return 1
    seen_ids: dict[str, str] = {}
    for p in targets:
        check_catalog(p, seen_ids)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
