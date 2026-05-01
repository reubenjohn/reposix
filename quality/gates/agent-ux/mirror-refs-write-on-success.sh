#!/usr/bin/env bash
# quality/gates/agent-ux/mirror-refs-write-on-success.sh — agent-ux
# verifier for catalog row `agent-ux/mirror-refs-write-on-success`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/mirror-refs-write-on-success
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   After a single-backend push via the existing handle_export
#              path, the cache's bare repo has BOTH refs/mirrors/<sot>-head
#              and refs/mirrors/<sot>-synced-at; the synced-at tag's
#              message body's first line matches `mirror synced at <RFC3339>`.
#
# Status until P80-01 T04: FAIL — wiring is scaffold-only in T01-T03;
# the integration tests + behavior coverage land in T04.
#
# Usage: bash quality/gates/agent-ux/mirror-refs-write-on-success.sh
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# Pick a free high-range port (avoids cross-runner collision per M4).
PORT=$(comm -23 <(seq 49152 65535 | sort) <(ss -tan 2>/dev/null | awk 'NR>1 {print $4}' | awk -F: '{print $NF}' | sort -u) | shuf -n 1)
WORK=$(mktemp -d -t reposix-mirror-refs-write.XXXXXX)
CACHE_DIR=$(mktemp -d -t reposix-mirror-refs-cache.XXXXXX)
SIM_PID=""
cleanup() {
    if [[ -n "${SIM_PID}" ]]; then kill "${SIM_PID}" 2>/dev/null || true; fi
    rm -rf "${WORK}" "${CACHE_DIR}"
}
trap cleanup EXIT

cargo build -p reposix-sim -p reposix-cli --quiet
SIM_BIN="${REPO_ROOT}/target/debug/reposix-sim"
CLI_BIN="${REPO_ROOT}/target/debug/reposix"
"${SIM_BIN}" --bind "127.0.0.1:${PORT}" --ephemeral &
SIM_PID=$!
sleep 1

# Init a working tree against sim::demo, edit, push.
#
# NOTE (H1 fix): `reposix init` does NOT honor REPOSIX_SIM_ORIGIN — it
# hardcodes DEFAULT_SIM_ORIGIN to http://127.0.0.1:7878 in
# crates/reposix-cli/src/init.rs. Only `reposix attach` reads the
# env var. The init's trailing best-effort `git fetch` against port
# 7878 will fail (sim is on ${PORT}); we re-point remote.origin.url
# AFTER init so the subsequent `git push` reaches our test sim.
# Precedent: crates/reposix-cli/tests/agent_flow.rs::dark_factory_sim_happy_path
# (explicit "We re-point the URL to our test sim below" comment).
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
"${CLI_BIN}" init "sim::demo" "${WORK}" > /dev/null 2>&1 || true
git -C "${WORK}" config remote.origin.url "reposix::http://127.0.0.1:${PORT}/projects/demo"
cd "${WORK}"
git fetch -q origin
git checkout origin/main -B main -q
echo "" >> issues/0001.md  # trivial trailing-newline change
git add . && git commit -q -m "trivial change for mirror-refs verifier"
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
git push -q origin main

# Locate the cache's bare repo.
CACHE_BARE=$(find "${CACHE_DIR}" -name '*.git' -type d -print -quit)
[[ -n "${CACHE_BARE}" ]] || { echo "FAIL: cache bare repo not found under ${CACHE_DIR}" >&2; exit 1; }

git -C "${CACHE_BARE}" for-each-ref refs/mirrors/ | grep -q "refs/mirrors/sim-head" \
    || { echo "FAIL: refs/mirrors/sim-head missing" >&2; exit 1; }
git -C "${CACHE_BARE}" for-each-ref refs/mirrors/ | grep -q "refs/mirrors/sim-synced-at" \
    || { echo "FAIL: refs/mirrors/sim-synced-at missing" >&2; exit 1; }

MSG=$(git -C "${CACHE_BARE}" log refs/mirrors/sim-synced-at -1 --format=%B 2>/dev/null | head -1)
[[ "${MSG}" =~ ^mirror\ synced\ at\ [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}.*Z$ ]] \
    || { echo "FAIL: synced-at tag message body did not match \`mirror synced at <RFC3339>\` (got: ${MSG})" >&2; exit 1; }

echo "PASS: mirror-refs written on push success; both refs resolvable; tag message body well-formed"
exit 0
