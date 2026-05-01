#!/usr/bin/env bash
# quality/gates/agent-ux/bus-write-fault-injection-mirror-fail.sh — agent-ux
# verifier for catalog row `agent-ux/bus-write-fault-injection-mirror-fail`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-fault-injection-mirror-fail
# CADENCE:     pre-pr (~10s wall time)
# INVARIANT:   Fault scenario (a) — mirror push fails between
#              confluence-write and ack. Helper exits zero with
#              `ok refs/heads/main` (Q3.6 SoT contract); audit op
#              helper_push_partial_fail_mirror_lag written;
#              refs/mirrors/<sot>-head advances; synced-at frozen;
#              mirror baseline preserved (failing-update-hook
#              rejects the push).
#
# Status until P83-02 T04: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_write_mirror_fail \
    bus_write_mirror_fail_returns_ok_with_lag_audit_row \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: fault-injection (a) mirror-fail produces correct end-state"
exit 0
