#!/usr/bin/env bash
# Release-tags-present verifier.
#
# Asserts that every "v0.X.Y shipped" claim in docs/development/roadmap.md
# is backed by a corresponding git tag. Cheap regression detector; nothing
# realistic deletes git tags, but this ratchets the bar and lives as
# committed evidence that the roadmap's milestone-shipping claims are
# falsifiable.
#
# Bound by:
#   docs-development-roadmap-md/v0-1-0-shipped
#   docs-development-roadmap-md/v0-2-0-alpha-shipped
#   docs-development-roadmap-md/v0-10-0-shipped
#
# Exit 0 if every required tag exists; 1 with diagnostic per miss.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

required_tags=(
  "v0.1.0"
  "v0.2.0-alpha"
  "v0.10.0"
)

fail=0
for tag in "${required_tags[@]}"; do
  if ! git tag --list "$tag" | grep -qx "$tag"; then
    echo "MISSING: git tag $tag (claimed shipped per docs/development/roadmap.md)" >&2
    fail=1
  fi
done

if [[ $fail -ne 0 ]]; then
  exit 1
fi

echo "OK: all ${#required_tags[@]} milestone-shipped tags present in git history."
