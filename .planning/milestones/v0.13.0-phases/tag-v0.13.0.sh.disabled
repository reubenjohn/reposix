#!/usr/bin/env bash
# tag-v0.13.0.sh -- v0.13.0 milestone tag-cut. Owner runs this; orchestrator does NOT push the tag.
#
# Mirrors v0.12.0/v0.11.x guard pattern. Run from repo root.
#
# Usage: bash .planning/milestones/v0.13.0-phases/tag-v0.13.0.sh
# After OK: git push origin v0.13.0

set -euo pipefail
VERSION="0.13.0"
TAG="v${VERSION}"

# Guard 1: clean working tree.
git diff --quiet && git diff --cached --quiet || { echo "FAIL: working tree not clean"; exit 1; }

# Guard 2: on main branch.
BRANCH=$(git rev-parse --abbrev-ref HEAD)
[ "$BRANCH" = "main" ] || { echo "FAIL: not on main (on $BRANCH)"; exit 1; }

# Guard 3: workspace version matches the tag.
# Accepts either workspace-level Cargo.toml or per-crate manifest carrying the bump.
grep -q "^version = \"${VERSION}\"" Cargo.toml 2>/dev/null \
  || grep -q "^version = \"${VERSION}\"" crates/reposix-cli/Cargo.toml 2>/dev/null \
  || { echo "FAIL: version $VERSION not found in Cargo.toml or crates/reposix-cli/Cargo.toml"; echo "owner_hint: bump [workspace.package].version to ${VERSION} before tagging"; exit 1; }

# Guard 4: CHANGELOG.md has [v0.13.0] entry.
grep -q "^## \[v${VERSION}\]" CHANGELOG.md || { echo "FAIL: CHANGELOG.md missing ## [v${VERSION}] entry"; exit 1; }

# Guard 5: full test suite GREEN. Sequential per CLAUDE.md "Build memory budget" rule 1.
echo "Running cargo test --workspace (sequential per CLAUDE.md memory budget)..."
cargo test --workspace --no-fail-fast \
  || { echo "FAIL: cargo test --workspace did not pass"; exit 1; }

# Guard 6: pre-push runner GREEN.
python3 quality/runners/run.py --cadence pre-push \
  || { echo "FAIL: pre-push runner did not pass"; exit 1; }

# Guard 7: P88 verdict GREEN (milestone-close gate).
ls quality/reports/verdicts/p88/*.md >/dev/null 2>&1 \
  && grep -qE 'GREEN|brightgreen' "$(ls -t quality/reports/verdicts/p88/*.md | head -1)" \
  || { echo "FAIL: P88 verdict missing or not GREEN"; exit 1; }

# Guard 8: milestone-close verdict GREEN.
ls quality/reports/verdicts/milestone-v0.13.0/*.md >/dev/null 2>&1 \
  && grep -qE 'GREEN|brightgreen' "$(ls -t quality/reports/verdicts/milestone-v0.13.0/*.md | head -1)" \
  || { echo "FAIL: milestone-v0.13.0 verdict missing or not GREEN"; exit 1; }

# Optional: signed tag if the user has a signing key configured.
git tag -s -a "$TAG" -m "v${VERSION} -- DVCS over REST" 2>/dev/null \
  || git tag -a "$TAG" -m "v${VERSION} -- DVCS over REST"

echo "OK: tag $TAG created locally."
echo "Owner pushes via: git push origin $TAG"
