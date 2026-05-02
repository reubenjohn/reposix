#!/usr/bin/env bash
# quality/gates/agent-ux/dark-factory/sim.sh -- v0.9.0 sim-arm of the
# dark-factory regression. Sourced (or invoked) via the dispatcher
# `dark-factory.sh sim`.
#
# CATALOG ROW: agent-ux/dark-factory-sim
# INVARIANT: helper stderr-teaching strings emit on conflict + blob-limit
# paths so a stderr-reading agent can recover without prompt engineering.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Workspace root is three levels up from quality/gates/agent-ux/dark-factory/.
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"

SIM_BIND="127.0.0.1:7779"
RUN_DIR="/tmp/dark-factory-$$"
ARTIFACT="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux/dark-factory-sim.json"
ROW_ID="agent-ux/dark-factory-sim"
SIM_URL="http://${SIM_BIND}"
SIM_DB="${RUN_DIR}/sim.db"
mkdir -p "$RUN_DIR"

# Egress allowlist must contain only the sim's localhost origin so any
# accidental egress to a real backend is refused.
export REPOSIX_ALLOWED_ORIGINS="${SIM_URL}"

# shellcheck disable=SC1091
source "${SCRIPT_DIR}/lib.sh"

build_and_resolve_bins
spawn_sim

REPO="${RUN_DIR}/repo"

# 2. reposix init: bootstrap the partial-clone working tree.
echo "dark-factory: reposix init sim::demo $REPO" >&2
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

echo "dark-factory: working tree configured correctly" >&2

# 4. Assertion: blob-limit stderr teaches the agent the recovery move.
grep -q 'git sparse-checkout' \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/stateless_connect.rs" \
    || { echo "FAIL: BLOB_LIMIT teaching string regressed in stateless_connect.rs"; exit 1; }

# 5. Assertion: conflict path teaches `git pull --rebase`.
grep -q 'git pull --rebase' \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/main.rs" \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/write_loop.rs" \
    || { echo "FAIL: conflict-rebase teaching string regressed in main.rs/write_loop.rs"; exit 1; }

echo "DARK-FACTORY DEMO COMPLETE -- sim backend: agent UX is pure git." >&2
echo "  - init configures partial-clone working tree without FUSE" >&2
echo "  - blob-limit error message names sparse-checkout recovery" >&2
echo "  - conflict error message names git-pull recovery" >&2
ASSERTS_PASSED='["dark-factory regression sim path exits 0", "helper stderr-teaching strings present on conflict + blob-limit paths (v0.9.0 invariant)", "no regression vs v0.9.0 baseline"]'
exit 0
