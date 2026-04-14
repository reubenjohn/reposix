#!/usr/bin/env bash
# scripts/demos/06-mount-real-confluence.sh — Tier 5 FUSE-mount-real-Confluence demo.
#
# AUDIENCE: developer
# RUNTIME_SEC: 45
# REQUIRES: cargo, fusermount3, reposix (release binary)
# ASSERTS: "DEMO COMPLETE" ".md"
#
# Narrative: Phase 11 wired the Confluence adapter end-to-end, so
# `reposix mount --backend confluence --project <SPACE_KEY>` mounts a
# real Atlassian Confluence space as a tree of Markdown files — exactly
# like `reposix mount --backend github` does for GitHub. This demo
# mounts the space, ls's it, cats the first page, and unmounts.
#
# Unlike 05-mount-real-github.sh this demo needs FOUR env vars (the
# GitHub version only needs `gh auth token`). Atlassian's Basic auth
# scheme requires both email + token, and Confluence is tenant-scoped
# so we also need the tenant subdomain and the space key.
#
# Not in smoke — requires real Atlassian credentials.
# Skips cleanly (exit 0) if any of the four required env vars are unset.

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

# ------------------------------------------------------------ SKIP check
# All four env vars are required. Missing any one => SKIP path.
# Never echoes the values of ATLASSIAN_API_KEY or ATLASSIAN_EMAIL; only
# tenant + space (non-secret identifiers) appear on stdout.
MISSING=()
[[ -z "${ATLASSIAN_API_KEY:-}"         ]] && MISSING+=("ATLASSIAN_API_KEY")
[[ -z "${ATLASSIAN_EMAIL:-}"           ]] && MISSING+=("ATLASSIAN_EMAIL")
[[ -z "${REPOSIX_CONFLUENCE_TENANT:-}" ]] && MISSING+=("REPOSIX_CONFLUENCE_TENANT")
[[ -z "${REPOSIX_CONFLUENCE_SPACE:-}"  ]] && MISSING+=("REPOSIX_CONFLUENCE_SPACE")
if (( ${#MISSING[@]} > 0 )); then
    echo "SKIP: env vars unset: ${MISSING[*]}"
    echo "      Set them (see .env.example and MORNING-BRIEF-v0.3.md) to"
    echo "      run this demo."
    echo "== DEMO COMPLETE =="  # smoke-style marker so a runner sees us finish.
    exit 0
fi

# ------------------------------------------------------------ config
MOUNT_PATH="/tmp/reposix-conf-demo-mnt"
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"

# Pre-clean: a stale mount from a prior aborted run would make `reposix
# mount` refuse. `fusermount3 -u` is a no-op when the path isn't mounted.
fusermount3 -u "$MOUNT_PATH" 2>/dev/null || true
rm -rf "$MOUNT_PATH"
mkdir -p "$MOUNT_PATH"

_REPOSIX_MOUNT_PATHS+=("$MOUNT_PATH")
_REPOSIX_TMP_PATHS+=("$MOUNT_PATH")
cleanup_trap

# ------------------------------------------------------------ 1/4 mount
section "[1/4] mount real Confluence at ${MOUNT_PATH}"
echo "tenant: ${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
echo "space:  ${REPOSIX_CONFLUENCE_SPACE}"
echo "allowlist: ${REPOSIX_ALLOWED_ORIGINS}"
reposix mount "$MOUNT_PATH" \
    --backend confluence \
    --project "$REPOSIX_CONFLUENCE_SPACE" \
    >/tmp/reposix-conf-demo-mnt.log 2>&1 &
MOUNT_PID=$!
# Reuse sim-kill list; cleanup_trap handles both SIGTERM reaping.
_REPOSIX_SIM_PIDS+=("$MOUNT_PID")

# Wait for the mount to expose at least one .md entry. 30s covers
# Atlassian's cold-connection latency (10-15s is normal on the first
# round-trip) plus MountProcess::wait_ready's own watchdog budget. The
# fuse daemon's per-request 5s SG-07 ceiling still applies, so a dead
# backend returns EIO fast.
if ! wait_for_mount "$MOUNT_PATH" 30; then
    echo "----- mount log -----"
    cat /tmp/reposix-conf-demo-mnt.log || true
    exit 1
fi

# ------------------------------------------------------------ 2/4 ls
section "[2/4] ls the mount — every page is a Markdown file"
# Phase-13 FUSE layout: Confluence mounts now expose `.gitignore` +
# `pages/` (real files) + `tree/` (read-only symlink overlay mirroring
# the Confluence parentId hierarchy). Real files live under `pages/` at
# 11-digit zero-padded filenames. We snapshot the `pages/` listing to
# avoid multiple readdir-driven Confluence round-trips (each `ls`
# re-fetches).
echo "root listing (expect .gitignore + pages/ + tree/):"
ls "$MOUNT_PATH" | sort
LISTING="$(ls "$MOUNT_PATH/pages")"
COUNT=$(echo "$LISTING" | wc -l)
echo "page count: $COUNT"
echo "$LISTING" | head -5

if [[ "$COUNT" -lt 1 ]]; then
    echo "FAIL: mount exposed 0 files under pages/"
    exit 1
fi

# ------------------------------------------------------------ 3/4 cat
section "[3/4] cat the first page — frontmatter renders from real Confluence"
# Unlike GitHub's `00000000001.md` (1-based issue numbers padded to 11
# digits), Confluence page IDs are per-space numeric IDs assigned by the
# server (also zero-padded to 11 digits in the mount). We cat whatever
# the `pages/` listing produced first — still proves the mount → inode
# → read path works end-to-end.
FIRST_PAGE="$(echo "$LISTING" | head -1)"
echo "+ cat ${MOUNT_PATH}/pages/${FIRST_PAGE}"
if ! cat "${MOUNT_PATH}/pages/${FIRST_PAGE}"; then
    echo "FAIL: cat pages/${FIRST_PAGE} did not succeed"
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
# Belt-and-braces: a zombie mount here would leak into subsequent demo
# runs and fail the plan's "no zombie FUSE mount" must_have. cleanup_trap
# will try again on EXIT but we assert here so the demo fails loudly.
if mountpoint -q "$MOUNT_PATH" 2>/dev/null; then
    echo "FAIL: mount still active after 3s"
    exit 1
fi
echo "unmounted cleanly"

echo
echo "== DEMO COMPLETE =="
