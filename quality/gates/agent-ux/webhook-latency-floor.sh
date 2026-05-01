#!/usr/bin/env bash
# CATALOG ROW: agent-ux/webhook-latency-floor
# CADENCE: pre-release
# INVARIANT: quality/reports/verifications/perf/webhook-latency.json
#            exists, parses, has p95_seconds <= 120 (falsifiable
#            threshold per ROADMAP P84 SC4).
#
# Status until P84-01 T05: FAIL (artifact does not exist yet).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

ARTIFACT="quality/reports/verifications/perf/webhook-latency.json"
test -f "$ARTIFACT" \
  || { echo "FAIL: $ARTIFACT does not exist"; exit 1; }
P95=$(python3 -c "import json,sys; print(json.load(open(sys.argv[1]))['p95_seconds'])" "$ARTIFACT" 2>/dev/null) \
  || { echo "FAIL: $ARTIFACT does not parse or lacks p95_seconds field"; exit 1; }
[ "$P95" -le 120 ] \
  || { echo "FAIL: p95_seconds=$P95 > 120s threshold (ROADMAP P84 SC4)"; exit 1; }

echo "PASS: $ARTIFACT p95=${P95}s within 120s threshold"
exit 0
