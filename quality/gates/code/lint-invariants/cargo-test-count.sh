#!/usr/bin/env bash
# quality/gates/code/lint-invariants/cargo-test-count.sh
# Binds catalog row: docs-development-contributing-md/cargo-test-133-tests
#
# Compile-only (D-05): `cargo test --workspace --no-run --message-format=json`,
# parse compiler-artifact events with target.test == true, count, and assert
# `count >= ${REPOSIX_LINT_TEST_FLOOR:-N}` where N is the documented test-binary
# count from contributing.md.
#
# D-06: monotone-friendly floor — test ADDITIONS never break the verifier;
# only test DELETIONS trigger BLOCK (the desired drift signal). The floor's
# default is the count measured at P72 task 10; future ratchets land via
# explicit env override + prose update (re-measure FIRST, then bump floor).
#
# Memory budget (D-04): heaviest verifier in this sub-area. Full-workspace
# `cargo test --no-run` link step uses 4-6 GB. Runner serializes; do not
# parallel-launch.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$REPO_ROOT"

# Default floor — re-measured at P72 task 10 (D-06): 368 unique test binaries
# at v0.12.1 P72 close. Environment override stays intact for future
# deliberate ratchets (re-measure prose FIRST, then bump floor).
floor="${REPOSIX_LINT_TEST_FLOOR:-368}"

json=$(cargo test --workspace --no-run --message-format=json 2>/dev/null || true)
if [ -z "$json" ]; then
  echo "FAIL: cargo test --workspace --no-run produced no JSON output" >&2
  exit 1
fi

# Count UNIQUE test binaries by target.name (cargo emits the same artifact
# event multiple times when the same target is exercised by more than one
# test scope). Unique-by-name yields the count a reader of contributing.md
# would expect ("the workspace ships N test binaries").
count=$(echo "$json" | jq -rs '
  [.[] | select(.reason=="compiler-artifact" and (.target.test // false)) | .target.name] | unique | length')

if [ -z "$count" ] || [ "$count" -lt "$floor" ]; then
  echo "FAIL: counted ${count:-?} test binaries; floor is $floor (test deletions trigger BLOCK per D-06)" >&2
  exit 1
fi

echo "PASS: $count test binaries (floor $floor)"
exit 0
