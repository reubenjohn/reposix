#!/usr/bin/env bash
# scripts/demos/07-mount-real-confluence-tree.sh — Tier 5 Confluence tree demo.
#
# AUDIENCE: developer
# RUNTIME_SEC: 60
# REQUIRES: cargo, fusermount3, reposix (release binary)
# ASSERTS: "DEMO COMPLETE" "/tree/" "_self.md"
#
# Narrative: Phase 13 introduces the nested mount layout. When mounting
# a Confluence space, the mount root now exposes THREE things side-by-side:
#
#   .gitignore   # reposix-synthesized, contains "/tree/\n"
#   pages/       # flat, writable, git-tracked — canonical <id>.md files
#   tree/        # read-only symlink overlay — mirrors Confluence parentId
#                # hierarchy with human-readable slug paths
#
# This demo captures the "hero.png" moment for the v0.4.0 release: an
# agent (or a human!) can `cd` into `tree/<space-slug>/` and read pages
# by title, while `git diff` stays clean because the symlink overlay is
# ignored. Matches the specifics block in 13-CONTEXT.md verbatim.
#
# NOT in smoke.sh — requires real Atlassian credentials.
# Skips cleanly (exit 0) if any of the four required env vars are unset,
# following the exact pattern of 06-mount-real-confluence.sh.

set -euo pipefail

# Self-wrap in `timeout 120` so a stuck sub-step cannot hang anything
# that chains demos. 120s > the 90s of demo 06 because we do more
# navigation (ls + readlink + cat) after the mount comes up.
if [[ -z "${REPOSIX_DEMO_INNER:-}" ]]; then
    exec timeout 120 env REPOSIX_DEMO_INNER=1 bash "$0" "$@"
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
    echo "      Set them (see .env.example) to run this demo against the"
    echo "      real REPOSIX space on reuben-john.atlassian.net."
    echo "== DEMO COMPLETE =="  # smoke-style marker so a runner sees us finish.
    exit 0
fi

# ------------------------------------------------------------ config
MOUNT_PATH="/tmp/reposix-tree-demo-mnt"
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"

# Pre-clean: a stale mount from a prior aborted run would make `reposix
# mount` refuse. `fusermount3 -u` is a no-op when the path isn't mounted.
fusermount3 -u "$MOUNT_PATH" 2>/dev/null || true
rm -rf "$MOUNT_PATH"
mkdir -p "$MOUNT_PATH"

_REPOSIX_MOUNT_PATHS+=("$MOUNT_PATH")
_REPOSIX_TMP_PATHS+=("$MOUNT_PATH")
cleanup_trap

# ------------------------------------------------------------ 1/6 mount
section "[1/6] mount real Confluence at ${MOUNT_PATH}"
echo "tenant: ${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
echo "space:  ${REPOSIX_CONFLUENCE_SPACE}"
echo "allowlist: ${REPOSIX_ALLOWED_ORIGINS}"
reposix mount "$MOUNT_PATH" \
    --backend confluence \
    --project "$REPOSIX_CONFLUENCE_SPACE" \
    >/tmp/reposix-tree-demo-mnt.log 2>&1 &
MOUNT_PID=$!
_REPOSIX_SIM_PIDS+=("$MOUNT_PID")

# Wait for the mount to expose at least one .md file somewhere under the
# mount. wait_for_mount only looks at the top level, and the top level
# is now .gitignore/pages/tree/ (no .md at the root), so we spin on
# pages/ directly.
DEADLINE=$((SECONDS + 30))
while (( SECONDS < DEADLINE )); do
    if ls "$MOUNT_PATH/pages" 2>/dev/null | grep -q '\.md$'; then
        break
    fi
    sleep 0.25
done
if ! ls "$MOUNT_PATH/pages" 2>/dev/null | grep -q '\.md$'; then
    echo "ERROR: mount at $MOUNT_PATH did not expose pages/*.md within 30s" >&2
    echo "----- mount log -----"
    cat /tmp/reposix-tree-demo-mnt.log || true
    exit 1
fi

# ------------------------------------------------------------ 2/6 root layout
section "[2/6] ls mount root — expect .gitignore, pages/, tree/"
ls -la "$MOUNT_PATH"

# Assert the three expected top-level entries are all there.
ROOT_LISTING="$(ls "$MOUNT_PATH")"
for expected in .gitignore pages tree; do
    # `.gitignore` won't show in bare `ls` without -a; use `ls -A`.
    if ! ls -A "$MOUNT_PATH" | grep -qx "$expected"; then
        echo "FAIL: expected root entry '$expected' not found" >&2
        echo "actual listing:" >&2
        ls -A "$MOUNT_PATH" >&2
        exit 1
    fi
done
echo "ok: .gitignore, pages/, tree/ all present"

# ------------------------------------------------------------ 3/6 .gitignore
section "[3/6] cat .gitignore — expect '/tree/'"
cat "$MOUNT_PATH/.gitignore"
if ! grep -q '^/tree/$' "$MOUNT_PATH/.gitignore"; then
    echo "FAIL: .gitignore does not contain '/tree/' literal" >&2
    exit 1
fi
echo "ok: git will skip the derived tree/ overlay"

# ------------------------------------------------------------ 4/6 flat pages
section "[4/6] flat view — ls pages/ and cat the first page"
PAGES_LISTING="$(ls "$MOUNT_PATH/pages")"
PAGE_COUNT="$(echo "$PAGES_LISTING" | wc -l)"
echo "pages/ count: $PAGE_COUNT"
echo "$PAGES_LISTING" | head -6
echo "..."

FIRST_PAGE="$(echo "$PAGES_LISTING" | head -1)"
echo "+ cat ${MOUNT_PATH}/pages/${FIRST_PAGE} | head -5"
if ! head -5 "${MOUNT_PATH}/pages/${FIRST_PAGE}"; then
    echo "FAIL: cat pages/${FIRST_PAGE} did not succeed" >&2
    exit 1
fi

# ------------------------------------------------------------ 5/6 tree walk
section "[5/6] hierarchical view — cd tree/ and readlink the hero file"
ls "$MOUNT_PATH/tree"
echo

# Pick the first subdirectory under tree/ (the space homepage slug).
TREE_DIR="$(find "$MOUNT_PATH/tree" -mindepth 1 -maxdepth 1 -type d | head -1)"
if [[ -z "$TREE_DIR" ]]; then
    echo "FAIL: tree/ has no subdirectories — hierarchy overlay missing" >&2
    exit 1
fi

echo "+ cd $TREE_DIR"
cd "$TREE_DIR"
echo "+ ls"
ls
echo

# Assert _self.md exists — it's the parent-page self-link synthesized
# for every interior node. If the layout regresses, this catches it.
if [[ ! -L "_self.md" ]]; then
    echo "FAIL: $TREE_DIR/_self.md is not a symlink (expected self-link)" >&2
    exit 1
fi

echo "+ readlink _self.md"
readlink _self.md
echo "+ cat _self.md | head -5    # follows symlink into pages/..."
head -5 _self.md

# If there's at least one non-_self symlink, readlink that too to prove
# sibling leaves also resolve. The REPOSIX space has 3 children under
# the homepage, so this should always find something.
FIRST_LEAF="$(find . -mindepth 1 -maxdepth 1 -type l ! -name '_self.md' | head -1)"
if [[ -n "$FIRST_LEAF" ]]; then
    echo
    echo "+ readlink $FIRST_LEAF"
    readlink "$FIRST_LEAF"
    echo "+ cat $FIRST_LEAF | head -5"
    head -5 "$FIRST_LEAF"
fi

cd /

# ------------------------------------------------------------ 6/6 unmount
section "[6/6] unmount"
fusermount3 -u "$MOUNT_PATH" || true
# Wait up to 3s for the mount to drop.
for _ in $(seq 1 30); do
    if ! mountpoint -q "$MOUNT_PATH" 2>/dev/null; then
        break
    fi
    sleep 0.1
done
if mountpoint -q "$MOUNT_PATH" 2>/dev/null; then
    echo "FAIL: mount still active after 3s" >&2
    exit 1
fi
echo "unmounted cleanly"

echo
echo "== DEMO COMPLETE =="
