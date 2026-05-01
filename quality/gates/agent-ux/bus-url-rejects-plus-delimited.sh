#!/usr/bin/env bash
# quality/gates/agent-ux/bus-url-rejects-plus-delimited.sh — agent-ux
# verifier for catalog row `agent-ux/bus-url-rejects-plus-delimited`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-url-rejects-plus-delimited
# CADENCE:     pre-pr
# INVARIANT:   bus_url::parse rejects + form (Q3.3) AND unknown query keys (Q-C);
#              error message names ?mirror= as the canonical form.
#
# Status until P82-01 T06: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_url \
    --quiet -- --nocapture \
    rejects_plus_delimited_bus_url rejects_unknown_query_param \
    2>&1 | tail -20

echo "PASS: bus_url::parse rejects + form AND unknown query keys"
exit 0
