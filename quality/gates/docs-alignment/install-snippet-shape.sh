#!/usr/bin/env bash
# P74 UX-BIND-01 (D-03): docs/index.md:19 advertises the 5-line-install
# claim. This verifier shape-checks line 19 — asserts the line lists
# all four install channels (curl|brew|cargo binstall|irm). Body-hash
# drift on either this verifier OR docs/index.md:19 fires STALE_DOCS_DRIFT
# via the walker.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
DOC="${REPO_ROOT}/docs/index.md"
LINE="$(sed -n '19p' "$DOC")"
for tok in 'curl' 'brew' 'cargo binstall' 'irm'; do
  if ! printf '%s' "$LINE" | grep -qF -- "$tok"; then
    echo "FAIL: docs/index.md:19 missing install-channel token '$tok' — line was: $LINE" >&2
    exit 1
  fi
done
echo "PASS: docs/index.md:19 advertises curl|brew|cargo binstall|irm"
exit 0
