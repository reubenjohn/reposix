#!/usr/bin/env bash
# quality/gates/code/lint-invariants/tests-green.sh
# Binds catalog row: README-md/tests-green
#
# D-05: COMPILE-ONLY signal — `cargo test --workspace --no-run`. The actual
# test-suite execution lives in `quality/gates/code/` nextest invocations
# at pre-push time. The docs-alignment binding here is a CHEAP "the test
# target compiles" assertion; if compilation fails, every other lint-config
# verifier in this sub-area would have failed already, so the value is the
# explicit baseline check.
#
# Memory budget (D-04): cargo-test-count.sh's earlier invocation populated
# the test-binary cache, so this run is fast on warm cache. Still serialized.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$REPO_ROOT"

if ! cargo test --workspace --no-run 2>&1; then
  echo "FAIL: cargo test --workspace --no-run failed (workspace test compilation broken)" >&2
  exit 1
fi

echo "PASS: workspace tests compile clean"
exit 0
