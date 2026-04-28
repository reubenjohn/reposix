#!/usr/bin/env python3
"""orphan-scripts-audit.py -- verifier for SIMPLIFY-12 (P63 MIGRATE-01).

Reads quality/catalogs/orphan-scripts.json and asserts each surviving-shim
row's contract:
  a. row.sources[0] is `scripts/<basename>` and the file exists.
  b. The file is <= row.expected.max_lines (default 40 for shims; higher for
     KEEP-AS-CANONICAL helpers per the audit doc).
  c. The first 10 lines contain a comment naming the canonical path
     (`canonical at`, `canonical home`, `migrated to`, `exec quality/gates/`,
     `subprocess.*quality/gates/`) OR document the keep-as-canonical rationale
     (`canonical for its own domain`, `meta-helper`, `no canonical home`).
  d. row.kind == "mechanical" and row.dimension == "meta".

Modes:
  --row-id <id>     verify a single row; write artifact + exit 0/1.
  --all             iterate every row; aggregate; exit 0 only if all PASS.

Stdlib only. No third-party deps.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
CATALOG = REPO_ROOT / "quality" / "catalogs" / "orphan-scripts.json"
ARTIFACT_DIR = REPO_ROOT / "quality" / "reports" / "verifications" / "structure"

CANONICAL_HINTS = (
    re.compile(r"canonical\s+(at|home|for|impl)", re.IGNORECASE),
    re.compile(r"migrated\s+to", re.IGNORECASE),
    re.compile(r"exec\s+.*quality/gates/", re.IGNORECASE),
    re.compile(r"subprocess.*quality/gates/", re.IGNORECASE),
    re.compile(r"meta-helper", re.IGNORECASE),
    re.compile(r"no\s+canonical\s+home", re.IGNORECASE),
    re.compile(r"thin\s+shim", re.IGNORECASE),
)


def now_rfc3339() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def load_catalog() -> dict:
    return json.loads(CATALOG.read_text(encoding="utf-8"))


def find_row(catalog: dict, row_id: str) -> dict | None:
    for r in catalog.get("rows", []):
        if r.get("id") == row_id:
            return r
    return None


def verify_row(row: dict) -> tuple[str, list[str]]:
    """Return (status, failed_asserts)."""
    failed: list[str] = []

    # (d) kind + dimension
    if row.get("kind") != "mechanical":
        failed.append(f"kind != mechanical (got {row.get('kind')!r})")
    if row.get("dimension") != "meta":
        failed.append(f"dimension != meta (got {row.get('dimension')!r})")

    sources = row.get("sources") or []
    if not sources:
        failed.append("sources[] is empty")
        return ("FAIL", failed)
    src_rel = sources[0]
    src_path = REPO_ROOT / src_rel
    # (a) source file exists
    if not src_path.is_file():
        failed.append(f"source file missing: {src_rel}")
        return ("FAIL", failed)

    # (b) line cap
    expected = row.get("expected") or {}
    max_lines = int(expected.get("max_lines", 40))
    lines = src_path.read_text(encoding="utf-8", errors="replace").splitlines()
    if len(lines) > max_lines:
        failed.append(
            f"line count {len(lines)} > max_lines {max_lines} for {src_rel}"
        )

    # (c) header hint
    header = "\n".join(lines[:10])
    if not any(h.search(header) for h in CANONICAL_HINTS):
        failed.append(
            f"first 10 lines of {src_rel} do not name a canonical path / "
            f"keep-as-canonical rationale (looked for: canonical at|home|for, "
            f"migrated to, exec quality/gates/, subprocess..quality/gates/, "
            f"meta-helper, no canonical home, thin shim)"
        )

    return ("PASS" if not failed else "FAIL", failed)


def write_artifact(row_id: str, status: str, failed: list[str], row: dict) -> Path:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    slug = row_id.split("/", 1)[1] if "/" in row_id else row_id
    out = ARTIFACT_DIR / f"orphan-scripts-{slug}.json"
    payload = {
        "claim_id": row_id,
        "phase": "p63",
        "verifier_kind": "mechanical",
        "verified_at": now_rfc3339(),
        "verifier_script": "quality/gates/structure/orphan-scripts-audit.py",
        "asserts": {
            "source_exists": True if row.get("sources") else False,
            "shim_shape_ok": status == "PASS",
        },
        "evidence": {
            "row_id": row_id,
            "source": (row.get("sources") or [None])[0],
            "failed_asserts": failed,
        },
        "status": status,
    }
    out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    return out


def cmd_row(row_id: str) -> int:
    catalog = load_catalog()
    row = find_row(catalog, row_id)
    if row is None:
        print(f"FAIL: row {row_id!r} not found in {CATALOG}", file=sys.stderr)
        return 1
    status, failed = verify_row(row)
    artifact = write_artifact(row_id, status, failed, row)
    print(f"[{status:<6}] {row_id} -> {artifact.relative_to(REPO_ROOT)}")
    if failed:
        for f in failed:
            print(f"  - {f}", file=sys.stderr)
    return 0 if status == "PASS" else 1


def cmd_all() -> int:
    catalog = load_catalog()
    rows = catalog.get("rows", [])
    if not rows:
        print("OK: orphan-scripts catalog has zero rows (P63 Wave 1 schema-only state).")
        return 0
    rc = 0
    for row in rows:
        rid = row.get("id", "<no-id>")
        status, failed = verify_row(row)
        write_artifact(rid, status, failed, row)
        marker = "[PASS  ]" if status == "PASS" else "[FAIL  ]"
        print(f"{marker} {rid}")
        if failed:
            rc = 1
            for f in failed:
                print(f"    - {f}")
    return rc


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Verify orphan-scripts.json shim-shape contract (SIMPLIFY-12 P63)."
    )
    g = parser.add_mutually_exclusive_group(required=True)
    g.add_argument("--row-id", help="verify a single row by id")
    g.add_argument("--all", action="store_true", help="iterate every row")
    args = parser.parse_args()
    if args.row_id:
        return cmd_row(args.row_id)
    return cmd_all()


if __name__ == "__main__":
    raise SystemExit(main())
