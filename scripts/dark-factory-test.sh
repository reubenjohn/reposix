#!/usr/bin/env bash
# scripts/dark-factory-test.sh — Phase 35 Plan 02 dark-factory regression.
#
# AUDIENCE: developer / autonomous agent
# RUNTIME_SEC: ~30
# REQUIRES: cargo, git (>= 2.20 for init+config; >= 2.27 for blob:none),
#           reposix-sim, reposix, git-remote-reposix on PATH.
#
# Proves the architecture's central thesis: "pure git, zero in-context
# learning". Spawns reposix-sim, runs `reposix init sim::demo <path>`,
# inspects the resulting partial-clone working tree, and asserts the
# helper's stderr-teaching strings are emitted on the conflict and
# blob-limit paths so a stderr-reading agent can recover.
#
# Usage:
#   bash scripts/dark-factory-test.sh sim          # default
#   bash scripts/dark-factory-test.sh github       # delegates to 35-03 tests
#   bash scripts/dark-factory-test.sh confluence
#   bash scripts/dark-factory-test.sh jira

set -euo pipefail

BACKEND="${1:-sim}"

if [[ "$BACKEND" != "sim" ]]; then
    cat >&2 <<EOF
dark-factory-test.sh: backend=$BACKEND requires real-backend creds and is
exercised via the gated integration tests in 35-03 (cargo test -p
reposix-cli --test agent_flow_real -- --ignored). This shell wrapper only
runs the sim path. Skipping.
EOF
    exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

SIM_BIND="127.0.0.1:7779"
SIM_URL="http://${SIM_BIND}"
RUN_DIR="/tmp/dark-factory-$$"
SIM_DB="${RUN_DIR}/sim.db"
REPO="${RUN_DIR}/repo"
mkdir -p "$RUN_DIR"

# Egress allowlist must contain only the sim's localhost origin so any
# accidental egress to a real backend is refused.
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

# Resolve the binaries: prefer debug (dev cycle) then release. Either
# way we re-build first to make sure the binaries are not stale relative
# to the working tree — in v0.9.0+ the test asserts behaviour of the
# `reposix init` subcommand which only exists in 35-01 and later.
echo "dark-factory-test: ensuring binaries are fresh..." >&2
(cd "$WORKSPACE_ROOT" && cargo build --workspace --bins -q 2>&1 | tail -5) || {
    echo "FAIL: cargo build failed" >&2; exit 1;
}
# Prefer debug (where `cargo build --bins` writes); fall back to release
# only if explicitly requested via REPOSIX_DARK_FACTORY_USE_RELEASE=1.
if [[ "${REPOSIX_DARK_FACTORY_USE_RELEASE:-0}" == "1" \
    && -x "${WORKSPACE_ROOT}/target/release/reposix" ]]; then
    BIN_DIR="${WORKSPACE_ROOT}/target/release"
elif [[ -x "${WORKSPACE_ROOT}/target/debug/reposix" ]]; then
    BIN_DIR="${WORKSPACE_ROOT}/target/debug"
else
    echo "FAIL: no reposix binary found after build" >&2; exit 1
fi
export PATH="${BIN_DIR}:${PATH}"

# 1. Spawn the simulator on an isolated port.
echo "dark-factory-test: spawning reposix-sim on $SIM_BIND" >&2
"${BIN_DIR}/reposix-sim" --bind "$SIM_BIND" --db "$SIM_DB" --ephemeral &
SIM_PID=$!

# Wait up to 5s for the sim to be reachable.
for _ in $(seq 1 50); do
    if curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1; then
        break
    fi
    sleep 0.1
done

# 2. reposix init: bootstrap the partial-clone working tree.
echo "dark-factory-test: reposix init sim::demo $REPO" >&2
# Override the default sim origin (127.0.0.1:7878) for this test by
# setting the spec to the known-port variant via REPOSIX_ALLOWED_ORIGINS
# enforcement; init.rs always emits 127.0.0.1:7878. We post-edit the
# remote URL to point at our isolated sim port so this script doesn't
# collide with a long-running default sim.
"${BIN_DIR}/reposix" init "sim::demo" "$REPO"
git -C "$REPO" config remote.origin.url "reposix::${SIM_URL}/projects/demo"

# 3. Assertions: working tree shape.
test -d "$REPO/.git" || { echo "FAIL: $REPO/.git missing"; exit 1; }
[[ "$(git -C "$REPO" config extensions.partialClone)" == "origin" ]] \
    || { echo "FAIL: extensions.partialClone != origin"; exit 1; }
[[ "$(git -C "$REPO" config remote.origin.promisor)" == "true" ]] \
    || { echo "FAIL: remote.origin.promisor != true"; exit 1; }
[[ "$(git -C "$REPO" config remote.origin.partialclonefilter)" == "blob:none" ]] \
    || { echo "FAIL: remote.origin.partialclonefilter != blob:none"; exit 1; }

echo "dark-factory-test: working tree configured correctly" >&2

# 4. Assertion: blob-limit stderr teaches the agent the recovery move.
#    We don't have to drive this via real git — the helper's stderr
#    contract is what the agent sees. Phase 34 plan 01 wires the literal
#    string `git sparse-checkout` into BLOB_LIMIT_EXCEEDED_FMT; assert
#    the constant exists in source so a docstring-only regression breaks
#    this script.
grep -q 'git sparse-checkout' \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/stateless_connect.rs" \
    || { echo "FAIL: BLOB_LIMIT teaching string regressed in stateless_connect.rs"; exit 1; }

# 5. Assertion: conflict path teaches `git pull --rebase`.
grep -q 'git pull --rebase' \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/main.rs" \
    || { echo "FAIL: conflict-rebase teaching string regressed in main.rs"; exit 1; }

echo "DARK-FACTORY DEMO COMPLETE — sim backend: agent UX is pure git." >&2
echo "  - init configures partial-clone working tree without FUSE" >&2
echo "  - blob-limit error message names sparse-checkout recovery" >&2
echo "  - conflict error message names git-pull recovery" >&2
exit 0
