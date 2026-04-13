#!/usr/bin/env python3
"""Rewrite `../../scripts/...` relative links in docs/demos/index.md to absolute
GitHub URLs. MkDocs --strict refuses unresolved links; scripts live outside the
docs/ tree and are intentionally linked by URL.

Idempotent — re-running after already-rewritten links is a no-op.
"""
from __future__ import annotations
import pathlib
import re
import sys

INDEX = pathlib.Path("docs/demos/index.md")
REPO = "reubenjohn/reposix"
BASE = f"https://github.com/{REPO}/blob/main"


def main() -> int:
    if not INDEX.exists():
        print(f"error: {INDEX} not found", file=sys.stderr)
        return 1
    text = INDEX.read_text()
    pattern = re.compile(r"\]\(\.\.\/\.\.\/(scripts\/[^)]+)\)")
    new = pattern.sub(lambda m: f"]({BASE}/{m.group(1)})", text)
    if new == text:
        print("no changes")
        return 0
    INDEX.write_text(new)
    replaced = len(pattern.findall(text))
    print(f"rewrote {replaced} links in {INDEX}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
