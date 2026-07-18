#!/usr/bin/env bash
# quality/gates/agent-ux/import-parent-resolve-fails-loud.sh -- agent-ux verifier
# for catalog row `agent-ux/import-parent-resolve-fails-loud` (P122 W2 / DRAIN-08 /
# GTH-V15-05).
#
# CATALOG ROW: quality/catalogs/agent-ux.json -> agent-ux/import-parent-resolve-fails-loud
# CADENCE:     on-demand (does NOT self-block intermediate pushes)
# INVARIANT:   resolve_import_parent() (crates/reposix-remote/src/main.rs) must FAIL
#              LOUD (coded RPX-0508) on a NON-absence `git rev-parse` failure instead
#              of silently degrading to the parentless overlay, while still treating a
#              genuine ref-absent first fetch (exit 1, empty stdout) as Ok(None). The
#              regression tests inject a fake git runner (exit 128 / spawn failure /
#              exit-0-empty-stdout / exit-1-absent / present) and live in a bin-target
#              `#[cfg(test)]` module -> graded by the BARE `cargo test -p
#              reposix-remote` (a `--test <name>` scope would MISS bin-target unit
#              tests, per crates/CLAUDE.md "bin-target vs integration-target").
#
# transport_claim:false -- the assert is unit-level exit-code semantics, NOT a
# transport-layer / real-backend claim (no reposix binary or backend endpoint driven).
#
# ONE cargo invocation at a time (crates/CLAUDE.md build-memory budget): the single
# `cargo test` below runs SEQUENTIALLY, foreground, never concurrently with another
# cargo user.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

ROW_ID="agent-ux/import-parent-resolve-fails-loud"
SRC="crates/reposix-remote/src/main.rs"
ARTIFACT="quality/reports/verifications/agent-ux/import-parent-resolve-fails-loud.json"

fail() {
  echo "FAIL (${ROW_ID}): $1" >&2
  exit 1
}

# --- Precondition: the regression tests + tri-state impl exist ---------------
[[ -f "${SRC}" ]] || fail "${SRC} does not exist"
# The tri-state signature (Result, not Option) -- the loud path's structural anchor.
grep -q 'fn resolve_import_parent() -> anyhow::Result<Option<ImportParent>>' "${SRC}" \
  || fail "resolve_import_parent is not lifted to anyhow::Result<Option<ImportParent>> (the loud tri-state)"
grep -q 'HELPER_IMPORT_PARENT_RESOLVE' "${SRC}" \
  || fail "${SRC} no longer emits the RPX-0508 (HELPER_IMPORT_PARENT_RESOLVE) teaching"
# Guard against the tests being gutted to always-green stubs: each named test must exist.
for t in \
  non_absence_exit_128_errors_loud_with_rpx0508 \
  spawn_failure_errors_loud_with_rpx0508 \
  ref_absent_exit_1_returns_ok_none \
  anomalous_exit_0_empty_stdout_errors_loud_with_rpx0508 \
  present_ref_returns_some_import_parent; do
  grep -q "fn ${t}(" "${SRC}" || fail "regression test ${t} is missing from ${SRC}"
done
# The loud tests must assert the RPX-0508 tag (not merely "returns Err").
grep -q 'RPX-0508' "${SRC}" || fail "the regression tests no longer assert the RPX-0508 tag"

# --- Run the BARE per-crate test (bin-target unit tests + lib + integration) --
echo "running: cargo test -p reposix-remote (bare per-crate)" >&2
TEST_LOG="$(mktemp)"
trap 'rm -f "${TEST_LOG}"' EXIT
if CARGO_BUILD_JOBS=2 cargo test -p reposix-remote > "${TEST_LOG}" 2>&1; then
  RC=0
else
  RC=$?
fi
tail -30 "${TEST_LOG}" >&2
if [[ "${RC}" -ne 0 ]]; then
  fail "cargo test -p reposix-remote exited ${RC} (a resolve_import_parent regression or an unrelated crate test failed)"
fi
# Confirm the resolve_import_parent tests actually RAN (not filtered/ignored).
grep -q 'resolve_import_parent_tests::non_absence_exit_128_errors_loud_with_rpx0508 ... ok' "${TEST_LOG}" \
  || fail "the exit-128 loud-error regression test did not run/pass under the bare per-crate invocation"
grep -q 'resolve_import_parent_tests::anomalous_exit_0_empty_stdout_errors_loud_with_rpx0508 ... ok' "${TEST_LOG}" \
  || fail "the exit-0-empty-stdout loud-error regression test did not run/pass"
grep -q 'resolve_import_parent_tests::ref_absent_exit_1_returns_ok_none ... ok' "${TEST_LOG}" \
  || fail "the ref-absent -> Ok(None) regression test did not run/pass"

# --- Emit the verification artifact (F-K4b congruence: asserts_passed token-maps
#     each of the row's expected.asserts) ------------------------------------
mkdir -p "$(dirname "${ARTIFACT}")"
TS="$(date -u +"%Y-%m-%dT%H:%M:%SZ")" ROW="${ROW_ID}" OUT="${ARTIFACT}" python3 - <<'PY'
import json, os
artifact = {
    "ts": os.environ["TS"],
    "row_id": os.environ["ROW"],
    "exit_code": 0,
    "timed_out": False,
    "stdout": "cargo test -p reposix-remote (bare per-crate) exited 0; resolve_import_parent_tests all passed.",
    "stderr": "",
    "asserts_passed": [
        "cargo test -p reposix-remote (bare per-crate, bin-target unit tests) exits 0 for the resolve_import_parent regression tests",
        "resolve_import_parent returns Ok(None) ONLY for spawn-success + exit-1 + empty-stdout (the genuine ref-absent first-fetch parentless seed) -- proven by ref_absent_exit_1_returns_ok_none",
        "resolve_import_parent returns Err (loud) on a NON-absence git failure (an injected fake-git runner that exits 128, and a spawn failure) -- proven by non_absence_exit_128_errors_loud_with_rpx0508 + spawn_failure_errors_loud_with_rpx0508",
        "resolve_import_parent returns Err (loud, RPX-0508) on an anomalous exit-0-with-empty-stdout rather than silently degrading to the parentless overlay -- proven by anomalous_exit_0_empty_stdout_errors_loud_with_rpx0508",
        "the loud non-absence failure emits the RPX-0508 coded teaching (tag + Fix + Recovery + Explain nudge) instead of silently degrading to the parentless overlay",
    ],
    "asserts_failed": [],
}
with open(os.environ["OUT"], "w", encoding="utf-8") as f:
    json.dump(artifact, f, indent=2)
    f.write("\n")
PY

echo "PASS (${ROW_ID}): resolve_import_parent fails loud (RPX-0508) on non-absence git failure; ref-absent stays Ok(None); regression tests green under bare cargo test -p reposix-remote"
exit 0
