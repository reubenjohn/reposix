#!/usr/bin/env bash
# scripts/webhook-latency-measure.sh — owner-runnable n=10
# manual-edit pass against TokenWorld + reposix-tokenworld-mirror.
# Shipped in P84 T05; produces the headline real-TokenWorld
# number for the v0.13.0 latency artifact refresh.
#
# Prerequisites:
#   - gh auth status confirms repo + workflow scopes.
#   - Confluence webhook configured to dispatch reposix-mirror-sync.
#   - Edit access to TokenWorld pages.
#   - v0.13.x reposix-cli published with working binstall artifacts
#     AND non-yanked gix dep (see .planning/milestones/v0.13.0-phases/
#     SURPRISES-INTAKE.md § 2026-05-01 16:43 — synthetic-dispatch
#     measurement is gated on this).
#
# Output: refreshed quality/reports/verifications/perf/webhook-latency.json
# with method="real-tokenworld-manual-edit", n=10, real timings.
set -euo pipefail
REPO="reubenjohn/reposix-tokenworld-mirror"
REF="refs/mirrors/confluence-synced-at"
TIMINGS=$(mktemp)
trap 'rm -f "$TIMINGS"' EXIT

for i in $(seq 1 10); do
  echo ""
  echo "Iteration $i / 10:"
  echo "  1. Edit a TokenWorld page in your browser."
  echo "  2. Save the edit."
  echo "  3. Press ENTER here when the save completes."
  read -r
  T_EDIT=$(date +%s)
  PRIOR=$(gh api "repos/${REPO}/git/refs/${REF}" -q .object.sha 2>/dev/null || echo "")
  while true; do
    NEW=$(gh api "repos/${REPO}/git/refs/${REF}" -q .object.sha 2>/dev/null || echo "")
    if [ -n "$NEW" ] && [ "$NEW" != "$PRIOR" ]; then
      T_DONE=$(date +%s)
      echo "  -> ref-update observed after $((T_DONE - T_EDIT))s"
      echo "$((T_DONE - T_EDIT))" >> "$TIMINGS"
      break
    fi
    sleep 2
    if [ $(($(date +%s) - T_EDIT)) -gt 180 ]; then
      echo "  -> TIMEOUT (>180s); skipping iteration"
      break
    fi
  done
done

N=$(wc -l < "$TIMINGS")
if [ "$N" -lt 1 ]; then
  echo "ABORT: no timings captured; nothing to write."
  exit 1
fi
P50=$(sort -n "$TIMINGS" | awk -v n="$N" 'NR==int(n*0.5)+1')
P95=$(sort -n "$TIMINGS" | awk -v n="$N" 'NR==int(n*0.95)+1')
MAX=$(sort -n "$TIMINGS" | tail -1)
TS=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
VERDICT="PASS"
[ "${P95:-9999}" -gt 120 ] && VERDICT="FAIL"

cat > quality/reports/verifications/perf/webhook-latency.json <<JSON
{
  "measured_at": "${TS}",
  "method": "real-tokenworld-manual-edit",
  "n": ${N},
  "p50_seconds": ${P50:-0},
  "p95_seconds": ${P95:-0},
  "max_seconds": ${MAX:-0},
  "target_seconds": 60,
  "verdict": "${VERDICT}"
}
JSON

echo ""
echo "Wrote quality/reports/verifications/perf/webhook-latency.json"
echo "  method=real-tokenworld-manual-edit n=${N} p50=${P50}s p95=${P95}s max=${MAX}s verdict=${VERDICT}"
