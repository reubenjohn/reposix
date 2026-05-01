#!/usr/bin/env bash
# P85 DVCS-DOCS-02: docs/guides/dvcs-mirror-setup.md ships an end-to-end
# webhook + GH Action walk-through; documents backends-without-webhooks
# fallback (Q4.2); documents cleanup procedure. The doc references the
# template at docs/guides/dvcs-mirror-setup-template.yml (P84).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
DOC="${REPO_ROOT}/docs/guides/dvcs-mirror-setup.md"
TEMPLATE="${REPO_ROOT}/docs/guides/dvcs-mirror-setup-template.yml"

if [[ ! -f "$DOC" ]]; then
  echo "FAIL: docs/guides/dvcs-mirror-setup.md does not exist (DVCS-DOCS-02)" >&2
  exit 1
fi
if [[ ! -f "$TEMPLATE" ]]; then
  echo "FAIL: docs/guides/dvcs-mirror-setup-template.yml (P84) missing — referenced by setup guide" >&2
  exit 1
fi

# Walk-through sections — each load-bearing step must be present.
for section in 'Step 1' 'Step 2' 'Step 3' 'Step 4' 'Step 5' 'Cleanup' 'Backends without webhooks'; do
  if ! grep -qF -- "$section" "$DOC"; then
    echo "FAIL: docs/guides/dvcs-mirror-setup.md missing section '$section'" >&2
    exit 1
  fi
done

# Required commands the owner runs (sample of the 80% most likely to drift).
for cmd in 'gh repo create' 'gh secret set' 'gh workflow disable'; do
  if ! grep -qF -- "$cmd" "$DOC"; then
    echo "FAIL: docs/guides/dvcs-mirror-setup.md missing command '$cmd'" >&2
    exit 1
  fi
done

# Cross-link to template must resolve.
if ! grep -qF 'dvcs-mirror-setup-template.yml' "$DOC"; then
  echo "FAIL: docs/guides/dvcs-mirror-setup.md does not reference the workflow template" >&2
  exit 1
fi

echo "PASS: docs/guides/dvcs-mirror-setup.md ships walk-through + cron-only fallback + cleanup"
exit 0
