#!/usr/bin/env bash
# tag-v0.12.0.sh -- v0.12.0 milestone tag-cut. Owner runs this; orchestrator does NOT push the tag.
#
# Mirrors v0.11.x guard pattern. Run from repo root.

set -euo pipefail
VERSION="0.12.0"
TAG="v${VERSION}"

# Guard 1: clean tree
git diff --quiet && git diff --cached --quiet || { echo "FAIL: working tree not clean"; exit 1; }

# Guard 2: on main
BRANCH=$(git rev-parse --abbrev-ref HEAD)
[ "$BRANCH" = "main" ] || { echo "FAIL: not on main (on $BRANCH)"; exit 1; }

# Guard 3: version match in workspace Cargo.toml or per-crate manifest
grep -q "^version = \"${VERSION}\"" Cargo.toml 2>/dev/null \
  || grep -q "^version = \"${VERSION}\"" crates/reposix-cli/Cargo.toml 2>/dev/null \
  || { echo "FAIL: version $VERSION not found in Cargo.toml or crates/reposix-cli/Cargo.toml"; exit 1; }

# Guard 4: CHANGELOG entry
grep -q "^## \[v${VERSION}\]" CHANGELOG.md || { echo "FAIL: CHANGELOG.md missing ## [v${VERSION}] entry"; exit 1; }

# Guard 5: ci.yml latest test job is success for the current branch
gh run list --workflow ci.yml --branch main --limit 1 --json conclusion --jq '.[0].conclusion' | grep -q success \
  || { echo "FAIL: latest ci.yml on main is not success"; exit 1; }

# Guard 6: P63 verdict GREEN (milestone-close gate per QG-06)
ls quality/reports/verdicts/p63/*.md >/dev/null 2>&1 \
  && grep -qE 'GREEN|brightgreen' "$(ls -t quality/reports/verdicts/p63/*.md | head -1)" \
  || { echo "FAIL: P63 verdict missing or not GREEN"; exit 1; }

# Optional: signed tag if the user has a signing key configured
git tag -s -a "$TAG" -m "v${VERSION} -- Quality Gates" 2>/dev/null \
  || git tag -a "$TAG" -m "v${VERSION} -- Quality Gates"

echo "OK: tag $TAG created locally."
echo "Owner pushes via: git push origin $TAG"
