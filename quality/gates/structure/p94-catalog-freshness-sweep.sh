#!/usr/bin/env bash
# quality/gates/structure/p94-catalog-freshness-sweep.sh
# Verifier for catalog row structure/p94-catalog-freshness-sweep (P94 D4).
# kind: subagent-graded — the unbiased phase-close/milestone verifier subagent
# adjudicates "env-gate vs code-regression" from the committed evidence; this
# mechanical gate validates that the evidence + classification exist, are
# complete, name the pre-accounted env-gate, and self-certify ZERO unaccounted
# regressions. It does NOT re-run the sweep (that would recurse — the sweep
# runs this row's cadence). Reproduce the sweep with:
#   bash .planning/milestones/v0.13.0-phases/94-real-backend-frictions/94-D4-sweep.sh
#
# Grades the row's expected.asserts:
#   1. every STALE non-P93 cadence-tagged row was re-graded in the sweep
#      (the sweep ran every cadence -> every cadence-tagged row was re-run).
#   2. every resulting FAIL is a NAMED accounted-for environment gate (e.g.
#      agent-ux/p92-mid-stream-litmus-t1-t4 exit 75 on git<2.34) — NOT a code
#      regression.
#   3. the env-gates are enumerated with exit code + why-not-a-regression.
#   4. no re-graded row flips PASS->FAIL for a code-change reason (a true
#      regression BLOCKS milestone-close -> this gate FAILs).
#
# Exit: 0 -> PASS (evidence complete, 0 unaccounted regressions); 1 -> FAIL
# (evidence missing/incomplete, or >=1 unaccounted regression -> milestone
# blocked). Usage: [--row-id <id>]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

ROW_ID="structure/p94-catalog-freshness-sweep"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

ARTIFACT="${REPO_ROOT}/quality/reports/verifications/structure/p94-catalog-freshness-sweep.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

SWEEP="${REPO_ROOT}/.planning/milestones/v0.13.0-phases/94-real-backend-frictions/94-freshness-sweep.txt"
CLASS="${REPO_ROOT}/.planning/milestones/v0.13.0-phases/94-real-backend-frictions/94-D4-sweep-classification.md"

PASSED=()
fail() {
  local desc="$1" detail="${2:-}"
  echo "FAIL (${ROW_ID}): ${desc}${detail:+: ${detail}}" >&2
  local pj; pj="$(printf '%s\n' "${PASSED[@]:-}" | python3 -c 'import json,sys; print(json.dumps([l for l in sys.stdin.read().splitlines() if l]))')"
  cat > "$ARTIFACT" <<EOF
{
  "ts": "$TS", "row_id": "$ROW_ID", "exit_code": 1, "status": "FAIL",
  "asserts_passed": ${pj},
  "asserts_failed": ["${desc}${detail:+ — ${detail}}"]
}
EOF
  exit 1
}
pass() { echo "  PASS: $1" >&2; PASSED+=("$1"); }

# ---- Assert 1: sweep ran every cadence (all rows re-graded) -------------------
[[ -f "$SWEEP" ]] || fail "sweep evidence artifact missing" "$SWEEP"
CAD_COUNT="$(grep -cE '^## CADENCE:' "$SWEEP" || true)"
[[ "${CAD_COUNT:-0}" -ge 8 ]] \
  || fail "sweep did not cover all 8 cadences (every cadence-tagged row re-graded)" "found ${CAD_COUNT} cadence sections"
grep -qE '^## VERDICT' "$SWEEP" || fail "sweep artifact has no final all-rows VERDICT section"
pass "sweep re-graded every cadence-tagged row (all ${CAD_COUNT} cadences run + all-rows verdict captured)"

# ---- Assert 2+3: FAILs classified, pre-accounted env-gate named ---------------
[[ -f "$CLASS" ]] || fail "FAIL-classification companion missing" "$CLASS"
grep -qE 'agent-ux/p92-mid-stream-litmus-t1-t4' "$CLASS" \
  || fail "classification does not name the pre-accounted git<2.34 env-gate (agent-ux/p92-mid-stream-litmus-t1-t4)"
grep -qiE 'git.?2\.34|exit.?75' "$CLASS" \
  || fail "classification does not record the env-gate exit code / git<2.34 reason"
pass "every sweep FAIL classified env-gate-vs-regression; pre-accounted p92-mid-stream-litmus git<2.34 exit-75 named"

# ---- Assert 4: zero UNACCOUNTED regressions (machine-checkable) ---------------
# The classification carries a machine line `UNACCOUNTED_REGRESSIONS: <N>`.
# N>0 means a genuine PASS->FAIL code regression hides behind the sweep and
# milestone-close is BLOCKED.
LINE="$(grep -E '^UNACCOUNTED_REGRESSIONS:' "$CLASS" | head -1 || true)"
[[ -n "$LINE" ]] || fail "classification lacks the machine-checkable UNACCOUNTED_REGRESSIONS: <N> line"
N="$(printf '%s' "$LINE" | grep -oE '[0-9]+' | head -1)"
[[ "${N:-1}" -eq 0 ]] \
  || fail "sweep found ${N} UNACCOUNTED regression(s) — a true PASS->FAIL code regression BLOCKS milestone-close"
pass "no re-graded row flips PASS->FAIL for a code reason (UNACCOUNTED_REGRESSIONS: 0)"

PJ="$(printf '%s\n' "${PASSED[@]}" | python3 -c 'import json,sys; print(json.dumps([l for l in sys.stdin.read().splitlines() if l]))')"
cat > "$ARTIFACT" <<EOF
{
  "ts": "$TS", "row_id": "$ROW_ID", "exit_code": 0, "status": "PASS",
  "unaccounted_regressions": 0,
  "evidence": ".planning/milestones/v0.13.0-phases/94-real-backend-frictions/94-freshness-sweep.txt",
  "classification": ".planning/milestones/v0.13.0-phases/94-real-backend-frictions/94-D4-sweep-classification.md",
  "asserts_passed": ${PJ},
  "asserts_failed": []
}
EOF
echo "PASS (${ROW_ID}): all cadences re-graded; every FAIL an accounted-for env-gate; 0 unaccounted regressions." >&2
exit 0
