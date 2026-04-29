#!/usr/bin/env bash
# Active-milestone vs workspace-version drift detector.
#
# Compares the most recent shipped CHANGELOG.md heading (skipping
# [Unreleased]) with the workspace package version in Cargo.toml.
# Catches the class of doc-drift where a milestone shipped (CHANGELOG
# entry exists) but Cargo.toml's workspace version was not bumped.
#
# Symptom this catches (real example, 2026-04-28):
#   CHANGELOG.md most-recent shipped: v0.12.0
#   Cargo.toml [workspace.package] version: 0.11.3
#   -> mismatch; v0.12.0 shipped per CHANGELOG but Cargo is still 0.11.3.
#
# Bound by docs-development-roadmap-md/v0-11-0-active-milestone in
# quality/catalogs/doc-alignment.json (the prose names "v0.11.0 active
# milestone" while v0.12.0 has shipped).
#
# Exit codes:
#   0 -- versions match (modulo patch -- major.minor)
#   1 -- mismatch surfaced; stderr prints both values for diagnosis

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Most recent NON-Unreleased CHANGELOG heading. Format: "## [v0.X.Y] -- ..."
changelog_version=$(
  grep -E '^## \[v[0-9]' CHANGELOG.md \
    | grep -v Unreleased \
    | head -1 \
    | sed -E 's/^## \[v([0-9]+\.[0-9]+\.[0-9]+)\].*/\1/'
)

# Cargo.toml [workspace.package] version. Format: 'version = "0.X.Y"'
cargo_version=$(
  awk '/^\[workspace\.package\]/{flag=1; next} /^\[/{flag=0} flag && /^version/ {gsub(/[" ]/,""); split($0,a,"="); print a[2]; exit}' Cargo.toml
)

if [[ -z "$changelog_version" ]]; then
  echo "ERROR: could not extract latest shipped version from CHANGELOG.md" >&2
  exit 1
fi
if [[ -z "$cargo_version" ]]; then
  echo "ERROR: could not extract version from Cargo.toml [workspace.package]" >&2
  exit 1
fi

# Compare on major.minor (allows patch drift -- 0.11.0 matches 0.11.3).
changelog_minor="${changelog_version%.*}"
cargo_minor="${cargo_version%.*}"

if [[ "$changelog_minor" == "$cargo_minor" ]]; then
  echo "OK: CHANGELOG latest shipped v${changelog_version}, Cargo workspace v${cargo_version} -- same minor (${changelog_minor})"
  exit 0
fi

echo "MISMATCH: active-milestone vs workspace-version drift" >&2
echo "  CHANGELOG.md most recent shipped: v${changelog_version}" >&2
echo "  Cargo.toml [workspace.package] version: ${cargo_version}" >&2
echo "  Major.minor mismatch (${changelog_minor} vs ${cargo_minor})." >&2
echo "" >&2
echo "Recovery: bump Cargo.toml [workspace.package] version to ${changelog_version}, OR" >&2
echo "          add the missing CHANGELOG entry for the in-flight version." >&2
exit 1
