#!/usr/bin/env python3
"""quality/runners/check_p60_red_rows.py -- P60 RED-row sentry.

Reads the 3 catalogs P60 touches (docs-build, code, freshness-invariants)
and reports the per-row grade for the 8 P60-touched rows. Exits 1 if any
P0+P1 row is NOT in {PASS, WAIVED}; exits 0 otherwise.

Used by P60 Waves G + H + the verifier subagent grading at phase close.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
CATALOGS = [
    REPO_ROOT / "quality" / "catalogs" / "docs-build.json",
    REPO_ROOT / "quality" / "catalogs" / "code.json",
    REPO_ROOT / "quality" / "catalogs" / "freshness-invariants.json",
]
P60_ROWS = {
    "docs-build/mkdocs-strict",
    "docs-build/mermaid-renders",
    "docs-build/link-resolution",
    "docs-build/badges-resolve",
    "code/cargo-fmt-check",
    "code/cargo-clippy-warnings",
    "structure/cred-hygiene",
    "structure/badges-resolve",
}


def main() -> int:
    red = 0
    found: set[str] = set()
    for c in CATALOGS:
        data = json.loads(c.read_text(encoding="utf-8"))
        for r in data["rows"]:
            if r["id"] in P60_ROWS:
                found.add(r["id"])
                status = r.get("status", "?")
                br = r.get("blast_radius", "?")
                print(f"  {br} {r['id']}: {status}")
                if br in ("P0", "P1") and status not in ("PASS", "WAIVED"):
                    red += 1
    missing = P60_ROWS - found
    if missing:
        print(f"WARN: missing P60 rows from catalogs: {sorted(missing)}")
    print(f"P0+P1 RED count: {red}")
    return 1 if red else 0


if __name__ == "__main__":
    sys.exit(main())
