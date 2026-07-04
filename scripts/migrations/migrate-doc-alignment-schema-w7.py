#!/usr/bin/env python3
"""One-shot W7/P71 migration: Row.test/test_body_hash singular -> tests/test_body_hashes plural arrays.

Schema cross-cut #1 + #2 from .planning/HANDOVER-v0.12.1.md:
  - `Row.test: Option<String>`            -> `Row.tests: Vec<String>`
  - `Row.test_body_hash: Option<String>`  -> `Row.test_body_hashes: Vec<String>`

Empty `tests` vec replaces the `None` semantics (no test bound). Single-test
rows become 1-element vectors. The 388-row v0.12.1 catalog has at most one
`test` per row pre-migration, so post-migration every row has either
`tests.len() == 0` or `tests.len() == 1` -- multi-test rows arrive only via
the new W7 `bind --test --test ...` repeatable surface (W7b).

Idempotent: running twice is a no-op (the singular keys are gone after the
first run; the second run sees nothing to migrate).

Bumps `summary.schema_version` from "1.0" -> "2.0".

Run once from repo root:
    python3 scripts/migrate-doc-alignment-schema-w7.py
"""
from __future__ import annotations

import json
import pathlib
import sys

CATALOG = pathlib.Path("quality/catalogs/doc-alignment.json")


def migrate() -> int:
    if not CATALOG.exists():
        print(f"ERROR: catalog not found at {CATALOG}", file=sys.stderr)
        return 1

    data = json.loads(CATALOG.read_text())
    rows = data.get("rows", [])

    pre_summary = data.get("summary", {})
    print("== pre-migration ==")
    for k in (
        "claims_total",
        "claims_bound",
        "claims_missing_test",
        "claims_retire_proposed",
        "claims_retired",
        "alignment_ratio",
    ):
        print(f"  {k}: {pre_summary.get(k)}")
    print()

    migrated = 0
    rows_with_tests = 0
    rows_with_hashes = 0
    rows_already_migrated = 0

    for row in rows:
        moved = False

        # tests
        if "test" in row:
            t = row.pop("test")
            if t:
                row["tests"] = [t]
                rows_with_tests += 1
            else:
                row["tests"] = []
            moved = True
        elif "tests" in row:
            rows_already_migrated += 1

        # test_body_hashes
        if "test_body_hash" in row:
            h = row.pop("test_body_hash")
            if h:
                row["test_body_hashes"] = [h]
                rows_with_hashes += 1
            else:
                row["test_body_hashes"] = []
            moved = True

        if moved:
            migrated += 1

    # Bump schema_version
    data.setdefault("summary", {})
    # schema_version lives at top-level, not inside summary.
    data["schema_version"] = "2.0"

    # Match the live formatter: 2-space indent, key order preserved
    # (sort_keys=False), trailing newline, ensure_ascii=False so
    # non-ASCII rationale text round-trips intact.
    body = json.dumps(data, indent=2, ensure_ascii=False, sort_keys=False)
    CATALOG.write_text(body + "\n")

    print("== migration ==")
    print(f"  rows_total           {len(rows)}")
    print(f"  rows_migrated        {migrated}")
    print(f"  rows_with_tests      {rows_with_tests}")
    print(f"  rows_with_hashes     {rows_with_hashes}")
    print(f"  rows_already_migrated {rows_already_migrated}  (no-op idempotency)")
    print()

    # Re-load and confirm summary unchanged.
    after = json.loads(CATALOG.read_text())
    post_summary = after.get("summary", {})
    print("== post-migration ==")
    for k in (
        "claims_total",
        "claims_bound",
        "claims_missing_test",
        "claims_retire_proposed",
        "claims_retired",
        "alignment_ratio",
    ):
        v_pre = pre_summary.get(k)
        v_post = post_summary.get(k)
        flag = "" if v_pre == v_post else "  <-- DRIFT (BAD)"
        print(f"  {k}: {v_post}{flag}")

    return 0


if __name__ == "__main__":
    sys.exit(migrate())
