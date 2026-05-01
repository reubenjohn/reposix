#!/usr/bin/env bash
# P85 DVCS-DOCS-01: docs/concepts/dvcs-topology.md ships and explains the
# three DVCS roles (SoT-holder, mirror-only consumer, round-tripper) plus
# the verbatim Q2.2 clarification about refs/mirrors/<sot-host>-synced-at.
# Body-hash drift on this verifier OR the doc fires STALE_DOCS_DRIFT via
# the walker.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
DOC="${REPO_ROOT}/docs/concepts/dvcs-topology.md"

if [[ ! -f "$DOC" ]]; then
  echo "FAIL: docs/concepts/dvcs-topology.md does not exist (DVCS-DOCS-01)" >&2
  exit 1
fi

# Three roles must each appear at least once (case-insensitive whole-token).
for role in 'SoT-holder' 'mirror-only consumer' 'round-tripper'; do
  if ! grep -qi -- "$role" "$DOC"; then
    echo "FAIL: docs/concepts/dvcs-topology.md missing role '$role'" >&2
    exit 1
  fi
done

# Q2.2 verbatim clarification: must contain the staleness-not-current phrasing.
# We check for the load-bearing phrase fragments rather than the exact verbatim
# string so prose can be reflowed without breaking the verifier.
for phrase in 'mirror last caught up' 'NOT a' 'current SoT state'; do
  if ! grep -qF -- "$phrase" "$DOC"; then
    echo "FAIL: docs/concepts/dvcs-topology.md missing Q2.2 phrase fragment '$phrase'" >&2
    exit 1
  fi
done

echo "PASS: docs/concepts/dvcs-topology.md ships with three roles + Q2.2 phrasing"
exit 0
