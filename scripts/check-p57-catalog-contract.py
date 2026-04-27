#!/usr/bin/env python3
"""Asserts P57 catalog-first contract invariants.

Verifies that quality/catalogs/freshness-invariants.json carries exactly the
9 rows P57 Wave A committed to (B-4 fix expansion from 8 to 9 rows), with
the specific status/waiver/blast_radius shape each downstream wave depends
on. Run as part of Wave A verify; useful for any future agent confirming
the contract has not drifted.

Stdlib only. Exit 0 on success, 1 on first violation.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CATALOG = REPO_ROOT / "quality" / "catalogs" / "freshness-invariants.json"
ORPHANS = REPO_ROOT / "quality" / "catalogs" / "orphan-scripts.json"

EXPECTED_FRESHNESS_IDS = {
    "structure/no-version-pinned-filenames",
    "structure/install-leads-with-pkg-mgr-docs-index",
    "structure/install-leads-with-pkg-mgr-readme",
    "structure/benchmarks-in-mkdocs-nav",
    "structure/no-loose-roadmap-or-requirements",
    "structure/no-orphan-docs",
    "structure/top-level-requirements-roadmap-scope",
    "structure/badges-resolve",
    "structure/banned-words",
}


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def check_freshness() -> None:
    data = json.loads(CATALOG.read_text(encoding="utf-8"))
    if data["dimension"] != "structure":
        fail(f"freshness catalog dimension != structure: {data['dimension']!r}")
    rows = data["rows"]
    if len(rows) != 9:
        fail(f"expected 9 rows (B-4 fix), got {len(rows)}")
    ids = {r["id"] for r in rows}
    if ids != EXPECTED_FRESHNESS_IDS:
        diff = ids ^ EXPECTED_FRESHNESS_IDS
        fail(f"row id set drift: symmetric difference = {sorted(diff)}")
    for r in rows:
        if r["status"] != "NOT-VERIFIED":
            fail(f"row {r['id']}: status must be NOT-VERIFIED at Wave A close, got {r['status']!r}")
        if r["cadence"] != "pre-push":
            fail(f"row {r['id']}: cadence must be pre-push, got {r['cadence']!r}")
    badge = next(r for r in rows if r["id"] == "structure/badges-resolve")
    if badge["waiver"] is None:
        fail("structure/badges-resolve waiver must be set (P60 carry-forward)")
    if badge["waiver"]["tracked_in"] != "BADGE-01 / P60":
        fail(f"badges-resolve waiver.tracked_in mismatch: {badge['waiver']['tracked_in']!r}")
    banned = next(r for r in rows if r["id"] == "structure/banned-words")
    if banned["waiver"] is not None:
        fail("structure/banned-words must NOT carry a waiver — SIMPLIFY-01 row is canonical")
    if banned["blast_radius"] != "P1":
        fail(f"banned-words blast_radius must be P1, got {banned['blast_radius']!r}")
    if banned["verifier"]["script"] != "quality/gates/structure/banned-words.sh":
        fail(f"banned-words verifier.script wired to wrong path: {banned['verifier']['script']!r}")
    qg08 = next(r for r in rows if r["id"] == "structure/top-level-requirements-roadmap-scope")
    if len(qg08["expected"]["asserts"]) < 3:
        fail("QG-08 row must carry at least 3 expected.asserts naming v0.11/v0.10/v0.9 historical sections")


def check_orphans() -> None:
    data = json.loads(ORPHANS.read_text(encoding="utf-8"))
    if data["dimension"] != "meta":
        fail(f"orphan-scripts dimension != meta: {data['dimension']!r}")
    rows = data["rows"]
    if len(rows) != 1:
        fail(f"orphan-scripts must seed exactly 1 W-2 waiver row, got {len(rows)}")
    r = rows[0]
    if r["id"] != "release/crates-io-max-version":
        fail(f"orphan-scripts seed id mismatch: {r['id']!r}")
    if r["status"] != "WAIVED":
        fail(f"orphan-scripts seed status must be WAIVED, got {r['status']!r}")
    if r["waiver"]["tracked_in"] != "SIMPLIFY-04/05 P58":
        fail(f"orphan-scripts seed tracked_in mismatch: {r['waiver']['tracked_in']!r}")
    if r["waiver"]["until"] != "2026-05-15T00:00:00Z":
        fail(f"orphan-scripts seed until mismatch: {r['waiver']['until']!r}")


def main() -> int:
    check_freshness()
    check_orphans()
    print("P57 catalog-first contract: OK (freshness 9 rows + orphans 1 W-2 waiver)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
