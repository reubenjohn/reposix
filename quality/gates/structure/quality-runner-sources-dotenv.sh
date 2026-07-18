#!/usr/bin/env bash
# quality/gates/structure/quality-runner-sources-dotenv.sh
# Verifier for catalog row structure/quality-runner-sources-dotenv (P123 SC1, DRAIN-03).
#
# Backs the invariant: quality/runners/run.py conditionally self-sources ./.env
# (present-only, non-clobbering, no value leak) so a `pre-release-real-backend`
# cadence run exercises real creds instead of silently skipping every real-backend
# row to NOT-VERIFIED when the caller did not pre-source .env — closing the
# false-green-preflight gap (scripts/preflight-real-backends.sh sourced .env;
# run.py did not, so preflight could report "backends reachable" while the actual
# cadence run silently skipped every row). SURPRISES-INTAKE 2026-07-14 20:43 HIGH.
#
# Layer A hermetic unit proof (mirrors catalog-immutable-on-read.sh): drives
# _env_load.load_dotenv_if_present over synthetic temp-dir .env fixtures via
# `python3 -m unittest quality.runners.test_run.TestEnvSelfSourcing`, proving all
# four halves: unset keys load; an already-set var wins (non-clobber); a missing
# .env is a silent no-op; no secret VALUE reaches stderr (KEY names only). Pure
# tempfile fixtures — no real .env, no real creds, no network, no subprocess sweep.
#
# Exit: 0 -> PASS; 1 -> FAIL. Usage: [--row-id <id>]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
cd "$REPO_ROOT"

ROW_ID="structure/quality-runner-sources-dotenv"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

ARTIFACT="${REPO_ROOT}/quality/reports/verifications/structure/quality-runner-sources-dotenv.json"
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

# ---- Layer A: hermetic unit proof ----------------------------------------------
UNITLOG="/tmp/quality-runner-sources-dotenv-unittest.$$.log"
if ! python3 -m unittest quality.runners.test_run.TestEnvSelfSourcing > "$UNITLOG" 2>&1; then
  echo "---- unittest output ----" >&2
  cat "$UNITLOG" >&2
  rm -f "$UNITLOG"
  fail "hermetic .env self-sourcing unittest RED — run.py failed to source ./.env, clobbered an already-set var, leaked a secret value, or errored on a missing .env"
fi
rm -f "$UNITLOG"

# asserts_passed crafted to token-map the backing row's 3 expected.asserts
# (F-K4b per-expected-assert congruence, _audit_field.asserts_congruent).
pass "a ./.env at repo root is parsed and its KEY=VALUE pairs are exported into os.environ only for keys not already present (explicit shell env always wins)"
pass "no .env present is a silent no-op — no error and no CI behavior change"
pass "no secret VALUE is ever printed to stdout or stderr — only which KEY names were loaded, if any"

emit_artifact 0 PASS
echo "PASS (${ROW_ID}): run.py conditionally self-sources ./.env (present-only, non-clobbering, no value leak)." >&2
exit 0
