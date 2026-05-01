#!/usr/bin/env bash
# quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh — agent-ux
# verifier for catalog row `agent-ux/mirror-refs-cited-in-reject-hint`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/mirror-refs-cited-in-reject-hint
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   After a successful push (refs populated), a SECOND push
#              with a stale prior triggers the conflict-reject path;
#              the helper's stderr cites refs/mirrors/<sot>-synced-at
#              with a parseable RFC3339 timestamp + `(N minutes ago)`.
#
# Status until P80-01 T04: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# Pick a free high-range port (M4); H1 — re-point URL after init.
PORT=$(comm -23 <(seq 49152 65535 | sort) <(ss -tan 2>/dev/null | awk 'NR>1 {print $4}' | awk -F: '{print $NF}' | sort -u) | shuf -n 1)
WORK1=$(mktemp -d -t reposix-mirror-refs-reject-w1.XXXXXX)
WORK2=$(mktemp -d -t reposix-mirror-refs-reject-w2.XXXXXX)
CACHE_DIR=$(mktemp -d -t reposix-mirror-refs-reject-cache.XXXXXX)
STDERR_CAP=$(mktemp -t reposix-mirror-refs-reject-stderr.XXXXXX)
SIM_PID=""
cleanup() {
    if [[ -n "${SIM_PID}" ]]; then kill "${SIM_PID}" 2>/dev/null || true; fi
    rm -rf "${WORK1}" "${WORK2}" "${CACHE_DIR}" "${STDERR_CAP}"
}
trap cleanup EXIT

cargo build -p reposix-sim -p reposix-cli --quiet
SIM_BIN="${REPO_ROOT}/target/debug/reposix-sim"
CLI_BIN="${REPO_ROOT}/target/debug/reposix"
"${SIM_BIN}" --bind "127.0.0.1:${PORT}" --ephemeral &
SIM_PID=$!
sleep 1

# First successful push from WORK1 — populates refs/mirrors/*.
# H1: re-point remote.origin.url after init (REPOSIX_SIM_ORIGIN no-op
# for `reposix init` — see shell #1's H1 comment block for rationale).
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
"${CLI_BIN}" init "sim::demo" "${WORK1}" > /dev/null 2>&1 || true
git -C "${WORK1}" config remote.origin.url "reposix::http://127.0.0.1:${PORT}/projects/demo"
( cd "${WORK1}" && git fetch -q origin && git checkout origin/main -B main -q && \
  echo "" >> issues/0001.md && git add . && git commit -q -m "first push" && \
  REPOSIX_CACHE_DIR="${CACHE_DIR}" \
  git push -q origin main )

sleep 2  # ensure non-zero "(N minutes ago)" math, even if N=0

# Second push from WORK2 against a STALE prior — conflict-reject path.
# WORK2's local clone never sees WORK1's push; pushing produces a
# version mismatch detected by handle_export's existing conflict logic.
# H1: same re-point dance for WORK2.
REPOSIX_CACHE_DIR="${CACHE_DIR}" \
"${CLI_BIN}" init "sim::demo" "${WORK2}" > /dev/null 2>&1 || true
git -C "${WORK2}" config remote.origin.url "reposix::http://127.0.0.1:${PORT}/projects/demo"
( cd "${WORK2}" && git fetch -q origin && git checkout origin/main -B main -q && \
  # WORK2 is now stale — the sim has advanced one version via WORK1's push.
  # Edit the same file WORK1 just modified to trigger conflict.
  echo "stale-prior" >> issues/0001.md && git add . && \
  git commit -q -m "stale push" && \
  REPOSIX_CACHE_DIR="${CACHE_DIR}" \
  git push origin main 2> "${STDERR_CAP}" || true )

grep -q "refs/mirrors/sim-synced-at" "${STDERR_CAP}" \
    || { echo "FAIL: reject stderr missing refs/mirrors/sim-synced-at citation" >&2; cat "${STDERR_CAP}" >&2; exit 1; }
grep -qE "[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}.*Z" "${STDERR_CAP}" \
    || { echo "FAIL: reject stderr missing RFC3339 timestamp" >&2; cat "${STDERR_CAP}" >&2; exit 1; }
grep -qE "[0-9]+ minutes ago" "${STDERR_CAP}" \
    || { echo "FAIL: reject stderr missing '(N minutes ago)' rendering" >&2; cat "${STDERR_CAP}" >&2; exit 1; }

echo "PASS: conflict-reject hint cites refs/mirrors/sim-synced-at with RFC3339 + (N minutes ago)"
exit 0
