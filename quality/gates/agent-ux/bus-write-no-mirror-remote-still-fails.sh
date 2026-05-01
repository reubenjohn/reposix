#!/usr/bin/env bash
# quality/gates/agent-ux/bus-write-no-mirror-remote-still-fails.sh — agent-ux
# verifier for catalog row `agent-ux/bus-write-no-mirror-remote-still-fails`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-no-mirror-remote-still-fails
# CADENCE:     pre-pr (~5s wall time)
# INVARIANT:   bus URL with no local `git remote` for the mirror
#              fails with verbatim Q3.5 hint after P83 lands;
#              regression check that P83's write fan-out doesn't
#              accidentally bypass P82's STEP 0 check.
#
# Status until P83-01 T06: FAIL.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

cargo test -p reposix-remote --test bus_write_no_mirror_remote \
    bus_write_no_mirror_remote_emits_q35_hint \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: P82 no-mirror-remote hint preserved end-to-end after P83 write fan-out"
exit 0
