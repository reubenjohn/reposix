#!/usr/bin/env bash
#
# Install the reposix git hooks (OP-7 from HANDOFF.md).
#
# Creates symlinks in .git/hooks/ pointing at scripts/hooks/*.
# Symlinks (not copies) so hook updates land the next time anyone
# pulls, without a second install step.
#
# Safe to re-run — symlinks are re-created idempotently.

set -euo pipefail

readonly repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly hooks_src="${repo_root}/scripts/hooks"
readonly hooks_dst="${repo_root}/.git/hooks"

if [[ ! -d "$hooks_dst" ]]; then
  printf '%s\n' "✖ .git/hooks not found — are you inside a git working tree?" >&2
  exit 1
fi

if [[ ! -d "$hooks_src" ]]; then
  printf '%s\n' "✖ scripts/hooks not found (expected at ${hooks_src})" >&2
  exit 1
fi

installed=0
for hook_path in "$hooks_src"/*; do
  [[ -f "$hook_path" ]] || continue
  hook_name="$(basename "$hook_path")"
  dst="${hooks_dst}/${hook_name}"

  # Make the source executable in case it was checked out without +x.
  chmod +x "$hook_path"

  # Replace any existing link/file with a fresh symlink.
  rm -f "$dst"
  ln -s "../../scripts/hooks/${hook_name}" "$dst"
  installed=$((installed + 1))

  printf '✓ installed hook: %s -> scripts/hooks/%s\n' "$hook_name" "$hook_name"
done

if [[ "$installed" -eq 0 ]]; then
  printf '%s\n' "(nothing to install — scripts/hooks/ is empty)" >&2
  exit 0
fi

printf '\n%s\n' "All hooks installed. To bypass a hook temporarily (discouraged):"
printf '%s\n' "  git push --no-verify    # skip pre-push checks"
