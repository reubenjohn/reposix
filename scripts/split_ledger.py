#!/usr/bin/env python3
"""Split an oversized `## `-delimited planning ledger into per-part child files.

OP-8 file-size-gate drain helper. Ledgers (SURPRISES-INTAKE / GOOD-TO-HAVES) are
append-only and grow past the *.md 20k progressive-disclosure budget. This tool
splits them **without content loss**: every `## ` entry is copied verbatim into a
part file, the original file is rewritten as a small INDEX (preamble + links to
each part + the entry titles), and an integrity check asserts the concatenated
parts reproduce the original entry bytes exactly.

Usage:
    scripts/split_ledger.py FILE --first-entry-line N --budget BYTES

- Lines [1..N-1] are the preamble (kept in the rewritten INDEX).
- From line N onward, every line matching `^## ` starts a new entry.
- Entries are greedily packed into parts, each part <= BUDGET bytes.
- Parts land in a sibling dir named after the file stem, lowercased.

Exit 0 on success (prints the resulting file sizes), 1 if a single entry
exceeds the budget (cannot be packed without intra-entry splitting) or if the
verbatim round-trip check fails.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ENTRY_RE = re.compile(r"^## ")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("file", type=Path)
    ap.add_argument("--first-entry-line", type=int, required=True,
                    help="1-based line number of the first `## ` entry")
    ap.add_argument("--budget", type=int, default=19000,
                    help="max bytes per part file (default 19000, margin under 20k)")
    args = ap.parse_args()

    src = args.file
    text = src.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)

    pre_end = args.first_entry_line - 1
    preamble = "".join(lines[:pre_end])
    body_lines = lines[pre_end:]

    # Partition body into entries at `^## ` boundaries.
    entries: list[str] = []
    cur: list[str] = []
    for ln in body_lines:
        if ENTRY_RE.match(ln) and cur:
            entries.append("".join(cur))
            cur = [ln]
        else:
            cur.append(ln)
    if cur:
        entries.append("".join(cur))

    if not entries or not ENTRY_RE.match(entries[0]):
        print(f"ERROR: no `## ` entry found at line {args.first_entry_line}", file=sys.stderr)
        return 1

    # A single entry over budget is sub-split at `---` horizontal-rule
    # boundaries (dated follow-ups) into verbatim continuation chunks. This
    # partitions bytes only — never modifies them — so the round-trip holds.
    def subsplit(entry: str) -> list[str]:
        if len(entry.encode()) <= args.budget:
            return [entry]
        elines = entry.splitlines(keepends=True)
        chunks: list[str] = []
        cur: list[str] = []
        for ln in elines:
            cur.append(ln)
            if ln.strip() == "---" and len("".join(cur).encode()) >= args.budget * 0.6:
                chunks.append("".join(cur))
                cur = []
        if cur:
            chunks.append("".join(cur))
        return chunks

    chunks: list[str] = []
    for e in entries:
        chunks.extend(subsplit(e))

    still_over = [(i, len(c.encode())) for i, c in enumerate(chunks)
                  if len(c.encode()) > args.budget]
    if still_over:
        for i, n in still_over:
            print(f"ERROR: chunk {i} is {n} bytes (> budget {args.budget}); no "
                  f"`---` sub-boundary found:\n  {chunks[i].splitlines()[0]}",
                  file=sys.stderr)
        return 1

    # Greedily pack chunks into parts.
    parts: list[list[str]] = []
    cur_part: list[str] = []
    cur_bytes = 0
    for e in chunks:
        eb = len(e.encode())
        if cur_part and cur_bytes + eb > args.budget:
            parts.append(cur_part)
            cur_part, cur_bytes = [], 0
        cur_part.append(e)
        cur_bytes += eb
    if cur_part:
        parts.append(cur_part)

    stem = src.stem.lower()
    out_dir = src.parent / stem
    out_dir.mkdir(exist_ok=True)
    n_parts = len(parts)
    title_line = lines[0].rstrip("\n") if lines else f"# {src.stem}"

    index_rows: list[str] = []
    for idx, part in enumerate(parts, start=1):
        part_name = f"part-{idx:02d}.md"
        part_path = out_dir / part_name
        header = (
            f"{title_line} — Part {idx} of {n_parts}\n\n"
            f"> Split from `{src.name}` for the file-size gate (OP-8 drain). "
            f"Index: `../{src.name}`. Entries preserved verbatim.\n\n"
        )
        part_path.write_text(header + "".join(part), encoding="utf-8")
        titles = []
        for e in part:
            first = e.splitlines()[0]
            if first.startswith("## "):
                titles.append(first[3:].strip())
            else:
                titles.append("(continued) " + first.strip()[:80])
        index_rows.append(f"- [`{stem}/{part_name}`]({stem}/{part_name}) — "
                          f"{len(part)} entries:")
        index_rows.extend(f"  - {t}" for t in titles)

    index = (
        preamble.rstrip("\n")
        + "\n\n## Split index (OP-8 file-size drain)\n\n"
        + f"This ledger exceeded the *.md 20k budget and was split into {n_parts} "
        + f"per-part child files under `{stem}/`. Every entry is preserved "
        + "verbatim; append new entries to the last part (or a new part) and add "
        + "the title here.\n\n"
        + "\n".join(index_rows)
        + "\n"
    )
    src.write_text(index, encoding="utf-8")

    # Round-trip integrity: concatenated entry bodies must equal the original body.
    orig_body = "".join(body_lines)
    rebuilt = "".join("".join(p) for p in parts)
    if rebuilt != orig_body:
        print("ERROR: round-trip mismatch — entry bytes changed during split!",
              file=sys.stderr)
        return 1

    print(f"OK: {src.name} -> INDEX ({len(index.encode())} bytes) + "
          f"{n_parts} parts in {out_dir}/  ({len(entries)} entries preserved)")
    for idx in range(1, n_parts + 1):
        p = out_dir / f"part-{idx:02d}.md"
        print(f"  {p.relative_to(src.parent)}: {p.stat().st_size} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
