#!/usr/bin/env python3
"""Prune stale Multi-source entries from doc-alignment catalog rows.

Architectural gap: `reposix-quality doc-alignment bind` is additive — when
an original cited line range no longer points at the relevant prose
(because the doc was edited), bind adds a new source entry but the old
one stays. The walker then reports STALE_DOCS_DRIFT on the legacy index.

This script: for each Multi-source row, recomputes the hash of each
cited line range and drops entries whose stored hash doesn't match
current source AND whose sibling source_hashes still has at least one
matching entry. Effect: rows whose claim moved to a different line
collapse from Multi → Single (or shorter Multi) at the still-resolving
citation.

Idempotent. Run after a docs-alignment refresh that produced multi-source
debris.

Usage:
    python3 scripts/prune-stale-multi-sources.py
"""
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CATALOG = REPO_ROOT / "quality" / "catalogs" / "doc-alignment.json"


def hash_source(file: str, line_start: int, line_end: int) -> str | None:
    """Return SHA-256 of cited line range (1-indexed, inclusive). Mirrors
    crates/reposix-quality/src/hash.rs::source_hash — splits on '\\n' and
    joins with '\\n' (no trailing newline). Returns None if file does
    not exist or range is out of bounds."""
    p = REPO_ROOT / file
    if not p.exists():
        return None
    raw = p.read_text(encoding="utf-8")
    lines = raw.split("\n")
    if line_start < 1 or line_end > len(lines):
        return None
    slice_ = lines[line_start - 1:line_end]
    joined = "\n".join(slice_)
    return hashlib.sha256(joined.encode("utf-8")).hexdigest()


def main() -> int:
    data = json.loads(CATALOG.read_text(encoding="utf-8"))
    pruned_rows = 0
    pruned_entries = 0
    for row in data["rows"]:
        sources = row.get("source")
        hashes = row.get("source_hashes")
        # Only Multi-source rows (list shape) qualify
        if not isinstance(sources, list) or not isinstance(hashes, list):
            continue
        if len(sources) < 2 or len(hashes) != len(sources):
            continue
        # Compute current hash per source; keep if matches stored OR file missing
        # (we can't validate). Drop if mismatch.
        keep_indices: list[int] = []
        drop_indices: list[int] = []
        for i, (src, stored_hash) in enumerate(zip(sources, hashes)):
            current = hash_source(src["file"], src["line_start"], src["line_end"])
            if current is None:
                # File missing — drop the citation
                drop_indices.append(i)
            elif current == stored_hash:
                keep_indices.append(i)
            else:
                drop_indices.append(i)
        # If at least one source still resolves, prune the others
        if keep_indices and drop_indices:
            new_sources = [sources[i] for i in keep_indices]
            new_hashes = [hashes[i] for i in keep_indices]
            if len(new_sources) == 1:
                row["source"] = new_sources[0]
            else:
                row["source"] = new_sources
            row["source_hashes"] = new_hashes
            row["source_hash"] = new_hashes[0]
            pruned_rows += 1
            pruned_entries += len(drop_indices)
            print(f"  pruned {len(drop_indices)} stale source(s) from {row['id']}")
    if pruned_rows == 0:
        print("no Multi-source rows had stale entries; catalog unchanged")
        return 0
    # Preserve catalog field order
    ordered: dict = {}
    for key in ("$schema", "comment", "dimension", "rows"):
        if key in data:
            ordered[key] = data[key]
    for key, val in data.items():
        if key not in ordered:
            ordered[key] = val
    CATALOG.write_text(
        json.dumps(ordered, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    print(f"pruned {pruned_entries} stale source entry/entries across {pruned_rows} row(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
