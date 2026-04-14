#!/usr/bin/env bash
# scripts/tag-v0.6.0.sh — create and push the v0.6.0 annotated tag.
#
# Only run this after:
#   1. cargo test --workspace --locked green
#   2. cargo clippy --workspace --all-targets -- -D warnings green
#   3. bash scripts/demos/smoke.sh 4/4 green
#   4. You've eyeballed the CHANGELOG [v0.6.0] section
#   5. Cargo.toml workspace version has been bumped to 0.6.0
#
# Safety guards (this script enforces all seven):
#   1. Current branch must be `main`
#   2. Working tree must be clean (no uncommitted or untracked changes)
#   3. v0.6.0 tag must NOT already exist locally
#   4. v0.6.0 tag must NOT already exist on origin
#   5. CHANGELOG.md must contain a `## [v0.6.0]` header
#   6. Cargo.toml workspace version must be 0.6.0
#   7. cargo test --workspace --locked green + scripts/demos/smoke.sh green
#
# Usage:  bash scripts/tag-v0.6.0.sh
#
# There is no --force flag by design. If a guard trips, fix the root cause.
#
# Milestone: v0.6.0 "Write Path + Full Sitemap" (milestone-start tag).
# Phase 16 ships: Confluence write path (create/update/delete), ADF<->Markdown
# converter, client-side audit log via SG-06 schema, ADF read path with storage
# fallback. Closes REQ WRITE-01..04. Phase 17 (swarm confluence-direct),
# Phase 18 (OP-2 remainder), Phase 19 (OP-1 remainder), and Phase 20 (OP-3)
# remain queued under this milestone — v0.6.0 is a milestone-start tag,
# not a milestone-complete tag.

set -euo pipefail

TAG="v0.6.0"

# 1. Branch guard
CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo "ERROR: not on main (current branch: $CURRENT_BRANCH)" >&2
    exit 1
fi
echo "[guard 1/7] branch = main"

# 2. Clean tree guard
if ! git diff --quiet HEAD 2>/dev/null || [[ -n "$(git status --porcelain)" ]]; then
    echo "ERROR: working tree is not clean" >&2
    git status --short >&2
    exit 1
fi
echo "[guard 2/7] working tree clean"

# 3. Tag-doesn't-exist guard (local)
if git rev-parse --verify "refs/tags/$TAG" >/dev/null 2>&1; then
    echo "ERROR: local tag $TAG already exists" >&2
    echo "       If you intend to re-tag, delete it first: git tag -d $TAG" >&2
    exit 1
fi
echo "[guard 3/7] no local tag $TAG"

# 4. Tag-doesn't-exist guard (remote)
if git ls-remote --tags origin "$TAG" 2>/dev/null | grep -q "refs/tags/$TAG"; then
    echo "ERROR: remote tag $TAG already exists on origin" >&2
    echo "       Investigate who pushed it before overwriting. The tag is visible" >&2
    echo "       and permanent; do not force-push without user approval." >&2
    exit 1
fi
echo "[guard 4/7] no remote tag $TAG on origin"

# 5. CHANGELOG section present
if ! grep -qE '^## \[v0\.6\.0\]' CHANGELOG.md; then
    echo "ERROR: CHANGELOG.md has no '## [v0.6.0]' section" >&2
    exit 1
fi
echo "[guard 5/7] CHANGELOG.md has [v0.6.0] section"

# 6. Cargo.toml workspace version bumped
#    The actual bump is a pre-tag commit the user makes (or a separate
#    small commit before running this script). This script only checks;
#    it does NOT edit.
if ! grep -qE '^version = "0\.6\.0"' Cargo.toml; then
    echo "ERROR: Cargo.toml workspace version is not 0.6.0. Bump it before tagging." >&2
    exit 1
fi
echo "[guard 6/7] Cargo.toml workspace version = 0.6.0"

# 7. All tests green (belt-and-suspenders; fast-fail before tagging)
echo "[guard 7/7] verifying workspace is green..."
cargo test --workspace --locked
bash scripts/demos/smoke.sh
echo "[guard 7/7] cargo test + smoke.sh green"

# 8. Extract v0.6.0 body from CHANGELOG for the tag message.
#    Stops at the next `## [` header (which is the previous release) and
#    drops that trailing line via `sed '$d'`.
CHANGELOG_BODY="$(sed -n '/^## \[v0.6.0\]/,/^## \[/p' CHANGELOG.md | sed '$d')"

echo "==> creating annotated tag $TAG"
git tag -a "$TAG" -m "reposix $TAG — Confluence write path + ADF converter (Phase 16, v0.6.0 milestone start)

See CHANGELOG.md for the full release notes.

$CHANGELOG_BODY
"

echo "==> pushing $TAG to origin"
git push origin "$TAG"

echo
echo "== TAG $TAG PUSHED =="
echo "Optional: create a GitHub release at"
echo "  https://github.com/reubenjohn/reposix/releases/new?tag=$TAG"
echo "and paste the CHANGELOG v0.6.0 section as the body."
