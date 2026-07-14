#!/usr/bin/env bash
# .planning/phases/94-real-backend-frictions/94-D4-sweep.sh
# P94 D4 — canonical all-rows catalog-freshness sweep (the reproduce driver for
# catalog row structure/p94-catalog-freshness-sweep).
#
# WHY A DRIVER (not `run.py --all`): the D4 catalog row's original `command`
# named `python3 quality/runners/run.py --all`, but run.py has NO `--all` flag
# (argparse REQUIRES --cadence; a row-level/all-rows scope flag is deferred
# GOOD-TO-HAVES-03 runner surgery). The canonical all-rows sweep today is
# "run every cadence once, in series" — their union covers every cadence-tagged
# catalog row (144 of them; the 393 cadence-less docs-alignment rows are graded
# by the doc-alignment walker, out of this sweep by design).
#
# BUILD-MEMORY BUDGET (CLAUDE.md, the #1 guardrail — VM OOM-crashed 3x on
# parallel cargo): every run.py invocation runs its verifiers SEQUENTIALLY
# (blocking subprocess.run), and this driver runs the cadences SEQUENTIALLY, so
# there is never more than ONE cargo invocation in flight. Do NOT launch this in
# parallel with any other cargo work.
#
# SELF-MUTATION: FIXED in P96 (D-P96-01). This driver runs each cadence with a
# bare `run.py --cadence <c>` (no --persist), which since the P96 GRADE/PERSIST
# split is VALIDATE-ONLY — it grades in memory and writes per-row artifacts but
# does NOT rewrite catalog JSON. The `git checkout HEAD -- quality/catalogs/`
# revert this comment used to mandate is NO LONGER NEEDED (and is now harmful in
# a worktree — see root CLAUDE.md); the P94 rows stay NOT-VERIFIED on disk on
# their own. Regression-locked by structure/catalog-immutable-on-read. The
# sweep's value is still the captured re-grade output.
#
# Usage: bash .planning/phases/94-real-backend-frictions/94-D4-sweep.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# .planning/phases/94-real-backend-frictions is 3 levels below the repo root.
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
cd "$REPO_ROOT"

ART="${REPO_ROOT}/.planning/phases/94-real-backend-frictions/94-freshness-sweep.txt"
: > "$ART"

{
  echo "# P94 D4 — catalog-freshness sweep (all cadences, sequential)"
  echo "# generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "# git HEAD: $(git rev-parse --short HEAD)"
  echo "# git version: $(git --version)"
  echo "# NOTE: run.py has no --all flag; canonical sweep = every cadence once (GOOD-TO-HAVES-03)."
  echo
} >> "$ART"

CADENCES=(pre-commit pre-push pre-pr weekly pre-release post-release on-demand pre-release-real-backend)

for cad in "${CADENCES[@]}"; do
  {
    echo "################################################################################"
    echo "## CADENCE: ${cad}   ($(date -u +%H:%M:%SZ))"
    echo "################################################################################"
  } >> "$ART"
  # Do NOT abort the sweep if a cadence exits nonzero — run.py exits 1 whenever
  # any P0/P1 row is not PASS/WAIVED (expected for the known NOT-VERIFIED rows).
  python3 quality/runners/run.py --cadence "$cad" >> "$ART" 2>&1
  rc=$?
  echo ">>> cadence ${cad} run.py exit=${rc}" >> "$ART"
  echo >> "$ART"
done

{
  echo "################################################################################"
  echo "## VERDICT (all rows, no cadence filter)   ($(date -u +%H:%M:%SZ))"
  echo "################################################################################"
} >> "$ART"
python3 quality/runners/verdict.py >> "$ART" 2>&1
echo ">>> verdict.py exit=$?" >> "$ART"

echo >> "$ART"
echo "# SWEEP COMPLETE $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$ART"
echo "SWEEP COMPLETE — evidence at $ART"
