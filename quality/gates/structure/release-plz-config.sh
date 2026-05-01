#!/usr/bin/env bash
# Release-plz-config verifier.
#
# Asserts that release-plz.toml exists and disables per-package GitHub
# release creation. Without git_release_enable = false, release-plz
# creates one zero-asset GH release per workspace package on every
# push to main, stealing releases/latest from the canonical v* release
# built by .github/workflows/release.yml. That breaks user install
# URLs (releases/latest/download/reposix-installer.sh) and any catalog
# row resolving through releases/latest.
#
# Bound by:
#   structure/release-plz-disables-gh-releases
#
# Original rationale: PR #34, commit f1d89e5.
# Exit 0 if all three assertions hold; 1 with diagnostic per miss.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

cfg="release-plz.toml"
fail=0

if [[ ! -f "$cfg" ]]; then
  echo "MISSING: $cfg not found at repo root" >&2
  echo "  recovery: re-add release-plz.toml — see PR #34 commit f1d89e5" >&2
  exit 1
fi

if ! grep -qE '^\[workspace\]' "$cfg"; then
  echo "MISSING: $cfg has no [workspace] table header" >&2
  echo "  recovery: add '[workspace]' section — see PR #34 commit f1d89e5" >&2
  fail=1
fi

if ! grep -qE '^[[:space:]]*git_release_enable[[:space:]]*=[[:space:]]*false' "$cfg"; then
  echo "MISSING: $cfg does not set 'git_release_enable = false'" >&2
  echo "  recovery: re-add the line under [workspace] — without it release-plz" >&2
  echo "  steals releases/latest on every push to main. See PR #34 commit f1d89e5." >&2
  fail=1
fi

if [[ $fail -ne 0 ]]; then
  exit 1
fi

echo "OK: release-plz.toml disables per-package GH release creation."
