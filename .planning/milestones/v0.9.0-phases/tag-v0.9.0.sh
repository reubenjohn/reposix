#!/usr/bin/env bash
# scripts/tag-v0.9.0.sh — create and push the v0.9.0 annotated tag.
#
# Only run this after:
#   1. cargo test --workspace --locked green
#   2. cargo clippy --workspace --all-targets -- -D warnings green
#   3. bash scripts/dark-factory-test.sh sim green
#   4. You've eyeballed the CHANGELOG [v0.9.0] section
#   5. Cargo.toml workspace version has been bumped to 0.9.0
#
# Safety guards (this script enforces all eight):
#   1. Current branch must be `main`
#   2. Working tree must be clean (no uncommitted or untracked changes)
#   3. v0.9.0 tag must NOT already exist locally
#   4. v0.9.0 tag must NOT already exist on origin
#   5. CHANGELOG.md must contain a `## [v0.9.0]` header
#   6. Cargo.toml workspace version must be 0.9.0
#   7. cargo test --workspace --locked green + dark-factory-test.sh sim green
#   8. docs/reference/testing-targets.md must exist (so we never tag a
#      release that lost the canonical test-target documentation)
#
# Usage:  bash scripts/tag-v0.9.0.sh
#
# There is no --force flag by design. If a guard trips, fix the root cause.
#
# Milestone: v0.9.0 "Architecture Pivot — Git-Native Partial Clone"
# Phases 31-36 ship: reposix-cache crate (ARCH-01..03), stateless-connect
# read path (ARCH-04..05), delta sync (ARCH-06..07), push-time conflict
# detection + blob-limit guardrail (ARCH-08..10), `reposix init` CLI pivot
# (ARCH-11..12), and FUSE deletion + CLAUDE.md update + reposix-agent-flow
# skill + release infrastructure (ARCH-13..19).

set -euo pipefail

TAG="v0.9.0"

# 1. Branch guard
CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo "ERROR: not on main (current branch: $CURRENT_BRANCH)" >&2
    exit 1
fi
echo "[guard 1/8] branch = main"

# 2. Clean tree guard
if ! git diff --quiet HEAD 2>/dev/null || [[ -n "$(git status --porcelain)" ]]; then
    echo "ERROR: working tree is not clean" >&2
    git status --short >&2
    exit 1
fi
echo "[guard 2/8] working tree clean"

# 3. Tag-doesn't-exist guard (local)
if git rev-parse --verify "refs/tags/$TAG" >/dev/null 2>&1; then
    echo "ERROR: local tag $TAG already exists" >&2
    echo "       If you intend to re-tag, delete it first: git tag -d $TAG" >&2
    exit 1
fi
echo "[guard 3/8] no local tag $TAG"

# 4. Tag-doesn't-exist guard (remote)
if git ls-remote --tags origin "$TAG" 2>/dev/null | grep -q "refs/tags/$TAG"; then
    echo "ERROR: remote tag $TAG already exists on origin" >&2
    echo "       Investigate who pushed it before overwriting. The tag is visible" >&2
    echo "       and permanent; do not force-push without user approval." >&2
    exit 1
fi
echo "[guard 4/8] no remote tag $TAG on origin"

# 5. CHANGELOG section present
if ! grep -qE '^## \[v0\.9\.0\]' CHANGELOG.md; then
    echo "ERROR: CHANGELOG.md has no '## [v0.9.0]' section" >&2
    exit 1
fi
echo "[guard 5/8] CHANGELOG.md has [v0.9.0] section"

# 6. Cargo.toml workspace version bumped
if ! grep -qE '^version = "0\.9\.0"' Cargo.toml; then
    echo "ERROR: Cargo.toml workspace version is not 0.9.0. Bump it before tagging." >&2
    exit 1
fi
echo "[guard 6/8] Cargo.toml workspace version = 0.9.0"

# 7. All tests green + dark-factory regression green (belt-and-suspenders;
#    fast-fail before tagging)
echo "[guard 7/8] verifying workspace is green..."
cargo test --workspace --locked
bash scripts/dark-factory-test.sh sim
echo "[guard 7/8] cargo test + dark-factory-test.sh sim green"

# 8. testing-targets doc must exist (per ARCH-18) — so we never tag a
#    release that lost the canonical real-backend documentation.
if [[ ! -f docs/reference/testing-targets.md ]]; then
    echo "ERROR: docs/reference/testing-targets.md missing — required for v0.9.0 ship per ARCH-18." >&2
    exit 1
fi
echo "[guard 8/8] docs/reference/testing-targets.md present"

# 9. Extract v0.9.0 body from CHANGELOG for the tag message.
CHANGELOG_BODY="$(sed -n '/^## \[v0.9.0\]/,/^## \[/p' CHANGELOG.md | sed '$d')"

echo "==> creating annotated tag $TAG"
git tag -a "$TAG" -m "reposix $TAG — Architecture Pivot: git-native partial clone (Phases 31-36)

See CHANGELOG.md for the full release notes.

$CHANGELOG_BODY
"

echo "==> pushing $TAG to origin"
git push origin "$TAG"

echo
echo "== TAG $TAG PUSHED =="
echo "Optional: create a GitHub release at"
echo "  https://github.com/reubenjohn/reposix/releases/new?tag=$TAG"
echo "and paste the CHANGELOG v0.9.0 section as the body."
