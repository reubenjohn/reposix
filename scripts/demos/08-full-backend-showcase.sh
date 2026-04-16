#!/usr/bin/env bash
# scripts/demos/08-full-backend-showcase.sh — Full backend showcase.
#
# AUDIENCE: developer
# RUNTIME_SEC: 120
# REQUIRES: cargo, fusermount3, gh, reposix, reposix-swarm (release binaries)
# ASSERTS: "DEMO COMPLETE" "00000000001.md" "_INDEX.md" "confluence-direct"
#
# Narrative: showcases every feature that shipped in v0.4–v0.6 (Phases 13–17)
# against BOTH real backends in a single script:
#
#   [1/7]  GitHub    list (table)                   — Phase 10 / trait decoupling (14)
#   [2/7]  Confluence list (table)                  — Phase 11 / ADF write path (16)
#   [3/7]  GitHub    FUSE mount: nested layout       — Phase 13 nested layout
#                    issues/, _INDEX.md, cat #1, grep
#   [4/7]  Confluence FUSE mount: flat pages/        — Phase 13 + ADF (16)
#                    _INDEX.md, cat first page
#   [5/7]  Confluence FUSE mount: tree/ hierarchy    — Phase 13 hero feature
#                    .gitignore, cd tree/, readlink _self.md
#   [6/7]  Swarm confluence-direct (3 clients × 5s)  — Phase 17 workload
#   [7/7]  reposix refresh --backend confluence       — Phase 14/16 git snapshot
#
# Requires: GITHUB_TOKEN (or `gh auth login`), and the four Atlassian env vars:
#   ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, REPOSIX_CONFLUENCE_TENANT,
#   REPOSIX_CONFLUENCE_SPACE.
# Skips cleanly if any required env var is absent.
# NOT in smoke — requires real credentials for both backends.

set -euo pipefail

# Self-wrap in `timeout 180` so a stuck sub-step cannot hang any caller.
if [[ -z "${REPOSIX_DEMO_INNER:-}" ]]; then
    exec timeout 180 env REPOSIX_DEMO_INNER=1 bash "$0" "$@"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/_lib.sh"

# ------------------------------------------------------------ prereqs
require reposix
require reposix-swarm
require fusermount3
require gh

# ------------------------------------------------------------ GitHub token
if ! GH_TOKEN_VAL="$(gh auth token 2>/dev/null)"; then
    GH_TOKEN_VAL=""
fi
if [[ -z "$GH_TOKEN_VAL" ]]; then
    echo "SKIP: 'gh auth token' returned empty; re-run after 'gh auth login'."
    echo "== DEMO COMPLETE =="
    exit 0
fi
export GITHUB_TOKEN="$GH_TOKEN_VAL"

# ------------------------------------------------------------ Atlassian creds check
MISSING=()
[[ -z "${ATLASSIAN_API_KEY:-}"         ]] && MISSING+=("ATLASSIAN_API_KEY")
[[ -z "${ATLASSIAN_EMAIL:-}"           ]] && MISSING+=("ATLASSIAN_EMAIL")
[[ -z "${REPOSIX_CONFLUENCE_TENANT:-}" ]] && MISSING+=("REPOSIX_CONFLUENCE_TENANT")
[[ -z "${REPOSIX_CONFLUENCE_SPACE:-}"  ]] && MISSING+=("REPOSIX_CONFLUENCE_SPACE")
if (( ${#MISSING[@]} > 0 )); then
    echo "SKIP: env vars unset: ${MISSING[*]}"
    echo "      Populate .env (see .env.example) and source it before running."
    echo "== DEMO COMPLETE =="
    exit 0
fi

# ------------------------------------------------------------ config
GH_PROJECT="octocat/Hello-World"
CONF_TENANT_ORIGIN="https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://api.github.com,${CONF_TENANT_ORIGIN}"

GH_MOUNT="/tmp/reposix-showcase-gh-mnt"
CONF_MOUNT="/tmp/reposix-showcase-conf-mnt"
REFRESH_DIR="/tmp/reposix-showcase-refresh"

_REPOSIX_MOUNT_PATHS+=("$GH_MOUNT" "$CONF_MOUNT")
_REPOSIX_TMP_PATHS+=("$GH_MOUNT" "$CONF_MOUNT" "$REFRESH_DIR")
cleanup_trap

# Pre-clean any stale mounts from prior aborted runs.
fusermount3 -u "$GH_MOUNT"   2>/dev/null || true
fusermount3 -u "$CONF_MOUNT" 2>/dev/null || true
rm -rf "$GH_MOUNT" "$CONF_MOUNT" "$REFRESH_DIR"

# ============================================================ [1/7] GitHub list
section "[1/7] GitHub — list issues (${GH_PROJECT}) — table"
echo "+ reposix list --backend github --project ${GH_PROJECT} --format table"
reposix list --backend github --project "$GH_PROJECT" --format table
echo
echo "+ reposix list --backend github --project ${GH_PROJECT} --format json | python3 -c '...count...'"
reposix list --backend github --project "$GH_PROJECT" --format json \
    | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'  {len(d)} issues returned')"

# ============================================================ [2/7] Confluence list
section "[2/7] Confluence — list pages (${REPOSIX_CONFLUENCE_SPACE}) — table"
echo "tenant: ${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
echo "+ reposix list --backend confluence --project ${REPOSIX_CONFLUENCE_SPACE} --format table"
reposix list \
    --backend confluence \
    --project "$REPOSIX_CONFLUENCE_SPACE" \
    --format table
echo
echo "+ reposix list --backend confluence --project ${REPOSIX_CONFLUENCE_SPACE} --format json | python3 -c '...count...'"
reposix list \
    --backend confluence \
    --project "$REPOSIX_CONFLUENCE_SPACE" \
    --format json \
    | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'  {len(d)} pages returned')"

# ============================================================ [3/7] GitHub FUSE mount
section "[3/7] GitHub FUSE — nested layout: issues/ + _INDEX.md"
mkdir -p "$GH_MOUNT"
echo "+ reposix mount ${GH_MOUNT} --backend github --project ${GH_PROJECT} &"
reposix mount "$GH_MOUNT" \
    --backend github \
    --project "$GH_PROJECT" \
    >/tmp/reposix-showcase-gh.log 2>&1 &
GH_MOUNT_PID=$!
_REPOSIX_SIM_PIDS+=("$GH_MOUNT_PID")

echo "(waiting for FUSE mount to populate issues/...)"
DEADLINE=$((SECONDS + 30))
while (( SECONDS < DEADLINE )); do
    if ls "$GH_MOUNT/issues" 2>/dev/null | grep -q '\.md$'; then break; fi
    sleep 0.25
done
if ! ls "$GH_MOUNT/issues" 2>/dev/null | grep -q '\.md$'; then
    echo "----- mount log -----"
    cat /tmp/reposix-showcase-gh.log || true
    echo "FAIL: GitHub mount did not expose issues/*.md within 30s" >&2
    exit 1
fi

echo
echo "+ ls -A ${GH_MOUNT}   # root: .gitignore + issues/"
ls -A "$GH_MOUNT"

echo
LISTING="$(ls "$GH_MOUNT/issues")"
COUNT="$(echo "$LISTING" | wc -l)"
echo "+ ls ${GH_MOUNT}/issues   # ${COUNT} issues, each a Markdown file"
echo "$LISTING" | head -5
echo "..."
echo "$LISTING" | tail -3

echo
echo "+ cat ${GH_MOUNT}/issues/_INDEX.md   # Phase 15 — synthesized bucket index"
cat "$GH_MOUNT/issues/_INDEX.md"

echo
echo "+ cat ${GH_MOUNT}/issues/00000000001.md   # issue #1 via IssueBackend.get_issue"
cat "$GH_MOUNT/issues/00000000001.md"

echo
echo "+ grep 'title:' ${GH_MOUNT}/issues/*.md | head -5   # POSIX grep over live issues"
grep 'title:' "$GH_MOUNT/issues/"*.md 2>/dev/null | head -5 || true

echo
echo "+ fusermount3 -u ${GH_MOUNT}"
fusermount3 -u "$GH_MOUNT" || true
for _ in $(seq 1 30); do
    if ! mountpoint -q "$GH_MOUNT" 2>/dev/null; then break; fi
    sleep 0.1
done
echo "unmounted cleanly"

# ============================================================ [4/7] Confluence FUSE flat
section "[4/7] Confluence FUSE — flat pages/ view + _INDEX.md (Phase 15)"
mkdir -p "$CONF_MOUNT"
echo "+ reposix mount ${CONF_MOUNT} --backend confluence --project ${REPOSIX_CONFLUENCE_SPACE} &"
reposix mount "$CONF_MOUNT" \
    --backend confluence \
    --project "$REPOSIX_CONFLUENCE_SPACE" \
    >/tmp/reposix-showcase-conf.log 2>&1 &
CONF_MOUNT_PID=$!
_REPOSIX_SIM_PIDS+=("$CONF_MOUNT_PID")

echo "(waiting for Confluence FUSE mount to populate pages/...)"
DEADLINE=$((SECONDS + 30))
while (( SECONDS < DEADLINE )); do
    if ls "$CONF_MOUNT/pages" 2>/dev/null | grep -q '\.md$'; then break; fi
    sleep 0.25
done
if ! ls "$CONF_MOUNT/pages" 2>/dev/null | grep -q '\.md$'; then
    echo "----- mount log -----"
    cat /tmp/reposix-showcase-conf.log || true
    echo "FAIL: Confluence mount did not expose pages/*.md within 30s" >&2
    exit 1
fi

echo
echo "+ ls -A ${CONF_MOUNT}   # root: .gitignore + pages/ + tree/"
ls -A "$CONF_MOUNT"

echo
echo "+ cat ${CONF_MOUNT}/pages/_INDEX.md   # synthesized page index"
cat "$CONF_MOUNT/pages/_INDEX.md"

PAGES_LISTING="$(ls "$CONF_MOUNT/pages" | grep '\.md$' | grep -v '_INDEX')"
FIRST_PAGE="$(echo "$PAGES_LISTING" | head -1)"
echo
echo "+ cat ${CONF_MOUNT}/pages/${FIRST_PAGE}   # first page, ADF → Markdown"
cat "$CONF_MOUNT/pages/${FIRST_PAGE}"

# ============================================================ [5/7] Confluence tree
section "[5/7] Confluence FUSE — tree/ hierarchy (Phase 13 nested layout)"
echo "(Confluence mount still live from [4/7])"
echo
echo "+ cat ${CONF_MOUNT}/.gitignore   # '/tree/' keeps overlay out of git"
cat "$CONF_MOUNT/.gitignore"

echo
echo "+ ls ${CONF_MOUNT}/tree   # space slug directories"
ls "$CONF_MOUNT/tree"

TREE_DIR="$(find "$CONF_MOUNT/tree" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | head -1)"
if [[ -z "$TREE_DIR" ]]; then
    echo "(tree/ has no subdirectories — empty space, skipping walk)"
else
    TREE_SLUG="$(basename "$TREE_DIR")"
    echo
    echo "+ ls ${CONF_MOUNT}/tree/${TREE_SLUG}   # human-readable page hierarchy"
    ls "$TREE_DIR"

    echo
    echo "+ readlink ${CONF_MOUNT}/tree/${TREE_SLUG}/_self.md"
    readlink "$TREE_DIR/_self.md"
    echo "+ head -5 ${CONF_MOUNT}/tree/${TREE_SLUG}/_self.md   # follows symlink → pages/<id>.md"
    head -5 "$TREE_DIR/_self.md"

    FIRST_LEAF="$(find "$TREE_DIR" -mindepth 1 -maxdepth 1 -type l \
        ! -name '_self.md' 2>/dev/null | head -1)"
    if [[ -n "$FIRST_LEAF" ]]; then
        LEAF_NAME="$(basename "$FIRST_LEAF")"
        echo
        echo "+ readlink ${CONF_MOUNT}/tree/${TREE_SLUG}/${LEAF_NAME}   # child page symlink"
        readlink "$FIRST_LEAF"
        echo "+ head -5 ${CONF_MOUNT}/tree/${TREE_SLUG}/${LEAF_NAME}"
        head -5 "$FIRST_LEAF"
    fi
fi

echo
echo "+ fusermount3 -u ${CONF_MOUNT}"
fusermount3 -u "$CONF_MOUNT" || true
for _ in $(seq 1 30); do
    if ! mountpoint -q "$CONF_MOUNT" 2>/dev/null; then break; fi
    sleep 0.1
done
echo "unmounted cleanly"

# ============================================================ [6/7] Swarm confluence-direct
section "[6/7] Swarm — confluence-direct mode (3 clients × 5s) — Phase 17"
echo "tenant: ${REPOSIX_CONFLUENCE_TENANT}.atlassian.net  space: ${REPOSIX_CONFLUENCE_SPACE}"
echo "+ reposix-swarm --mode confluence-direct --target ${CONF_TENANT_ORIGIN}"
echo "    --project ${REPOSIX_CONFLUENCE_SPACE} --clients 3 --duration 5"
reposix-swarm \
    --mode confluence-direct \
    --target "${CONF_TENANT_ORIGIN}" \
    --project "$REPOSIX_CONFLUENCE_SPACE" \
    --clients 3 \
    --duration 5
echo "swarm run complete"

# ============================================================ [7/7] refresh → git snapshot
section "[7/7] reposix refresh --backend confluence → git snapshot (Phase 14/16)"
mkdir -p "$REFRESH_DIR"
echo "+ git init ${REFRESH_DIR} && git commit --allow-empty -m 'init'"
git -C "$REFRESH_DIR" init -q
git -C "$REFRESH_DIR" -c user.email="demo@reposix" -c user.name="reposix-demo" \
    commit --allow-empty -q -m "init"
echo "initialized empty git repo at ${REFRESH_DIR}"

echo
echo "+ reposix refresh ${REFRESH_DIR} --backend confluence --project ${REPOSIX_CONFLUENCE_SPACE}"
reposix refresh "$REFRESH_DIR" \
    --backend confluence \
    --project "$REPOSIX_CONFLUENCE_SPACE"

echo
echo "+ git -C ${REFRESH_DIR} log --oneline -3"
git -C "$REFRESH_DIR" log --oneline -3

echo
echo "+ git -C ${REFRESH_DIR} show --stat HEAD"
git -C "$REFRESH_DIR" show --stat HEAD

echo
echo "+ ls ${REFRESH_DIR}"
ls "$REFRESH_DIR"

echo
echo "== DEMO COMPLETE =="
