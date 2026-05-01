#!/usr/bin/env bash
# quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh — agent-ux
# verifier for catalog row `agent-ux/mirror-refs-readable-by-vanilla-fetch`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/mirror-refs-readable-by-vanilla-fetch
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   After a single-backend push has populated mirror refs,
#              a fresh `git clone --bare` of the cache's bare repo
#              (or `git fetch` from an existing clone) brings BOTH
#              refs/mirrors/<sot>-head and refs/mirrors/<sot>-synced-at
#              into the new clone — proves vanilla-git readers can
#              observe mirror lag without any reposix awareness.
#
# Status until P80-01 T04: FAIL — wiring is scaffold-only in T01-T03.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# Pick a free high-range port (M4); H1 — re-point URL after init.
PORT=$(comm -23 <(seq 49152 65535 | sort) <(ss -tan 2>/dev/null | awk 'NR>1 {print $4}' | awk -F: '{print $NF}' | sort -u) | shuf -n 1)
WORK=$(mktemp -d -t reposix-mirror-refs-fetch.XXXXXX)
CACHE_DIR=$(mktemp -d -t reposix-mirror-refs-fetch-cache.XXXXXX)
CLONE=$(mktemp -d -t reposix-mirror-refs-fetch-clone.XXXXXX)
SIM_PID=""
cleanup() {
    if [[ -n "${SIM_PID}" ]]; then kill "${SIM_PID}" 2>/dev/null || true; fi
    rm -rf "${WORK}" "${CACHE_DIR}" "${CLONE}"
}
trap cleanup EXIT

cargo build -p reposix-sim -p reposix-cli --quiet
SIM_BIN="${REPO_ROOT}/target/debug/reposix-sim"
CLI_BIN="${REPO_ROOT}/target/debug/reposix"
"${SIM_BIN}" --bind "127.0.0.1:${PORT}" --ephemeral &
SIM_PID=$!
sleep 1

# H1: REPOSIX_SIM_ORIGIN is a no-op for `reposix init`; we re-point
# remote.origin.url after init so the subsequent `git push` reaches
# our sim. See shell #1's H1 comment block for the full rationale.
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
"${CLI_BIN}" init "sim::demo" "${WORK}" > /dev/null 2>&1 || true
git -C "${WORK}" config remote.origin.url "reposix::http://127.0.0.1:${PORT}/projects/demo"
cd "${WORK}"
git fetch -q origin
git checkout origin/main -B main -q
echo "" >> issues/0001.md
git add . && git commit -q -m "trivial change for mirror-refs-fetch verifier"
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
git push -q origin main

CACHE_BARE=$(find "${CACHE_DIR}" -name '*.git' -type d -print -quit)
[[ -n "${CACHE_BARE}" ]] || { echo "FAIL: cache bare repo not found" >&2; exit 1; }

# Vanilla `git clone --bare` — no reposix involvement.
git clone --bare -q "${CACHE_BARE}" "${CLONE}/mirror.git"
git -C "${CLONE}/mirror.git" for-each-ref refs/mirrors/ | grep -q "refs/mirrors/sim-head" \
    || { echo "FAIL: vanilla-clone missing refs/mirrors/sim-head" >&2; exit 1; }
git -C "${CLONE}/mirror.git" for-each-ref refs/mirrors/ | grep -q "refs/mirrors/sim-synced-at" \
    || { echo "FAIL: vanilla-clone missing refs/mirrors/sim-synced-at" >&2; exit 1; }

echo "PASS: vanilla-fetch brings refs/mirrors/* along to a fresh bare clone"
exit 0
