#!/usr/bin/env bash
# quality/gates/structure/persist-catalog-write-locked.sh
# Verifier for catalog row structure/persist-catalog-write-locked (P123 SC3, DRAIN-05).
#
# Backs the invariant: two concurrent quality-runner --persist MINT runs targeting
# the same catalog file MUST NOT interleave writes -- an OS-level advisory flock
# (quality/runners/_persist_guard.py::catalog_persist_lock) wraps the WHOLE
# per-catalog read-modify-write (load_catalog -> grade -> save_catalog), so the
# second writer's read cannot begin until the first writer's write has committed
# and released. Closes GTH-V15-01 (P104: two runners observed minting the same
# catalog file mid-verification), a live lost-update hazard held back only by
# orchestration convention until now.
#
# Load-bearing scope: the lock is acquired BEFORE load_catalog and released AFTER
# save_catalog. A narrower lock (only around the write) would still lose updates --
# both runners could read the same pre-mutation snapshot before either writes. A
# validate-only (no --persist) run takes a nullcontext branch and never contends.
#
# Layer-A hermetic-unit-proof shape (mirrors structure/persist-refuses-downgrade.sh):
# runs the deterministic concurrency repro -- a REAL second subprocess blocking on
# the held flock (wall-clock >= ~1.8s, not a mock), a validate-only run staying
# lock-free, single-writer minting unchanged, and TWO concurrent --persist
# processes on one catalog leaving both writers' flips intact (no lost update). On
# the pre-guard runner the concurrency case FAILS (one flip is overwritten).
#
# Exit: 0 -> PASS; 1 -> FAIL. Usage: [--row-id <id>]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
cd "$REPO_ROOT"

ROW_ID="structure/persist-catalog-write-locked"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

ARTIFACT="${REPO_ROOT}/quality/reports/verifications/structure/persist-catalog-write-locked.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# asserts_passed are fixed for a PASS: each token-covers one expected.asserts
# entry on the backing row (F-K4b per-pair congruence; _audit_field.asserts_congruent).
PASSED=(
  "a second --persist invocation targeting the same catalog while a first holds the lock BLOCKS until release, rather than interleaving writes (TestPersistCatalogLock test_1 measures a real subprocess acquire wall-clock >= 1.8s; test_4 runs two concurrent --persist processes on one catalog and both writers' row flips survive with no lost update)"
  "the lock is scoped to --persist runs only -- a validate-only run never contends for it (test_2 a validate-only run completes without blocking while a separate process holds the lock, and never opens the lock file)"
  "single-writer --persist behavior (mint / no-op / RED, from the existing TestPersistGate suite) is unchanged with the lock in place (test_3 uncontended --persist still mints the PASS grade)"
)

emit_artifact() {  # <exit_code> <status> <failed_json>
  local ec="$1" st="$2" failed="${3:-[]}"
  local pj
  pj="$(printf '%s\n' "${PASSED[@]:-}" | python3 -c 'import json,sys; print(json.dumps([l for l in sys.stdin.read().splitlines() if l]))')"
  cat > "$ARTIFACT" <<EOF
{
  "ts": "$TS", "row_id": "$ROW_ID", "exit_code": $ec, "status": "$st",
  "asserts_passed": ${pj},
  "asserts_failed": ${failed}
}
EOF
}
fail() {
  local desc="$1"
  echo "FAIL (${ROW_ID}): ${desc}" >&2
  PASSED=()  # a RED run proves no assert; emit an empty asserts_passed
  emit_artifact 1 FAIL "$(python3 -c 'import json,sys; print(json.dumps([sys.argv[1]]))' "$desc")"
  exit 1
}

# ---- Layer A: hermetic unit proof (the deterministic concurrency repro) ----
LOG="/tmp/persist-catalog-write-locked-unittest.$$.log"
if ! python3 -m unittest quality.runners.test_run.TestPersistCatalogLock > "$LOG" 2>&1; then
  echo "---- unittest output ----" >&2
  cat "$LOG" >&2
  rm -f "$LOG"
  fail "TestPersistCatalogLock RED -- the --persist lock either did not block a second real writer, was taken by a validate-only run, regressed single-writer minting, or let two concurrent --persist writers lost-update the same catalog"
fi
rm -f "$LOG"

emit_artifact 0 PASS
echo "PASS (${ROW_ID}): concurrent --persist runners serialize on the flock; validate-only stays lock-free; no lost update." >&2
exit 0
