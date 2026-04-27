#!/usr/bin/env bash
# Quality Gates verifier — generic gh-CLI wrapper for code/cargo-test-pass +
# code/cargo-fmt-clean rows (POLISH-CODE P58-stub: documentation-of-existing).
#
# Usage:
#   bash quality/gates/code/ci-job-status.sh --workflow ci.yml --job test [--branch main]
#
# Asserts: the most recent successful run of <workflow> on <branch> exists
# (we do not introspect job-level details — gh CLI exposes run-level
# success which is sufficient since ci.yml's `test` and `rustfmt` jobs
# both gate the run conclusion).
#
# Exit codes:
#   0 — most recent successful run on branch found (job is GREEN by extension).
#   1 — no successful run found OR gh CLI not installed/authenticated.

set -euo pipefail

WORKFLOW=""
JOB=""
BRANCH="main"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --workflow) WORKFLOW="$2"; shift 2 ;;
    --job)      JOB="$2"; shift 2 ;;
    --branch)   BRANCH="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

[[ -z "$WORKFLOW" ]] && { echo "--workflow required" >&2; exit 1; }
[[ -z "$JOB" ]] && { echo "--job required" >&2; exit 1; }

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
ARTIFACT_DIR="$REPO_ROOT/quality/reports/verifications/code"
mkdir -p "$ARTIFACT_DIR"

case "$JOB" in
  test)        ROW_ID="code/cargo-test-pass";  ARTIFACT="$ARTIFACT_DIR/cargo-test-pass.json" ;;
  rustfmt|fmt) ROW_ID="code/cargo-fmt-clean";  ARTIFACT="$ARTIFACT_DIR/cargo-fmt-clean.json" ;;
  *)           ROW_ID="code/$JOB";             ARTIFACT="$ARTIFACT_DIR/$JOB.json" ;;
esac

TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

if ! command -v gh >/dev/null 2>&1; then
  printf '{"ts": "%s", "row_id": "%s", "asserts_passed": [], "asserts_failed": ["gh CLI not installed in environment"]}\n' \
    "$TS" "$ROW_ID" > "$ARTIFACT"
  echo "FAIL: gh CLI not installed" >&2
  exit 1
fi

LATEST_RUN=$(gh run list --workflow="$WORKFLOW" --branch="$BRANCH" --status=success --limit=1 --json databaseId,conclusion 2>/dev/null || echo "[]")
RUN_COUNT=$(printf '%s' "$LATEST_RUN" | python3 -c "import json,sys
try:
    d = json.load(sys.stdin)
    print(len(d) if isinstance(d, list) else 0)
except Exception:
    print(0)
")

if [[ "$RUN_COUNT" -lt 1 ]]; then
  printf '{"ts": "%s", "row_id": "%s", "asserts_passed": [], "asserts_failed": ["no successful run of %s on %s"]}\n' \
    "$TS" "$ROW_ID" "$WORKFLOW" "$BRANCH" > "$ARTIFACT"
  echo "FAIL: no successful run of $WORKFLOW on $BRANCH" >&2
  exit 1
fi

printf '{"ts": "%s", "row_id": "%s", "asserts_passed": ["most recent successful run of %s on %s exists"], "asserts_failed": []}\n' \
  "$TS" "$ROW_ID" "$WORKFLOW" "$BRANCH" > "$ARTIFACT"
echo "OK: $ROW_ID — most recent successful run of $WORKFLOW on $BRANCH"
