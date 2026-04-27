#!/usr/bin/env python3
"""_patch_plan_block.py — surgical block replacement in PLAN.md files.

This is a small helper used by gsd-planner during plan authoring/revision
to swap one task block for another without rewriting the whole PLAN.md.
Public-API stability not required; underscore-prefixed because internal.

Usage:
    python3 scripts/_patch_plan_block.py <plan_path> <before_path> <after_path>

Where:
    <plan_path>   = the PLAN.md to patch (in-place edit).
    <before_path> = file containing the exact block to find (must match once).
    <after_path>  = file containing the replacement.

Exit codes:
    0  one match found, replaced, written.
    1  zero matches OR multiple matches OR file IO error.

Why this exists (CLAUDE.md OP-4 grounding): the gsd-tools plan-structure
verifier requires <files>/<action>/<verify>/<done> on every <task>; old
plans authored with checkpoint:human-verify shape sometimes lack those
elements. Hand-editing via inline `python3 -c '...'` triggers the
project's deny-ad-hoc-bash hook because those edits exceed 300 chars.
This committed helper is the "named command the next agent recognizes"
that closes that loop.
"""

from __future__ import annotations

import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 4:
        print(__doc__, file=sys.stderr)
        return 1

    plan_path, before_path, after_path = (Path(p) for p in sys.argv[1:4])

    plan = plan_path.read_text()
    before = before_path.read_text()
    after = after_path.read_text()

    count = plan.count(before)
    if count == 0:
        print(f"FAIL: before-block not found in {plan_path}", file=sys.stderr)
        return 1
    if count > 1:
        print(
            f"FAIL: before-block matches {count} times in {plan_path}; "
            "must be unique. Add more surrounding context to before-block.",
            file=sys.stderr,
        )
        return 1

    plan_path.write_text(plan.replace(before, after))
    print(f"OK: patched {plan_path} (1 block replaced)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
