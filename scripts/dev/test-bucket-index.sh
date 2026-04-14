#!/usr/bin/env bash
# scripts/dev/test-bucket-index.sh — live-mount + cat _INDEX.md proof.
#
# Phase 15 Wave A verification: mount the simulator via FUSE, read the
# synthesized `_INDEX.md` under `mount/issues/`, assert the markdown
# structure, and verify the write path is hard-rejected.
#
# Requires: reposix-sim, reposix-fuse, fusermount3 on PATH.
# Exit 0 on success, non-zero on any assertion failure.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# shellcheck source=../demos/_lib.sh
source "${REPO_ROOT}/scripts/demos/_lib.sh"

require reposix-sim
require reposix-fuse
require fusermount3

SIM_BIND="127.0.0.1:7866"
SIM_URL="http://${SIM_BIND}"
SIM_DB="/tmp/reposix-15-bucket-index-sim.db"
MNT="/tmp/reposix-15-bucket-index-mnt"
export SIM_BIND SIM_URL
cleanup_trap

fusermount3 -u "$MNT" 2>/dev/null || true
rm -rf "$MNT" "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm" 2>/dev/null || true
pkill -f "reposix-sim --bind ${SIM_BIND}" 2>/dev/null || true
sleep 0.3

section "[1/4] start sim + mount FUSE"
setup_sim "$SIM_DB"
setup_mount "$MNT" --project demo
wait_for_mount "$MNT" 10
echo "ready: sim=$SIM_URL mount=$MNT"

section "[2/4] ls $MNT/issues/"
ls "$MNT/issues"
echo

section "[3/4] cat $MNT/issues/_INDEX.md"
INDEX="$(cat "$MNT/issues/_INDEX.md")"
echo "$INDEX"
echo

# Assertions on the rendered index.
if ! grep -q '^---$' <<<"$INDEX"; then
  echo "FAIL: no YAML frontmatter delimiter" >&2; exit 1
fi
if ! grep -q '^backend: ' <<<"$INDEX"; then
  echo "FAIL: no backend frontmatter key" >&2; exit 1
fi
if ! grep -q '^issue_count: ' <<<"$INDEX"; then
  echo "FAIL: no issue_count frontmatter key" >&2; exit 1
fi
if ! grep -qE '^\| id ' <<<"$INDEX"; then
  echo "FAIL: no markdown table header" >&2; exit 1
fi

section "[4/4] verify _INDEX.md is read-only"
if touch "$MNT/issues/_INDEX.md" 2>/dev/null; then
  echo "FAIL: touch on _INDEX.md should have returned an error" >&2; exit 1
fi
if rm "$MNT/issues/_INDEX.md" 2>/dev/null; then
  echo "FAIL: rm on _INDEX.md should have returned an error" >&2; exit 1
fi

echo
echo "== BUCKET INDEX PROOF OK =="
