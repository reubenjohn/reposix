#!/usr/bin/env bash
# quality/gates/perf/latency-bench.sh -- perf dimension latency benchmark (dispatcher).
#
# MIGRATED FROM: scripts/latency-bench.sh per SIMPLIFY-11 (P59) -- file move only.
# CATALOG ROW:   quality/catalogs/perf-targets.json -> perf/latency-bench (WAIVED until 2026-07-26)
# CADENCE:       weekly (per .github/workflows/bench-latency-cron.yml; ~5min wall time)
# STATUS:        v0.12.0 file-relocate stub; full gate logic deferred to v0.12.1 via MIGRATE-03.
#                The cross-check that compares bench output against headline copy in
#                docs/index.md + README.md is the v0.12.1 deliverable.
#
# FACTORED layout (file-size-limits gate — shell budget 10k):
#   latency-bench/lib.sh           -- timing helpers, fmt_ms, count_blob_materializations
#   latency-bench/sim.sh           -- sim block (default; always runs)
#   latency-bench/github.sh        -- github block (skipped if GITHUB_TOKEN unset)
#   latency-bench/confluence.sh    -- confluence block (skipped if Atlassian bundle unset)
#   latency-bench/jira.sh          -- jira block (skipped if JIRA bundle unset)
#   latency-bench/emit-markdown.sh -- formats collected timings as docs/benchmarks/latency.md
#
# Predecessor preserved as scripts/latency-bench.sh shim per OP-5 reversibility.
#
# AUDIENCE: developer / sales asset author
# RUNTIME_SEC: ~15s sim-only, ~60s with all three real-backend bundles.
# REQUIRES: cargo, git, jq, sqlite3, curl, reposix-sim, reposix on PATH (or built in target/).
#
# Spawns reposix-sim --ephemeral, runs the v0.9.0 golden path against it,
# and emits per-step latency rows in Markdown table format. When real-backend
# credential bundles are present in env, additionally probes the matching
# real backend (github / confluence / jira) and stamps its column.
#
# Each timed step is taken as the median of 3 samples (network jitter on
# real backends is the dominant flake source — Phase 54 plan §"Risk areas").
#
# Soft thresholds:
#   sim cold init       < 500ms   (regression-flagged via WARN, not exit)
#   real-backend step   < 3s      (regression-flagged via WARN, not exit)
#
# Real-backend columns are populated when the relevant env vars are
# present; otherwise they're empty / "n/a". Default is sim-only.
#
# Output: a fully-formatted Markdown file at docs/benchmarks/latency.md
# (running this script is the regenerator).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="${SCRIPT_DIR}/latency-bench"
# Workspace root is three levels up from quality/gates/perf/ (was one
# level up from scripts/ in the predecessor).
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
OUT="${WORKSPACE_ROOT}/docs/benchmarks/latency.md"

SIM_BIND="127.0.0.1:7780"
SIM_URL="http://${SIM_BIND}"
RUN_DIR="/tmp/v090-latency-$$"
SIM_DB="${RUN_DIR}/sim.db"
REPO="${RUN_DIR}/repo"
# Pin the cache directory so the script can locate `cache.db` deterministically
# for the blob-materialization counter (matches resolve_cache_path in
# crates/reposix-cache/src/path.rs: <root>/reposix/<backend>-<project>.git).
CACHE_ROOT="${RUN_DIR}/cache"
mkdir -p "$RUN_DIR" "$CACHE_ROOT"
export REPOSIX_CACHE_DIR="$CACHE_ROOT"

# Default allowlist — sim only. Per-backend blocks below extend this for
# their REST probes via a local override (the helper-spawned cache reads
# the env var fresh on each invocation).
export REPOSIX_ALLOWED_ORIGINS="${SIM_URL}"

cleanup() {
    local rc=$?
    if [[ -n "${SIM_PID:-}" ]]; then
        kill "$SIM_PID" 2>/dev/null || true
        wait "$SIM_PID" 2>/dev/null || true
    fi
    rm -rf "$RUN_DIR"
    exit "$rc"
}
trap cleanup EXIT

# Shared helpers (now_ms, median3_step, count_blob_materializations, fmt_ms, fmt_ms_n).
# shellcheck source=latency-bench/lib.sh
source "${LIB_DIR}/lib.sh"

echo "latency-bench: ensuring binaries are fresh..." >&2
(cd "$WORKSPACE_ROOT" && cargo build --workspace --bins -q 2>&1 | tail -3)
BIN_DIR="${WORKSPACE_ROOT}/target/debug"
export PATH="${BIN_DIR}:${PATH}"

echo "latency-bench: spawning reposix-sim on $SIM_BIND" >&2
SEED="${WORKSPACE_ROOT}/crates/reposix-sim/fixtures/seed.json"
"${BIN_DIR}/reposix-sim" --bind "$SIM_BIND" --db "$SIM_DB" --ephemeral --seed-file "$SEED" &
SIM_PID=$!
for _ in $(seq 1 50); do
    if curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1; then
        break
    fi
    sleep 0.1
done

# Backend probe blocks. Each populates a fixed set of <prefix>_<step>_MS vars
# (or leaves them empty when the backend is gated off by missing creds).
# shellcheck source=latency-bench/sim.sh
source "${LIB_DIR}/sim.sh"
# shellcheck source=latency-bench/github.sh
source "${LIB_DIR}/github.sh"
# shellcheck source=latency-bench/confluence.sh
source "${LIB_DIR}/confluence.sh"
# shellcheck source=latency-bench/jira.sh
source "${LIB_DIR}/jira.sh"

# Format the collected timings into docs/benchmarks/latency.md.
# shellcheck source=latency-bench/emit-markdown.sh
source "${LIB_DIR}/emit-markdown.sh"

exit 0
