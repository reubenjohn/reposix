#!/usr/bin/env bash
# probe-confluence.sh — one-command Atlassian Confluence credentials probe.
#
# Purpose: after you set/fix ATLASSIAN_EMAIL and REPOSIX_CONFLUENCE_TENANT in
# .env, run this to verify the token authenticates against your tenant before
# running `reposix mount --backend confluence`. Writes NO files outside /tmp
# and never echoes the token value.
#
# Why this script exists: Phase 11 (v0.3) shipped the Confluence adapter on an
# evening where we couldn't authenticate to discover the user's tenant — the
# git-email wasn't the Atlassian-login-email. This script is the 30-second
# recovery path: set the right env vars, run it, read the green/red result.
#
# Exits 0 on auth success, 1 on any auth/connectivity failure.

set -euo pipefail

cd "$(dirname "$0")/.."

# Load .env if present.
if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

: "${ATLASSIAN_API_KEY:?missing — add to .env}"
: "${ATLASSIAN_EMAIL:?missing — set to the email shown at https://id.atlassian.com/manage-profile/security/api-tokens}"
: "${REPOSIX_CONFLUENCE_TENANT:?missing — set to the <foo> in https://<foo>.atlassian.net}"

TENANT="$REPOSIX_CONFLUENCE_TENANT"
BASE="https://${TENANT}.atlassian.net"

echo "== probe: ${ATLASSIAN_EMAIL} @ ${BASE} =="

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# Auth + reachability in one shot: Confluence REST v2 spaces list.
# Note: do NOT use /rest/api/3/myself (Jira) — it returns 404 on a
# Confluence-only site, which looks like an auth failure but isn't.
# id.atlassian.com API tokens authenticate tenant-specifically under Basic;
# /wiki/api/v2/spaces is the canonical tenant-scoped probe.
HTTP=$(curl -sS -o "$TMP/spaces.json" -w '%{http_code}' \
  -u "${ATLASSIAN_EMAIL}:${ATLASSIAN_API_KEY}" \
  -H 'Accept: application/json' \
  "${BASE}/wiki/api/v2/spaces?limit=5")
echo "  GET /wiki/api/v2/spaces → HTTP ${HTTP}"

if [[ "$HTTP" != "200" ]]; then
  echo "  FAIL: auth rejected or Confluence unreachable. Verify:"
  echo "    - ATLASSIAN_EMAIL matches the account that issued the token"
  echo "    - REPOSIX_CONFLUENCE_TENANT is the subdomain (no .atlassian.net suffix)"
  echo "    - the token has not been revoked at id.atlassian.com"
  head -c 400 "$TMP/spaces.json" 2>/dev/null || true
  echo
  exit 1
fi

# Count spaces and extract the first key (if any).
SPACE_COUNT=$(grep -o '"key":"[^"]*"' "$TMP/spaces.json" | wc -l)
FIRST_KEY=$(grep -o '"key":"[^"]*"' "$TMP/spaces.json" | head -1 | cut -d'"' -f4 || echo "")
FIRST_ID=$(grep -o '"id":"[^"]*"' "$TMP/spaces.json" | head -1 | cut -d'"' -f4 || echo "")
echo "  spaces visible: ${SPACE_COUNT}"
if [[ -n "$FIRST_KEY" ]]; then
  echo "  first space: key=${FIRST_KEY} id=${FIRST_ID}"
fi

# 3. If --create-space, create a fresh space for the demo.
if [[ "${1:-}" == "--create-space" ]]; then
  KEY="${2:-REPOSIX}"
  NAME="${3:-reposix demo space}"
  echo "== creating space key=${KEY} name='${NAME}' =="
  HTTP=$(curl -sS -o "$TMP/create.json" -w '%{http_code}' \
    -u "${ATLASSIAN_EMAIL}:${ATLASSIAN_API_KEY}" \
    -H 'Accept: application/json' \
    -H 'Content-Type: application/json' \
    -X POST \
    --data "{\"key\":\"${KEY}\",\"name\":\"${NAME}\"}" \
    "${BASE}/wiki/api/v2/spaces")
  echo "  POST /wiki/api/v2/spaces → HTTP ${HTTP}"
  case "$HTTP" in
    200|201)
      echo "  created."
      CREATED_ID=$(grep -o '"id":"[^"]*"' "$TMP/create.json" | head -1 | cut -d'"' -f4)
      echo "  id=${CREATED_ID} key=${KEY}"
      ;;
    409)
      echo "  space key already exists — that's fine, reuse it."
      ;;
    403)
      echo "  FAIL: permission denied (token lacks space:create scope, or Confluence admin required)."
      head -c 400 "$TMP/create.json" 2>/dev/null || true
      echo
      exit 1
      ;;
    *)
      echo "  FAIL: unexpected status"
      head -c 400 "$TMP/create.json" 2>/dev/null || true
      echo
      exit 1
      ;;
  esac
fi

echo
echo "== OK — auth works, Confluence reachable. =="
echo "Next: REPOSIX_ALLOWED_ORIGINS=\"http://127.0.0.1:*,${BASE}\" \\"
echo "      reposix list --backend confluence --project <SPACE_KEY>"
