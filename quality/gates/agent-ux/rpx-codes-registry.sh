#!/usr/bin/env bash
# quality/gates/agent-ux/rpx-codes-registry.sh — agent-ux verifier for catalog row
# `agent-ux/rpx-codes-registry` (Phase 121 / P121: RPX error codes + `reposix explain`).
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/rpx-codes-registry
# CADENCE:     on-demand (does NOT self-block intermediate pushes while P121 impl lands)
# INVARIANT:   the RPX error-code namespace is coherent + queryable. Three legs,
#              all reality-first:
#                (a) the explain integration suite passes (every REGISTRY code
#                    prints a non-empty cause+fix+recovery; --list enumerates;
#                    unknown-code teaches; rustc-parity-of-shape);
#                (b) a REAL `reposix explain <code>` prints a code header + Fix: +
#                    Recovery: on stdout, and a REAL `reposix explain RPX-9999`
#                    (unknown) teaches `reposix explain --list` + exits non-zero;
#                (c) rpx_registry_check.py finds every emitted `.code`/`teach_coded`/
#                    `[RPX-xxxx]`/`*_FMT` code registered + entries non-empty + codes
#                    unique + `RPX-\d{4}`, and every user-facing teaching site coded.
#
# CATALOG-FIRST: this gate is committed in the phase's FIRST commit, before the
# `reposix explain` subcommand + the registry exist. Run over the pre-impl tree it
# EXITS NON-ZERO (legs a/b need a binary that does not yet build the `explain`
# target; leg c flags every not-yet-coded teaching site). That is the honest
# NOT-VERIFIED state; the catalog row flips PASS only after a green run in W4.
#
# Leaf isolation: leg (b) runs the binary from a throwaway /tmp dir with the `cd`
# in the SAME invocation (the explain path is hermetic — no tree, no network — but
# we cd out of the shared repo anyway, matching the dark-factory precedent).
# Build memory budget: exactly ONE cargo invocation, FOREGROUND (crates/CLAUDE.md).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

KNOWN_CODE="RPX-0201"   # a real CLI code (cache-build) — present once W3 mints it.
UNKNOWN_CODE="RPX-9999" # never registered — the unknown-code teaching path.

# (c) FIRST — the source-grep checker needs no build, so it gives the fastest
#     signal (registry integrity + every-teaching-site-coded) before we spend a
#     compile. Self-test the checker's own logic, then run it over the tree.
python3 "${REPO_ROOT}/quality/gates/agent-ux/rpx_registry_check.py" --self-test
python3 "${REPO_ROOT}/quality/gates/agent-ux/rpx_registry_check.py"

# (a) the explain integration suite (also builds the `reposix` binary). `cargo
#     test` (not nextest) matches the existing agent-ux gate precedent and runs
#     whether or not cargo-nextest is installed locally. ONE FOREGROUND cargo.
cargo test -p reposix-cli --test explain -- --nocapture 2>&1 | tail -30

BIN="${REPO_ROOT}/target/debug/reposix"
if [[ ! -x "${BIN}" ]]; then
    echo "FAIL: reposix binary not built at ${BIN} after the explain suite" >&2
    echo "Fix: run \`cargo build -p reposix-cli --bin reposix\` (one foreground cargo)" >&2
    exit 1
fi

# (b) exercise the REAL explain paths end-to-end from a throwaway /tmp dir.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

KNOWN_OUT="$(cd "${TMP}" && "${BIN}" explain "${KNOWN_CODE}" 2>&1 || true)"
if ! grep -q "${KNOWN_CODE}" <<<"${KNOWN_OUT}"; then
    echo "FAIL: \`reposix explain ${KNOWN_CODE}\` output has no ${KNOWN_CODE} code header" >&2
    echo "----- captured output -----" >&2
    echo "${KNOWN_OUT}" >&2
    exit 1
fi
if ! grep -q 'Fix:' <<<"${KNOWN_OUT}"; then
    echo "FAIL: \`reposix explain ${KNOWN_CODE}\` teaches no 'Fix:' line" >&2
    echo "${KNOWN_OUT}" >&2
    exit 1
fi
if ! grep -Eq 'Recovery:' <<<"${KNOWN_OUT}"; then
    echo "FAIL: \`reposix explain ${KNOWN_CODE}\` gives no copy-paste 'Recovery:' line" >&2
    echo "${KNOWN_OUT}" >&2
    exit 1
fi

# unknown code: teaches how to list, exits non-zero, does not panic.
UNKNOWN_OUT="$(cd "${TMP}" && "${BIN}" explain "${UNKNOWN_CODE}" 2>&1 || true)"
UNKNOWN_STATUS="$(cd "${TMP}" && "${BIN}" explain "${UNKNOWN_CODE}" >/dev/null 2>&1; echo $?)"
if [[ "${UNKNOWN_STATUS}" == "0" ]]; then
    echo "FAIL: \`reposix explain ${UNKNOWN_CODE}\` (unknown) exited 0 — must fail non-zero" >&2
    exit 1
fi
if grep -qi 'panicked' <<<"${UNKNOWN_OUT}"; then
    echo "FAIL: \`reposix explain ${UNKNOWN_CODE}\` PANICKED instead of teaching" >&2
    echo "${UNKNOWN_OUT}" >&2
    exit 1
fi
if ! grep -q 'reposix explain --list' <<<"${UNKNOWN_OUT}"; then
    echo "FAIL: \`reposix explain ${UNKNOWN_CODE}\` does not teach \`reposix explain --list\`" >&2
    echo "${UNKNOWN_OUT}" >&2
    exit 1
fi

echo "PASS: RPX registry coherent, every teaching site coded, \`reposix explain\` teaches"
exit 0
