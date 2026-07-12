#!/usr/bin/env bash
# ci-wait.sh — bounded-poll wait for a GitHub Actions run to conclude.
#
# WHY THIS EXISTS
# ---------------
# Background `gh run watch` HANGS indefinitely when it is pointed at a run
# that has ALREADY concluded (it waits for an in-progress transition that
# never comes). This burned two autonomous sessions — hang IDs
# `bulqmsyrv` and `biy9yxt33` — where a GREEN-already run left the watcher
# blocked forever. This helper is the promote-ad-hoc-bash fix (CLAUDE.md
# OP-4): a committed, bounded-poll replacement that returns IMMEDIATELY on
# an already-`completed` run and can NEVER hang past its hard timeout.
#
# It replaces flaky background `gh run watch` in autonomous CI-wait loops.
#
# USAGE
#   scripts/ci-wait.sh [<run-id>]
#     <run-id>  optional. Default: the latest `ci.yml` run on branch main.
#
# ENV KNOBS
#   CI_WAIT_INTERVAL  poll interval in seconds   (default 20)
#   CI_WAIT_TIMEOUT   hard timeout in seconds     (default 900)
#   CI_WAIT_BRANCH    branch for latest-run lookup (default main)
#   CI_WAIT_WORKFLOW  workflow file for lookup     (default ci.yml)
#
# EXIT CODES
#   0  the run concluded with conclusion == success
#   1  the run concluded non-success (failure/cancelled/timed_out/
#      action_required/startup_failure/...) OR a lookup/query error
#   2  the hard timeout elapsed while the run was still in progress
#      (the run URL is printed for manual follow-up)
set -euo pipefail

INTERVAL="${CI_WAIT_INTERVAL:-20}"
TIMEOUT="${CI_WAIT_TIMEOUT:-900}"
BRANCH="${CI_WAIT_BRANCH:-main}"
WORKFLOW="${CI_WAIT_WORKFLOW:-ci.yml}"

command -v gh > /dev/null 2>&1 || {
  echo "ci-wait: 'gh' CLI not found on PATH — install it or run 'gh auth login'" >&2
  exit 1
}

RUN_ID="${1:-}"

if [[ -z "${RUN_ID}" ]]; then
  # Resolve the latest ci.yml run on the target branch.
  RUN_ID="$(gh run list --branch "${BRANCH}" --workflow "${WORKFLOW}" \
    --limit 1 --json databaseId --jq '.[0].databaseId' 2>/dev/null || true)"
  if [[ -z "${RUN_ID}" || "${RUN_ID}" == "null" ]]; then
    echo "ci-wait: no '${WORKFLOW}' run found on branch '${BRANCH}'" >&2
    exit 1
  fi
  echo "ci-wait: resolved latest '${WORKFLOW}' run on '${BRANCH}' -> ${RUN_ID}"
fi

STATUS=""
CONCLUSION=""
URL=""

# Populate STATUS / CONCLUSION / URL for RUN_ID. Returns non-zero on a
# gh query failure so the caller can fail closed rather than loop on stale
# values.
query_run() {
  local json
  json="$(gh run view "${RUN_ID}" --json status,conclusion,url 2>/dev/null)" || return 1
  STATUS="$(printf '%s' "${json}" | gh_jq '.status')"
  CONCLUSION="$(printf '%s' "${json}" | gh_jq '.conclusion')"
  URL="$(printf '%s' "${json}" | gh_jq '.url')"
}

# Small jq shim via `gh`-independent parsing: prefer jq, fall back to a
# minimal python parse so the helper works without jq installed.
gh_jq() {
  local filter="$1" input
  input="$(cat)"
  if command -v jq > /dev/null 2>&1; then
    printf '%s' "${input}" | jq -r "${filter} // \"\""
  else
    CI_WAIT_JSON="${input}" CI_WAIT_KEY="${filter#.}" python3 -c '
import json, os
d = json.loads(os.environ["CI_WAIT_JSON"] or "{}")
print(d.get(os.environ["CI_WAIT_KEY"]) or "")'
  fi
}

START="$(date +%s)"

while true; do
  if ! query_run; then
    echo "ci-wait: failed to query run ${RUN_ID} (gh run view error)" >&2
    exit 1
  fi
  ELAPSED="$(( $(date +%s) - START ))"

  # ALREADY-CONCLUDED FAST-PATH — this is the exact case that hung
  # `gh run watch`. If the first query already shows `completed`, decide
  # NOW; never loop-wait on a concluded run.
  if [[ "${STATUS}" == "completed" ]]; then
    printf 'ci-wait: run %s completed (conclusion=%s, elapsed=%ss)\n' \
      "${RUN_ID}" "${CONCLUSION}" "${ELAPSED}"
    if [[ "${CONCLUSION}" == "success" ]]; then
      exit 0
    fi
    echo "ci-wait: run ${RUN_ID} concluded non-success (${CONCLUSION}) — ${URL}" >&2
    exit 1
  fi

  printf 'ci-wait: run %s status=%s elapsed=%ss (interval=%ss timeout=%ss)\n' \
    "${RUN_ID}" "${STATUS:-unknown}" "${ELAPSED}" "${INTERVAL}" "${TIMEOUT}"

  if [[ "${ELAPSED}" -ge "${TIMEOUT}" ]]; then
    echo "ci-wait: hard timeout (${TIMEOUT}s) — run ${RUN_ID} still '${STATUS}': ${URL}" >&2
    exit 2
  fi

  sleep "${INTERVAL}"
done
