#!/usr/bin/env python3
"""Lightweight schema validator for quality/catalogs/*.json.

Checks structural invariants the runner depends on:
- Wrapper keys ($schema, comment, dimension, rows) present.
- Each row has the required unified-schema fields.
- Cadence/status/blast_radius vocabularies are valid.
- Waiver shape (when present) has until/reason/dimension_owner/tracked_in.

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
    "structure", "agent-ux", "perf", "security", "meta",
}
VALID_CADENCES = {
    "pre-push", "pre-pr", "weekly", "pre-release", "post-release", "on-demand",
}
VALID_KINDS = {"mechanical", "container", "asset-exists", "subagent-graded", "manual"}
VALID_STATUSES = {"PASS", "FAIL", "PARTIAL", "NOT-VERIFIED", "WAIVED"}
VALID_BLAST = {"P0", "P1", "P2"}
WAIVER_KEYS = {"until", "reason", "dimension_owner", "tracked_in"}
ROW_REQUIRED = [
    "id", "dimension", "cadence", "kind", "sources", "command", "expected",
    "verifier", "artifact", "status", "last_verified", "freshness_ttl",
    "blast_radius", "owner_hint", "waiver",
]


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def check_row(catalog_path: Path, idx: int, row: dict) -> None:
    where = f"{catalog_path.name}[{idx}] id={row.get('id', '<missing>')}"
    for key in ROW_REQUIRED:
        if key not in row:
            fail(f"{where}: missing field {key!r}")
    if row["dimension"] not in VALID_DIMENSIONS:
        fail(f"{where}: dimension {row['dimension']!r} not in {sorted(VALID_DIMENSIONS)}")
    if row["cadence"] not in VALID_CADENCES:
        fail(f"{where}: cadence {row['cadence']!r} not in {sorted(VALID_CADENCES)}")
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


def check_catalog(path: Path) -> int:
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
    for idx, row in enumerate(data["rows"]):
        check_row(path, idx, row)
    print(f"OK: {path.name} dimension={data['dimension']} rows={len(data['rows'])}")
    return 0


def main() -> int:
    if len(sys.argv) > 1:
        targets = [Path(p) for p in sys.argv[1:]]
    else:
        targets = sorted(p for p in CATALOG_DIR.glob("*.json"))
    if not targets:
        print(f"no catalogs found under {CATALOG_DIR}", file=sys.stderr)
        return 1
    for p in targets:
        check_catalog(p)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
