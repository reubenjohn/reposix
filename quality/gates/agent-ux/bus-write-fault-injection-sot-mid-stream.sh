#!/usr/bin/env bash
# quality/gates/agent-ux/bus-write-fault-injection-sot-mid-stream.sh — agent-ux
# verifier for catalog row `agent-ux/bus-write-fault-injection-sot-mid-stream`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-fault-injection-sot-mid-stream
# CADENCE:     pre-pr (~10s wall time)
# INVARIANT:   Fault scenario (b) — confluence write fails
#              mid-stream (5xx on second PATCH). Helper exits
#              non-zero with `error refs/heads/main some-actions-failed`;
#              NO mirror push attempted; mirror baseline preserved;
#              NO helper_push_accepted row; NO
#              helper_push_partial_fail_mirror_lag row;
#              wiremock saw 2 PATCH requests (id=1 succeeded,
#              id=2 returned 500).
#
# Status until P83-02 T04: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_write_sot_fail \
    bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: fault-injection (b) SoT-mid-stream produces correct end-state"
exit 0
