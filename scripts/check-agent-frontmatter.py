#!/usr/bin/env python3
"""Guard: every `.claude/agents/*.md` must have YAML frontmatter that parses.

A subagent definition whose frontmatter fails to parse is SILENTLY DROPPED from the
agent registry — the type never appears and dispatch fails with "agent type not found",
with no error at load time. The classic footgun is a bare `: ` (colon-space) inside a
multi-line plain-scalar `description:` (e.g. a literal `` `model: opus` ``), which YAML
reads as a mapping separator. Fix by quoting the value or using a block scalar (`>-`).

Exit 0 if all defs parse; exit 1 (listing the offenders) if any fail. Wire into
pre-commit / CI so a def can never regress to un-launchable unnoticed.

Usage: scripts/check-agent-frontmatter.py [AGENTS_DIR ...]
       (defaults to .claude/agents)
"""
import sys
import glob
import os

try:
    import yaml
except ImportError:
    sys.stderr.write("check-agent-frontmatter: PyYAML not installed; cannot validate\n")
    sys.exit(2)


def extract_frontmatter(path):
    with open(path, encoding="utf-8") as f:
        text = f.read()
    if not text.startswith("---"):
        return None
    parts = text.split("---", 2)
    if len(parts) < 3:
        return None
    return parts[1]


def main(argv):
    dirs = argv[1:] or [".claude/agents"]
    paths = []
    for d in dirs:
        paths.extend(sorted(glob.glob(os.path.join(d, "*.md"))))

    if not paths:
        sys.stderr.write(f"check-agent-frontmatter: no agent files under {dirs}\n")
        return 0

    failures = []
    for path in paths:
        fm = extract_frontmatter(path)
        if fm is None:
            failures.append((path, "no YAML frontmatter (--- ... --- block missing)"))
            continue
        try:
            data = yaml.safe_load(fm)
        except yaml.YAMLError as e:
            failures.append((path, str(e).replace("\n", " ").strip()[:200]))
            continue
        if not isinstance(data, dict) or "name" not in data:
            failures.append((path, "frontmatter parsed but has no `name` key"))

    if failures:
        sys.stderr.write("check-agent-frontmatter: FAIL — un-registerable agent def(s):\n")
        for path, why in failures:
            sys.stderr.write(f"  {path}\n    {why}\n")
        sys.stderr.write(
            "  Fix: a bare `: ` in a plain-scalar `description:` breaks YAML; "
            "use a block scalar (`>-`) or quote the value.\n"
        )
        return 1

    print(f"check-agent-frontmatter: OK — {len(paths)} agent def(s) parse cleanly")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
