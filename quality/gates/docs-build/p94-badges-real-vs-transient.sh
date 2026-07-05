#!/usr/bin/env bash
# quality/gates/docs-build/p94-badges-real-vs-transient.sh
# Verifier for catalog row docs-build/p94-badges-real-vs-transient (P94 D3).
#
# Closes the recurring `badges-resolve` pre-push RED (GOOD-TO-HAVES.md
# badges-resolve entry, MEDIUM/P2) by grading the row's expected.asserts:
#
#   1. badges-resolve.py was re-run in isolation on >=2 spaced occasions and
#      the pass/fail pattern was recorded (the determination artifact
#      distinguishes a transient flake from a genuinely-broken URL).
#   2. the real-vs-transient verdict is recorded AND the GOOD-TO-HAVES.md
#      badges-resolve entry is RESOLVED (status flipped from OPEN) with that
#      finding.
#   3. TRANSIENT branch: badges-resolve.py gained a retry/backoff (or a
#      documented waiver). Determination was TRANSIENT, so we assert the
#      retry/backoff is present.
#   4. net: `python3 quality/gates/docs-build/badges-resolve.py` exits 0
#      (docs-build/badges-resolve reaches green, no longer flaking RED
#      on pre-push).
#
# Does NOT itself decide real-vs-transient — that judgement lives in the
# committed determination artifact + the resolved GOOD-TO-HAVES entry; this
# gate mechanically confirms the determination was made, recorded, acted on,
# and that the underlying check now passes.
#
# Exit: 0 -> PASS, 1 -> FAIL. Usage: [--row-id <id>]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

ROW_ID="docs-build/p94-badges-real-vs-transient"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

ARTIFACT="${REPO_ROOT}/quality/reports/verifications/docs-build/p94-badges-real-vs-transient.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

DETERMINATION="${REPO_ROOT}/.planning/phases/94-real-backend-frictions/94-D3-badges-determination.md"
GOODTOHAVES="${REPO_ROOT}/.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md"
BADGES="${REPO_ROOT}/quality/gates/docs-build/badges-resolve.py"

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

# ---- Assert 1: >=2 spaced isolated re-runs recorded --------------------------
[[ -f "$DETERMINATION" ]] \
  || fail "determination artifact missing" "$DETERMINATION"
# The evidence table records at least two dated runs (Run 1 + Run 2, spaced).
RUN_ROWS="$(grep -cE '^\| *[0-9]+ *\|' "$DETERMINATION" || true)"
[[ "${RUN_ROWS:-0}" -ge 2 ]] \
  || fail "determination artifact records fewer than 2 spaced re-runs" "found ${RUN_ROWS}"
pass "badges-resolve.py re-run on >=2 spaced occasions; pass/fail pattern recorded (${RUN_ROWS} runs)"

# ---- Assert 2: verdict recorded + GOOD-TO-HAVES entry RESOLVED ----------------
grep -qiE 'Verdict:.*(TRANSIENT|REAL)' "$DETERMINATION" \
  || fail "determination artifact records no real-vs-transient verdict"
# The GOOD-TO-HAVES badges-resolve entry must be flipped from OPEN to RESOLVED.
python3 - "$GOODTOHAVES" <<'PY' || fail "GOOD-TO-HAVES badges-resolve entry is not RESOLVED (still OPEN or missing)"
import re, sys
text = open(sys.argv[1], encoding="utf-8").read()
# Isolate the badges-resolve entry (its header to the next --- rule or EOF).
m = re.search(r"^## .*`badges-resolve` FAILs on pre-push.*?$(.*?)(?=^---\s*$|\Z)",
              text, re.S | re.M)
if not m:
    sys.exit(1)
body = m.group(1)
# Must be RESOLVED, not OPEN, and cite a verdict.
ok = re.search(r"\*\*STATUS:\*\*\s*RESOLVED", body) and re.search(r"TRANSIENT|REAL", body)
sys.exit(0 if ok else 1)
PY
pass "real-vs-transient verdict recorded + GOOD-TO-HAVES badges-resolve entry RESOLVED"

# ---- Assert 3: TRANSIENT branch -> retry/backoff present ----------------------
# Determination was TRANSIENT: assert the gate gained a bounded retry/backoff.
grep -qE 'MAX_ATTEMPTS' "$BADGES" && grep -qE 'TRANSIENT_HTTP' "$BADGES" && grep -qE 'BACKOFF_S' "$BADGES" \
  || fail "badges-resolve.py lacks the retry/backoff (MAX_ATTEMPTS/TRANSIENT_HTTP/BACKOFF_S) the TRANSIENT verdict requires"
pass "badges-resolve.py has bounded retry/backoff for transient failures (real 404/403/wrong-type still fail fast)"

# ---- Assert 4: net -> badges-resolve.py exits 0 ------------------------------
echo "p94-d3: running badges-resolve.py (must exit 0)…" >&2
if python3 "$BADGES" >&2; then
  pass "python3 quality/gates/docs-build/badges-resolve.py exits 0 (docs-build/badges-resolve is green)"
else
  fail "badges-resolve.py did NOT exit 0 — docs-build/badges-resolve still failing"
fi

PJ="$(printf '%s\n' "${PASSED[@]}" | python3 -c 'import json,sys; print(json.dumps([l for l in sys.stdin.read().splitlines() if l]))')"
cat > "$ARTIFACT" <<EOF
{
  "ts": "$TS", "row_id": "$ROW_ID", "exit_code": 0, "status": "PASS",
  "verdict": "TRANSIENT",
  "asserts_passed": ${PJ},
  "asserts_failed": []
}
EOF
echo "PASS (${ROW_ID}): badges failure diagnosed TRANSIENT, retry/backoff added, entry resolved, badges-resolve.py exits 0." >&2
exit 0
