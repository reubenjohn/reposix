#!/usr/bin/env bash
# quality/gates/code/lint-invariants/cargo-test-count.sh
# Binds catalog row: docs-development-contributing-md/cargo-test-133-tests
#
# TODO(P72 task 8): compile-only (`cargo test --workspace --no-run
# --message-format=json`), count `compiler-artifact` events with
# `target.test == true`, and assert `count >= ${REPOSIX_LINT_TEST_FLOOR:-N}`
# where N is the documented count from contributing.md (re-measured at
# task 10 per D-06).

set -euo pipefail

echo "STUB: $0 not yet implemented" >&2
exit 1
