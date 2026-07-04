#!/usr/bin/env bash
# quality/gates/agent-ux/milestone-close-vision-litmus.sh — RBF-FW-03 SLOT
#
# Substrate dependency: P91 (real-backend attach), P92 (audit log fixes),
# P93 (cache-coherence), P94 (bus tree), P95 (claim qualifier).
# Until those land, this verifier legitimately returns NOT-VERIFIED via
# one of two paths:
#   1. cadence: pre-release-real-backend env-gate short-circuits before
#      the script runs (no env -> runner sets NOT-VERIFIED via _realbackend.is_skipped)
#   2. env IS set but substrate not landed -> script writes NOT-VERIFIED
#      artifact directly AND exits 75 (NOT-VERIFIED convention) -> runner
#      maps exit-75 -> NOT-VERIFIED status via _realbackend.map_exit_code_to_status
#      (exit 1 would let the runner overwrite to FAIL, destroying the
#      honest deferral signal)
#
# blast_radius: P0 on the catalog row STILL blocks milestone-close grading
# because NOT-VERIFIED at P0 fails the milestone-close gate (any P0 row
# not GREEN blocks).
#
# NEVER add a `waiver` block to the corresponding catalog row — that is
# the explicit anti-C7 (self-licensing-deferral-loop) cut.
#
# Implements catalog row agent-ux/milestone-close-vision-litmus-real-backend.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

ARTIFACT="${REPO_ROOT}/quality/reports/verifications/agent-ux/milestone-close-vision-litmus-real-backend.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Defensive: if we got past the runner's env-gate short-circuit, env IS set.
# But the substrate to actually run the third-arm scenario doesn't exist yet.
# Write a NOT-VERIFIED artifact with explicit substrate-deferred reason.
cat > "$ARTIFACT" <<EOF
{"ts":"${TS}","row_id":"agent-ux/milestone-close-vision-litmus-real-backend","exit_code":75,"status":"NOT-VERIFIED","reason":"substrate_not_landed","blocked_on":["P91","P92","P93","P94","P95"],"asserts_passed":[],"asserts_failed":["substrate not landed: P91 attach + P92 audit log + P93 cache-coherence + P94 bus tree + P95 claim qualifier all required"]}
EOF

echo "NOT-VERIFIED: substrate not landed (depends on P91+P92+P93+P94+P95)" >&2
echo "  artifact: ${ARTIFACT}" >&2
echo "  see: quality/dispatch/milestone-close-verdict.md probe #9" >&2
echo "  exit 75 (sysexits.h EX_TEMPFAIL repurposed) -> runner preserves NOT-VERIFIED" >&2
# exit 75 (NOT-VERIFIED convention), NOT exit 1.
# 89-03 ships _realbackend.map_exit_code_to_status which maps 75 ->
# "NOT-VERIFIED" in run.py's exit-code -> status branch. Combined with
# blast_radius: P0 on the row, this guarantees the milestone-close grading
# attempt cannot succeed until the substrate lands AND the runner does NOT
# report this as FAIL (which would muddy the C7 anti-pattern guard signal).
exit 75
