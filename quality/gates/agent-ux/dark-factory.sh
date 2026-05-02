#!/usr/bin/env bash
# quality/gates/agent-ux/dark-factory.sh -- agent-ux dimension dark-factory regression.
#
# DISPATCHER: routes by $1 to per-arm scripts under dark-factory/.
# Per-arm logic + assertions live in:
#   dark-factory/sim.sh             (v0.9.0; agent-ux/dark-factory-sim)
#   dark-factory/dvcs-third-arm.sh  (v0.13.0 P86; agent-ux/dvcs-third-arm)
#   dark-factory/lib.sh             (shared helpers — sim spawn, build, cleanup)
#
# This file's path is preserved so all catalog rows + CLAUDE.md
# invocations continue to resolve. The split is purely organizational
# (file-size-limits factoring per the *.sh 10k char budget).
#
# MIGRATED FROM: scripts/dark-factory-test.sh per SIMPLIFY-07 (P59).
# CATALOG ROWS:
#   sim arm           -> agent-ux/dark-factory-sim     (v0.9.0; pre-pr; mechanical)
#   dvcs-third-arm    -> agent-ux/dvcs-third-arm       (v0.13.0 P86; pre-pr; subagent-graded)
# CADENCE:       pre-pr (per CI dark-factory job; ~30s wall time per arm)
# AUDIENCE: developer / autonomous agent / quality runner
# RUNTIME_SEC: ~30 (sim arm) / ~45 (dvcs-third-arm)
# REQUIRES: cargo, git (>= 2.20 for init+config; >= 2.27 for blob:none),
#           reposix-sim, reposix, git-remote-reposix on PATH, sqlite3 (third arm).
#
# Usage:
#   bash quality/gates/agent-ux/dark-factory.sh sim          # default (v0.9.0 arm)
#   bash quality/gates/agent-ux/dark-factory.sh dvcs-third-arm  # v0.13.0 P86 arm
#   bash quality/gates/agent-ux/dark-factory.sh github       # delegates to 35-03 tests
#   bash quality/gates/agent-ux/dark-factory.sh confluence
#   bash quality/gates/agent-ux/dark-factory.sh jira

set -euo pipefail

BACKEND="${1:-sim}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

case "$BACKEND" in
    sim)
        exec bash "${SCRIPT_DIR}/dark-factory/sim.sh"
        ;;
    dvcs-third-arm)
        exec bash "${SCRIPT_DIR}/dark-factory/dvcs-third-arm.sh"
        ;;
    github|confluence|jira)
        cat >&2 <<EOF
dark-factory.sh: backend=$BACKEND requires real-backend creds and is
exercised via the gated integration tests in 35-03 (cargo test -p
reposix-cli --test agent_flow_real -- --ignored). This shell wrapper only
runs the sim and dvcs-third-arm paths. Skipping.
EOF
        exit 0
        ;;
    *)
        cat >&2 <<EOF
dark-factory.sh: unknown backend '$BACKEND'.
Usage: dark-factory.sh {sim|dvcs-third-arm|github|confluence|jira}
EOF
        exit 2
        ;;
esac
