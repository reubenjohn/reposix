#!/usr/bin/env bash
# quality/gates/agent-ux/bus-no-remote-configured-error.sh — agent-ux
# verifier for catalog row `agent-ux/bus-no-remote-configured-error`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-no-remote-configured-error
# CADENCE:     pre-pr (~5s wall time)
# INVARIANT:   bus URL with no local `git remote` for the mirror
#              fails with verbatim Q3.5 hint
#              "configure the mirror remote first: git remote add <name> <mirror-url>";
#              NO auto-mutation of git config.
#
# Status until P82-01 T06: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_precheck_a \
    bus_no_remote_configured_emits_q35_hint \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: bus URL with no local git remote emits Q3.5 hint"
exit 0
