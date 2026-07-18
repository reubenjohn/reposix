#!/usr/bin/env bash
# quality/gates/structure/persist-refuses-downgrade.sh
# Verifier for catalog row structure/persist-refuses-downgrade (P123 SC2, DRAIN-04).
#
# Backs the invariant: a quality-runner --persist MINT run must REFUSE to write a
# committed-GREEN (PASS/WAIVED, per `git show HEAD:<catalog>`) row back at an
# EXPLICIT worse grade (FAIL/PARTIAL) unless --allow-downgrade is passed -- and
# even the override prints a loud per-row notice, never a silent overwrite. This
# closes the silent-catalog-corruption near-miss where a --persist run downgraded
# vision-litmus PASS->FAIL on an env-skip false negative, caught only because the
# diff happened to be reviewed before staging (SURPRISES-INTAKE 2026-07-14 20:44).
#
# Load-bearing distinction (deadlock prevention): a demotion to NOT-VERIFIED
# (freshness-TTL expiry / missing verifier / env-skip / exit-75) is NOT a
# downgrade and is ALWAYS allowed -- otherwise the phase's own freshness-invariant
# mints (which produce NOT-VERIFIED) would deadlock against this guard.
#
# Layer-A hermetic-unit-proof shape (mirrors structure/catalog-immutable-on-read.sh):
# runs the deterministic in-process repro over a throwaway /tmp git repo with a real
# `git show HEAD:` committed baseline, proving refuse / allow-with-flag / NOT-VERIFIED-
# always-allowed / brand-new-exempt / exit-code semantics. On the pre-guard runner it
# FAILS (the synthetic downgrade would persist).
#
# Exit: 0 -> PASS; 1 -> FAIL. Usage: [--row-id <id>]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
cd "$REPO_ROOT"

ROW_ID="structure/persist-refuses-downgrade"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

ARTIFACT="${REPO_ROOT}/quality/reports/verifications/structure/persist-refuses-downgrade.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# asserts_passed are fixed for a PASS: each token-covers one expected.asserts
# entry on the backing row (F-K4b per-pair congruence; _audit_field.asserts_congruent).
PASSED=(
  "a committed PASS/WAIVED row whose fresh grade is worse is NOT persisted unless --allow-downgrade is passed (TestPersistDowngradeGuard test_1 PASS->FAIL + test_3 WAIVED->FAIL refuse; on-disk status unchanged)"
  "the refusal message prints the row id, old status, new status, and the literal recovery command python3 quality/runners/run.py --cadence <c> --persist --allow-downgrade (test_1 asserts the row id / PASS / FAIL / --allow-downgrade substrings)"
  "a row absent from the git-HEAD committed catalog (brand-new) is exempt from the downgrade guard and mints freely (test_4 head_rows=[] mints its fresh grade with no refusal)"
  "--allow-downgrade restores the prior unconditional-persist behavior, still printing a loud per-row ALLOWED notice (test_2 persists FAIL with a non-silent notice)"
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

# ---- Layer A: hermetic unit proof (the deterministic downgrade-guard repro) ----
LOG="/tmp/persist-refuses-downgrade-unittest.$$.log"
if ! python3 -m unittest quality.runners.test_run.TestPersistDowngradeGuard > "$LOG" 2>&1; then
  echo "---- unittest output ----" >&2
  cat "$LOG" >&2
  rm -f "$LOG"
  fail "TestPersistDowngradeGuard RED -- --persist either silently downgraded a committed-GREEN row, printed a non-teaching refusal, blocked a legitimate NOT-VERIFIED demotion, or wrongly blocked a brand-new row"
fi
rm -f "$LOG"

emit_artifact 0 PASS
echo "PASS (${ROW_ID}): --persist refuses a committed-GREEN downgrade without --allow-downgrade; NOT-VERIFIED demotions stay allowed." >&2
exit 0
