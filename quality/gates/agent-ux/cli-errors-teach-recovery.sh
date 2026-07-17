#!/usr/bin/env bash
# quality/gates/agent-ux/cli-errors-teach-recovery.sh — agent-ux verifier for
# catalog row `agent-ux/cli-errors-teach-recovery` (Phase 120 / P120).
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/cli-errors-teach-recovery
# CADENCE:     on-demand (does NOT self-block intermediate pushes while P120 impl lands)
# INVARIANT:   every enumerated `reposix` CLI subcommand error meets the 3-part
#              Rust-compiler-grade bar (teach the fix / name the alternative /
#              copy-paste recovery). Three legs, all reality-first:
#                (a) the continuous-regression nextest suite passes;
#                (b) a REAL `reposix` error path emits Fix:/Recovery: on stderr;
#                (c) teach_scan.py finds no un-dispositioned bail!/anyhow! block
#                    (single- or multi-line) over the CLI scope.
#
# Leaf isolation: leg (b) runs `reposix init` in a throwaway /tmp dir with the
# `cd` in the SAME invocation (never the shared repo).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# (a) continuous-regression integration suite (also builds the `reposix` binary).
# `cargo test` (not nextest) matches the existing agent-ux gate precedent and
# runs whether or not cargo-nextest is installed locally.
cargo test -p reposix-cli --test errors_teach_recovery -- --nocapture 2>&1 | tail -30

# (b) exercise a REAL CLI error path end-to-end. A malformed spec (`foo` with no
#     `::`) at a FRESH path reaches translate_spec_to_url and is retrofitted to
#     teach() in W1 — so the emitted stderr carries Fix: + an indented Recovery:.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
BIN="${REPO_ROOT}/target/debug/reposix"
if [[ ! -x "${BIN}" ]]; then
    cargo build -p reposix-cli --bin reposix 2>&1 | tail -5
fi
STDERR="$(cd "${TMP}" && "${BIN}" init foo "${TMP}/fresh-clone" 2>&1 1>/dev/null || true)"
if ! grep -q 'Fix:' <<<"${STDERR}"; then
    echo "FAIL: a malformed-spec 'reposix init' error lacks a 'Fix:' teaching line" >&2
    echo "----- captured stderr -----" >&2
    echo "${STDERR}" >&2
    echo "Fix: route translate_spec_to_url's error arms through reposix_core::errmsg::teach" >&2
    exit 1
fi
if ! grep -Eq 'Recovery:' <<<"${STDERR}"; then
    echo "FAIL: the 'reposix init' error teaches no copy-paste 'Recovery:' line" >&2
    echo "${STDERR}" >&2
    exit 1
fi

# (c) source-scan the CLI surface — no un-dispositioned bail!/anyhow! block.
python3 "${REPO_ROOT}/quality/gates/agent-ux/teach_scan.py" --scope cli

echo "PASS: reposix-cli subcommand errors teach fix + alternative + recovery"
exit 0
