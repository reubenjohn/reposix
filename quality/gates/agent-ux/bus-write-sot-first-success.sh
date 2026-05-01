#!/usr/bin/env bash
# quality/gates/agent-ux/bus-write-sot-first-success.sh — agent-ux
# verifier for catalog row `agent-ux/bus-write-sot-first-success`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-sot-first-success
# CADENCE:     pre-pr (~10s wall time)
# INVARIANT:   bus_handler::handle_bus_export reads stdin via
#              parse_export_stream, calls write_loop::apply_writes,
#              writes refs/mirrors/<sot>-head AND refs/mirrors/<sot>-synced-at
#              on SoT-success + mirror-success; audit_events_cache
#              has helper_push_started + helper_push_accepted +
#              mirror_sync_written rows; helper exits zero with
#              `ok refs/heads/main` on stdout.
#
# Status until P83-01 T06: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_write_happy \
    happy_path_writes_both_refs_and_acks_ok \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: bus write SoT-first-success path writes both refs + dual audit table"
exit 0
