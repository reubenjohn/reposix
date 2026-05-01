#!/usr/bin/env python3
"""Validate P83 catalog rows in quality/catalogs/agent-ux.json.

Used by P83-02:
- T01: confirm 4 new P83-02 rows exist (initial FAIL is fine)
- T04: confirm all 8 P83 rows have status=PASS after the gate runner re-grades

Modes:
- `--mode t01`: just check the 4 P83-02 rows are present (any status)
- `--mode t04`: check all 8 P83 rows are PASS (final phase-close gate)

Stdlib only. Per CLAUDE.md §4: a named script the next agent recognizes.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CATALOG = REPO_ROOT / "quality" / "catalogs" / "agent-ux.json"

P83_02_ROW_IDS = [
    "agent-ux/bus-write-fault-injection-mirror-fail",
    "agent-ux/bus-write-fault-injection-sot-mid-stream",
    "agent-ux/bus-write-fault-injection-post-precheck-409",
    "agent-ux/bus-write-audit-completeness",
]

P83_01_PASS_PREEXISTING = [
    "agent-ux/bus-write-sot-first-success",
    "agent-ux/bus-write-no-helper-retry",
    "agent-ux/bus-write-no-mirror-remote-still-fails",
]

P83_01_LINGERING = "agent-ux/bus-write-mirror-fail-returns-ok"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=["t01", "t04"], required=True)
    args = ap.parse_args()

    with open(CATALOG, encoding="utf-8") as f:
        data = json.load(f)
    rows = {r["id"]: r["status"] for r in data["rows"]}

    if args.mode == "t01":
        missing = [r for r in P83_02_ROW_IDS if r not in rows]
        if missing:
            print(f"FAIL: missing rows: {missing}")
            return 1
        print(f"PASS: all 4 P83-02 rows present in agent-ux.json (total rows: {len(rows)})")
        return 0

    # t04: full P83 phase-close gate.
    required = (
        [P83_01_LINGERING] + P83_02_ROW_IDS + P83_01_PASS_PREEXISTING
    )
    not_pass = [(r, rows.get(r)) for r in required if rows.get(r) != "PASS"]
    if not_pass:
        print(f"FAIL: rows not PASS: {not_pass}")
        return 1

    # Sanity: P83-01 PASS rows did not regress.
    regressions = [
        r for r in P83_01_PASS_PREEXISTING if rows.get(r) != "PASS"
    ]
    if regressions:
        print(f"FAIL: P83-01 rows regressed: {regressions}")
        return 1

    print("PASS: all 8 P83 rows now PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
