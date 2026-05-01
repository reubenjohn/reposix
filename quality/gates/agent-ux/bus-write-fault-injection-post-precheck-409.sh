#!/usr/bin/env bash
# quality/gates/agent-ux/bus-write-fault-injection-post-precheck-409.sh — agent-ux
# verifier for catalog row `agent-ux/bus-write-fault-injection-post-precheck-409`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-fault-injection-post-precheck-409
# CADENCE:     pre-pr (~10s wall time)
# INVARIANT:   Fault scenario (c) — confluence 409 after
#              PRECHECK B passed. Helper exits non-zero;
#              NO mirror push; error names the failing record
#              id (D-09 / Pitfall 3 documented behavior); NO
#              helper_push_accepted row; mirror baseline
#              preserved.
#
# Status until P83-02 T04: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_write_post_precheck_409 \
    bus_write_post_precheck_conflict_409_no_mirror_push \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: fault-injection (c) post-precheck-409 produces correct end-state"
exit 0
