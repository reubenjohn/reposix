#!/usr/bin/env bash
# real-backend-env-gate.sh — kcov coverage harness for the B4/B5
# pre-release-real-backend verifiers' no-creds env-gate path (quick
# 260712-phc). Drives BOTH new scripts with every real-backend cred/allowlist
# var unset so kcov credits their env-gate setup + missing-var loop +
# artifact-write lines, without ever reaching cargo/init/network. The REAL
# scenario body (two-cache Confluence conflict scenario / GitHub front-door
# round-trip) stays unexercised here by design -- that PASS grading is a
# separate, owner-gated real-backend run, not this coverage harness's job.
#
# Harness contract: exit 0 regardless of target exits. See 00-probe.sh.
set -eu

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"

run_env_gated() {
  # run_env_gated <path> — execute target with ALL real-backend creds +
  # allowlist unset; ignore its exit (75/NOT-VERIFIED is the honest outcome,
  # a real gate result reported by the gate's own catalog row, not by this
  # coverage harness).
  local target="$1"
  env -u GITHUB_TOKEN -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL \
      -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_ALLOWED_ORIGINS \
      -u JIRA_EMAIL -u JIRA_API_TOKEN -u REPOSIX_JIRA_INSTANCE \
    bash "$REPO_ROOT/$target" >/dev/null 2>&1 || true
}

run_env_gated quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh
run_env_gated quality/gates/agent-ux/github-front-door-real-backend.sh

exit 0
