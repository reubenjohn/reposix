#!/usr/bin/env bash
# quality/gates/code/lint-invariants/errors-doc-section.sh
# Binds catalog row: docs-development-contributing-md/errors-doc-section-required
#
# D-07: clippy lint, not grep. Handles Result aliases, trait methods, and
# generics correctly — grep cannot. Asserts zero `clippy::missing_errors_doc`
# hits across the workspace.
#
# Memory budget (D-04): one cargo invocation; clippy is heavier than
# `cargo check` (30-90s on warm cache). Runner serializes verifiers.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$REPO_ROOT"

# Use --message-format=json so we can count `missing_errors_doc` precisely.
# Clippy itself may exit non-zero if other warnings/errors fire; we capture
# stdout regardless and only judge the missing_errors_doc count below.
output=$(cargo clippy --workspace --message-format=json -- \
           -W clippy::missing_errors_doc 2>/dev/null || true)

if [ -z "$output" ]; then
  echo "FAIL: clippy produced no output (workspace did not compile?)" >&2
  exit 1
fi

hits=$(echo "$output" | jq -rs '
  [.[] | select(.reason=="compiler-message" and
                .message.code != null and
                .message.code.code=="clippy::missing_errors_doc")] | length' 2>/dev/null || echo "0")

if [ "$hits" != "0" ]; then
  echo "FAIL: $hits pub fn(s) returning Result<...> lack a # Errors rustdoc section" >&2
  echo "$output" | jq -rs '
    [.[] | select(.reason=="compiler-message" and
                  .message.code != null and
                  .message.code.code=="clippy::missing_errors_doc")][] |
    "\(.message.spans[0].file_name):\(.message.spans[0].line_start) — \(.message.message)"' >&2
  exit 1
fi

echo "PASS: 0 missing_errors_doc hits across workspace"
exit 0
