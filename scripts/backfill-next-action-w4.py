#!/usr/bin/env python3
"""W4/P68: heuristically populate Row.next_action across all rows.

Maps each existing row in `quality/catalogs/doc-alignment.json` to one of the
five `NextAction` enum variants based on its `last_verdict` + rationale prefix:

  - last_verdict == BOUND                          -> BIND_GREEN
  - last_verdict in (RETIRE_PROPOSED, RETIRE_CONFIRMED) -> RETIRE_FEATURE
  - rationale starts with "IMPL_GAP:"              -> FIX_IMPL_THEN_BIND
  - rationale starts with "DOC_DRIFT:"             -> UPDATE_DOC
  - otherwise                                       -> WRITE_TEST

Idempotent: every row is set unconditionally, so re-running yields the same
catalog. The catalog `schema_version` is NOT bumped — `next_action` is an
additive field guarded by `serde(default)` on the Rust side, so pre-W4
catalogs (no field) deserialize as `WRITE_TEST` cleanly.

Run once from repo root:
    python3 scripts/backfill-next-action-w4.py
"""
from __future__ import annotations

import json
import pathlib
import sys

CATALOG = pathlib.Path("quality/catalogs/doc-alignment.json")


def backfill() -> int:
    if not CATALOG.exists():
        print(f"ERROR: catalog not found at {CATALOG}", file=sys.stderr)
        return 1

    data = json.loads(CATALOG.read_text())
    rows = data.get("rows", [])

    counts = {
        "WRITE_TEST": 0,
        "FIX_IMPL_THEN_BIND": 0,
        "UPDATE_DOC": 0,
        "RETIRE_FEATURE": 0,
        "BIND_GREEN": 0,
    }

    for row in rows:
        verdict = row.get("last_verdict")
        rationale = (row.get("rationale") or "").strip()
        if verdict == "BOUND":
            action = "BIND_GREEN"
        elif verdict in ("RETIRE_PROPOSED", "RETIRE_CONFIRMED"):
            action = "RETIRE_FEATURE"
        elif rationale.startswith("IMPL_GAP:"):
            action = "FIX_IMPL_THEN_BIND"
        elif rationale.startswith("DOC_DRIFT:"):
            action = "UPDATE_DOC"
        else:
            action = "WRITE_TEST"
        row["next_action"] = action
        counts[action] += 1

    # Match the live formatter: 2-space indent, key order preserved
    # (sort_keys=False), trailing newline, ensure_ascii=False so
    # non-ASCII rationale text round-trips intact.
    body = json.dumps(data, indent=2, ensure_ascii=False, sort_keys=False)
    CATALOG.write_text(body + "\n")

    total = sum(counts.values())
    print(f"backfill complete: {total} rows")
    for k, v in counts.items():
        print(f"  {k:<22} {v}")
    return 0


if __name__ == "__main__":
    sys.exit(backfill())
