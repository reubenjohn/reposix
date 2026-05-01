#!/usr/bin/env bash
# quality/gates/agent-ux/bus-write-mirror-fail-returns-ok.sh — agent-ux
# verifier for catalog row `agent-ux/bus-write-mirror-fail-returns-ok`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-mirror-fail-returns-ok
# CADENCE:     pre-pr (~10s wall time)
# INVARIANT:   On SoT-success + mirror-fail, helper writes
#              helper_push_partial_fail_mirror_lag audit row,
#              advances refs/mirrors/<sot>-head, leaves
#              refs/mirrors/<sot>-synced-at FROZEN, emits stderr
#              warn, returns ok refs/heads/main to git.
#
# Status until P83-02 T04: FAIL.
# NOTE: This row is minted in P83-01 T01 (catalog-first contract);
# its test ships in P83-02 T02. P83-01 T06 leaves row 2 FAIL;
# P83-02 T04 flips to PASS.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_write_mirror_fail \
    bus_write_mirror_fail_returns_ok_with_lag_audit_row \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: bus write SoT-success+mirror-fail path returns ok with lag audit row"
exit 0
