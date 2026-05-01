#!/usr/bin/env bash
# quality/gates/agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first.sh — agent-ux
# verifier for catalog row `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first
# CADENCE:     pre-pr (~5s wall time)
# INVARIANT:   PRECHECK A (mirror drift via git ls-remote) emits
#              `error refs/heads/main fetch first` on stdout
#              + hint on stderr; helper exits BEFORE PRECHECK B
#              + BEFORE stdin read.
#
# Status until P82-01 T06: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_precheck_a \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: PRECHECK A emits fetch first on mirror drift"
exit 0
