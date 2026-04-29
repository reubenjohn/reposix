#!/usr/bin/env bash
# P74 UX-BIND-04 (D-06): polish2-06-landing claim asserts the connector
# capability matrix lives on docs/index.md. Verifier dual-greps for
# (a) a heading mentioning "connector" (the live heading is
# "## Connector capability matrix" — literal claim+heading match
# (P77 narrow following P74 widen)) AND
# (b) at least one markdown table row. Both required. Fires
# STALE_DOCS_DRIFT if either disappears.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
DOC="${REPO_ROOT}/docs/index.md"
if ! grep -qE '^## .*[Cc]onnector' "$DOC"; then
  echo "FAIL: docs/index.md has no '## ...connector...' heading — capability matrix likely missing" >&2
  exit 1
fi
if ! grep -qE '^\| .* \| .* \|' "$DOC"; then
  echo "FAIL: docs/index.md has no markdown table rows — capability matrix table missing" >&2
  exit 1
fi
echo "PASS: docs/index.md has connector heading + table row"
exit 0
