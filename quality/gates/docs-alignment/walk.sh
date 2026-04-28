#!/usr/bin/env bash
# Wrapper: invoke the compiled docs-alignment hash drift walker.
#
# Detects target/release/reposix-quality first (CI / steady state); falls back
# to target/debug/reposix-quality when only debug artifacts exist (developer
# loop). Exits non-zero with a clear stderr message if neither binary exists --
# the runner forwards stderr verbatim so the slash-command hint reaches the
# user.
#
# Source: crates/reposix-quality/src/commands/doc_alignment.rs (verb `walk`).

set -euo pipefail

readonly REPO_ROOT="$(git rev-parse --show-toplevel)"
readonly RELEASE_BIN="${REPO_ROOT}/target/release/reposix-quality"
readonly DEBUG_BIN="${REPO_ROOT}/target/debug/reposix-quality"

if [[ -x "$RELEASE_BIN" ]]; then
  BIN="$RELEASE_BIN"
elif [[ -x "$DEBUG_BIN" ]]; then
  BIN="$DEBUG_BIN"
else
  printf '%s\n' "docs-alignment/walk: neither target/release/reposix-quality nor target/debug/reposix-quality exists" >&2
  printf '%s\n' "  build the binary first: cargo build -p reposix-quality --release" >&2
  exit 1
fi

# The walker exits non-zero on any blocking row state (STALE_DOCS_DRIFT,
# MISSING_TEST, STALE_TEST_GONE, TEST_MISALIGNED, RETIRE_PROPOSED) and prints
# the slash-command hint to stderr verbatim. Forward stderr unchanged so the
# runner surfaces it.
exec "$BIN" walk "$@"
