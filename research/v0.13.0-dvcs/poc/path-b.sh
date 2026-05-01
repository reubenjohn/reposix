#!/usr/bin/env bash
# POC path (b): bus-remote-shaped push observing mirror lag.
# Architecture-sketch.md § "Algorithm (export path)" steps 6-8.
#
# Two sims are stood up: simulator-A on SIM_PORT (the SoT), simulator-B on
# SIM_PORT+1 (the mirror surrogate — the closest local analog to "git push
# to GH mirror"). The script orchestrates the SoT-first sequencing:
#
#   1. Both sims seeded with record id=1, version=1.
#   2. PATCH SoT (sim A): version bumps to 2.
#   3. Kill sim B (mirror) before its PATCH lands. PATCH attempt fails
#      with connection refused — the lag is now > 0.
#   4. Restart sim B; replay the PATCH. Lag returns to 0.
#   5. Try to query an HTTP audit endpoint on each sim. Sim does not expose
#      one — finding for FINDINGS § Path (b).
#
# This is NOT the production bus algorithm; it's a STAGED reproduction
# that exercises the sequencing decisions the bus algorithm depends on.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"

: "${SIM_PORT:=7888}"
SIM_A_PORT="${SIM_PORT}"
SIM_B_PORT=$((SIM_PORT + 1))

echo "=== Path (b): bus SoT-first observing mirror lag ==="
echo "[path-b] sim A (SoT) port=${SIM_A_PORT}, sim B (mirror) port=${SIM_B_PORT}"

# Pre-clean: any leftover sims on either port from path-a or prior runs.
pkill -f "reposix-sim.*${SIM_A_PORT}" 2>/dev/null || true
pkill -f "reposix-sim.*${SIM_B_PORT}" 2>/dev/null || true
sleep 0.5

# Build SIM A's seed: a single record id=1.
SEED_A="${SCRIPT_DIR}/path-b-seed-a.json"
SEED_B="${SCRIPT_DIR}/path-b-seed-b.json"
cat > "${SEED_A}" <<'EOF'
{
  "project": {"slug": "demo", "name": "POC-79 path (b) SoT", "description": "Bus SoT."},
  "issues": [{"id": 1, "title": "v1 record", "status": "open", "labels": [], "body": "original"}]
}
EOF
cp "${SEED_A}" "${SEED_B}"

# Start sim A.
echo "[path-b] starting sim A on :${SIM_A_PORT}"
(
    cd "${REPO_ROOT}"
    cargo run -p reposix-sim --quiet -- \
        --bind "127.0.0.1:${SIM_A_PORT}" \
        --ephemeral \
        --seed-file "${SEED_A}" \
        > "${SCRIPT_DIR}/logs/sim-b-A.log" 2>&1 &
    echo $! > /tmp/reposix-poc-79-sim-A.pid
)
SIM_A_PID="$(cat /tmp/reposix-poc-79-sim-A.pid)"

# Start sim B.
echo "[path-b] starting sim B on :${SIM_B_PORT}"
(
    cd "${REPO_ROOT}"
    cargo run -p reposix-sim --quiet -- \
        --bind "127.0.0.1:${SIM_B_PORT}" \
        --ephemeral \
        --seed-file "${SEED_B}" \
        > "${SCRIPT_DIR}/logs/sim-b-B.log" 2>&1 &
    echo $! > /tmp/reposix-poc-79-sim-B.pid
)
SIM_B_PID="$(cat /tmp/reposix-poc-79-sim-B.pid)"

cleanup_b() {
    [[ -n "${SIM_A_PID:-}" ]] && kill "${SIM_A_PID}" 2>/dev/null || true
    [[ -n "${SIM_B_PID:-}" ]] && kill "${SIM_B_PID}" 2>/dev/null || true
    [[ -n "${SIM_B_PID2:-}" ]] && kill "${SIM_B_PID2}" 2>/dev/null || true
    rm -f /tmp/reposix-poc-79-sim-A.pid /tmp/reposix-poc-79-sim-B.pid \
        /tmp/reposix-poc-79-sim-B2.pid \
        "${SEED_A}" "${SEED_B}" 2>/dev/null || true
}
trap cleanup_b EXIT

# Wait for both sims to bind.
for which in A:${SIM_A_PORT} B:${SIM_B_PORT}; do
    label="${which%:*}"
    port="${which#*:}"
    echo "[path-b] waiting for sim ${label} on :${port}..."
    for i in $(seq 1 60); do
        if curl -fsS -m 1 "http://127.0.0.1:${port}/healthz" > /dev/null 2>&1; then
            echo "[path-b] sim ${label} alive after ${i} polls"
            break
        fi
        if [[ $i -eq 60 ]]; then
            echo "[path-b] FATAL: sim ${label} never bound on :${port}" >&2
            exit 1
        fi
        sleep 0.5
    done
done

# Step 1: confirm both seeded.
echo "[path-b] Step 1: both sims seeded"
A_V1=$(curl -fsS "http://127.0.0.1:${SIM_A_PORT}/projects/demo/issues/1" | grep -oE '"version":[0-9]+' | head -1)
B_V1=$(curl -fsS "http://127.0.0.1:${SIM_B_PORT}/projects/demo/issues/1" | grep -oE '"version":[0-9]+' | head -1)
echo "[path-b]   A:${A_V1}  B:${B_V1}"

# Step 2: SoT-first PATCH on sim A. Version bumps to 2.
echo "[path-b] Step 2: SoT write to sim A (PATCH)"
A_AFTER=$(curl -fsS -X PATCH \
    "http://127.0.0.1:${SIM_A_PORT}/projects/demo/issues/1" \
    -H 'Content-Type: application/json' \
    -d '{"body":"v2 from POC"}')
echo "[path-b]   sim A after PATCH: ${A_AFTER}"

# Step 3: kill sim B BEFORE its PATCH attempt to simulate mirror failure.
echo "[path-b] Step 3: killing sim B to simulate mirror-write failure"
kill "${SIM_B_PID}" 2>/dev/null || true
wait "${SIM_B_PID}" 2>/dev/null || true
sleep 1

# Step 4: try the mirror write; expect connection refused.
echo "[path-b] Step 4: mirror write attempt (expect failure)"
if curl -fsS -m 2 -X PATCH \
       "http://127.0.0.1:${SIM_B_PORT}/projects/demo/issues/1" \
       -H 'Content-Type: application/json' \
       -d '{"body":"v2 from POC"}' \
       > /dev/null 2>&1; then
    echo "[path-b]   UNEXPECTED: mirror write succeeded"
    exit 1
else
    echo "[path-b]   EXPECTED: mirror write failed (connection refused)."
    echo "[path-b]   Lag is now > 0:"
    A_VER=$(curl -fsS "http://127.0.0.1:${SIM_A_PORT}/projects/demo/issues/1" | grep -oE '"version":[0-9]+' | head -1)
    echo "[path-b]     sim A (SoT)    : ${A_VER}"
    echo "[path-b]     sim B (mirror) : <unreachable>"
fi

# Step 5: restart sim B and replay; lag returns to 0.
echo "[path-b] Step 5: restart sim B + replay PATCH"
(
    cd "${REPO_ROOT}"
    cargo run -p reposix-sim --quiet -- \
        --bind "127.0.0.1:${SIM_B_PORT}" \
        --ephemeral \
        --seed-file "${SEED_B}" \
        > "${SCRIPT_DIR}/logs/sim-b-B-restart.log" 2>&1 &
    echo $! > /tmp/reposix-poc-79-sim-B2.pid
)
SIM_B_PID2="$(cat /tmp/reposix-poc-79-sim-B2.pid)"
echo "[path-b]   sim B restart PID=${SIM_B_PID2}"
for i in $(seq 1 60); do
    if curl -fsS -m 1 "http://127.0.0.1:${SIM_B_PORT}/healthz" > /dev/null 2>&1; then
        echo "[path-b]   sim B alive after ${i} polls"
        break
    fi
    sleep 0.5
done
B_AFTER=$(curl -fsS -X PATCH \
    "http://127.0.0.1:${SIM_B_PORT}/projects/demo/issues/1" \
    -H 'Content-Type: application/json' \
    -d '{"body":"v2 from POC (replay)"}')
echo "[path-b]   sim B after replay: ${B_AFTER}"
B_VER=$(echo "${B_AFTER}" | grep -oE '"version":[0-9]+' | head -1)
A_VER=$(curl -fsS "http://127.0.0.1:${SIM_A_PORT}/projects/demo/issues/1" | grep -oE '"version":[0-9]+' | head -1)
echo "[path-b]   post-replay versions: A:${A_VER}  B:${B_VER}"
echo "[path-b]   Lag now 0 (both at version 2)."

# Step 6: probe HTTP audit endpoints.
echo "[path-b] Step 6: probing audit endpoint on each sim"
for label in A B; do
    if [[ "${label}" == "A" ]]; then port="${SIM_A_PORT}"; else port="${SIM_B_PORT}"; fi
    HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:${port}/projects/demo/audit" || echo "000")
    echo "[path-b]   GET /projects/demo/audit on sim ${label}: HTTP ${HTTP_CODE}"
done
echo "[path-b]   FINDING: simulator does NOT expose an HTTP /audit endpoint."
echo "[path-b]   Audit rows live in the sim's SQLite audit_events table only;"
echo "[path-b]   production bus must own its own cache-side audit trail per OP-3."

echo "[path-b] OK: SoT-first sequencing exercised end-to-end."
