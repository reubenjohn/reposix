#!/usr/bin/env bash
# quality/gates/agent-ux/bus-precheck-b-sot-drift-emits-fetch-first.sh — agent-ux
# verifier for catalog row `agent-ux/bus-precheck-b-sot-drift-emits-fetch-first`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-precheck-b-sot-drift-emits-fetch-first
# CADENCE:     pre-pr (~10s wall time)
# INVARIANT:   PRECHECK B (SoT drift via list_changed_since) emits
#              `error refs/heads/main fetch first` on stdout
#              + hint citing refs/mirrors/<sot>-synced-at on stderr;
#              helper exits BEFORE stdin read; ZERO PATCH/PUT calls
#              hit wiremock.
#
# Status until P82-01 T06: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_precheck_b \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: PRECHECK B emits fetch first on SoT drift"
exit 0
