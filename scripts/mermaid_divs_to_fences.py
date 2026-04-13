#!/usr/bin/env python3
"""Rewrite `<div class="mermaid">...</div>` blocks in docs/*.md to `\\`\\`\\`mermaid`
fenced code blocks. MkDocs-Material's `pymdownx.superfences` renders mermaid
from fenced blocks; raw HTML divs render as plain text unless mermaid.js is
loaded manually.

Runs idempotently — existing fenced blocks are untouched.
"""
from __future__ import annotations
import pathlib
import re
import sys

DOCS = pathlib.Path("docs")
PATTERN = re.compile(r'<div class="mermaid">\n(.*?)\n</div>', re.DOTALL)


def main() -> int:
    total_blocks = 0
    touched = 0
    for p in DOCS.rglob("*.md"):
        original = p.read_text()
        new = PATTERN.sub(lambda m: f"```mermaid\n{m.group(1)}\n```", original)
        if new != original:
            p.write_text(new)
            touched += 1
            print(f"rewrote {p}")
        total_blocks += new.count("```mermaid")
    print(f"done: {touched} files rewritten, {total_blocks} total mermaid fences in docs/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
