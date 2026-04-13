#!/usr/bin/env bash
# scripts/demos/03-conflict-resolution.sh — Tier 1 conflict-is-git-merge demo.
#
# AUDIENCE: skeptic
# RUNTIME_SEC: 60
# REQUIRES: cargo, fusermount3, jq, sqlite3, git, curl, sed
# ASSERTS: "409" "version_mismatch" "DEMO COMPLETE"
#
# Narrative:
#   [1/5] setup sim.
#   [2/5] curl GET issue 1 -> record V1.
#   [3/5] "agent A" PATCHes issue 1 with If-Match: "V1" -> success, V2.
#   [4/5] "agent B" PATCHes issue 1 with stale If-Match: "V1" -> 409.
#   [5/5] pretty-print the 409 body and explain that in the git-push
#         path this is exactly what becomes a merge conflict.

set -euo pipefail

# Self-wrap in `timeout 90` so a stuck step can't hang smoke.sh.
if [[ -z "${REPOSIX_DEMO_INNER:-}" ]]; then
    exec timeout 90 env REPOSIX_DEMO_INNER=1 bash "$0" "$@"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/_lib.sh"

require reposix-sim
require curl
require jq
require fusermount3

SIM_BIND="127.0.0.1:7803"
SIM_URL="http://${SIM_BIND}"
SIM_DB="/tmp/reposix-demo-03-sim.db"
export SIM_BIND SIM_URL
cleanup_trap

# Idempotent pre-clean.
rm -f "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm" 2>/dev/null || true
pkill -f "reposix-sim --bind ${SIM_BIND}" 2>/dev/null || true
sleep 0.2

section "[1/5] start simulator"
setup_sim "$SIM_DB"
echo "sim ready at $SIM_URL"

section "[2/5] read issue 1 and record its current version V1"
V1=$(curl -s "${SIM_URL}/projects/demo/issues/1" | jq -r '.version')
echo "V1 = $V1"

section "[3/5] agent A PATCHes with If-Match: \"$V1\" — expects success, version bumps to V2"
RESP_A=$(curl -s -w "\n__HTTP__ %{http_code}" -X PATCH \
    -H "content-type: application/json" \
    -H "If-Match: \"$V1\"" \
    -d '{"status":"in_progress"}' \
    "${SIM_URL}/projects/demo/issues/1")
CODE_A=$(echo "$RESP_A" | awk '/^__HTTP__/{print $2}')
BODY_A=$(echo "$RESP_A" | sed '/^__HTTP__/d')
echo "agent A HTTP code: $CODE_A"
echo "agent A response:"
echo "$BODY_A" | jq '{id, status, version}'
V2=$(echo "$BODY_A" | jq -r '.version')
echo "V2 = $V2"
if [[ "$CODE_A" != "200" ]]; then
    echo "ERROR: agent A PATCH did not return 200" >&2
    exit 1
fi

section "[4/5] agent B PATCHes with STALE If-Match: \"$V1\" — expects 409 version_mismatch"
RESP_B=$(curl -s -w "\n__HTTP__ %{http_code}" -X PATCH \
    -H "content-type: application/json" \
    -H "If-Match: \"$V1\"" \
    -d '{"status":"in_review"}' \
    "${SIM_URL}/projects/demo/issues/1")
CODE_B=$(echo "$RESP_B" | awk '/^__HTTP__/{print $2}')
BODY_B=$(echo "$RESP_B" | sed '/^__HTTP__/d')
echo "agent B HTTP code: $CODE_B"
echo "agent B body (pretty-printed):"
echo "$BODY_B" | jq . 2>/dev/null || echo "$BODY_B"
if [[ "$CODE_B" != "409" ]]; then
    echo "ERROR: agent B PATCH returned $CODE_B; expected 409" >&2
    exit 1
fi
if ! echo "$BODY_B" | grep -q "version_mismatch"; then
    echo "ERROR: 409 body did not mention version_mismatch" >&2
    exit 1
fi

section "[5/5] what git sees on the same path"
cat <<'EOF'
In the git-push path, this same 409 is what git turns into a merge
conflict in the real push path: the helper reads the current server
version before mutating, sends an If-Match with that version, and
surfaces the 409 as a "remote rejected" so the agent can `git pull
--rebase` to resolve. No special UI, no bespoke reconciliation
protocol — optimistic concurrency on HTTP etags, rendered back
through git.
EOF

echo
echo "== DEMO COMPLETE =="
