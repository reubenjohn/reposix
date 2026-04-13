#!/usr/bin/env bash
# scripts/demos/02-guardrails.sh — Tier 1 security-story demo.
#
# AUDIENCE: security
# RUNTIME_SEC: 60
# REQUIRES: cargo, fusermount3, jq, sqlite3, git, curl, sed
# ASSERTS: "Permission denied" "refusing to push" "allow-bulk-delete" "DEMO COMPLETE"
#
# Narrative:
#   [1/4] setup sim + primary mount, bootstrap a repo via the helper.
#   [2/4] SG-01 — second mount with REPOSIX_ALLOWED_ORIGINS=
#         http://127.0.0.1:9999 (mismatch) -> ls on that mount fails
#         with Permission denied. Primary mount unaffected.
#   [3/4] SG-02 — rm 6 files + push is refused ("refusing to push ...
#         commit message tag '[allow-bulk-delete]' overrides").
#         Amend with the tag -> succeeds.
#   [4/4] SG-03 — curl PATCH with body frontmatter containing
#         version: 999999 -> GET shows server version unchanged
#         (sanitize-on-egress stripped it).

set -euo pipefail

# Self-wrap in `timeout 90` so a stuck step can't hang smoke.sh.
if [[ -z "${REPOSIX_DEMO_INNER:-}" ]]; then
    exec timeout 90 env REPOSIX_DEMO_INNER=1 bash "$0" "$@"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/_lib.sh"

require reposix-sim
require reposix-fuse
require git-remote-reposix
require git
require curl
require jq
require fusermount3

SIM_BIND="127.0.0.1:7802"
SIM_URL="http://${SIM_BIND}"
SIM_DB="/tmp/reposix-demo-02-sim.db"
MNT="/tmp/reposix-demo-02-mnt"
ALLOW_MNT="/tmp/reposix-demo-02-allow-mnt"
REPO="/tmp/reposix-demo-02-repo"
export SIM_BIND SIM_URL
_REPOSIX_TMP_PATHS+=("$REPO")
cleanup_trap

# Idempotent pre-clean.
fusermount3 -u "$MNT" 2>/dev/null || true
fusermount3 -u "$ALLOW_MNT" 2>/dev/null || true
rm -rf "$MNT" "$ALLOW_MNT" "$REPO" "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm" 2>/dev/null || true
pkill -f "reposix-sim --bind ${SIM_BIND}" 2>/dev/null || true
sleep 0.2

section "[1/4] start simulator + primary mount + bootstrap repo"
setup_sim "$SIM_DB"
setup_mount "$MNT" --project demo
wait_for_mount "$MNT" 10
mkdir -p "$REPO"
(
    cd "$REPO"
    git init -q
    git symbolic-ref HEAD refs/heads/main
    git config user.email "demo@reposix.local"
    git config user.name "reposix-demo"
    git remote add origin "reposix::${SIM_URL}/projects/demo"
    git fetch origin 2>&1 | sed 's/^/    /' || true
    git checkout -q -B main refs/reposix/origin/main
)
echo "primary mount + repo ready"

section "[2/4] SG-01 — outbound HTTP allowlist refusal"
echo "spawning a second mount with REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:9999 (mismatch)"
setup_mount "$ALLOW_MNT" --project demo --allowed-origins "http://127.0.0.1:9999"
sleep 1.5
echo "ls on allowlist-constrained mount (expect Permission denied):"
# `|| true` so set -e doesn't abort; we grep the output in assert.sh.
ls "$ALLOW_MNT" 2>&1 || true
echo "(stderr tail from the constrained mount)"
tail -5 "/tmp/reposix-demo-fuse-$(basename "$ALLOW_MNT").log" 2>/dev/null | sed 's/^/    /' || true
# Clean up the constrained mount before we move on so the primary mount
# isn't holding two FUSE daemons for the rest of the demo.
fusermount3 -u "$ALLOW_MNT" 2>/dev/null || true
sleep 0.3

section "[3/4] SG-02 — bulk-delete cap (cap=5; deleting 6 is refused)"
(
    cd "$REPO"
    git rm -q 0001.md 0002.md 0003.md 0004.md 0005.md 0006.md
    git commit -am "cleanup" -q
    set +e
    git push origin main 2>/tmp/reposix-demo-02-sg02.log
    PUSH_RC=$?
    set -e
    echo "first push exit code: $PUSH_RC (expect non-zero)"
    echo "stderr:"
    sed 's/^/    /' /tmp/reposix-demo-02-sg02.log
    if grep -q "refusing to push" /tmp/reposix-demo-02-sg02.log; then
        echo "SG-02 fired: server (or helper) refused the bulk delete"
    else
        echo "SG-02 did NOT fire -- failing demo" >&2
        exit 1
    fi
    git commit --amend -q -m "[allow-bulk-delete] cleanup"
    echo "second push (with override tag):"
    git push origin main 2>&1 | sed 's/^/    /'
)
echo "server issue count after override push:"
curl -s "${SIM_URL}/projects/demo/issues" | jq 'length'

section "[4/4] SG-03 — sanitize-on-egress: client-supplied version is stripped"
# Re-seed the simulator with a single issue so we have something to PATCH
# against after the bulk-delete above cleared the board.
POST_RESP=$(curl -s -X POST -H "content-type: application/json" \
    -d '{"title":"sg03-probe","body":"probe body","status":"open","labels":[]}' \
    "${SIM_URL}/projects/demo/issues")
ISSUE_ID=$(echo "$POST_RESP" | jq -r '.id')
echo "created probe issue id=$ISSUE_ID:"
echo "$POST_RESP" | jq '{id, version}'

# PATCH with frontmatter claiming version: 999999 in the body. The server
# must NOT let this overwrite its authoritative version — Tainted<T>
# sanitize() strips id/version/created_at/updated_at on ingress.
echo "PATCHing with body that contains \"version: 999999\" in-frontmatter:"
BODY=$(jq -nc --arg b $'---\nid: 99\ntitle: sg03-probe\nstatus: open\nversion: 999999\n---\nhello' '{body:$b}')
curl -s -X PATCH -H "content-type: application/json" \
    -H 'If-Match: "1"' \
    -d "$BODY" \
    "${SIM_URL}/projects/demo/issues/${ISSUE_ID}" | jq '{id, version}'

echo "server authoritative state (version MUST NOT be 999999):"
SERVER_VERSION=$(curl -s "${SIM_URL}/projects/demo/issues/${ISSUE_ID}" | jq -r '.version')
echo "server.version = $SERVER_VERSION"
if [[ "$SERVER_VERSION" == "999999" ]]; then
    echo "SG-03 FAILED: client overwrote authoritative version" >&2
    exit 1
fi
echo "SG-03 held: attacker-supplied version was stripped."

echo
echo "== DEMO COMPLETE =="
