#!/usr/bin/env python3
"""Migrate quality catalog rows from `cadence: str` to `cadences: list[str]`.

Companion to commit "quality(runner): cadences as list + add pre-commit
cadence". The runner now reads `cadences` only — catalogs must follow.

Behaviour:
  1. Walk every quality/catalogs/*.json; skip orphan-scripts.json and
     *-allowlist.json (these are not consumed by the runner).
  2. For each row carrying a scalar `cadence` key, replace it with
     `cadences: [<old-value>]`. Special case: rows previously tagged
     `cadence: "pre-push"` migrate to `cadences: ["pre-push", "pre-pr"]`
     so existing pre-push gates start firing in CI too — closing the
     gap where CI's `--cadence pre-pr` invocation silently skipped them.
  3. Preserve wrapper field order ($schema, comment, dimension, rows)
     to match quality/runners/run.py:save_catalog().
  4. Idempotent: rows already on the new shape are skipped; running a
     second time is a no-op.

Usage:
  python3 scripts/migrations/2026-05-cadence-to-list.py
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CATALOG_DIR = REPO_ROOT / "quality" / "catalogs"


def discover_catalogs() -> list[Path]:
    """Glob catalog files. Mirror the runner's discover_catalogs() filter."""
    out: list[Path] = []
    for p in sorted(CATALOG_DIR.glob("*.json")):
        if p.stem == "orphan-scripts" or p.stem.endswith("-allowlist"):
            continue
        out.append(p)
    return out


def save_catalog(path: Path, data: dict[str, Any]) -> None:
    """Write back, preserving wrapper field order. Mirrors run.py:save_catalog."""
    ordered: dict[str, Any] = {}
    for key in ("$schema", "comment", "dimension", "rows"):
        if key in data:
            ordered[key] = data[key]
    for key, val in data.items():
        if key not in ordered:
            ordered[key] = val
    path.write_text(
        json.dumps(ordered, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def migrate_row(row: dict[str, Any]) -> bool:
    """Mutate row in place. Return True iff row changed."""
    if "cadences" in row:
        # Already migrated — idempotent skip.
        return False
    if "cadence" not in row:
        # Row predates the cadence field entirely (rare; e.g. pseudo-rows).
        return False
    old = row.pop("cadence")
    if old == "pre-push":
        row["cadences"] = ["pre-push", "pre-pr"]
    else:
        row["cadences"] = [old]
    return True


def main() -> int:
    catalogs = discover_catalogs()
    if not catalogs:
        print(f"no catalogs found under {CATALOG_DIR}", file=sys.stderr)
        return 1

    grand_total = 0
    files_changed = 0
    for cat_path in catalogs:
        try:
            data = json.loads(cat_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as e:
            print(f"FAIL: {cat_path}: invalid JSON: {e}", file=sys.stderr)
            return 1
        rows = data.get("rows", [])
        touched = 0
        for row in rows:
            if migrate_row(row):
                touched += 1
        if touched:
            save_catalog(cat_path, data)
            files_changed += 1
            grand_total += touched
            print(f"{cat_path.relative_to(REPO_ROOT)}: {touched} row(s) migrated")
        else:
            print(f"{cat_path.relative_to(REPO_ROOT)}: no rows to migrate")

    print(f"summary: {grand_total} row(s) across {files_changed} file(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
