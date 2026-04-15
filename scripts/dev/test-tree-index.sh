#!/usr/bin/env bash
# scripts/dev/test-tree-index.sh — live-mount + cat _INDEX.md proof for
# mount-root and tree-subdir synthetic files.
#
# Phase 18 verification: mount the simulator via FUSE, read the synthesized
# `_INDEX.md` at the mount root, assert the markdown structure, and verify
# the write path is hard-rejected.
#
# Tree-subdir section note:
#   `tree/` is only populated when at least one issue carries a non-null
#   `parent_id`. The reposix-sim backend always returns `parent_id: null`
#   (the sim models a flat issue tracker, not a page hierarchy). To exercise
#   the tree-dir `_INDEX.md` in a live mount you need the Confluence backend
#   against a tenant with parent/child pages, or a future sim extension that
#   persists `parent_id`. See scripts/dev/probe-confluence.sh.
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

SIM_BIND="127.0.0.1:7867"
SIM_URL="http://${SIM_BIND}"
SIM_DB="/tmp/reposix-18-tree-index-sim.db"
MNT="/tmp/reposix-18-tree-index-mnt"
export SIM_BIND SIM_URL
cleanup_trap

fusermount3 -u "$MNT" 2>/dev/null || true
rm -rf "$MNT" "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm" 2>/dev/null || true
pkill -f "reposix-sim --bind ${SIM_BIND}" 2>/dev/null || true
sleep 0.3

section "[1/5] start sim + mount FUSE"
setup_sim "$SIM_DB"
setup_mount "$MNT" --project demo
wait_for_mount "$MNT" 10
echo "ready: sim=$SIM_URL mount=$MNT"

section "[2/5] ls $MNT to trigger readdir refresh"
ls "$MNT"
echo

section "[3/5] cat $MNT/_INDEX.md (mount-root overview)"
ROOT_INDEX="$(cat "$MNT/_INDEX.md")"
echo "$ROOT_INDEX"
echo

# Assertions on the rendered root index.
if ! grep -q '^---$' <<<"$ROOT_INDEX"; then
  echo "FAIL: no YAML frontmatter delimiter" >&2; exit 1
fi
if ! grep -q '^kind: mount-index$' <<<"$ROOT_INDEX"; then
  echo "FAIL: 'kind: mount-index' missing from frontmatter" >&2; exit 1
fi
if ! grep -q '^backend: ' <<<"$ROOT_INDEX"; then
  echo "FAIL: no 'backend' frontmatter key" >&2; exit 1
fi
if ! grep -q '^project: ' <<<"$ROOT_INDEX"; then
  echo "FAIL: no 'project' frontmatter key" >&2; exit 1
fi
if ! grep -qE '^\| entry ' <<<"$ROOT_INDEX"; then
  echo "FAIL: no markdown table header" >&2; exit 1
fi

section "[4/5] verify $MNT/_INDEX.md is read-only"
if touch "$MNT/_INDEX.md" 2>/dev/null; then
  echo "FAIL: touch on _INDEX.md should have returned an error" >&2; exit 1
fi
# Use a subshell so the failure does not abort the script.
if (echo x > "$MNT/_INDEX.md") 2>/dev/null; then
  echo "FAIL: write to _INDEX.md should have returned an error" >&2; exit 1
fi

section "[5/5] tree-dir _INDEX.md (sim note)"
echo "  INFO: reposix-sim returns parent_id=null for all issues."
echo "  INFO: tree/ overlay is only populated when parent_id is present."
echo "  INFO: tree-dir _INDEX.md live-mount test requires the Confluence backend."
echo "  INFO: See scripts/dev/probe-confluence.sh for credential setup."
echo "  INFO: The render path is unit-tested in crates/reposix-fuse/src/fs.rs"
echo "        (render_tree_index_frontmatter_and_table, tree_index_full_dfs,"
echo "         tree_index_empty — six tests covering kind: tree-index, DFS depth,"
echo "         entry_count, and read-only enforcement)."
echo "  SKIP: tree-dir live-mount assertion skipped (requires Confluence backend)"

echo
echo "== TREE + MOUNT-ROOT INDEX PROOF OK =="
