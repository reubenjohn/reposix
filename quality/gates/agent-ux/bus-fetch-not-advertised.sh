#!/usr/bin/env bash
# quality/gates/agent-ux/bus-fetch-not-advertised.sh — agent-ux
# verifier for catalog row `agent-ux/bus-fetch-not-advertised`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-fetch-not-advertised
# CADENCE:     pre-pr (~5s wall time)
# INVARIANT:   capability list emitted for bus URL contains
#              import / export / refspec / object-format=sha1
#              but NOT stateless-connect (DVCS-BUS-FETCH-01 / Q3.4).
#              Capability list for bare reposix:: URL DOES contain
#              stateless-connect (regression check on single-backend).
#
# Status until P82-01 T06: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_capabilities \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: bus URL omits stateless-connect; single-backend retains it"
exit 0
