#!/usr/bin/env bash
# quality/gates/agent-ux/bus-write-no-helper-retry.sh — agent-ux
# verifier for catalog row `agent-ux/bus-write-no-helper-retry`.
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/bus-write-no-helper-retry
# CADENCE:     pre-pr (~15s wall time)
# INVARIANT:   P92 SC5 upgrade -- a BEHAVIORAL assertion, not a source
#              grep: fault-inject a mirror-push failure (the
#              `common::make_failing_mirror_fixture` failing `update`
#              hook) and assert the helper's OWN process made EXACTLY
#              ONE `git push` attempt against the mirror (counted via
#              the hook's own invocation log,
#              `common::count_mirror_hook_invocations` --
#              `crates/reposix-remote/tests/bus_write_no_helper_retry.rs`).
#              The source-grep below is KEPT as a cheap pre-check (a
#              retry construct textually present would be a code smell
#              even if it happened not to fire in this scenario) but is
#              no longer the sole assertion.
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

echo "pre-check PASS: no retry-shaped tokens found in ${FILE} (cheap grep, not the sole assertion)"

# BEHAVIORAL assertion (P92 SC5): fault-inject a mirror-push failure and
# assert the helper's OWN process made EXACTLY ONE git-push attempt
# against the mirror remote, counted via the failing update hook's own
# invocation log -- not a source-grep proxy.
cargo test -p reposix-remote --test bus_write_no_helper_retry \
    bus_write_no_helper_retry_makes_exactly_one_push_attempt \
    --quiet -- --nocapture 2>&1 | tail -20

echo "PASS: bus_handler.rs contains no retry constructs adjacent to push_mirror (grep pre-check) AND the helper made exactly 1 git-push attempt against a faulted mirror (behavioral)"
exit 0
