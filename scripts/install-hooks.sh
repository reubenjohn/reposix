#!/usr/bin/env bash
#
# Install the reposix git hooks by pointing core.hooksPath at the
# tracked .githooks/ directory.
#
# Composition rule (.githooks/pre-{commit,push}):
#   - Project hooks are the base layer that ALWAYS runs.
#   - Personal global hooks at ~/.git-hooks/<event> chain in at the END
#     as optional add-ons (recursion-guarded against the
#     delegate-to-project shim pattern).
#
# Safe to re-run -- core.hooksPath is set idempotently.

set -euo pipefail

readonly repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly hooks_src="${repo_root}/.githooks"

if [[ ! -d "${repo_root}/.git" ]]; then
  printf '%s\n' "✖ ${repo_root}/.git not found — are you inside a git working tree?" >&2
  exit 1
fi

if [[ ! -d "$hooks_src" ]]; then
  printf '%s\n' "✖ .githooks/ not found (expected at ${hooks_src})" >&2
  exit 1
fi

# Point this repo's core.hooksPath at .githooks (relative path so the
# value works inside worktrees too). Idempotent.
git -C "$repo_root" config core.hooksPath .githooks

# Ensure executable bits on every tracked hook.
installed=0
for hook_path in "$hooks_src"/*; do
  [[ -f "$hook_path" ]] || continue
  name="$(basename "$hook_path")"
  # Skip the test harness and any non-hook helpers (they have a "."
  # in the name). Real git hook events have no extension.
  if [[ "$name" == *.* ]]; then continue; fi
  chmod +x "$hook_path"
  installed=$((installed + 1))
  printf '✓ enabled hook: .githooks/%s\n' "$name"
done

if [[ "$installed" -eq 0 ]]; then
  printf '%s\n' "(nothing to enable — .githooks/ has no event hooks)" >&2
  exit 0
fi

printf '\n%s\n' "core.hooksPath = $(git -C "$repo_root" config --get core.hooksPath)"
printf '%s\n' "All hooks enabled. To bypass a hook temporarily (discouraged):"
printf '%s\n' "  git push --no-verify       # skip pre-push checks"
printf '%s\n' "  git commit --no-verify     # skip pre-commit checks"
