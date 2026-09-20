#!/usr/bin/env bash
# Verify the scoped Claude Code instruction files are also exposed to Codex.
#
# CLAUDE.md is the sole canonical instruction source.  Each scoped AGENTS.md
# must therefore be a tracked relative symlink to its sibling CLAUDE.md, not a
# copied policy that could drift.  Keep this explicit list deliberately small:
# it names every scoped instruction surface the repository currently intends
# agents to consume.  Adding another scoped CLAUDE.md is a deliberate policy
# change and must add it here in the same commit.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

scoped_dirs=(
  ".planning"
  "crates"
  "quality"
  ".claude/hooks"
)

fail=0
for dir in "${scoped_dirs[@]}"; do
  claude="$dir/CLAUDE.md"
  agents="$dir/AGENTS.md"

  if ! git ls-files --error-unmatch "$claude" "$agents" >/dev/null 2>&1; then
    echo "MISSING: $claude and $agents must both be tracked" >&2
    fail=1
    continue
  fi

  if [[ ! -L "$agents" ]]; then
    echo "INVALID: $agents must be a symlink to CLAUDE.md, not a copied file" >&2
    fail=1
    continue
  fi

  target="$(readlink "$agents")"
  if [[ "$target" != "CLAUDE.md" ]]; then
    echo "INVALID: $agents points to '$target'; expected 'CLAUDE.md'" >&2
    fail=1
    continue
  fi

  if [[ ! -f "$agents" ]]; then
    echo "BROKEN: $agents -> $target does not resolve to a regular file" >&2
    fail=1
  fi
done

if [[ $fail -ne 0 ]]; then
  echo "Recovery: replace each scoped AGENTS.md with 'ln -s CLAUDE.md <dir>/AGENTS.md' and stage both files." >&2
  exit 1
fi

echo "PASS: ${#scoped_dirs[@]} scoped AGENTS.md files are tracked symlinks to their local CLAUDE.md."
