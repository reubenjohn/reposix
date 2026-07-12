#!/usr/bin/env bash
# quality/gates/code/ci-green-on-main.sh
# Binds catalog row: code/ci-green-on-main  (kind: mechanical, cadence: post-push)
#
# WHAT: assert the LATEST ci.yml run on `main` concluded success. Grades the
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
# WHY the LATEST run, not any green run:
#   Uses `gh run list --workflow=ci.yml --branch=main --limit=1` with NO --status
#   filter and parses the SINGLE most-recent run. A `--status=success` query (as
#   ci-job-status.sh deliberately uses for a DIFFERENT purpose) would surface the
#   newest GREEN run even when the LATEST run is RED -- masking a red HEAD. This
#   verifier must fail on a red HEAD, so it inspects only the true latest run.
#
# Exit codes (mapped by quality/runners/run.py):
#   0  -> PASS         : latest run's conclusion == "success".
#   1  -> FAIL         : latest run concluded failure/cancelled/timed_out/etc.
#   75 -> NOT-VERIFIED : gh missing/unauthenticated, no runs found, or the
#                        latest run is still in-progress (status != completed /
#                        conclusion null). Per the runner's exit-75 convention
#                        (quality/PROTOCOL.md "Verifier exit-code conventions"),
#                        this is the honest "cannot determine" state -- NEVER a
#                        skip-as-PASS. A P0 NOT-VERIFIED still grades RED at the
#                        verdict layer, so an unknowable main does not close a phase.
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
ARTIFACT_DIR="$REPO_ROOT/quality/reports/verifications/code"
mkdir -p "$ARTIFACT_DIR"
ARTIFACT="$ARTIFACT_DIR/ci-green-on-main.json"
ROW_ID="code/ci-green-on-main"
WORKFLOW="ci.yml"
BRANCH="main"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

emit() {
  # emit <exit_code> <json_passed_array> <json_failed_array>
  printf '{"ts": "%s", "row_id": "%s", "exit_code": %s, "asserts_passed": %s, "asserts_failed": %s}\n' \
    "$TS" "$ROW_ID" "$1" "$2" "$3" > "$ARTIFACT"
}

if ! command -v gh >/dev/null 2>&1; then
  emit 75 '[]' '["gh CLI not installed -- cannot determine main CI state (run: gh auth login on a machine with gh)"]'
  echo "NOT-VERIFIED: gh CLI not installed" >&2
  exit 75
fi

# No --status filter: we need the SINGLE latest run, red or green.
RAW=$(gh run list --workflow="$WORKFLOW" --branch="$BRANCH" --limit=1 \
        --json databaseId,conclusion,status 2>/dev/null) || {
  emit 75 '[]' '["gh run list failed (unauthenticated or network error) -- cannot determine main CI state"]'
  echo "NOT-VERIFIED: gh run list failed (auth/network)" >&2
  exit 75
}

# Parse the single latest run. VERDICT is one of: success | failure | in-progress | none.
VERDICT=$(printf '%s' "$RAW" | python3 -c '
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

case "$VERDICT" in
  success)
    emit 0 "[\"latest $WORKFLOW run on $BRANCH concluded success\"]" '[]'
    echo "PASS: $ROW_ID -- latest $WORKFLOW run on $BRANCH is GREEN"
    exit 0
    ;;
  in-progress)
    emit 75 '[]' "[\"latest $WORKFLOW run on $BRANCH is still in-progress -- CI not concluded yet\"]"
    echo "NOT-VERIFIED: latest $WORKFLOW run on $BRANCH still in-progress" >&2
    exit 75
    ;;
  none)
    emit 75 '[]' "[\"no $WORKFLOW run found on $BRANCH -- cannot determine CI state\"]"
    echo "NOT-VERIFIED: no $WORKFLOW run found on $BRANCH" >&2
    exit 75
    ;;
  failure:*)
    CONCL="${VERDICT#failure:}"
    emit 1 '[]' "[\"latest $WORKFLOW run on $BRANCH concluded '$CONCL' (not success) -- main is RED\"]"
    echo "FAIL: $ROW_ID -- latest $WORKFLOW run on $BRANCH concluded '$CONCL'" >&2
    exit 1
    ;;
  *)
    emit 1 '[]' "[\"unexpected verdict parse: '$VERDICT'\"]"
    echo "FAIL: $ROW_ID -- unexpected verdict '$VERDICT'" >&2
    exit 1
    ;;
esac
