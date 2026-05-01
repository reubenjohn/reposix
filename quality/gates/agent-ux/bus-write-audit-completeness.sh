#!/usr/bin/env bash
# quality/gates/agent-ux/bus-write-audit-completeness.sh — agent-ux
# verifier for catalog row `agent-ux/bus-write-audit-completeness`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-audit-completeness
# CADENCE:     pre-pr (~10s wall time)
# INVARIANT:   Happy-path bus push writes expected rows to BOTH
#              audit tables per OP-3:
#              audit_events_cache: helper_push_started +
#                helper_push_accepted + mirror_sync_written +
#                helper_backend_instantiated;
#              audit_events (sim DB or wiremock-equivalent at the
#                wire boundary): one row per executed REST mutation
#                (create_record / update_record / delete_or_close).
#
# Status until P83-02 T04: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_write_audit_completeness \
    bus_write_audit_completeness_happy_path_writes_both_tables \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: audit-completeness happy-path writes both tables"
exit 0
