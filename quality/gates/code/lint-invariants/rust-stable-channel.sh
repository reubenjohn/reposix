#!/usr/bin/env bash
# quality/gates/code/lint-invariants/rust-stable-channel.sh
# Binds catalog row: docs-development-contributing-md/rust-stable-no-nightly
#
# Asserts `rust-toolchain.toml` declares `channel = "stable"`. Nightly is
# banned per CLAUDE.md "Coding conventions" — if anyone flips the toolchain
# to nightly, this verifier blocks the next walk.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
readonly TOOLCHAIN="${REPO_ROOT}/rust-toolchain.toml"

if [ ! -f "$TOOLCHAIN" ]; then
  echo "FAIL: rust-toolchain.toml not found at $TOOLCHAIN" >&2
  exit 1
fi

if ! grep -qE '^[[:space:]]*channel[[:space:]]*=[[:space:]]*"stable"' "$TOOLCHAIN"; then
  echo "FAIL: rust-toolchain.toml missing 'channel = \"stable\"' — nightly is banned per CLAUDE.md" >&2
  cat "$TOOLCHAIN" >&2
  exit 1
fi

echo "PASS: toolchain channel = stable"
exit 0
