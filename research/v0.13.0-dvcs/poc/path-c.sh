#!/usr/bin/env bash
# POC path (c): cheap precheck refusing fast on SoT version mismatch.
# Architecture-sketch.md § "Algorithm (export path)" steps 2-3 (CHEAP
# PRECHECK B).
#
# 1. Snapshot the local view of record id=1 (version=1).
# 2. SoT drifts: a third party PATCHes record id=1 server-side (version
#    bumps to 2).
# 3. Cheap precheck: GET the record; compare server version to local
#    version. Mismatch → emit the production-shaped error message and
#    EXIT non-zero — without reading stdin, without attempting any REST
#    write.
#
# The POC uses a single-record GET as the precheck shape. Production's
# `list_changed_since` is the L1 target (P81); the POC stays single-call
# to keep findings sharp on the version-mismatch behavior.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"

: "${SIM_PORT:=7888}"

echo "=== Path (c): cheap precheck on SoT mismatch ==="
echo "[path-c] SIM_PORT=${SIM_PORT}"

# Pre-clean.
pkill -f "reposix-sim.*${SIM_PORT}" 2>/dev/null || true
sleep 0.5

SEED="${SCRIPT_DIR}/path-c-seed.json"
cat > "${SEED}" <<'EOF'
{
  "project": {"slug": "demo", "name": "POC-79 path (c)", "description": "Precheck."},
  "issues": [{"id": 1, "title": "v1", "status": "open", "labels": [], "body": "original"}]
}
EOF

echo "[path-c] starting sim on :${SIM_PORT}"
(
    cd "${REPO_ROOT}"
    cargo run -p reposix-sim --quiet -- \
        --bind "127.0.0.1:${SIM_PORT}" \
        --ephemeral \
        --seed-file "${SEED}" \
        > "${SCRIPT_DIR}/logs/sim-c.log" 2>&1 &
    echo $! > /tmp/reposix-poc-79-sim-c.pid
)
SIM_PID="$(cat /tmp/reposix-poc-79-sim-c.pid)"

cleanup_c() {
    [[ -n "${SIM_PID:-}" ]] && kill "${SIM_PID}" 2>/dev/null || true
    rm -f /tmp/reposix-poc-79-sim-c.pid "${SEED}" 2>/dev/null || true
}
trap cleanup_c EXIT

# Wait for bind.
for i in $(seq 1 60); do
    if curl -fsS -m 1 "http://127.0.0.1:${SIM_PORT}/healthz" > /dev/null 2>&1; then
        echo "[path-c] sim alive after ${i} polls"; break
    fi
    if [[ $i -eq 60 ]]; then
        echo "[path-c] FATAL: sim never bound" >&2; exit 1
    fi
    sleep 0.5
done

# Step 1: snapshot local view (developer ran `reposix sync` previously;
# local cache says version=1).
LOCAL_VERSION=1
echo "[path-c] Step 1: local snapshot version=${LOCAL_VERSION}"

# Step 2: SoT drifts — PATCH the record server-side; version bumps to 2.
echo "[path-c] Step 2: SoT drift (a third party PATCHes record id=1)"
PATCH_RESP=$(curl -fsS -X PATCH \
    "http://127.0.0.1:${SIM_PORT}/projects/demo/issues/1" \
    -H 'Content-Type: application/json' \
    -d '{"body":"server-side bump"}')
SERVER_VERSION=$(echo "${PATCH_RESP}" | grep -oE '"version":[0-9]+' | head -1 | grep -oE '[0-9]+')
echo "[path-c]   server is now version=${SERVER_VERSION}"

# Step 3: cheap precheck. Single GET; compare; refuse fast.
echo "[path-c] Step 3: cheap precheck"
GET_RESP=$(curl -fsS "http://127.0.0.1:${SIM_PORT}/projects/demo/issues/1")
GET_VERSION=$(echo "${GET_RESP}" | grep -oE '"version":[0-9]+' | head -1 | grep -oE '[0-9]+')
echo "[path-c]   precheck GET returned version=${GET_VERSION}"
echo "[path-c]   local=${LOCAL_VERSION}  server=${GET_VERSION}"

if [[ "${GET_VERSION}" != "${LOCAL_VERSION}" ]]; then
    # Emit production-shaped error message: matches the architecture-sketch
    # "error refs/heads/main fetch first" + hint pattern.
    cat <<EOF
[path-c]   MISMATCH: refusing fast — SoT drifted (local=${LOCAL_VERSION}, server=${GET_VERSION})
[path-c]   error refs/heads/main fetch first
[path-c]   hint: confluence has changes since your last fetch; git pull --rebase
[path-c]   (precheck completed without reading stdin or attempting any write)
EOF
    echo "[path-c] OK: precheck refused fast on SoT mismatch."
else
    echo "[path-c]   UNEXPECTED: versions match (precheck would not refuse)"
    exit 1
fi
