#!/usr/bin/env bash
# CATALOG ROW: agent-ux/webhook-cron-fallback
# CADENCE: pre-pr
# INVARIANT: Workflow YAML has literal cron '*/30 * * * *' (D-06; NEVER vars.*),
#            actions/checkout@v6 with fetch-depth: 0 (D-04 / Pitfall 4),
#            concurrency: { group: reposix-mirror-sync, cancel-in-progress: false } (D-01).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

TEMPLATE="docs/guides/dvcs-mirror-setup-template.yml"
test -f "$TEMPLATE" || { echo "FAIL: $TEMPLATE missing"; exit 1; }

grep -qF "'*/30 * * * *'" "$TEMPLATE" \
  || { echo "FAIL: missing literal cron '*/30 * * * *' (D-06)"; exit 1; }
if grep -E 'cron:\s*.*\$\{\{' "$TEMPLATE" >/dev/null 2>&1; then
  echo "FAIL: cron field uses \${{ ... }} interpolation (Pitfall 3)"
  exit 1
fi
grep -q "actions/checkout@v6" "$TEMPLATE" \
  || { echo "FAIL: missing actions/checkout@v6"; exit 1; }
grep -qE "fetch-depth:\s*0" "$TEMPLATE" \
  || { echo "FAIL: missing fetch-depth: 0 (D-04 / Pitfall 4)"; exit 1; }
grep -q "group: reposix-mirror-sync" "$TEMPLATE" \
  || { echo "FAIL: missing concurrency group (D-01)"; exit 1; }
grep -qE "cancel-in-progress:\s*false" "$TEMPLATE" \
  || { echo "FAIL: missing cancel-in-progress: false (D-01)"; exit 1; }

echo "PASS: cron literal + fetch-depth + concurrency invariants hold"
exit 0
