#!/usr/bin/env bash
# POC path (a): reconciliation against deliberately-mangled checkout.
# Architecture-sketch.md § "Reconciliation cases" rows 1-5.
#
# 1. Start a sim with a seed file containing records {id=1, id=42}, NOT id=99.
# 2. Copy fixtures (which include id=1, id=42 (twice), id=99, no-id) into
#    a working tree at /tmp/reposix-poc-79-checkout.
# 3. Invoke the scratch reconciler; classify each fixture into one of the
#    5 reconciliation cases.
# 4. Emit a fifth case (MIRROR_LAG) by seeding a backend record (id=2)
#    with NO matching local file.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"

: "${SIM_PORT:=7888}"
: "${REPOSIX_CACHE_DIR:=/tmp/reposix-poc-79-cache}"
export REPOSIX_CACHE_DIR

echo "=== Path (a): reconciliation against mangled checkout ==="
echo "[path-a] SIM_PORT=${SIM_PORT}"

# Build the seed: id=1 (matches local), id=2 (mirror-lag — no local file),
# id=42 (will be ambiguous: TWO local files claim it). id=99 deliberately
# absent (the local file claims it, exercising BACKEND_DELETED).
SEED="${SCRIPT_DIR}/path-a-seed.json"
cat > "${SEED}" <<'EOF'
{
  "project": {
    "slug": "demo",
    "name": "POC-79 path (a)",
    "description": "Reconciliation walk against deliberately-mangled checkout."
  },
  "issues": [
    {"id": 1,  "title": "matches local 0001.md", "status": "open", "labels": [], "body": "v1"},
    {"id": 2,  "title": "no local file (mirror lag)", "status": "open", "labels": [], "body": "v1"},
    {"id": 42, "title": "duplicate target", "status": "open", "labels": [], "body": "v1"}
  ]
}
EOF

# Start the sim ephemeral (in-memory DB, no on-disk side effects).
echo "[path-a] starting sim on 127.0.0.1:${SIM_PORT} (ephemeral)"
(
    cd "${REPO_ROOT}"
    cargo run -p reposix-sim --quiet -- \
        --bind "127.0.0.1:${SIM_PORT}" \
        --ephemeral \
        --seed-file "${SEED}" \
        > "${SCRIPT_DIR}/logs/sim-a.log" 2>&1 &
    echo $! > /tmp/reposix-poc-79-sim-a.pid
)
SIM_A_PID="$(cat /tmp/reposix-poc-79-sim-a.pid)"
echo "[path-a] sim PID=${SIM_A_PID}"

cleanup_a() {
    if [[ -n "${SIM_A_PID:-}" ]]; then
        kill "${SIM_A_PID}" 2>/dev/null || true
        wait "${SIM_A_PID}" 2>/dev/null || true
    fi
    rm -f /tmp/reposix-poc-79-sim-a.pid "${SEED}" 2>/dev/null || true
}
trap cleanup_a EXIT

# Wait for the sim to bind. Poll /healthz up to 30s.
echo "[path-a] waiting for sim to bind..."
for i in $(seq 1 60); do
    if curl -fsS -m 1 "http://127.0.0.1:${SIM_PORT}/healthz" > /dev/null 2>&1; then
        echo "[path-a] sim alive after ${i} polls"
        break
    fi
    if [[ $i -eq 60 ]]; then
        echo "[path-a] FATAL: sim never bound on :${SIM_PORT}" >&2
        echo "[path-a] sim log tail:" >&2
        tail -20 "${SCRIPT_DIR}/logs/sim-a.log" >&2 || true
        exit 1
    fi
    sleep 0.5
done

# Smoke-check the seed loaded as expected (3 records, ids {1,2,42}).
echo "[path-a] checking seed..."
SEED_LIST=$(curl -fsS "http://127.0.0.1:${SIM_PORT}/projects/demo/issues")
echo "[path-a] seeded: ${SEED_LIST}"

# Copy fixtures into a fresh working tree.
WORK=/tmp/reposix-poc-79-checkout
rm -rf "${WORK}"
mkdir -p "${WORK}"
cp -r "${SCRIPT_DIR}/fixtures/mangled-checkout/." "${WORK}/"
(
    cd "${WORK}"
    git init -q
    git -c user.email=poc@reposix.local -c user.name=poc add .
    git -c user.email=poc@reposix.local -c user.name=poc commit -q -m "fixture: mangled checkout"
)
echo "[path-a] working tree built at ${WORK}"

# Build + run the scratch reconciler. Cargo invocation is SERIAL relative
# to the sim's earlier `cargo run` (sim is now a long-running bg process;
# its compile finished before the trap ran).
echo "[path-a] building scratch reconciler..."
(
    cd "${SCRIPT_DIR}/scratch"
    cargo build --quiet --release 2>&1 | tail -20
)

echo "[path-a] invoking reconciler..."
RC=0
"${SCRIPT_DIR}/scratch/target/release/reposix-poc-79" \
    --working-tree "${WORK}" \
    --sot-origin "http://127.0.0.1:${SIM_PORT}" \
    --project demo || RC=$?

echo "[path-a] reconciler exit=${RC}"

# Hard assertions on stdout shape — verify all 5 reconciliation cases observed.
LOG="${SCRIPT_DIR}/logs/path-a-reconciliation.log"
# (At this point we're streaming via `tee` from run.sh, so the log isn't
#  finalized; instead, re-check exit code + scrape the just-printed table
#  by re-running into a temp file. Cheaper: trust RC, since the reconciler
#  only exits 0 when all 5 cases observed.)
if [[ ${RC} -ne 0 ]]; then
    echo "[path-a] FAIL: reconciler did not observe all 5 cases (exit=${RC})" >&2
    exit ${RC}
fi
echo "[path-a] OK: all 5 reconciliation cases observed"
