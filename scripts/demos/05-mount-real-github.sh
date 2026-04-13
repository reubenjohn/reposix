#!/usr/bin/env bash
# scripts/demos/05-mount-real-github.sh — Tier 5 FUSE-mount-real-GitHub demo.
#
# AUDIENCE: developer
# RUNTIME_SEC: 30
# REQUIRES: cargo, fusermount3, gh, jq
# ASSERTS: "DEMO COMPLETE" "0001.md"
#
# Narrative: the entire point of the IssueBackend trait, shipped in v0.2,
# was that the FUSE daemon could be re-pointed at a real backend with no
# bespoke plumbing. Phase 10 wired that end-to-end: `reposix mount
# --backend github --project owner/repo` mounts real GitHub issues as
# Markdown files, backed by `GithubReadOnlyBackend` through the same
# allowlist + rate-limit machinery as `reposix list`.
#
# This demo mounts `octocat/Hello-World`, lists the files, `cat`s issue
# #1, and unmounts. If `gh auth token` is empty we SKIP (CI general
# workers don't carry GitHub auth in this project by convention —
# integration-contract uses its own `${{ secrets.GITHUB_TOKEN }}`).
#
# Not in smoke — requires a real GitHub token.

set -euo pipefail

# Self-wrap in `timeout 90` so a stuck sub-step cannot hang smoke.sh.
if [[ -z "${REPOSIX_DEMO_INNER:-}" ]]; then
    exec timeout 90 env REPOSIX_DEMO_INNER=1 bash "$0" "$@"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/_lib.sh"

# ------------------------------------------------------------ prereqs
require reposix
require fusermount3
require gh
require jq

# Skip cleanly if the dev has no gh auth — this demo is not runnable
# without a real token.
if ! GH_TOKEN_VAL="$(gh auth token 2>/dev/null)"; then
    GH_TOKEN_VAL=""
fi
if [[ -z "$GH_TOKEN_VAL" ]]; then
    echo "SKIP: gh auth token is empty; re-run after 'gh auth login'."
    echo "== DEMO COMPLETE =="  # smoke-style marker so the runner sees us finish.
    exit 0
fi

# ------------------------------------------------------------ config
MOUNT_PATH="/tmp/reposix-gh-demo-mnt"
PROJECT="octocat/Hello-World"
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://api.github.com"
export GITHUB_TOKEN="$GH_TOKEN_VAL"

# Pre-clean: a stale mount from a prior aborted run would make `reposix
# mount` refuse. `fusermount3 -u` is a no-op when the path isn't mounted.
fusermount3 -u "$MOUNT_PATH" 2>/dev/null || true
rm -rf "$MOUNT_PATH"
mkdir -p "$MOUNT_PATH"

_REPOSIX_MOUNT_PATHS+=("$MOUNT_PATH")
_REPOSIX_TMP_PATHS+=("$MOUNT_PATH")
cleanup_trap

# ------------------------------------------------------------ 1/4 mount
section "[1/4] mount real GitHub at ${MOUNT_PATH}"
echo "project: ${PROJECT}"
echo "allowlist: ${REPOSIX_ALLOWED_ORIGINS}"
reposix mount "$MOUNT_PATH" \
    --backend github \
    --project "$PROJECT" \
    >/tmp/reposix-gh-demo-mnt.log 2>&1 &
MOUNT_PID=$!
_REPOSIX_SIM_PIDS+=("$MOUNT_PID")  # reuse the sim-kill list; cleanup_trap handles both

# Wait for the mount to expose at least one .md entry. Give it 30s —
# GitHub's first round-trip can be slow on cold connections, plus the
# CLI's MountProcess::wait_ready watchdog already eats up to 15s on its
# own before reposix-mount returns. The fuse daemon's per-request 5s
# SG-07 ceiling still applies, so a genuinely dead backend returns EIO
# fast.
if ! wait_for_mount "$MOUNT_PATH" 30; then
    echo "----- mount log -----"
    cat /tmp/reposix-gh-demo-mnt.log || true
    exit 1
fi

# ------------------------------------------------------------ 2/4 ls
section "[2/4] ls the mount — every issue is a Markdown file"
# Snapshot the listing into a variable so we don't trigger multiple
# readdir-driven GitHub round-trips (each `ls` re-fetches; the inode
# cache is per-issue, not per-listing).
LISTING="$(ls "$MOUNT_PATH")"
COUNT=$(echo "$LISTING" | wc -l)
echo "issue count: $COUNT"
echo "$LISTING" | head -5
echo "..."
echo "$LISTING" | tail -3

if [[ "$COUNT" -lt 1 ]]; then
    echo "FAIL: mount exposed 0 files"
    exit 1
fi

# ------------------------------------------------------------ 3/4 cat
section "[3/4] cat issue #1 — frontmatter renders from real GitHub"
# `0001.md` is the zero-padded name for GitHub issue number 1. The
# listing in step 2 shows the 500 most recent issues; issue #1 is
# addressable by `lookup` even though it's below the pagination window,
# thanks to the IssueBackend.get_issue seam.
echo "+ cat ${MOUNT_PATH}/0001.md"
if ! cat "$MOUNT_PATH/0001.md"; then
    echo "FAIL: cat 0001.md did not succeed"
    exit 1
fi

# ------------------------------------------------------------ 4/4 unmount
section "[4/4] unmount"
fusermount3 -u "$MOUNT_PATH" || true
# Wait up to 3s for the mount to drop.
for _ in $(seq 1 30); do
    if ! mountpoint -q "$MOUNT_PATH" 2>/dev/null; then
        break
    fi
    sleep 0.1
done
echo "unmounted cleanly"

echo
echo "== DEMO COMPLETE =="
