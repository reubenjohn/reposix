#!/usr/bin/env bash
# scripts/demos/01-edit-and-push.sh — Tier 1 core value prop demo.
#
# AUDIENCE: developer
# RUNTIME_SEC: 60
# REQUIRES: cargo, fusermount3, jq, sqlite3, git, curl, sed
# ASSERTS: "DEMO COMPLETE" "status: in_progress" "in_review"
#
# Narrative: setup -> ls -> cat -> sed-edit-via-FUSE -> curl verifies
# server state -> git clone via reposix:: remote -> git commit -> git push
# -> curl verifies again.

set -euo pipefail

# Self-wrap in `timeout 90` so a stuck sub-step cannot hang smoke.sh.
# REPOSIX_DEMO_INNER=1 marks us as already wrapped so we don't recurse.
if [[ -z "${REPOSIX_DEMO_INNER:-}" ]]; then
    exec timeout 90 env REPOSIX_DEMO_INNER=1 bash "$0" "$@"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/_lib.sh"

# ------------------------------------------------------------ prereqs
require reposix-sim
require reposix-fuse
require git-remote-reposix
require git
require curl
require jq
require fusermount3

# ------------------------------------------------------------ config
SIM_BIND="127.0.0.1:7801"
SIM_URL="http://${SIM_BIND}"
SIM_DB="/tmp/reposix-demo-01-sim.db"
MNT="/tmp/reposix-demo-01-mnt"
REPO="/tmp/reposix-demo-01-repo"
export SIM_BIND SIM_URL
_REPOSIX_TMP_PATHS+=("$REPO")
cleanup_trap

# Pre-clean any debris from a prior aborted run so we start idempotent.
fusermount3 -u "$MNT" 2>/dev/null || true
rm -rf "$MNT" "$REPO" "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm" 2>/dev/null || true
pkill -f "reposix-sim --bind ${SIM_BIND}" 2>/dev/null || true
sleep 0.2

section "[1/6] start simulator + mount FUSE"
setup_sim "$SIM_DB"
setup_mount "$MNT" --project demo
wait_for_mount "$MNT" 10
echo "sim ready at $SIM_URL; mount ready at $MNT"

section "[2/6] agent-style read path: ls + cat"
ls "$MNT" | sort
echo "--- head of 0001.md ---"
head -8 "$MNT/0001.md"

section "[3/6] edit through FUSE (sed-style in-memory write)"
echo "before: status = $(curl -s "${SIM_URL}/projects/demo/issues/1" | jq -r .status)"
NEW="$(sed 's/^status: open$/status: in_progress/' "$MNT/0001.md")"
printf '%s\n' "$NEW" > "$MNT/0001.md"
sleep 0.3
echo "after FUSE write, frontmatter head:"
head -6 "$MNT/0001.md"
echo "server confirms:"
curl -s "${SIM_URL}/projects/demo/issues/1" | jq '{id, status, version}'

section "[4/6] git clone via reposix:: remote"
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
    echo "working tree after fetch:"
    ls
)

section "[5/6] edit + commit + push"
(
    cd "$REPO"
    sed -i 's/^status: in_progress$/status: in_review/' 0001.md
    git commit -am "request review" -q
    echo "pushing..."
    git push origin main 2>&1 | sed 's/^/    /'
)

section "[6/6] verify server state reflects the push"
echo "issue 1 status after push (expect in_review):"
curl -s "${SIM_URL}/projects/demo/issues/1" | jq -r '.status'

echo
echo "== DEMO COMPLETE =="
