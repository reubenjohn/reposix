#!/usr/bin/env bash
# quality/gates/agent-ux/helper-errors-teach-recovery.sh — agent-ux verifier for
# catalog row `agent-ux/helper-errors-teach-recovery` (Phase 120 / P120).
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/helper-errors-teach-recovery
# CADENCE:     on-demand (does NOT self-block intermediate pushes while P120 impl lands)
# INVARIANT:   every enumerated `reposix-remote` (git-remote-reposix) helper error
#              is dispositioned (RETROFIT through the shared builder, or EXEMPT-
#              marked for a git-protocol-contract / internal site) and meets the
#              3-part bar where retrofit. Three legs, all reality-first:
#                (a) the continuous-regression nextest suite passes;
#                (b) a REAL malformed bus URL, driven through the built
#                    `git-remote-reposix` binary, emits Fix:/Recovery: on stderr;
#                (c) teach_scan.py finds no un-dispositioned bail!/anyhow! block
#                    (single- or multi-line) over the helper scope.
#
# The helper parses argv[2] (the remote URL) BEFORE any git/network context, so
# leg (b) is fully hermetic — no repo/seed needed.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# (a) continuous-regression integration suite (also builds the helper binary).
# `cargo test` (not nextest) matches the existing agent-ux gate precedent and
# runs whether or not cargo-nextest is installed locally.
cargo test -p reposix-remote --test errors_teach_recovery -- --nocapture 2>&1 | tail -30

# (b) exercise a REAL helper error path: a malformed bus URL (dropped `+`-form,
#     `base.contains('+')`) is rejected by bus_url::parse BEFORE any base-parse
#     or network; W4 routes all six reject arms through malformed_bus_url_error
#     so the emitted stderr carries Fix: + Recovery:.
BIN="${REPO_ROOT}/target/debug/git-remote-reposix"
if [[ ! -x "${BIN}" ]]; then
    cargo build -p reposix-remote --bin git-remote-reposix 2>&1 | tail -5
fi
STDERR="$("${BIN}" origin 'reposix::sim+mirror' 2>&1 1>/dev/null </dev/null || true)"
if ! grep -q 'Fix:' <<<"${STDERR}"; then
    echo "FAIL: a malformed bus URL error lacks a 'Fix:' teaching line" >&2
    echo "----- captured stderr -----" >&2
    echo "${STDERR}" >&2
    echo "Fix: route bus_url.rs reject arms through reposix_core::errmsg via malformed_bus_url_error" >&2
    exit 1
fi
if ! grep -Eq 'Recovery:' <<<"${STDERR}"; then
    echo "FAIL: the malformed bus URL error teaches no copy-paste 'Recovery:' line" >&2
    echo "${STDERR}" >&2
    exit 1
fi

# (c) source-scan the helper surface — no un-dispositioned bail!/anyhow! block.
python3 "${REPO_ROOT}/quality/gates/agent-ux/teach_scan.py" --scope helper

echo "PASS: git-remote-reposix helper errors teach fix + alternative + recovery"
exit 0
