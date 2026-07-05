#!/usr/bin/env bash
# quality/gates/agent-ux/real-git-push-e2e.sh -- agent-ux/real-git-push-e2e verifier.
#
# Drives a REAL `git commit` + `git push` from a `reposix init`'d working
# tree (zero manual fast-export) and asserts the backend received exactly
# one PATCH with no spurious Create/Delete, and that a subsequent no-op
# push (pull, no edits, push) writes nothing.
#
# KNOWN BLOCKER (documented, expected to FAIL once the git-version gate
# below is satisfied): QL-001 in
# .planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md (2026-07-04
# entry, "discovered-by: Stage-1 Lane-A"). The push diff planner's
# path-shape mismatch (crates/reposix-remote/src/diff.rs:104-107 spells
# issue paths as bare 4-zero-padded `0042.md`, but the real on-disk shape
# everywhere else is `issues/42.md`) makes a real push against a
# `reposix init`'d tree misclassify every record as Create+Delete, and a
# separate stream-parser bug (fast_import.rs:156-157) drops the first
# M-line on a genuine no-op push (silent data loss). This verifier is
# INTENTIONALLY expected to FAIL until P90/P91 land the fix described in
# that entry's "Sketched resolution" -- the catalog row anchors that
# contract (see the row's `waiver` block). Write this script so that once
# the planner is fixed it goes GREEN WITHOUT EDITS to this file.
#
# GIT VERSION GATE: git >= 2.34 is required for the stateless-connect
# fetch path this scenario depends on (`reposix init` -> `git checkout -B
# main refs/reposix/origin/main`). git < 2.34 fails with cryptic
# ref/gitdir errors (FINDING-A in the same SURPRISES-INTAKE entry) that
# are an ENVIRONMENT gap, not a code regression -- this script detects
# that up front and reports NOT-VERIFIED (exit 75) rather than FAIL.
#
# Exit-code convention: quality/PROTOCOL.md "Verifier exit-code
# conventions" -- the runner's map_exit_code_to_status
# (quality/runners/_realbackend.py) maps exit 75 -> NOT-VERIFIED for
# EVERY verifier's subprocess result, not only pre-release-real-backend
# cadence rows. This script relies on that runner-wide convention.
#
# Implements catalog row agent-ux/real-git-push-e2e.
#
# Usage: real-git-push-e2e.sh [--row-id <id>]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

ROW_ID="agent-ux/real-git-push-e2e"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

ARTIFACT="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux/real-git-push-e2e.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# --- git version precondition ----------------------------------------------
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
    "git $GIT_VERSION < 2.34 -- reposix's stateless-connect partial-clone fetch path requires git >= 2.34 (CLAUDE.md Tech stack); the real-push scenario cannot complete \`reposix init && git checkout -B main refs/reposix/origin/main\` on this box (FINDING-A, .planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md 2026-07-04 entry) -- upgrade git and re-run"
  ]
}
EOF
  echo "NOT-VERIFIED: git ${GIT_VERSION} < 2.34 required. Exit 75 (NOT-VERIFIED convention -- quality/runners/_realbackend.py:map_exit_code_to_status applies runner-wide; see quality/PROTOCOL.md 'Verifier exit-code conventions')." >&2
  echo "  artifact: $ARTIFACT" >&2
  exit 75
fi

# --- git >= 2.34: real end-to-end scenario ---------------------------------
# SIM_BIND MUST equal reposix-cli's DEFAULT_SIM_ORIGIN (crates/reposix-cli/
# src/init.rs:24 = 127.0.0.1:7878): `reposix init sim::demo` bakes that default
# origin into remote.origin.url and translate_spec_to_url does NOT honour
# REPOSIX_SIM_ORIGIN (unlike sync.rs:84). Binding the sim on any other port
# makes init's partial-clone fetch target 7878 (where nothing is listening)
# AND trips the egress allowlist (REPOSIX_ALLOWED_ORIGINS below is derived from
# SIM_BIND) with "blocked origin: http://127.0.0.1:7878" -- observed as the
# real-git-push-e2e FAIL in CI run 28723720784. run.py runs rows sequentially
# with a kill+wait cleanup, so sharing 7878 with the dvcs-third-arm row is safe
# (and spawn_sim now fails loud if the port is somehow still occupied).
# GOOD-TO-HAVES: teach translate_spec_to_url to honour REPOSIX_SIM_ORIGIN so
# this scenario can use a collision-proof dedicated port.
SIM_BIND="127.0.0.1:7878"
RUN_DIR="/tmp/real-git-push-e2e-$$"
SIM_URL="http://${SIM_BIND}"
SIM_DB="${RUN_DIR}/sim.db"
mkdir -p "$RUN_DIR"

export REPOSIX_ALLOWED_ORIGINS="${SIM_URL}"

# Reuse the shared dark-factory helpers (build/resolve bins, spawn sim,
# fail_with, cleanup-and-write-artifact trap). These expect WORKSPACE_ROOT,
# RUN_DIR, ARTIFACT, ROW_ID, SIM_BIND, SIM_URL, SIM_DB to be set (done above).
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/dark-factory/lib.sh"

build_and_resolve_bins
spawn_sim seeded

REPO="${RUN_DIR}/repo"

echo "real-git-push-e2e: reposix init sim::demo $REPO" >&2
"${BIN_DIR}/reposix" init "sim::demo" "$REPO"

git -C "$REPO" config user.email "e2e@example.invalid"
git -C "$REPO" config user.name "real-git-push-e2e"

git -C "$REPO" checkout -B main refs/reposix/origin/main \
  || fail_with "git checkout -B main refs/reposix/origin/main failed" \
    "requires git >= 2.34 stateless-connect fetch to have populated refs/reposix/origin/main"

# Pick the lowest-id issue file present and append a real edit (not a
# synthetic fast-export stream).
ISSUE_FILE=$(find "$REPO/issues" -maxdepth 1 -name '*.md' | sort | head -1)
[[ -n "$ISSUE_FILE" ]] || fail_with "no issues/*.md file found after checkout" "$REPO/issues"
{ echo ""; echo "e2e edit $(date -u +%s)"; } >> "$ISSUE_FILE"
git -C "$REPO" add "$ISSUE_FILE"
git -C "$REPO" commit --quiet -m "real-git-push-e2e: edit one issue"

echo "real-git-push-e2e: git push origin main" >&2
git -C "$REPO" push origin main \
  || fail_with "git push origin main failed" "expected a clean single-record PATCH round-trip"

# --- Assertion 1: exactly one PATCH, zero POST/DELETE for this push -------
PATCH_COUNT=$(sqlite3 "$SIM_DB" \
  "SELECT COUNT(*) FROM audit_events WHERE method='PATCH' AND path LIKE '/projects/demo/issues/%';" 2>/dev/null || echo "-1")
POST_COUNT=$(sqlite3 "$SIM_DB" \
  "SELECT COUNT(*) FROM audit_events WHERE method='POST' AND path='/projects/demo/issues';" 2>/dev/null || echo "-1")
DELETE_COUNT=$(sqlite3 "$SIM_DB" \
  "SELECT COUNT(*) FROM audit_events WHERE method='DELETE' AND path LIKE '/projects/demo/issues/%';" 2>/dev/null || echo "-1")

[[ "$PATCH_COUNT" -eq 1 ]] || fail_with "expected exactly 1 PATCH to the backend, got ${PATCH_COUNT}" "misclassification: a real push should round-trip as a single PATCH"
[[ "$POST_COUNT" -eq 0 ]] || fail_with "expected 0 spurious POST (Create) actions, got ${POST_COUNT}" "QL-001 BUG-1 path-shape create/delete storm"
[[ "$DELETE_COUNT" -eq 0 ]] || fail_with "expected 0 spurious DELETE actions, got ${DELETE_COUNT}" "QL-001 BUG-1 path-shape create/delete storm"
ASSERT_PATCH_LOG="real push round-tripped as exactly 1 PATCH (misclassification count == 0: 0 Create, 0 Delete)"
echo "  PASS: ${ASSERT_PATCH_LOG}" >&2

# --- Assertion 2: no-op push writes nothing --------------------------------
echo "real-git-push-e2e: no-op push (pull, no edits, push)" >&2
git -C "$REPO" pull --quiet --no-rebase origin main || true
git -C "$REPO" push origin main > "${RUN_DIR}/noop-push.log" 2>&1 || true

TOTAL_AFTER=$(sqlite3 "$SIM_DB" \
  "SELECT COUNT(*) FROM audit_events WHERE method IN ('POST','PATCH','DELETE') AND path LIKE '/projects/demo/issues%';" 2>/dev/null || echo "-1")
EXPECTED_TOTAL=$((PATCH_COUNT + POST_COUNT + DELETE_COUNT))
[[ "$TOTAL_AFTER" -eq "$EXPECTED_TOTAL" ]] \
  || fail_with "no-op push wrote backend mutations" "expected ${EXPECTED_TOTAL} total mutating requests, got ${TOTAL_AFTER} (QL-001 BUG-3 stream-parser data loss)"
ASSERT_NOOP_LOG="no-op push (pull, no edits, push) wrote zero additional backend mutations"
echo "  PASS: ${ASSERT_NOOP_LOG}" >&2

ASSERTS_PASSED=$(python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" "$ASSERT_PATCH_LOG" "$ASSERT_NOOP_LOG")

echo "REAL-GIT-PUSH-E2E COMPLETE -- real git push round-trips with zero misclassification." >&2
exit 0
