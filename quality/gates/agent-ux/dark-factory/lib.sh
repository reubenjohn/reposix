#!/usr/bin/env bash
# quality/gates/agent-ux/dark-factory/lib.sh -- shared helpers for
# dark-factory.sh per-arm scripts (sim.sh, dvcs-third-arm.sh).
#
# Sourced by the dispatcher AFTER per-arm constants are set
# (BACKEND, SIM_BIND, RUN_DIR, ARTIFACT, ROW_ID, WORKSPACE_ROOT).
# Defines: cleanup trap, binary build/resolve, sim spawn, sim-ready probe,
# JSON-artifact writer.
#
# Not invokable directly. Per the file-size-limits factoring (hint:
# "factor into composable scripts") this lives at < 10k chars.

# --- artifact + assertion state ---------------------------------------------
EXIT_CODE=0
ASSERTS_PASSED='[]'
ASSERTS_FAILED='[]'
mkdir -p "$(dirname "$ARTIFACT")"

# Emit a FAIL line to stderr, set ASSERTS_FAILED to a single-element JSON
# array, and exit 1. Centralizes the boilerplate the per-arm scripts used
# to inline at every assertion failure point.
fail_with() {
    local desc="$1"
    local detail="${2:-}"
    if [[ -n "$detail" ]]; then
        echo "FAIL: ${desc}: ${detail}" >&2
    else
        echo "FAIL: ${desc}" >&2
    fi
    ASSERTS_FAILED='["'"${desc}"'"]'
    exit 1
}

cleanup() {
    EXIT_CODE=$?
    if [[ -n "${SIM_PID:-}" ]]; then
        kill "$SIM_PID" 2>/dev/null || true
        wait "$SIM_PID" 2>/dev/null || true
    fi
    rm -rf "$RUN_DIR"
    if [[ -n "${THIRD_ARM_CACHE_DIR:-}" ]]; then
        rm -rf "$THIRD_ARM_CACHE_DIR"
    fi
    cat > "$ARTIFACT" <<EOF
{
  "ts": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "row_id": "${ROW_ID}",
  "exit_code": $EXIT_CODE,
  "asserts_passed": ${ASSERTS_PASSED},
  "asserts_failed": ${ASSERTS_FAILED}
}
EOF
    exit "$EXIT_CODE"
}
trap cleanup EXIT

# --- binary build + PATH resolve --------------------------------------------
# Resolve the binaries: prefer debug (dev cycle) then release. Either
# way we re-build first to make sure the binaries are not stale relative
# to the working tree.
build_and_resolve_bins() {
    echo "dark-factory: ensuring binaries are fresh..." >&2
    (cd "$WORKSPACE_ROOT" && cargo build --workspace --bins -q 2>&1 | tail -5) || {
        echo "FAIL: cargo build failed" >&2; exit 1;
    }
    if [[ "${REPOSIX_DARK_FACTORY_USE_RELEASE:-0}" == "1" \
        && -x "${WORKSPACE_ROOT}/target/release/reposix" ]]; then
        BIN_DIR="${WORKSPACE_ROOT}/target/release"
    elif [[ -x "${WORKSPACE_ROOT}/target/debug/reposix" ]]; then
        BIN_DIR="${WORKSPACE_ROOT}/target/debug"
    else
        echo "FAIL: no reposix binary found after build" >&2; exit 1
    fi
    export PATH="${BIN_DIR}:${PATH}"
    export BIN_DIR
}

# --- simulator spawn --------------------------------------------------------
# Args: $1 = "seeded" or "" — seeded variant loads the canonical fixture so
# issues/0001.md has body the agent can edit (used by the dvcs-third-arm).
spawn_sim() {
    local mode="${1:-}"
    echo "dark-factory: spawning reposix-sim on $SIM_BIND" >&2
    if [[ "$mode" == "seeded" ]]; then
        "${BIN_DIR}/reposix-sim" --bind "$SIM_BIND" --db "$SIM_DB" --ephemeral \
            --seed-file "${WORKSPACE_ROOT}/crates/reposix-sim/fixtures/seed.json" &
    else
        "${BIN_DIR}/reposix-sim" --bind "$SIM_BIND" --db "$SIM_DB" --ephemeral &
    fi
    SIM_PID=$!
    export SIM_PID

    # Wait up to 5s for the sim to be reachable.
    for _ in $(seq 1 50); do
        if curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1; then
            return 0
        fi
        sleep 0.1
    done
    echo "FAIL: sim did not come up at ${SIM_URL} within 5s" >&2
    return 1
}
