#!/usr/bin/env bash
# quality/gates/code/lint-invariants/rust-msrv.sh
# Binds catalog row: README-md/rust-1-82-requirement
#
# Asserts workspace `Cargo.toml` pins `rust-version = "1.82"` under the
# [workspace.package] table. If MSRV-01 (P70 carry-forward) ever bumps the
# pin to 1.85, this verifier breaks LOUD — that is the desired drift signal,
# at which point the README + contributing.md prose must update first.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
readonly CARGO_TOML="${REPO_ROOT}/Cargo.toml"

if [ ! -f "$CARGO_TOML" ]; then
  echo "FAIL: workspace Cargo.toml not found at $CARGO_TOML" >&2
  exit 1
fi

# Match `rust-version = "1.82"` (allow leading whitespace; Cargo.toml lives
# under [workspace.package] so the line is typically un-indented but tolerate
# either shape).
if ! grep -qE '^[[:space:]]*rust-version[[:space:]]*=[[:space:]]*"1\.82"' "$CARGO_TOML"; then
  echo "FAIL: workspace Cargo.toml missing 'rust-version = \"1.82\"'" >&2
  echo "current rust-version line(s):" >&2
  grep -nE 'rust-version' "$CARGO_TOML" >&2 || echo "  (none)" >&2
  exit 1
fi

echo "PASS: workspace MSRV pinned to 1.82"
exit 0
