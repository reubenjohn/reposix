#!/usr/bin/env bash
# quality/gates/code/lint-invariants/cargo-check-workspace.sh
# Binds catalog row: docs-development-contributing-md/cargo-check-workspace-available
#
# Runs `cargo check --workspace -q` from REPO_ROOT and asserts exit 0.
# Cheap compile-only signal — if the workspace doesn't type-check, every
# other lint-config verifier downstream will fail too.
#
# Memory budget (D-04): one cargo invocation. The runner serializes
# verifiers; the executor must not run multiple cargo invocations in
# parallel (CLAUDE.md "Build memory budget"). Expect ~10-30s on warm cache.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$REPO_ROOT"

if ! cargo check --workspace -q 2>&1; then
  echo "FAIL: cargo check --workspace -q exited non-zero" >&2
  exit 1
fi

echo "PASS: cargo check --workspace -q clean"
exit 0
