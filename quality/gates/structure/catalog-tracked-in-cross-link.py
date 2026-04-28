#!/usr/bin/env python3
# KEEP-AS-CANONICAL (P63 MIGRATE-03): meta-helper / catalog<->REQUIREMENTS bidirectional cross-link verifier.
"""catalog-tracked-in-cross-link.py -- P63 MIGRATE-03 cross-link verifier.

Asserts that every catalog row in the v0.12.1 carry-forward stub catalogs
(`quality/catalogs/perf-targets.json`, `quality/catalogs/security-gates.json`,
`quality/catalogs/cross-platform.json`) whose `waiver.tracked_in` cites a
v0.12.1 REQ-ID resolves to a `**REQ-ID**` checkbox in
`.planning/milestones/v0.12.1-phases/REQUIREMENTS.md`.

Stdlib only. Exits 0 if all links resolve, 1 otherwise.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent.parent
REQS = REPO / ".planning" / "milestones" / "v0.12.1-phases" / "REQUIREMENTS.md"
CATALOGS = [
    REPO / "quality" / "catalogs" / "perf-targets.json",
    REPO / "quality" / "catalogs" / "security-gates.json",
    REPO / "quality" / "catalogs" / "cross-platform.json",
]

PATTERN = re.compile(r"v0\.12\.1\s+([A-Z]+-\d+)")


def main() -> int:
    if not REQS.is_file():
        print(f"FAIL: {REQS} missing", file=sys.stderr)
        return 1
    reqs_text = REQS.read_text(encoding="utf-8")
    bad: list[tuple[str, str, str]] = []
    total = 0
    for f in CATALOGS:
        if not f.is_file():
            continue
        for r in json.loads(f.read_text(encoding="utf-8")).get("rows", []):
            ti = (r.get("waiver") or {}).get("tracked_in", "") or ""
            m = PATTERN.search(ti)
            if not m:
                continue
            total += 1
            req_id = m.group(1)
            if not re.search(r"\*\*" + re.escape(req_id) + r"\*\*", reqs_text):
                bad.append((r["id"], ti, req_id))
    if bad:
        print(f"FAIL: {len(bad)}/{total} catalog tracked_in REQ-IDs do not resolve in {REQS.relative_to(REPO)}", file=sys.stderr)
        for row_id, ti, req_id in bad:
            print(f"  - {row_id}: tracked_in={ti!r} -> REQ-ID {req_id!r} not found", file=sys.stderr)
        return 1
    print(f"OK: {total} catalog tracked_in REQ-IDs all resolve in {REQS.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
