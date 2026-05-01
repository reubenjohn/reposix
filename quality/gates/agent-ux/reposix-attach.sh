#!/usr/bin/env bash
# quality/gates/agent-ux/reposix-attach.sh — agent-ux verifier for the
# `reposix attach` subcommand (DVCS-ATTACH-01).
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/reposix-attach-against-vanilla-clone
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   `reposix attach <spec>` against a vanilla `git init` checkout
#              configures `extensions.partialClone=<remote-name>` (NOT
#              `origin`) and a `reposix::` remote URL — the dark-factory
#              partial-clone surface for the v0.13.0 DVCS topology.
#
# Status until P79-03 T03: FAIL — attach is scaffold-only in P79-02; the
# integration tests + behaviour coverage land in P79-03 and only then does
# the runner re-grade this row to PASS.
#
# Usage: bash quality/gates/agent-ux/reposix-attach.sh
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

PORT=7898
WORK=$(mktemp -d -t reposix-attach-verify.XXXXXX)
SIM_PID=""
cleanup() {
    if [[ -n "${SIM_PID}" ]]; then kill "${SIM_PID}" 2>/dev/null || true; fi
    rm -rf "${WORK}"
}
trap cleanup EXIT

cargo build -p reposix-sim -p reposix-cli --quiet
SIM_BIN="${REPO_ROOT}/target/debug/reposix-sim"
CLI_BIN="${REPO_ROOT}/target/debug/reposix"
"${SIM_BIN}" --bind "127.0.0.1:${PORT}" --ephemeral --no-seed &
SIM_PID=$!
sleep 1

cd "${WORK}"
git init -q
# Verifier-isolated cache + sim origin so a stale ~/.cache/reposix dir
# from a previous run doesn't poison Cache::open's identity check.
CACHE_DIR=$(mktemp -d -t reposix-attach-cache.XXXXXX)
trap 'cleanup; rm -rf "${CACHE_DIR}"' EXIT
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
REPOSIX_SIM_ORIGIN="http://127.0.0.1:${PORT}" \
"${CLI_BIN}" attach "sim::demo" --remote-name reposix > /dev/null

PCLONE=$(git config --get extensions.partialClone || true)
RURL=$(git config --get remote.reposix.url || true)
[[ "${PCLONE}" == "reposix" ]] || { echo "FAIL: extensions.partialClone=${PCLONE} expected reposix" >&2; exit 1; }
[[ "${RURL}" == reposix::* ]] || { echo "FAIL: remote.reposix.url=${RURL} expected reposix:: prefix" >&2; exit 1; }
echo "PASS: reposix attach sets extensions.partialClone=reposix and reposix:: remote URL"
exit 0
