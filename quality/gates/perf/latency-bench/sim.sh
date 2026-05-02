#!/usr/bin/env bash
# quality/gates/perf/latency-bench/sim.sh -- sim-backend probe block.
#
# Sourced by ../latency-bench.sh after lib.sh and after the sim has been
# spawned. Reads: BIN_DIR, RUN_DIR, REPO, SIM_URL, CACHE_ROOT.
# Writes (exports): SIM_INIT_MS, SIM_LIST_MS, SIM_GET_MS, SIM_PATCH_MS,
# SIM_CAP_MS, SIM_N, SIM_BLOBS.

SIM_PROJECT="demo"
SIM_REPO="$REPO"

# init cold — single sample (one-shot bootstrap; re-running into the same
# path would no-op git init and skew the number low).
T0=$(now_ms)
"${BIN_DIR}/reposix" init "sim::${SIM_PROJECT}" "$SIM_REPO" >/dev/null 2>&1 || true
git -C "$SIM_REPO" config remote.origin.url "reposix::${SIM_URL}/projects/${SIM_PROJECT}"
T1=$(now_ms)
SIM_INIT_MS=$((T1 - T0))
SIM_BLOBS=$(count_blob_materializations "${CACHE_ROOT}/reposix/sim-${SIM_PROJECT}.git")

# REST round-trips against the sim — 3 samples each.
SIM_LIST_BODY="$(curl -fsS "${SIM_URL}/projects/${SIM_PROJECT}/issues")"
SIM_N=$(echo "$SIM_LIST_BODY" | jq 'length' 2>/dev/null || echo "0")
SIM_LIST_MS=$(median3_step curl -fsS "${SIM_URL}/projects/${SIM_PROJECT}/issues")
SIM_GET_MS=$(median3_step curl -fsS "${SIM_URL}/projects/${SIM_PROJECT}/issues/1")

# PATCH no-op: re-write the title to whatever it already is. The sim's
# expected_version is checked, so we read it first.
SIM_TITLE=$(echo "$SIM_LIST_BODY" | jq -r '.[0].title // "latency-bench-ping"')
SIM_VERSION=$(curl -fsS "${SIM_URL}/projects/${SIM_PROJECT}/issues/1" | jq -r '.version // 1')
sim_patch() {
    curl -fsS -X PATCH \
        -H 'Content-Type: application/json' \
        -d "{\"title\":$(printf '%s' "$SIM_TITLE" | jq -Rsc .),\"expected_version\":${SIM_VERSION}}" \
        "${SIM_URL}/projects/${SIM_PROJECT}/issues/1"
}
SIM_PATCH_MS=$(median3_step sim_patch)

# Helper capabilities probe (proxy for clone bootstrap). Local-only — no
# network, identical across columns; kept as the runner-variance control.
sim_cap() {
    echo "capabilities" | "${BIN_DIR}/git-remote-reposix" \
        origin "reposix::${SIM_URL}/projects/${SIM_PROJECT}"
}
SIM_CAP_MS=$(median3_step sim_cap)

# Soft threshold warnings (non-fatal)
[[ $SIM_INIT_MS -gt 500 ]] && echo "WARN: sim cold init ${SIM_INIT_MS}ms > 500ms threshold" >&2 || true
[[ $SIM_LIST_MS -gt 500 ]] && echo "WARN: sim list ${SIM_LIST_MS}ms > 500ms threshold" >&2 || true
