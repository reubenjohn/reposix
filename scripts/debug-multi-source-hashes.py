#!/usr/bin/env python3
"""Debug helper for prune-stale-multi-sources.py.

Prints per-row hash status for Multi-source rows whose walker reports
STALE_DOCS_DRIFT. Use to diagnose why prune-stale-multi-sources.py
isn't detecting expected stale entries.
"""
import hashlib
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CATALOG = REPO_ROOT / "quality" / "catalogs" / "doc-alignment.json"


def hash_source(file, line_start, line_end):
    """Mirror Rust's source_hash: split on '\\n', join with '\\n', no trailing newline."""
    p = REPO_ROOT / file
    if not p.exists():
        return None
    raw = p.read_text(encoding="utf-8")
    lines = raw.split("\n")
    if line_start < 1 or line_end > len(lines):
        return None
    return hashlib.sha256("\n".join(lines[line_start - 1:line_end]).encode("utf-8")).hexdigest()


def main():
    data = json.loads(CATALOG.read_text(encoding="utf-8"))
    target_ids = sys.argv[1:] if len(sys.argv) > 1 else None
    for row in data["rows"]:
        if target_ids and row["id"] not in target_ids:
            continue
        sources = row.get("source")
        hashes = row.get("source_hashes")
        if not isinstance(sources, list):
            continue
        if not hashes:
            continue
        print(f"\nrow {row['id']} ({row.get('last_verdict', '?')}):")
        for i, (src, stored) in enumerate(zip(sources, hashes)):
            current = hash_source(src["file"], src["line_start"], src["line_end"])
            status = "MATCH" if current == stored else "DRIFT" if current else "FILE-MISSING"
            print(f"  [{i}] {src['file']}:{src['line_start']}-{src['line_end']}")
            print(f"      stored:  {stored[:32]}...")
            print(f"      current: {(current or 'N/A')[:32]}...")
            print(f"      → {status}")


if __name__ == "__main__":
    main()
