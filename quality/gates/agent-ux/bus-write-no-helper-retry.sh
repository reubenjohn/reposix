#!/usr/bin/env bash
# quality/gates/agent-ux/bus-write-no-helper-retry.sh — agent-ux
# verifier for catalog row `agent-ux/bus-write-no-helper-retry`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-no-helper-retry
# CADENCE:     pre-pr (~2s wall time)
# INVARIANT:   crates/reposix-remote/src/bus_handler.rs contains
#              NO retry constructs adjacent to push_mirror calls
#              (no `for _ in 0..` loops, no `loop {`, no
#              `tokio::time::sleep`, no `--force-with-lease`,
#              no `--force` in args).
#
# Status until P83-01 T06: FAIL (until T04 lands push_mirror
# AND the grep confirms no retry constructs).
#
# Per Q3.6 RATIFIED 2026-04-30: surface, no helper-side retry.
# User retries the whole push.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

FILE="crates/reposix-remote/src/bus_handler.rs"
if [[ ! -f "${FILE}" ]]; then
    echo "FAIL: ${FILE} does not exist" >&2
    exit 1
fi

# Grep for retry-shaped patterns. Any hit fails RED.
# Filter out comments first to avoid false positives on doc text.
FILTERED=$(grep -v '^\s*//' "${FILE}" || true)

if echo "${FILTERED}" | grep -qE 'for[[:space:]]+_[[:space:]]+in[[:space:]]+0\.\.'; then
    echo "FAIL: retry construct found (for _ in 0..) — Q3.6 RATIFIED no-retry violated" >&2
    exit 1
fi
if echo "${FILTERED}" | grep -qE '^\s*loop[[:space:]]*\{'; then
    echo "FAIL: bare loop construct found — Q3.6 RATIFIED no-retry violated" >&2
    exit 1
fi
if echo "${FILTERED}" | grep -qE 'tokio::time::sleep'; then
    echo "FAIL: tokio::time::sleep found — retry-via-sleep is Q3.6 violated" >&2
    exit 1
fi
if echo "${FILTERED}" | grep -qE -- '--force-with-lease|--force[^-]'; then
    echo "FAIL: --force / --force-with-lease found — D-08 RATIFIED plain push violated" >&2
    exit 1
fi

# Confirm push_mirror exists (T04 must land before T06 catalog flip).
if ! grep -q 'fn push_mirror' "${FILE}"; then
    echo "FAIL: fn push_mirror not found in ${FILE} (T04 not yet shipped)" >&2
    exit 1
fi

echo "PASS: bus_handler.rs contains no retry constructs adjacent to push_mirror"
exit 0
