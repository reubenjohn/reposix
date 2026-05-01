#!/usr/bin/env bash
# P85 DVCS-DOCS-03: docs/guides/troubleshooting.md gains a "DVCS push/pull
# issues" section covering the four required entries: bus-remote `fetch
# first` rejection, attach reconciliation warnings, webhook race
# conditions, cache-desync recovery via `reposix sync --reconcile`.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
DOC="${REPO_ROOT}/docs/guides/troubleshooting.md"

if [[ ! -f "$DOC" ]]; then
  echo "FAIL: docs/guides/troubleshooting.md does not exist" >&2
  exit 1
fi

# Section anchor must exist.
if ! grep -qF '## DVCS push/pull issues' "$DOC"; then
  echo "FAIL: docs/guides/troubleshooting.md missing '## DVCS push/pull issues' section" >&2
  exit 1
fi

# Each of the four required entries must be present (sub-section headings).
for entry in \
  'Bus-remote `fetch first` rejection' \
  'Attach reconciliation warnings' \
  'Webhook race conditions' \
  'Cache-desync recovery via `reposix sync --reconcile`'; do
  if ! grep -qF -- "$entry" "$DOC"; then
    echo "FAIL: docs/guides/troubleshooting.md missing entry: $entry" >&2
    exit 1
  fi
done

# Reconciliation cases — five rows in the attach table per P79 architecture.
for case in 'no-id' 'backend-deleted' 'duplicate-id' 'mirror-lag'; do
  if ! grep -qF -- "$case" "$DOC"; then
    echo "FAIL: docs/guides/troubleshooting.md attach table missing case '$case'" >&2
    exit 1
  fi
done

# --force-with-lease semantics must be cited (webhook race section).
if ! grep -qF -- '--force-with-lease' "$DOC"; then
  echo "FAIL: docs/guides/troubleshooting.md does not cite --force-with-lease" >&2
  exit 1
fi

echo "PASS: docs/guides/troubleshooting.md DVCS section covers 4 required entries + 4 attach cases"
exit 0
