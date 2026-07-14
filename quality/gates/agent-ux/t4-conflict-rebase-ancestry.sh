#!/usr/bin/env bash
# quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh -- agent-ux/t4-conflict-rebase-ancestry
# verifier.
#
# T4 prove-before-fix regression lock (P92, DP-2 discipline). The heavy mechanism fix
# already landed pre-phase (`cb630e5`: scrub inherited GIT_*  env before the bare-cache
# `git config` shell-out in `Cache::open`). This verifier does NOT re-fix anything -- it
# PROVES the fix holds by driving the exact two-writer conflict + refetch scenario the
# original dark-factory T4 finding (2026-05-02, HIGH-1) exercised, and asserts the specific
# regression: a helper-side refetch after a rejected/conflicting push must NOT mint a fresh,
# disconnected root commit.
#
# Two INDEPENDENT working trees (A, B), each with its OWN REPOSIX_CACHE_DIR (two separate
# bare caches) -- the realistic two-agent/two-machine topology, matching the original T4
# test's structure. (A shared-cache, single-machine topology was tried first and found NOT
# to trigger conflict detection at all -- the shared cache's own delta-sync absorbs the
# other writer's change before the second push's precheck runs; that is a different,
# separately-filed finding, not this regression's scope.)
#
# Sequence: A edits + pushes (baseline, succeeds) -> B edits the SAME record (stale base,
# its cache never saw A's write) + pushes (must be REJECTED: version mismatch / fetch
# first) -> B recovers via `git fetch origin` -> assert `refs/reposix/origin/main`'s ROOT
# commit (`git rev-list --max-parents=0`) is IDENTICAL before and after the refetch, AND
# that the ref actually ADVANCED (proving the assertion isn't vacuously true because nothing
# happened).
#
# NOTE: this verifier deliberately does NOT drive `git pull --rebase` to completion --
# `git rebase`'s own 3-way merge hits a SEPARATE, newly-discovered bug (cache delta-sync
# under-reports changed records, "not our ref" on the blob the merge needs) that is
# unrelated to the HIGH-1 ancestry mechanism this row locks. See
# .planning/milestones/v0.13.0-phases/92-push-flow-correctness/92-T4-REPRO-NOTES.md for the full finding and
# .planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md for the filed follow-up.
#
# GIT VERSION GATE: git >= 2.34 is required (CLAUDE.md Tech stack; same convention as
# agent-ux/real-git-push-e2e -- see that script's header for the exit-75 NOT-VERIFIED
# rationale, which this script reuses verbatim).
#
# Implements catalog row agent-ux/t4-conflict-rebase-ancestry.
#
# Usage: t4-conflict-rebase-ancestry.sh [--row-id <id>]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

ROW_ID="agent-ux/t4-conflict-rebase-ancestry"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

ARTIFACT="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# --- git version precondition (verbatim convention: agent-ux/real-git-push-e2e) -----------
GIT_VERSION="$(git --version | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?' | head -1)"
GIT_MAJOR="${GIT_VERSION%%.*}"
GIT_REST="${GIT_VERSION#*.}"
GIT_MINOR="${GIT_REST%%.*}"

git_too_old=0
if [[ "$GIT_MAJOR" -lt 2 ]]; then
  git_too_old=1
elif [[ "$GIT_MAJOR" -eq 2 && "$GIT_MINOR" -lt 34 ]]; then
  git_too_old=1
fi

if [[ "$git_too_old" -eq 1 ]]; then
  cat > "$ARTIFACT" <<EOF
{
  "ts": "$TS",
  "row_id": "$ROW_ID",
  "exit_code": 75,
  "status": "NOT-VERIFIED",
  "reason": "git_too_old",
  "git_version": "$GIT_VERSION",
  "required": ">= 2.34",
  "asserts_passed": [],
  "asserts_failed": [
    "git $GIT_VERSION < 2.34 -- the two-writer conflict + refetch scenario requires the stateless-connect partial-clone fetch path (CLAUDE.md Tech stack) -- upgrade git and re-run"
  ]
}
EOF
  echo "NOT-VERIFIED: git ${GIT_VERSION} < 2.34 required. Exit 75 (see quality/PROTOCOL.md 'Verifier exit-code conventions')." >&2
  echo "  artifact: $ARTIFACT" >&2
  exit 75
fi

# --- git >= 2.34: real two-writer scenario -------------------------------------------------
SIM_BIND="127.0.0.1:7878"
RUN_DIR="/tmp/t4-conflict-rebase-ancestry-$$"
SIM_URL="http://${SIM_BIND}"
SIM_DB="${RUN_DIR}/sim.db"
mkdir -p "$RUN_DIR"

export REPOSIX_ALLOWED_ORIGINS="${SIM_URL}"

# shellcheck disable=SC1091
source "${SCRIPT_DIR}/dark-factory/lib.sh"

build_and_resolve_bins
export SIM_PERSIST=1
spawn_sim seeded

A="${RUN_DIR}/A"
B="${RUN_DIR}/B"
CACHE_A="${RUN_DIR}/cacheA"
CACHE_B="${RUN_DIR}/cacheB"

echo "t4-conflict-rebase-ancestry: reposix init A (own cache)" >&2
REPOSIX_CACHE_DIR="$CACHE_A" "${BIN_DIR}/reposix" init "sim::demo" "$A"
git -C "$A" config user.email "writer-a@example.invalid"
git -C "$A" config user.name "writer-A"

echo "t4-conflict-rebase-ancestry: reposix init B (own cache)" >&2
REPOSIX_CACHE_DIR="$CACHE_B" "${BIN_DIR}/reposix" init "sim::demo" "$B"
git -C "$B" config user.email "writer-b@example.invalid"
git -C "$B" config user.name "writer-B"

git -C "$A" checkout -B main refs/reposix/origin/main \
  || fail_with "git checkout -B main (A)" "requires git >= 2.34 stateless-connect fetch"
git -C "$B" checkout -B main refs/reposix/origin/main \
  || fail_with "git checkout -B main (B)" "requires git >= 2.34 stateless-connect fetch"

# NOTE: A and B use SEPARATE caches, each independently built from its own
# `sync(sim:demo): N issues at <wall-clock timestamp>` commit -- ROOT_A and
# ROOT_B are NOT expected to match each other (different cache, different
# build timestamp, different hash). The regression this row locks is about
# B's OWN root commit staying identical ACROSS B's OWN refetch, not about A
# and B converging on a shared root.
ROOT_B="$(git -C "$B" rev-list --max-parents=0 refs/heads/main | tail -1)"

ISSUE_FILE_A=$(find "$A/issues" -maxdepth 1 -name '*.md' | sort | head -1)
[[ -n "$ISSUE_FILE_A" ]] || fail_with "no issues/*.md file found after A's checkout" "$A/issues"
ISSUE_BASENAME="$(basename "$ISSUE_FILE_A")"
ISSUE_FILE_B="${B}/issues/${ISSUE_BASENAME}"

# B's conflict-detection precheck compares the backend's `updated_at` against
# B's `last_fetched_at` cursor (list_changed_since). A 2s gap here guarantees
# A's upcoming edit lands in a STRICTLY later wall-clock second than B's
# cursor, so the comparison can't collide at whatever timestamp precision the
# sim stores -- without this gap the scenario is flaky (observed: B's stale
# push occasionally NOT rejected when A's edit and B's cursor land in the
# same second).
sleep 2

echo "t4-conflict-rebase-ancestry: A edits + pushes (baseline)" >&2
{ echo ""; echo "A-edit-$(date -u +%s)"; } >> "$ISSUE_FILE_A"
git -C "$A" add "issues/${ISSUE_BASENAME}"
git -C "$A" commit --quiet -m "A: edit ${ISSUE_BASENAME}"
REPOSIX_CACHE_DIR="$CACHE_A" git -C "$A" push origin main \
  || fail_with "A's baseline push failed" "expected a clean single-writer push to succeed"
ASSERT_A_PUSH_LOG="A's baseline push (no conflict) succeeded"
echo "  PASS: ${ASSERT_A_PUSH_LOG}" >&2

echo "t4-conflict-rebase-ancestry: B edits the SAME record (stale base) + pushes (expect rejection)" >&2
{ echo ""; echo "B-edit-$(date -u +%s)"; } >> "$ISSUE_FILE_B"
git -C "$B" add "issues/${ISSUE_BASENAME}"
git -C "$B" commit --quiet -m "B: edit ${ISSUE_BASENAME}"

set +e
B_PUSH1_LOG="${RUN_DIR}/b-push1.log"
REPOSIX_CACHE_DIR="$CACHE_B" git -C "$B" push origin main > "$B_PUSH1_LOG" 2>&1
B_PUSH1_EXIT=$?
set -e
if [[ "$B_PUSH1_EXIT" -eq 0 ]]; then
  fail_with "B's stale-base push should have been REJECTED but exited 0" \
    "$(cat "$B_PUSH1_LOG")"
fi
grep -qE 'version mismatch|fetch first' "$B_PUSH1_LOG" \
  || fail_with "B's rejected push did not name the expected conflict (version mismatch / fetch first)" \
    "$(cat "$B_PUSH1_LOG")"
ASSERT_REJECT_LOG="B's stale-base push against the SAME record A just pushed was correctly rejected (version mismatch / fetch first)"
echo "  PASS: ${ASSERT_REJECT_LOG}" >&2

echo "t4-conflict-rebase-ancestry: B recovers via git fetch origin" >&2
REPOSIX_CACHE_DIR="$CACHE_B" git -C "$B" fetch origin \
  || fail_with "B's recovery git fetch origin failed" \
    "this is the exact HIGH-1 regression path -- a refetch after a rejected push must succeed"

NEW_ROOT_B="$(git -C "$B" rev-list --max-parents=0 refs/reposix/origin/main | tail -1)"
[[ "$NEW_ROOT_B" == "$ROOT_B" ]] \
  || fail_with "HIGH-1 REGRESSED: refetch produced a NEW root commit (no ancestry to the prior tip)" \
    "ROOT_B(before)=$ROOT_B ROOT_B(after)=$NEW_ROOT_B"
ASSERT_ANCESTRY_LOG="refetched refs/reposix/origin/main's root commit is IDENTICAL before and after the recovery fetch (no fresh disconnected root -- HIGH-1 stays fixed)"
echo "  PASS: ${ASSERT_ANCESTRY_LOG}" >&2

# Courtesy check: the ref must have actually ADVANCED (new commits beyond the shared root)
# -- otherwise the ancestry assertion above would be vacuously true because nothing happened.
COMMIT_COUNT_AFTER="$(git -C "$B" rev-list --count refs/reposix/origin/main)"
[[ "$COMMIT_COUNT_AFTER" -gt 1 ]] \
  || fail_with "refetch did not advance refs/reposix/origin/main at all" "commit count = $COMMIT_COUNT_AFTER"
ASSERT_ADVANCED_LOG="refs/reposix/origin/main genuinely advanced past the shared root (commit count=${COMMIT_COUNT_AFTER}) -- the ancestry assertion is not vacuous"
echo "  PASS: ${ASSERT_ADVANCED_LOG}" >&2

ASSERTS_PASSED=$(python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" \
  "$ASSERT_A_PUSH_LOG" "$ASSERT_REJECT_LOG" "$ASSERT_ANCESTRY_LOG" "$ASSERT_ADVANCED_LOG")

echo "T4-CONFLICT-REBASE-ANCESTRY COMPLETE -- two-writer conflict correctly rejected; recovery refetch preserves ancestry (no fresh root)." >&2
exit 0
