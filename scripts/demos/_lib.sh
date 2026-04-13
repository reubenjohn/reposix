#!/usr/bin/env bash
# scripts/demos/_lib.sh — shared helpers for the reposix demo suite.
#
# Sourced by every Tier 1 demo (and by full.sh). The caller is expected to
# have already set `set -euo pipefail` before sourcing this file.
#
# Exposed verbs:
#   section "[N/M] <description>"      — print a visually separated banner.
#   wait_for_url <url> <timeout_sec>   — curl readiness poll.
#   wait_for_mount <path> <timeout_sec>— ensure the mount renders *.md files.
#   setup_sim [<db-path>] [<seed-file>]— spawn reposix-sim, export SIM_PID/
#                                        SIM_DB/SIM_URL/SIM_BIND. Waits
#                                        for /healthz.
#   setup_mount <path> [--project X]   — spawn reposix-fuse, export
#                                        MOUNT_PID/MOUNT_PATH.
#   cleanup_trap                       — idempotent EXIT-trap installer.
#   require <cmd>                      — precheck, exit 2 if missing.
#
# Conventions:
#   - Release binaries are expected on $PATH (the demo scripts do the
#     cargo build themselves before recording).
#   - All temp state lives under /tmp so the trap can rm -rf it.
#   - The trap is idempotent: double-sourcing or re-running is safe.

# Guard against double-source.
if [[ -n "${_REPOSIX_LIB_SOURCED:-}" ]]; then
    return 0
fi
_REPOSIX_LIB_SOURCED=1

# -------------------------------------------------------------- defaults
: "${SIM_BIND:=127.0.0.1:7878}"
: "${SIM_URL:=http://${SIM_BIND}}"
: "${SIM_DB:=/tmp/reposix-demo-sim.db}"
: "${DEMO_SEED:=crates/reposix-sim/fixtures/seed.json}"

# Track child PIDs + mount paths so cleanup_trap can reap them.
_REPOSIX_SIM_PIDS=()
_REPOSIX_MOUNT_PATHS=()
_REPOSIX_TMP_PATHS=()

# -------------------------------------------------------------- section
section() {
    local title="$1"
    echo
    echo "================================================================"
    echo "  $title"
    echo "================================================================"
    # Small beat so `script(1)` recordings don't collapse banners.
    sleep 0.25
}

# -------------------------------------------------------------- require
require() {
    local cmd="$1"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "ERROR: required command '$cmd' not found on PATH" >&2
        echo "       (if this is a reposix binary, run: cargo build --release --workspace --bins)" >&2
        exit 2
    fi
}

# -------------------------------------------------------------- waits
wait_for_url() {
    local url="$1" timeout="${2:-10}"
    local deadline=$((SECONDS + timeout))
    while ((SECONDS < deadline)); do
        if curl -sf "$url" >/dev/null 2>&1; then
            return 0
        fi
        sleep 0.1
    done
    echo "ERROR: timeout waiting for $url" >&2
    return 1
}

wait_for_mount() {
    local path="$1" timeout="${2:-10}"
    local deadline=$((SECONDS + timeout))
    while ((SECONDS < deadline)); do
        if ls "$path" 2>/dev/null | grep -q '\.md$'; then
            return 0
        fi
        sleep 0.1
    done
    echo "ERROR: timeout waiting for FUSE mount at $path" >&2
    return 1
}

# -------------------------------------------------------------- setup_sim
# Usage: setup_sim [<db-path>] [<seed-file>]
# Exports: SIM_PID, SIM_DB, SIM_URL.
setup_sim() {
    local db="${1:-$SIM_DB}"
    local seed="${2:-$DEMO_SEED}"
    SIM_DB="$db"
    rm -f "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm"
    _REPOSIX_TMP_PATHS+=("$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm")
    local log="/tmp/reposix-demo-sim.log"
    reposix-sim --bind "$SIM_BIND" --db "$SIM_DB" --seed-file "$seed" \
        >"$log" 2>&1 &
    SIM_PID=$!
    _REPOSIX_SIM_PIDS+=("$SIM_PID")
    if ! wait_for_url "${SIM_URL}/healthz" 10; then
        echo "----- sim log -----" >&2
        tail -20 "$log" >&2 || true
        return 1
    fi
    export SIM_PID SIM_DB SIM_URL SIM_BIND
}

# -------------------------------------------------------------- setup_mount
# Usage: setup_mount <path> [--project <name>] [--allowed-origins <csv>]
# Exports: MOUNT_PID, MOUNT_PATH.
setup_mount() {
    local mount_path="$1"; shift
    local project="demo"
    local allowed=""
    while (($# > 0)); do
        case "$1" in
            --project)          project="$2"; shift 2 ;;
            --allowed-origins)  allowed="$2"; shift 2 ;;
            *) echo "setup_mount: unknown arg '$1'" >&2; return 1 ;;
        esac
    done
    mkdir -p "$mount_path"
    _REPOSIX_MOUNT_PATHS+=("$mount_path")
    _REPOSIX_TMP_PATHS+=("$mount_path")
    local log="/tmp/reposix-demo-fuse-$(basename "$mount_path").log"
    if [[ -n "$allowed" ]]; then
        REPOSIX_ALLOWED_ORIGINS="$allowed" \
            reposix-fuse "$mount_path" --backend "$SIM_URL" --project "$project" \
            >"$log" 2>&1 &
    else
        reposix-fuse "$mount_path" --backend "$SIM_URL" --project "$project" \
            >"$log" 2>&1 &
    fi
    MOUNT_PID=$!
    export MOUNT_PID MOUNT_PATH="$mount_path"
}

# -------------------------------------------------------------- cleanup_trap
# Install an idempotent EXIT trap that tears down every mount + sim + tmp
# path we've registered, in reverse order.
_reposix_cleanup() {
    local rc=$?
    set +e
    # Unmount every FUSE mount we tracked.
    local p
    for p in "${_REPOSIX_MOUNT_PATHS[@]}"; do
        fusermount3 -u "$p" 2>/dev/null
    done
    # Politely kill every child (SIGTERM, then SIGKILL if still alive).
    local pid
    for pid in "${_REPOSIX_SIM_PIDS[@]}"; do
        kill "$pid" 2>/dev/null
    done
    # Belt-and-braces: scrub any reposix-fuse/sim bound to these paths or
    # this SIM_BIND in case setup_* hasn't recorded a PID yet.
    pkill -f "reposix-fuse " 2>/dev/null
    pkill -f "reposix-sim --bind $SIM_BIND" 2>/dev/null
    sleep 0.2
    for pid in "${_REPOSIX_SIM_PIDS[@]}"; do
        kill -9 "$pid" 2>/dev/null
    done
    pkill -9 -f "reposix-fuse " 2>/dev/null
    pkill -9 -f "reposix-sim --bind $SIM_BIND" 2>/dev/null
    # Remove tmp paths.
    for p in "${_REPOSIX_TMP_PATHS[@]}"; do
        rm -rf "$p" 2>/dev/null
    done
    set -e
    exit $rc
}

cleanup_trap() {
    trap _reposix_cleanup EXIT INT TERM
}
