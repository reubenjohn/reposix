#!/usr/bin/env bash
# quality/gates/code/ci-green-on-main.sh
# Binds catalog row: code/ci-green-on-main  (kind: mechanical, cadence: post-push)
#
# WHAT: assert the LATEST run of EVERY workflow in the required-workflow list
# (WORKFLOWS below) on `main` concluded success. Grades the
# `code/ci-green-on-main` P0 row that closes the systemic hole where a phase
# shipped GREEN while its push turned main RED and nobody re-checked CI.
#
# WHY post-push, and why NON-CIRCULAR (do NOT re-demote to pre-push):
#   This runs at the `post-push` cadence -- at phase/milestone-close, AFTER
#   `git push origin main` has LANDED, orchestrator/verifier-side. It reads the
#   conclusion of the run CI has ALREADY started for the just-landed commit.
#   That is the OPPOSITE of D-CONV-1's circularity concern: D-CONV-1 demoted
#   `code/cargo-*` / `ci-job-status` out of the pre-* path because a CI-green
#   check running BEFORE/INSIDE the CI run for the commit under test is circular
#   (CI has not concluded). Asking "did main's latest CI go green?" once the push
#   is on main is not circular. Keep this on post-push.
#
# WHY the LATEST run per workflow, not any green run:
#   Uses `gh run list --workflow=<wf> --branch=main --limit=1` with NO --status
#   filter and parses the SINGLE most-recent run for each watched workflow. A
#   `--status=success` query (as ci-job-status.sh deliberately uses for a
#   DIFFERENT purpose) would surface the newest GREEN run even when the LATEST
#   run is RED -- masking a red HEAD. This verifier must fail on a red HEAD, so
#   it inspects only the true latest run of each workflow.
#
# WHY a required-workflow LIST, not a single hardcoded WORKFLOW (P123/SC5a,
# GTH-V15-07, DRAIN-01): a lone `WORKFLOW="ci.yml"` let a persistently-RED
# `release-plz.yml` on main rot unnoticed -- this row only ever watched ci.yml.
# WORKFLOWS below is exactly the two workflows CONFIRMED (2026-07-18, by
# reading both workflow files) to fire unconditionally on `push: branches:
# [main]` with NO path filter -- i.e. every push to main is guaranteed to spawn
# a run of both:
#   - .github/workflows/ci.yml            -- `on: push: branches: [main]`
#   - .github/workflows/release-plz.yml   -- `on: push: branches: [main]`
#     (`gh run list --workflow=release-plz.yml --branch=main --limit=8` on
#     2026-07-18 showed 8/8 recent main runs concluded `success`, never
#     `skipped` -- a no-op release-plz run still concludes `success`.)
# Deliberately EXCLUDED (do NOT add these without re-verifying their trigger
# shape first):
#   - .github/workflows/audit.yml               -- path-filtered to
#     `**/Cargo.toml`/`**/Cargo.lock`; does NOT fire on every push to main, so
#     "no run for this push" is the EXPECTED steady state, not a gap.
#   - .github/workflows/docs.yml                -- `on: workflow_run: workflows:
#     ["CI"]`, not `push` -- fires only after ci.yml completes, never 1:1 with
#     the push itself.
#   - .github/workflows/quality-post-release.yml -- `on: workflow_run:
#     workflows: ["release"]`, not `push` -- same reasoning as docs.yml.
#
# AGGREGATION SEMANTICS (per-workflow verdict -> one overall verdict):
#   - ANY watched workflow's `gh` call failing (missing/unauth), reporting NO
#     run at all on `main` (the "no run for this commit/branch yet" edge case
#     -- e.g. a race where the push landed a beat before Actions registered
#     the run), or still in-progress -> overall NOT-VERIFIED. Never a silent
#     PASS (an unknowable workflow must not be treated as green) and never a
#     hang (each `gh` call still returns -- there is nothing left to poll).
#   - Else if ANY watched workflow's latest run concluded non-success ->
#     overall FAIL, naming WHICH workflow is red and its conclusion.
#   - Else (every watched workflow's latest run concluded success) -> PASS.
#   NOT-VERIFIED takes priority over FAIL: if one workflow is unknowable and a
#   DIFFERENT workflow is independently red, the row reports NOT-VERIFIED (the
#   unknowable workflow blocks a confident verdict either way) -- re-run once
#   gh/the run resolves to get the true FAIL.
#
# Exit codes (mapped by quality/runners/run.py):
#   0  -> PASS         : every watched workflow's latest run concluded success.
#   1  -> FAIL         : gh + every watched workflow resolved, but at least one
#                        latest run concluded failure/cancelled/timed_out/etc.
#   75 -> NOT-VERIFIED : gh missing/unauthenticated, OR any watched workflow
#                        reported no run / is still in-progress. Per the
#                        runner's exit-75 convention (quality/PROTOCOL.md
#                        "Verifier exit-code conventions"), this is the honest
#                        "cannot determine" state -- NEVER a skip-as-PASS. A P0
#                        NOT-VERIFIED still grades RED at the verdict layer, so
#                        an unknowable main does not close a phase.
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

if ! command -v gh >/dev/null 2>&1; then
  emit 75 '[]' '["gh CLI not installed -- cannot determine main CI state (run: gh auth login on a machine with gh)"]' '{}'
  echo "NOT-VERIFIED: gh CLI not installed" >&2
  exit 75
fi

# Per-workflow verdict: one of success | in-progress | none | gh-error | failure:<conclusion>.
#
# `env -u GH_TOKEN -u GITHUB_TOKEN` (Rule 1 fix, discovered running this row
# through `quality/runners/run.py`, which self-sources ./.env per P123/
# DRAIN-03): this row's threat model explicitly claims `gh run list` output is
# "authenticated GitHub API via the operator's own `gh auth`" -- but `gh`
# actually prioritizes GH_TOKEN/GITHUB_TOKEN env vars over the stored keyring
# session when EITHER is set. .env's GITHUB_TOKEN is a REST bearer PAT for
# OTHER real-backend rows (github-front-door-real-backend.sh, perf/
# latency-bench/github.sh) that call the GitHub REST API directly with
# `curl -H "Authorization: Bearer ..."` -- it is not guaranteed valid for the
# `gh` CLI's own auth resolution, and unconditionally letting it leak into
# this row's `gh` calls contradicts the row's own documented trust boundary.
# Explicitly unsetting both env vars for the `gh` invocation (mirroring the
# `env -u GITHUB_TOKEN ...` isolation precedent in
# quality/gates/agent-ux/real-backend-env-gate.selftest.sh) makes this row
# actually read the operator's `gh auth login` session, as claimed.
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
WF_JSON_PARTS=()
for wf in "${WORKFLOWS[@]}"; do
  v="${VERDICTS[$wf]}"
  case "$v" in
    success)
      GREEN_MSGS+=("latest $wf run on $BRANCH concluded success")
      WF_JSON_PARTS+=("\"$wf\": \"success\"")
      ;;
    gh-error)
      UNKNOWABLE+=("$wf (gh run list failed -- unauthenticated or network error)")
      WF_JSON_PARTS+=("\"$wf\": \"gh-error\"")
      ;;
    in-progress)
      UNKNOWABLE+=("$wf (latest run still in-progress -- CI not concluded yet)")
      WF_JSON_PARTS+=("\"$wf\": \"in-progress\"")
      ;;
    none)
      # "No run for this commit/branch" -- e.g. release-plz.yml has never
      # fired on main yet, or a registration race just after the push landed.
      # This must NOT silently PASS (an unwatched workflow isn't green by
      # default) and must NOT hang (gh already returned; nothing to poll) --
      # it is an honest NOT-VERIFIED naming which workflow was unknowable.
      UNKNOWABLE+=("$wf (no run found on $BRANCH -- cannot determine CI state)")
      WF_JSON_PARTS+=("\"$wf\": \"none\"")
      ;;
    failure:*)
      concl="${v#failure:}"
      RED+=("$wf (concluded '$concl' -- not success)")
      WF_JSON_PARTS+=("\"$wf\": \"failure:$concl\"")
      ;;
    *)
      UNKNOWABLE+=("$wf (unexpected verdict parse: '$v')")
      WF_JSON_PARTS+=("\"$wf\": \"unknown:$v\"")
      ;;
  esac
done

WF_JSON="{$(IFS=,; echo "${WF_JSON_PARTS[*]}")}"

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
