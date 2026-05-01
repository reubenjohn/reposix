#!/usr/bin/env python3
"""Validate the 6 P84 webhook-mirror-sync catalog rows.

Run during T01 (after minting) and T06 (after flipping FAIL -> PASS).

Usage:
  python3 scripts/check_p84_catalog_rows.py              # presence-only check (T01)
  python3 scripts/check_p84_catalog_rows.py --require-pass  # also assert all status=PASS (T06)
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CATALOG = REPO_ROOT / "quality" / "catalogs" / "agent-ux.json"

REQUIRED_ROW_IDS = [
    "agent-ux/webhook-trigger-dispatch",
    "agent-ux/webhook-cron-fallback",
    "agent-ux/webhook-force-with-lease-race",
    "agent-ux/webhook-first-run-empty-mirror",
    "agent-ux/webhook-backends-without-webhooks",
    "agent-ux/webhook-latency-floor",
]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--require-pass",
        action="store_true",
        help="Assert each P84 row's status is PASS (T06 phase-close gate).",
    )
    args = parser.parse_args()

    data = json.loads(CATALOG.read_text())
    rows_by_id = {r["id"]: r for r in data["rows"]}

    missing = [rid for rid in REQUIRED_ROW_IDS if rid not in rows_by_id]
    if missing:
        print(f"FAIL: missing P84 rows: {missing}", file=sys.stderr)
        return 1

    print(f"All {len(REQUIRED_ROW_IDS)} P84 rows present in {CATALOG.relative_to(REPO_ROOT)}.")
    print(f"Total rows in catalog: {len(data['rows'])}")
    for rid in REQUIRED_ROW_IDS:
        r = rows_by_id[rid]
        print(f"  {rid}: status={r['status']} cadence={r['cadence']} kind={r['kind']}")

    if args.require_pass:
        not_pass = [rid for rid in REQUIRED_ROW_IDS if rows_by_id[rid]["status"] != "PASS"]
        if not_pass:
            print(f"FAIL: rows not PASS: {not_pass}", file=sys.stderr)
            return 1
        print("All P84 rows have status=PASS (T06 phase-close gate satisfied).")

    return 0


if __name__ == "__main__":
    sys.exit(main())
