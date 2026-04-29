#!/usr/bin/env python3
"""Verify relative Markdown links in user-facing docs resolve to existing files.

Used by Phase 44 doc-clarity audit and useful as a pre-commit/CI guard going
forward. Skips http(s)://, mailto:, and pure-anchor links. Anchor fragments
on .md links are stripped before resolution (this script does NOT validate
anchors — mkdocs --strict does that).

Migrated from scripts/check_doc_links.py per P60 SIMPLIFY-08 (2026-04-27).
Renamed underscore-to-hyphen per .planning/research/v0.12.0/naming-and-architecture.md
directory-layout convention (mirrors quality/gates/release/gh-assets-present.py).
"""

from __future__ import annotations

import glob
import os
import re
import sys

DEFAULT_GLOBS = [
    "docs/index.md",
    "docs/concepts/*.md",
    "docs/tutorials/*.md",
    "docs/how-it-works/*.md",
    "docs/guides/*.md",
    "docs/reference/*.md",
    "docs/benchmarks/*.md",
]

LINK_RE = re.compile(r"\]\(([^)]+)\)")


def collect(patterns: list[str]) -> list[str]:
    files: list[str] = []
    for pat in patterns:
        files.extend(sorted(glob.glob(pat)))
    return files


def check(files: list[str]) -> list[tuple[str, str, str]]:
    broken: list[tuple[str, str, str]] = []
    for path in files:
        with open(path) as fh:
            text = fh.read()
        for match in LINK_RE.finditer(text):
            link = match.group(1).strip()
            if link.startswith(("http://", "https://", "mailto:", "#")):
                continue
            rel = link.split("#", 1)[0]
            if not rel:
                continue
            if rel.startswith("/"):
                abs_path = rel.lstrip("/")
            else:
                abs_path = os.path.normpath(os.path.join(os.path.dirname(path), rel))
            if not os.path.exists(abs_path):
                broken.append((path, link, abs_path))
    return broken


def main(argv: list[str]) -> int:
    patterns = argv[1:] if len(argv) > 1 else DEFAULT_GLOBS
    files = collect(patterns)
    broken = check(files)
    for path, link, abs_path in broken:
        print(f"BROKEN in {path}: {link} -> {abs_path}")
    print(f"\n{len(broken)} broken link(s) across {len(files)} file(s)")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
