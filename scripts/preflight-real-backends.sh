#!/usr/bin/env bash
# preflight-real-backends.sh — verify the three sanctioned real-backend test targets
# are reachable + auth works + the named test target exists.
#
# No canonical home under quality/gates/ -- canonical for its own domain
# (dev-loop pre-flight check registered at orphan-scripts/preflight-real-backends-sh).
#
# Sanctioned targets (per docs/reference/testing-targets.md):
#   - Confluence: TokenWorld space (also probes REPOSIX defensively per the May 2 audit gap)
#   - GitHub:     reubenjohn/reposix issues
#   - JIRA:       project ${JIRA_TEST_PROJECT:-TEST}
#
# Reads creds from .env (auto-sourced if present) or the existing shell env.
# Exits 0 if every probe returns HTTP 200 + a non-empty result.
# Exits 1 if any sanctioned target is unreachable / auth fails / target missing.
# Exits 2 if no creds are configured at all (fail-closed default).
#
# Use BEFORE kicking off any phase that depends on a real backend (P91 onward).
# Run BEFORE milestone-close (P97) per RBF-FW-03 / OP-1.
#
# Idempotent + read-only — issues GET-only HTTP requests; no mutations.

set -euo pipefail

# Source .env if present and creds not already exported.
if [ -f ./.env ] && [ -z "${ATLASSIAN_API_KEY:-}${GITHUB_TOKEN:-}${JIRA_API_TOKEN:-}" ]; then
  set -a
  # shellcheck disable=SC1091
  . ./.env
  set +a
fi

PASS=0
FAIL=0
SKIP=0

probe_pass() { printf "  \033[32mPASS\033[0m  %s\n" "$1"; PASS=$((PASS+1)); }
probe_fail() { printf "  \033[31mFAIL\033[0m  %s\n" "$1"; FAIL=$((FAIL+1)); }
probe_skip() { printf "  \033[33mSKIP\033[0m  %s\n" "$1"; SKIP=$((SKIP+1)); }

echo "=== Confluence (sanctioned: TokenWorld) ==="
if [ -n "${ATLASSIAN_EMAIL:-}" ] && [ -n "${ATLASSIAN_API_KEY:-}" ] && [ -n "${REPOSIX_CONFLUENCE_TENANT:-}" ]; then
  body=$(curl -sS -u "$ATLASSIAN_EMAIL:$ATLASSIAN_API_KEY" \
    "https://$REPOSIX_CONFLUENCE_TENANT.atlassian.net/wiki/api/v2/spaces?keys=TokenWorld" \
    -w "\nHTTP_CODE:%{http_code}\n" 2>&1) || {
      probe_fail "Confluence — curl error"
      :
    }
  code=$(printf '%s' "$body" | awk -F: '/^HTTP_CODE:/ {print $2}')
  json=$(printf '%s' "$body" | sed '/^HTTP_CODE:/d')
  if [ "$code" = "200" ]; then
    result=$(printf '%s' "$json" | python3 -c "import sys,json; d=json.load(sys.stdin); rs=d.get('results',[]); print(rs[0]['name'] if rs else 'EMPTY')" 2>/dev/null || echo "PARSE_ERROR")
    if [ "$result" = "EMPTY" ]; then
      probe_fail "Confluence key=TokenWorld — auth OK but space not found (was the space renamed?)"
    else
      probe_pass "Confluence key=TokenWorld — space \"$result\" reachable"
    fi
  else
    probe_fail "Confluence key=TokenWorld — HTTP $code"
  fi
else
  probe_skip "Confluence — set ATLASSIAN_EMAIL + ATLASSIAN_API_KEY + REPOSIX_CONFLUENCE_TENANT in .env"
fi

echo
echo "=== GitHub (reubenjohn/reposix issues) ==="
if [ -n "${GITHUB_TOKEN:-}" ]; then
  body=$(curl -sS -H "Authorization: token $GITHUB_TOKEN" \
    -H "Accept: application/vnd.github+json" \
    "https://api.github.com/repos/reubenjohn/reposix" \
    -w "\nHTTP_CODE:%{http_code}\n" 2>&1) || {
      probe_fail "GitHub — curl error"
      :
    }
  code=$(printf '%s' "$body" | awk -F: '/^HTTP_CODE:/ {print $2}')
  json=$(printf '%s' "$body" | sed '/^HTTP_CODE:/d')
  if [ "$code" = "200" ]; then
    result=$(printf '%s' "$json" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('full_name','?'), 'open_issues='+str(d.get('open_issues_count','?')), 'private='+str(d.get('private','?')))" 2>/dev/null || echo "PARSE_ERROR")
    probe_pass "GitHub — $result"
  else
    probe_fail "GitHub — HTTP $code"
  fi
else
  probe_skip "GitHub — set GITHUB_TOKEN in .env"
fi

echo
echo "=== JIRA (project ${JIRA_TEST_PROJECT:-TEST}) ==="
if [ -n "${JIRA_EMAIL:-}" ] && [ -n "${JIRA_API_TOKEN:-}" ] && [ -n "${REPOSIX_JIRA_INSTANCE:-}" ]; then
  proj=${JIRA_TEST_PROJECT:-${REPOSIX_JIRA_PROJECT:-TEST}}
  # Mirror reposix-jira's contract (client.rs: format!("https://{tenant}.atlassian.net")):
  # REPOSIX_JIRA_INSTANCE is a bare subdomain; accept a full host for back-compat.
  case "$REPOSIX_JIRA_INSTANCE" in
    *.*) jira_host=$REPOSIX_JIRA_INSTANCE ;;
    *)   jira_host="$REPOSIX_JIRA_INSTANCE.atlassian.net" ;;
  esac
  body=$(curl -sS -u "$JIRA_EMAIL:$JIRA_API_TOKEN" \
    "https://$jira_host/rest/api/3/project/$proj" \
    -w "\nHTTP_CODE:%{http_code}\n" 2>&1) || {
      probe_fail "JIRA — curl error"
      :
    }
  code=$(printf '%s' "$body" | awk -F: '/^HTTP_CODE:/ {print $2}')
  json=$(printf '%s' "$body" | sed '/^HTTP_CODE:/d')
  if [ "$code" = "200" ]; then
    result=$(printf '%s' "$json" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('key','?'), '\"'+d.get('name','?')+'\"', d.get('projectTypeKey','?'))" 2>/dev/null || echo "PARSE_ERROR")
    probe_pass "JIRA — $result"
  else
    probe_fail "JIRA — HTTP $code (project=$proj)"
  fi
else
  probe_skip "JIRA — set JIRA_EMAIL + JIRA_API_TOKEN + REPOSIX_JIRA_INSTANCE in .env"
fi

echo
echo "=== Summary ==="
printf "  pass=%d  fail=%d  skip=%d\n" "$PASS" "$FAIL" "$SKIP"

if [ "$FAIL" -gt 0 ]; then
  echo "  RESULT: FAIL — fix auth or backend reachability before any real-backend phase (P91+)."
  exit 1
fi
if [ "$PASS" -eq 0 ]; then
  echo "  RESULT: NO CREDS — populate .env per docs/reference/testing-targets.md and re-run."
  exit 2
fi
echo "  RESULT: PASS — sanctioned real-backend targets reachable. Safe to start P91+."
exit 0
