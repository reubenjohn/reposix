#!/usr/bin/env bash
# POC-01 runner (P79 / v0.13.0). Throwaway end-to-end against the simulator.
# See research/v0.13.0-dvcs/poc/README.md for the throwaway contract.
#
# This script orchestrates three integration paths:
#   path-a.sh  — reconciliation against deliberately-mangled checkout
#   path-b.sh  — bus SoT-first observing mirror lag
#   path-c.sh  — cheap precheck on SoT version mismatch
#
# Each path captures stdout/stderr to logs/<path>.log; FINDINGS is updated
# manually after the run by reading the transcripts.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# Isolation: dedicated cache + non-default sim port; aggressive cleanup on EXIT.
export REPOSIX_CACHE_DIR=/tmp/reposix-poc-79-cache
export SIM_PORT="${POC_SIM_PORT:-7888}"

cleanup_residual() {
    # Tear down any sims still bound to our test ports, plus scratch state.
    pkill -f "reposix-sim.*${SIM_PORT}" 2>/dev/null || true
    pkill -f "reposix-sim.*$((SIM_PORT + 1))" 2>/dev/null || true
    rm -rf /tmp/reposix-poc-79-cache /tmp/reposix-poc-79-* 2>/dev/null || true
}
trap cleanup_residual EXIT
cleanup_residual  # pre-clean in case a prior run left state

mkdir -p "${SCRIPT_DIR}/logs"
echo "POC-01 starting at $(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee "${SCRIPT_DIR}/logs/run.log"

# Path (a): reconciliation against mangled checkout.
echo "" | tee -a "${SCRIPT_DIR}/logs/run.log"
echo "[run.sh] dispatching path-a..." | tee -a "${SCRIPT_DIR}/logs/run.log"
bash "${SCRIPT_DIR}/path-a.sh" 2>&1 | tee "${SCRIPT_DIR}/logs/path-a-reconciliation.log"

# Path (b): bus SoT-first observing mirror lag.
echo "" | tee -a "${SCRIPT_DIR}/logs/run.log"
echo "[run.sh] dispatching path-b..." | tee -a "${SCRIPT_DIR}/logs/run.log"
bash "${SCRIPT_DIR}/path-b.sh" 2>&1 | tee "${SCRIPT_DIR}/logs/path-b-bus-mirror-lag.log"

# Path (c): cheap precheck on SoT mismatch.
echo "" | tee -a "${SCRIPT_DIR}/logs/run.log"
echo "[run.sh] dispatching path-c..." | tee -a "${SCRIPT_DIR}/logs/run.log"
bash "${SCRIPT_DIR}/path-c.sh" 2>&1 | tee "${SCRIPT_DIR}/logs/path-c-cheap-precheck.log"

echo "" | tee -a "${SCRIPT_DIR}/logs/run.log"
echo "POC-01 complete at $(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a "${SCRIPT_DIR}/logs/run.log"
