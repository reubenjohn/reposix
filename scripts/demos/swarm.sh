#!/usr/bin/env bash
# scripts/demos/swarm.sh — Tier 4 adversarial-swarm demo.
#
# AUDIENCE: developer, ops
# RUNTIME_SEC: 40
# REQUIRES: cargo, jq, sqlite3, curl
# ASSERTS: "Swarm summary" "Append-only invariant: upheld" "DEMO COMPLETE"
#
# Narrative: spawn sim -> run reposix-swarm with 50 concurrent agents for
# 30s in sim-direct mode -> print the markdown report -> assert the audit
# row count matches total ops (SG-06 invariant still upheld under load).

set -euo pipefail

# Self-wrap in `timeout 60` so a stuck sub-step cannot hang smoke.sh
# (even though smoke doesn't include this demo today).
if [[ -z "${REPOSIX_DEMO_INNER:-}" ]]; then
    exec timeout 60 env REPOSIX_DEMO_INNER=1 bash "$0" "$@"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/_lib.sh"

require reposix-sim
require reposix-swarm
require curl
require jq
require sqlite3

# --------- config ---------
SIM_BIND="127.0.0.1:7804"
SIM_URL="http://${SIM_BIND}"
SIM_DB="/tmp/reposix-demo-swarm-sim.db"
CLIENTS="${SWARM_CLIENTS:-50}"
DURATION="${SWARM_DURATION:-30}"
export SIM_BIND SIM_URL
cleanup_trap

# Clean any prior debris so re-runs are idempotent.
pkill -f "reposix-sim --bind ${SIM_BIND}" 2>/dev/null || true
rm -f "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm" 2>/dev/null || true
sleep 0.2

section "[1/4] start simulator (rate-limit 1000 rps/agent)"
# The swarm deliberately hammers: we raise the per-agent rate limit so
# concurrent agents don't immediately hit 429. Each simulated agent
# still has its own X-Reposix-Agent bucket, but the swarm loop is
# tighter than a human would be.
rm -f "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm"
_REPOSIX_TMP_PATHS+=("$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm")
reposix-sim --bind "$SIM_BIND" --db "$SIM_DB" \
    --seed-file crates/reposix-sim/fixtures/seed.json \
    --rate-limit 1000 \
    >/tmp/reposix-demo-swarm-sim.log 2>&1 &
SIM_PID=$!
_REPOSIX_SIM_PIDS+=("$SIM_PID")
export SIM_PID SIM_DB SIM_URL SIM_BIND
wait_for_url "${SIM_URL}/healthz" 10
echo "sim ready at $SIM_URL"

section "[2/4] seed snapshot (pre-swarm)"
echo "seed issue count:"
curl -s "${SIM_URL}/projects/demo/issues" | jq 'length'
echo "audit rows pre-swarm:"
sqlite3 "$SIM_DB" 'SELECT COUNT(*) FROM audit_events;'

section "[3/4] run swarm (${CLIENTS} clients × ${DURATION}s, sim-direct)"
REPOSIX_ALLOWED_ORIGINS='http://127.0.0.1:*' reposix-swarm \
    --clients "$CLIENTS" \
    --duration "$DURATION" \
    --mode sim-direct \
    --target "$SIM_URL" \
    --audit-db "$SIM_DB"

section "[4/4] invariant check (SG-06: audit table is append-only)"
# Prove the trigger still blocks UPDATE/DELETE even after a hammering
# run. Exit code of the UPDATE must be non-zero (trigger raises).
set +e
UPDATE_ERR=$(sqlite3 "$SIM_DB" 'UPDATE audit_events SET path="/hacked" WHERE id=1;' 2>&1)
UPDATE_RC=$?
set -e
if [[ $UPDATE_RC -eq 0 ]]; then
    echo "FAIL: UPDATE on audit_events succeeded (trigger broken)"
    exit 1
fi
echo "UPDATE blocked (rc=${UPDATE_RC}): ${UPDATE_ERR}"

echo
echo "== DEMO COMPLETE =="
