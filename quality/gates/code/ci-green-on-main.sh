#!/usr/bin/env bash
# quality/gates/code/ci-green-on-main.sh
# Binds catalog row: code/ci-green-on-main  (kind: mechanical, cadence: post-push)
#
# WHAT: assert the LATEST run of EVERY workflow in the required-workflow list
# (WORKFLOWS below) on `main` concluded success. Grades the
# `code/ci-green-on-main` P0 row that closes the systemic hole where a phase
# shipped GREEN while its push turned main RED and nobody re-checked CI.
#
# WHY post-push, NON-CIRCULAR (do NOT re-demote to pre-push): runs AFTER
# `git push origin main` has LANDED, orchestrator/verifier-side -- reads the
# conclusion of the run CI already started for the just-landed commit. That is
# the OPPOSITE of D-CONV-1's circularity concern (which demoted `code/cargo-*`
# / `ci-job-status` out of pre-* because a CI-green check running BEFORE/INSIDE
# the CI run under test is circular). Keep this on post-push.
#
# WHY the LATEST run per workflow, not any green run: uses `gh run list
# --workflow=<wf> --branch=main --limit=1` with NO --status filter, per
# watched workflow. A `--status=success` query (as ci-job-status.sh
# deliberately uses, for a different purpose) would surface an OLDER green run
# and mask a red HEAD -- this verifier must fail on a red HEAD.
#
# WHY a required-workflow LIST, not a single hardcoded WORKFLOW (P123/SC5a,
# GTH-V15-07, DRAIN-01): a lone `WORKFLOW="ci.yml"` let a persistently-RED
# `release-plz.yml` on main rot unnoticed. WORKFLOWS below is exactly the two
# workflows CONFIRMED (2026-07-18, by reading both workflow files) to fire
# unconditionally on `push: branches: [main]` with NO path filter:
#   - .github/workflows/ci.yml            -- `on: push: branches: [main]`
#   - .github/workflows/release-plz.yml   -- `on: push: branches: [main]`
#     (8/8 recent main runs concluded `success`, checked 2026-07-18 -- a no-op
#     release-plz run still concludes `success`, never `skipped`)
# Deliberately EXCLUDED (re-verify trigger shape before ever adding these):
#   - .github/workflows/audit.yml -- path-filtered to Cargo.toml/lock, does NOT
#     fire on every push ("no run for this push" is the expected steady state).
#   - .github/workflows/docs.yml, quality-post-release.yml -- both trigger via
#     `workflow_run`, not `push`, never 1:1 with the push itself.
#
# AGGREGATION (per-workflow verdict -> one overall verdict):
#   - ANY watched workflow's `gh` call failing (missing/unauth), reporting NO
#     run at all (the "no run for this commit/branch yet" edge case -- a race
#     where the push landed before Actions registered the run, or a workflow
#     that has simply never fired), or still in-progress -> overall
#     NOT-VERIFIED, naming which workflow(s). Never a silent PASS (unknowable
#     is not green) and never a hang (each `gh` call still returns).
#   - Else if ANY watched workflow's latest run concluded non-success ->
#     overall FAIL, naming which workflow and its conclusion.
#   - Else (every watched workflow succeeded) -> PASS.
#   NOT-VERIFIED beats FAIL: if one workflow is unknowable and a DIFFERENT one
#   is independently red, the row reports NOT-VERIFIED -- re-run once gh/the
#   run resolves to get the true FAIL.
#
# Exit codes (mapped by quality/runners/run.py):
#   0  -> PASS         : every watched workflow's latest run concluded success.
#   1  -> FAIL         : all resolved, but >=1 latest run concluded
#                        failure/cancelled/timed_out/etc.
#   75 -> NOT-VERIFIED : gh missing/unauthenticated, OR any watched workflow
#                        reported no run / is still in-progress (exit-75
#                        convention, quality/PROTOCOL.md "Verifier exit-code
#                        conventions") -- NEVER a skip-as-PASS. A P0
#                        NOT-VERIFIED still grades RED at the verdict layer.
set -uo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
ARTIFACT_DIR="$REPO_ROOT/quality/reports/verifications/code"
mkdir -p "$ARTIFACT_DIR"
ARTIFACT="$ARTIFACT_DIR/ci-green-on-main.json"
ROW_ID="code/ci-green-on-main"
WORKFLOWS=("ci.yml" "release-plz.yml")
BRANCH="main"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

emit() {
  # emit <exit_code> <json_passed_array> <json_failed_array> <json_workflows_obj>
  printf '{"ts": "%s", "row_id": "%s", "exit_code": %s, "asserts_passed": %s, "asserts_failed": %s, "workflows": %s}\n' \
    "$TS" "$ROW_ID" "$1" "$2" "$3" "$4" > "$ARTIFACT"
}

json_array() {
  # json_array <str...> -> a JSON string array, [] for zero args.
  if [ "$#" -eq 0 ]; then
    printf '[]'
    return
  fi
  python3 -c 'import json, sys; print(json.dumps(sys.argv[1:]))' "$@"
}

json_object() {
  # json_object <k1> <v1> <k2> <v2> ... -> a JSON object {"k1":"v1", ...}, {} for
  # zero args. Serialized via json.dumps (mirrors json_array) so a per-workflow
  # verdict value derived from a TAINTED `gh` API byte (the run conclusion string)
  # is properly ESCAPED, never hand-interpolated into a JSON string literal
  # (tainted-by-default; a hand-built quote/backslash in that byte would otherwise
  # produce malformed or injected JSON in the artifact).
  if [ "$#" -eq 0 ]; then
    printf '{}'
    return
  fi
  python3 -c 'import json, sys; a = sys.argv[1:]; print(json.dumps(dict(zip(a[0::2], a[1::2]))))' "$@"
}

if ! command -v gh >/dev/null 2>&1; then
  emit 75 '[]' '["gh CLI not installed -- cannot determine main CI state (run: gh auth login on a machine with gh)"]' '{}'
  echo "NOT-VERIFIED: gh CLI not installed" >&2
  exit 75
fi

# Per-workflow verdict: one of success | in-progress | none | gh-error | failure:<conclusion>.
#
# `env -u GH_TOKEN -u GITHUB_TOKEN` (Rule 1 fix, found running this row via
# run.py, which self-sources ./.env per P123/DRAIN-03): `gh` prioritizes
# GH_TOKEN/GITHUB_TOKEN env vars over the stored keyring session when EITHER
# is set. .env's GITHUB_TOKEN is a REST bearer PAT for OTHER real-backend rows
# (github-front-door-real-backend.sh, perf/latency-bench/github.sh) -- not
# guaranteed valid for `gh` CLI auth, and letting it leak in here contradicts
# this row's documented trust boundary ("authenticated via the operator's own
# `gh auth`"). Unsetting both mirrors the isolation precedent in
# quality/gates/agent-ux/real-backend-env-gate.selftest.sh.
declare -A VERDICTS
for wf in "${WORKFLOWS[@]}"; do
  RAW=""
  RAW=$(env -u GH_TOKEN -u GITHUB_TOKEN gh run list --workflow="$wf" --branch="$BRANCH" --limit=1 \
          --json databaseId,conclusion,status 2>/dev/null) || RAW=""
  if [ -z "$RAW" ]; then
    VERDICTS["$wf"]="gh-error"
    continue
  fi
  VERDICTS["$wf"]=$(printf '%s' "$RAW" | python3 -c '
import json, sys
try:
    runs = json.load(sys.stdin)
except Exception:
    print("none"); sys.exit(0)
if not isinstance(runs, list) or not runs:
    print("none"); sys.exit(0)
r = runs[0]
status = r.get("status")
concl = r.get("conclusion")
if status != "completed" or concl is None:
    print("in-progress"); sys.exit(0)
print("success" if concl == "success" else "failure:" + str(concl))
')
done

# Partition into unknowable / red / green, and build the per-workflow JSON
# object the artifact carries alongside the existing top-level asserts_*.
UNKNOWABLE=()
RED=()
GREEN_MSGS=()
WF_KV=()   # flat [wf1, verdict1, wf2, verdict2, ...] -> json_object serializes it
for wf in "${WORKFLOWS[@]}"; do
  v="${VERDICTS[$wf]}"
  case "$v" in
    success)
      GREEN_MSGS+=("latest $wf run on $BRANCH concluded success")
      WF_KV+=("$wf" "success")
      ;;
    gh-error)
      UNKNOWABLE+=("$wf (gh run list failed -- unauthenticated or network error)")
      WF_KV+=("$wf" "gh-error")
      ;;
    in-progress)
      UNKNOWABLE+=("$wf (latest run still in-progress -- CI not concluded yet)")
      WF_KV+=("$wf" "in-progress")
      ;;
    none)
      # "No run for this commit/branch" -- e.g. release-plz.yml has never
      # fired on main yet, or a registration race just after the push landed.
      # This must NOT silently PASS (an unwatched workflow isn't green by
      # default) and must NOT hang (gh already returned; nothing to poll) --
      # it is an honest NOT-VERIFIED naming which workflow was unknowable.
      UNKNOWABLE+=("$wf (no run found on $BRANCH -- cannot determine CI state)")
      WF_KV+=("$wf" "none")
      ;;
    failure:*)
      concl="${v#failure:}"
      RED+=("$wf (concluded '$concl' -- not success)")
      # `failure:$concl` embeds a TAINTED gh byte -- json_object escapes it.
      WF_KV+=("$wf" "failure:$concl")
      ;;
    *)
      UNKNOWABLE+=("$wf (unexpected verdict parse: '$v')")
      # `unknown:$v` likewise embeds the tainted verdict byte -- serialized, not
      # hand-interpolated.
      WF_KV+=("$wf" "unknown:$v")
      ;;
  esac
done

# json_object serializes the per-workflow verdict map so any tainted gh-derived
# conclusion byte is escaped, never hand-interpolated into a JSON literal.
WF_JSON="$(json_object "${WF_KV[@]}")"

if [ "${#UNKNOWABLE[@]}" -gt 0 ]; then
  FAILED_JSON="$(json_array "${UNKNOWABLE[@]}")"
  emit 75 '[]' "$FAILED_JSON" "$WF_JSON"
  echo "NOT-VERIFIED: $ROW_ID -- unknowable workflow(s): ${UNKNOWABLE[*]}" >&2
  exit 75
fi

if [ "${#RED[@]}" -gt 0 ]; then
  FAILED_JSON="$(json_array "${RED[@]}")"
  emit 1 '[]' "$FAILED_JSON" "$WF_JSON"
  echo "FAIL: $ROW_ID -- red workflow(s) on $BRANCH: ${RED[*]}" >&2
  exit 1
fi

PASSED_JSON="$(json_array "${GREEN_MSGS[@]}")"
emit 0 "$PASSED_JSON" '[]' "$WF_JSON"
echo "PASS: $ROW_ID -- latest run of every required workflow on $BRANCH is GREEN (${WORKFLOWS[*]})"
exit 0
