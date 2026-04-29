#!/usr/bin/env bash
# P73 CONNECTOR-GAP-04 path (a): existence verifier for the JIRA real
# adapter, bound to docs/benchmarks/token-economy/jira-real-adapter-not-implemented.
#
# The bench row at docs/benchmarks/token-economy.md:23-28 was authored
# when the JIRA real adapter was not yet implemented. v0.11.x Phase 29
# shipped it. This verifier asserts the manifest exists; if a future
# change deletes the crate, the doc-alignment walker fires
# STALE_TEST_DRIFT and a maintainer reviews.
#
# Bench-number re-measurement (the actual token counts for the row's
# remaining columns) remains deferred to perf-dim P67.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
MANIFEST="${REPO_ROOT}/crates/reposix-jira/Cargo.toml"
if [[ ! -f "$MANIFEST" ]]; then
  echo "FAIL: ${MANIFEST} missing — JIRA real adapter crate was deleted; bench row at docs/benchmarks/token-economy.md:23-28 needs immediate update or retire" >&2
  exit 1
fi
echo "PASS: reposix-jira crate present (real adapter shipped v0.11.x; bench numbers tracked under perf-dim P67)"
exit 0
