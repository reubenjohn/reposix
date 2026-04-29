#!/usr/bin/env bash
# P74 UX-BIND-04 (D-06): polish2-06-landing claim asserts the connector
# capability matrix lives on docs/index.md. Verifier dual-greps for
# (a) a heading mentioning "connector" OR "backend" (the live heading is
# "## What each backend can do" — same intent, different word) AND
# (b) at least one markdown table row. Both required. Fires
# STALE_DOCS_DRIFT if either disappears.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
DOC="${REPO_ROOT}/docs/index.md"
if ! grep -qE '^## .*([Cc]onnector|[Bb]ackend)' "$DOC"; then
  echo "FAIL: docs/index.md has no '## ...connector...' or '## ...backend...' heading — capability matrix likely missing" >&2
  exit 1
fi
if ! grep -qE '^\| .* \| .* \|' "$DOC"; then
  echo "FAIL: docs/index.md has no markdown table rows — capability matrix table missing" >&2
  exit 1
fi
echo "PASS: docs/index.md has connector/backend heading + table row"
exit 0
