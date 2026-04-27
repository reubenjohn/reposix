#!/usr/bin/env python3
"""Test invariants for quality/runners/run.py.

P57 Wave B introduced quality/runners/run.py. Two non-obvious invariants
must hold across pre-push runs to keep `git status` clean:

1. Idempotent on no-op runs: when no row's status changes, the runner
   does NOT rewrite the catalog file. Otherwise json.dumps reformatting
   leaves dirty diffs after every pre-push run.

2. Catalog formatting survives round-trips: ensure_ascii=False so em-dashes
   etc. are preserved (no \\u2014 escapes).

This script tests those invariants against the live
quality/catalogs/freshness-invariants.json catalog. Stdlib only.

Usage: python3 scripts/test-runner-invariants.py
Exit:  0 on PASS, 1 on FAIL.
"""

from __future__ import annotations

import importlib.util
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
RUNNER_PATH = REPO_ROOT / "quality" / "runners" / "run.py"
CATALOG_PATH = REPO_ROOT / "quality" / "catalogs" / "freshness-invariants.json"


def load_runner_module():
    spec = importlib.util.spec_from_file_location("quality_runner", RUNNER_PATH)
    if spec is None or spec.loader is None:
        raise SystemExit(f"FAIL: cannot import {RUNNER_PATH}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def test_idempotent_no_op_run() -> None:
    """Running the runner twice in a row must produce a byte-identical
    catalog file on the second run. The FIRST run may legitimately mutate
    statuses (e.g., a row with an active waiver flips from NOT-VERIFIED to
    WAIVED on first verify); the SECOND run should be a no-op."""
    mod = load_runner_module()
    orig = mod.load_catalog(CATALOG_PATH)
    data = json.loads(json.dumps(orig))
    now = datetime.now(timezone.utc)
    for row in data["rows"]:
        if mod.is_in_scope(row, "pre-push", now):
            mod.run_row(row, REPO_ROOT, now)
    # Snapshot post-first-run state.
    first_run = json.loads(json.dumps(data))
    # Run AGAIN against the post-first-run state; this should be no-op.
    data2 = json.loads(json.dumps(first_run))
    for row in data2["rows"]:
        if mod.is_in_scope(row, "pre-push", now):
            mod.run_row(row, REPO_ROOT, now)
    dirty = mod.catalog_dirty(first_run, data2)
    if dirty:
        fail(
            f"catalog_dirty returned True on a SECOND run after stable state. "
            f"first-run statuses={[r['status'] for r in first_run['rows']]}, "
            f"second-run statuses={[r['status'] for r in data2['rows']]}"
        )
    print("PASS: idempotent on second run (no spurious catalog rewrites)")


def test_unicode_preservation() -> None:
    """save_catalog must preserve em-dash and other Unicode characters
    (ensure_ascii=False)."""
    mod = load_runner_module()
    sample = {
        "$schema": "x",
        "comment": "em-dash — and unicode preserved",
        "dimension": "structure",
        "rows": [],
    }
    tmp = REPO_ROOT / "quality" / "reports" / ".test-roundtrip.json"
    try:
        mod.save_catalog(tmp, sample)
        text = tmp.read_text(encoding="utf-8")
        if "\\u2014" in text:
            fail("save_catalog produced \\u2014 escape — ensure_ascii=False not active")
        if "—" not in text:
            fail("save_catalog stripped the em-dash")
        print("PASS: Unicode preservation (em-dash survives round-trip)")
    finally:
        if tmp.exists():
            tmp.unlink()


def main() -> int:
    test_idempotent_no_op_run()
    test_unicode_preservation()
    print("OK: all runner invariants hold")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
