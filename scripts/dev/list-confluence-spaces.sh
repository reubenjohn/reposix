#!/usr/bin/env bash
# scripts/dev/list-confluence-spaces.sh — list available Confluence spaces.
#
# Useful for discovering the correct space key when setting up .env.
# Reads credentials from env vars (source .env first) or accepts them inline.
#
# Usage:
#   source .env  # or set -a; . .env; set +a (tilde-safe workaround)
#   bash scripts/dev/list-confluence-spaces.sh

set -euo pipefail

: "${ATLASSIAN_API_KEY:?need ATLASSIAN_API_KEY}"
: "${ATLASSIAN_EMAIL:?need ATLASSIAN_EMAIL}"
: "${REPOSIX_CONFLUENCE_TENANT:?need REPOSIX_CONFLUENCE_TENANT}"

URL="https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net/wiki/api/v2/spaces?limit=50"
echo "Fetching spaces from ${URL} ..."
curl -sf -u "${ATLASSIAN_EMAIL}:${ATLASSIAN_API_KEY}" "$URL" \
  | python3 -c "
import sys, json
d = json.load(sys.stdin)
results = d.get('results', [])
print(f'Found {len(results)} spaces:')
print(f'  {\"KEY\":<20} {\"NAME\":<40} {\"TYPE\":<12}')
print(f'  {\"-\"*20} {\"-\"*40} {\"-\"*12}')
for s in results:
    print(f'  {s[\"key\"]:<20} {s[\"name\"][:40]:<40} {s[\"type\"]:<12}')
"
