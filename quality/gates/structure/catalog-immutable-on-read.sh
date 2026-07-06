#!/usr/bin/env bash
# quality/gates/structure/catalog-immutable-on-read.sh
# Verifier for catalog row structure/catalog-immutable-on-read (P96, D-P96-01).
#
# Backs the invariant: a quality-runner cadence GATE run (the pre-push / pre-pr
# path) must NOT write graded status back to quality/catalogs/*.json. Only the
# explicit `--persist` MINT invocation (phase-close / verifier-subagent grading)
# may mutate the committed catalog. This closes the HIGH self-mutation bug where
# a read-only pre-push flipped docs-build.json (NOT-VERIFIED->PASS) and dirtied
# the tree at push time (3 live repros, each dropped with the recurring
# `git checkout HEAD -- quality/catalogs/` band-aid — now unnecessary).
#
# Two layers of evidence:
#   (A) Hermetic unit proof — python3 -m unittest quality.runners.test_run.
#       Drives run.main() in-process over a one-row synthetic catalog and proves
#       all three halves: validate-only does not mutate; validate-only STILL
#       blocks RED (gate integrity); --persist STILL mints (grades not frozen).
#       This is the deterministic flip repro (DP-2): on the pre-fix runner it
#       FAILS because the synthetic flip is persisted.
#   (B) Real-tree breadth check — snapshot md5 of every real quality/catalogs/
#       *.json, run the REAL runner over a real cadence, assert ZERO catalog
#       bytes changed. Uses --cadence pre-commit deliberately: it is cargo-free
#       (fast) AND this row is tagged [pre-push, pre-pr] but NOT pre-commit, so
#       the inner run never re-invokes this verifier (no recursion). A full
#       --cadence pre-push run here would (1) recurse and (2) run cargo clippy a
#       second time inside a pre-push hook — both footguns the split avoids.
#
# Exit: 0 -> PASS; 1 -> FAIL. Usage: [--row-id <id>]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
cd "$REPO_ROOT"

ROW_ID="structure/catalog-immutable-on-read"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

ARTIFACT="${REPO_ROOT}/quality/reports/verifications/structure/catalog-immutable-on-read.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

PASSED=()
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
  emit_artifact 1 FAIL "$(python3 -c 'import json,sys; print(json.dumps([sys.argv[1]]))' "$desc")"
  exit 1
}
pass() { echo "  PASS: $1" >&2; PASSED+=("$1"); }

# ---- Layer A: hermetic unit proof (the deterministic flip repro) --------------
if ! python3 -m unittest quality.runners.test_run > /tmp/catalog-immutable-unittest.$$.log 2>&1; then
  echo "---- unittest output ----" >&2
  cat /tmp/catalog-immutable-unittest.$$.log >&2
  rm -f /tmp/catalog-immutable-unittest.$$.log
  fail "hermetic persist-gate unittest RED — a bare cadence run mutates the catalog, or --persist no longer mints, or a validate-only RED no longer blocks"
fi
rm -f /tmp/catalog-immutable-unittest.$$.log
pass "a bare cadence run (run.py --cadence pre-push, no --persist) leaves every quality/catalogs/*.json byte-identical (validate-only, no self-mutation)"
pass "a validate-only cadence run still blocks RED via compute_exit_code exit 1 (gate integrity preserved without persistence)"
pass "the explicit mint path (run.py --cadence <c> --persist) still writes graded status back to the catalog (grades not frozen)"

# ---- Layer B: real-tree breadth check (charter md5 snapshot) -------------------
CAT_DIR="${REPO_ROOT}/quality/catalogs"
BACKUP="$(mktemp -d)"
cp "${CAT_DIR}"/*.json "${BACKUP}/"
BEFORE="$(md5sum "${CAT_DIR}"/*.json | sed "s#${CAT_DIR}/##" | sort)"

# Real runner, real catalogs, cargo-free cadence, validate-only (no --persist).
python3 "${REPO_ROOT}/quality/runners/run.py" --cadence pre-commit > /dev/null 2>&1 || true

AFTER="$(md5sum "${CAT_DIR}"/*.json | sed "s#${CAT_DIR}/##" | sort)"
# Unconditionally restore from the byte-snapshot so this gate is non-destructive
# even against a pre-fix runner that would have persisted a flip.
cp "${BACKUP}"/*.json "${CAT_DIR}/"
rm -rf "${BACKUP}"

if [[ "$BEFORE" != "$AFTER" ]]; then
  echo "---- catalog md5 drift ----" >&2
  diff <(printf '%s\n' "$BEFORE") <(printf '%s\n' "$AFTER") >&2 || true
  fail "a real run.py cadence run mutated quality/catalogs/*.json bytes (self-mutation on read)"
fi
pass "a real run.py cadence run over the real quality/catalogs/ leaves all catalog md5 sums unchanged (breadth check)"

emit_artifact 0 PASS
echo "PASS (${ROW_ID}): cadence runs are validate-only (catalog byte-immutable); --persist still mints." >&2
exit 0
