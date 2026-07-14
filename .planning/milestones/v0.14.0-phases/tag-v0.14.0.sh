#!/usr/bin/env bash
# tag-v0.14.0.sh -- v0.14.0 milestone tag-cut.
#
# *** THIS SCRIPT IS THE RELEASE MANAGER'S TO RUN. ***
# The authoring agent (the session that wrote this file) does NOT execute it and does
# NOT push the tag -- authoring this file is not the same as running it. Pushing tag
# v0.14.0 triggers the canonical multi-platform release at .github/workflows/release.yml
# (tag pattern `v*`). Per-package release-plz tags/crates.io publishes are unaffected
# (release-plz.toml keeps git_release_enable = false; see that file's header for why).
#
# Mirrors the v0.12.0 guard pattern
# (.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh, the active reference) plus the
# milestone-close-verdict guard from the disabled v0.13.0 template
# (.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh.disabled). Run from repo root.
#
# v0.14.0 ships GREEN-WITH-RECORDED-CAVEATS, not a clean GREEN -- see Guard 7/8 below
# and the tag message this script writes. This is expected and does not block tagging;
# it is the milestone's honestly-graded state per Manager Ruling #4 (Option B),
# .planning/CONSULT-DECISIONS.md.
#
# Usage: bash .planning/milestones/v0.14.0-phases/tag-v0.14.0.sh
# (this script performs the push itself, after an interactive confirmation -- see the
# final lines. There is no separate "owner pushes via ..." manual step.)

set -euo pipefail
VERSION="0.14.0"
TAG="v${VERSION}"

# Guard 1: clean working tree.
git diff --quiet && git diff --cached --quiet \
  || { echo "FAIL: working tree not clean"; echo "owner_hint: commit or stash pending changes before tagging"; exit 1; }

# Guard 2: on main branch.
BRANCH=$(git rev-parse --abbrev-ref HEAD)
[ "$BRANCH" = "main" ] || { echo "FAIL: not on main (on $BRANCH)"; exit 1; }

# Guard 3: workspace version matches the tag.
# Accepts either workspace-level Cargo.toml or per-crate manifest carrying the bump.
grep -q "^version = \"${VERSION}\"" Cargo.toml 2>/dev/null \
  || grep -q "^version = \"${VERSION}\"" crates/reposix-cli/Cargo.toml 2>/dev/null \
  || { echo "FAIL: version $VERSION not found in Cargo.toml or crates/reposix-cli/Cargo.toml"; echo "owner_hint: bump [workspace.package].version to ${VERSION} before tagging (release-plz version-bump PR)"; exit 1; }

# Guard 4: CHANGELOG.md has the [v0.14.0] entry.
grep -q "^## \[v${VERSION}\]" CHANGELOG.md || { echo "FAIL: CHANGELOG.md missing ## [v${VERSION}] entry"; exit 1; }

# Guard 5: ci.yml latest run on main is success.
gh run list --workflow ci.yml --branch main --limit 1 --json conclusion --jq '.[0].conclusion' | grep -q success \
  || { echo "FAIL: latest ci.yml on main is not success"; exit 1; }

# Guard 6: milestone-close verdict exists and is GREEN (GREEN-WITH-RECORDED-CAVEATS
# qualifies -- the substring match accepts it same as a clean GREEN; the caveat itself
# is recorded, not hidden, per Manager Ruling #4 / Option B).
VERDICT_FILE="quality/reports/verdicts/milestone-v${VERSION}/VERDICT.md"
[ -f "$VERDICT_FILE" ] || { echo "FAIL: milestone verdict missing at $VERDICT_FILE"; exit 1; }
grep -qE 'GREEN|brightgreen' "$VERDICT_FILE" || { echo "FAIL: $VERDICT_FILE is not GREEN"; exit 1; }

# Guard 7: independent milestone-close ratification exists and is GREEN.
RATIFICATION_FILE="quality/reports/verdicts/milestone-v${VERSION}/RATIFICATION.md"
[ -f "$RATIFICATION_FILE" ] || { echo "FAIL: milestone ratification missing at $RATIFICATION_FILE"; exit 1; }
grep -qE 'GREEN|brightgreen' "$RATIFICATION_FILE" || { echo "FAIL: $RATIFICATION_FILE is not GREEN"; exit 1; }

echo "All guards passed."

TAG_MSG="v${VERSION} -- Wave-2 hardening

Ships GREEN-WITH-RECORDED-CAVEATS, not a clean GREEN (Manager Ruling #4 / Option B --
.planning/CONSULT-DECISIONS.md, \"2026-07-13 [MANAGER] Ruling #4\"). Full verdict:
quality/reports/verdicts/milestone-v${VERSION}/VERDICT.md
(independent ratification alongside it in the same directory).

Both P0 real-backend rows PASS against live TokenWorld:
  - agent-ux/milestone-close-vision-litmus-real-backend
  - agent-ux/p93-partial-failure-recovery-real-confluence

pre-release-real-backend cadence: 5 PASS / 0 FAIL / 1 NOT-VERIFIED (cadence exit 1,
recorded honestly -- never claimed as exit 0). The sole NOT-VERIFIED row is
agent-ux/t4-conflict-rebase-ancestry-real-backend's two-writer conflict+refetch
scenario, which bails PRE-mutation at a VM git-version floor (git 2.25.1 < 2.34 --
below the stateless-connect partial-clone floor the scenario needs). This is an
environment limitation of the tagging host, NOT a product regression: the sim-arm
twin of the same scenario is green in CI on every push, and \`reposix doctor\` itself
treats sub-2.34 git as WARN, not ERROR."

echo ""
echo "About to create tag $TAG with message:"
echo "---"
echo "$TAG_MSG"
echo "---"
read -r -p "Type 'yes' to create and push tag $TAG (this triggers .github/workflows/release.yml): " CONFIRM
[ "$CONFIRM" = "yes" ] || { echo "Aborted: no tag created."; exit 1; }

# Signed, annotated tag (falls back to unsigned only if no signing key is configured).
git tag -s -a "$TAG" -m "$TAG_MSG" 2>/dev/null \
  || git tag -a "$TAG" -m "$TAG_MSG"

echo "OK: tag $TAG created locally."
echo "Pushing $TAG to origin -- this triggers .github/workflows/release.yml ..."

git push origin "$TAG"
