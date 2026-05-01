#!/usr/bin/env bash
# quality/gates/agent-ux/sync-reconcile-subcommand.sh — agent-ux
# verifier for catalog row `agent-ux/sync-reconcile-subcommand`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/sync-reconcile-subcommand
# CADENCE:     pre-pr (~30s wall time)
# INVARIANT:   `reposix sync --reconcile --help` exits 0 AND the
#              integration smoke test
#              `reposix-cli/tests/sync.rs::sync_reconcile_advances_cursor`
#              passes (cache last_fetched_at advances after running the
#              subcommand against a sim).
#
# Status until P81-01 T04: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# Build the CLI binary so --help exits cleanly without running the
# subcommand body. Quiet build to keep verifier output focused.
LOG="$(mktemp)"
trap 'rm -f "${LOG}"' EXIT
if ! cargo build -p reposix-cli --quiet > "${LOG}" 2>&1; then
    echo "FAIL: cargo build -p reposix-cli failed" >&2
    tail -40 "${LOG}" >&2
    exit 1
fi
CLI_BIN="${REPO_ROOT}/target/debug/reposix"
if ! "${CLI_BIN}" sync --reconcile --help > /dev/null 2>>"${LOG}"; then
    echo "FAIL: reposix sync --reconcile --help nonzero exit" >&2
    tail -40 "${LOG}" >&2
    exit 1
fi

if ! cargo test -p reposix-cli --test sync sync_reconcile_advances_cursor \
        --quiet -- --nocapture > "${LOG}" 2>&1; then
    echo "FAIL: sync_reconcile_advances_cursor did not pass" >&2
    tail -40 "${LOG}" >&2
    exit 1
fi

echo "PASS: reposix sync --reconcile help+subcommand smoke green"
exit 0
