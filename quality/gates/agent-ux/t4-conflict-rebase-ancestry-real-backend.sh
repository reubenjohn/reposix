#!/usr/bin/env bash
# quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh --
# agent-ux/t4-conflict-rebase-ancestry-real-backend verifier (B4, P0,
# quick 260712-phc).
#
# REAL-BACKEND arm of the sim-arm sibling
# quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh (P92, DP-2
# prove-before-fix discipline) -- same two-independent-writer topology,
# ported onto the sanctioned Confluence TokenWorld space
# (docs/reference/testing-targets.md) instead of the local sim.
#
# Two INDEPENDENT working trees (A, B), each with its OWN REPOSIX_CACHE_DIR
# (two separate bare caches). A edits + pushes (baseline, succeeds); B edits
# the SAME record with a stale base + pushes (must be REJECTED: version
# mismatch / fetch first); B recovers via `git fetch origin`; asserts B's
# root commit (`git rev-list --max-parents=0`) is IDENTICAL before and after
# the refetch (no fresh disconnected root -- the original HIGH-1 symptom),
# AND that the ref genuinely advanced (non-vacuous).
#
# CONFLUENCE BUCKET (load-bearing): confluence records live under `pages/`,
# NOT `issues/` (`bucket_for_backend("confluence") == "pages"`,
# crates/reposix-core/src/path.rs). This script globs the record from the
# pages/ bucket -- never hardcode issues/ (that spelling is sim/GitHub/JIRA
# specific).
#
# MASS-DELETE GUARD (critical -- real TokenWorld safety, learned the hard
# way in quality/gates/agent-ux/milestone-close-vision-litmus.sh): a
# confluence export/push whose diff mis-recognises the working tree's
# bucket can make every cached record look DELETED and mass-delete the
# space. `assert_safe_push_diff` below refuses (hard exit 1, never pushes)
# any push whose pending commit diff (HEAD~1..HEAD) contains a deletion, or
# touches a page id on the protected-fixture denylist (7766017 / 7798785,
# docs/reference/testing-targets.md). This scenario only ever appends a
# line to ONE existing page in place -- it never creates or deletes pages.
# NOTICING (per the ownership charter): as of the Wave-5.5 fix documented in
# lib/litmus-flow.sh's GUARD B, the push planner is now id-keyed
# (bucket-agnostic), so the historical issues/-only-recognition bug that
# motivated this guard is believed already closed at the planner layer --
# this guard remains as defense-in-depth, not because the planner bug is
# known-live. If a FUTURE real run of this script ever hits the guard, that
# is itself a real finding worth a fresh SURPRISES-INTAKE entry, not a
# reason to loosen the guard.
#
# GIT VERSION GATE: git >= 2.34 required (same convention + rationale as the
# sim-arm sibling and agent-ux/real-git-push-e2e).
#
# Exit-code discipline (quality/runners/_realbackend.py:map_exit_code_to_status):
#   0  -> PASS          full two-writer round-trip against real TokenWorld
#   75 -> NOT-VERIFIED  env-gate (creds/allowlist unset) OR git < 2.34 --
#                       NEVER a soft-skip for creds-present-but-wrong-target
#   1  -> FAIL          non-sanctioned target/tenant, mass-delete guard trip,
#                       or the documented happy path disagrees with the
#                       binary's actual behaviour (OD-2 hard-RED)
#
# NEVER add a `waiver` block to this row (OD-2 / anti-C7) -- this verifier
# never edits the catalog; the runner grades it.
#
# Implements catalog row agent-ux/t4-conflict-rebase-ancestry-real-backend.
#
# Usage: t4-conflict-rebase-ancestry-real-backend.sh
set -uo pipefail   # deliberately NOT errexit: every exit path below is
                    # explicit (not_verified_exit / hard_fail_exit / the
                    # final `exit 0`) so the EXIT trap can ALWAYS write a
                    # well-formed artifact, even on an unguarded failure.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

ROW_ID="agent-ux/t4-conflict-rebase-ancestry-real-backend"
ARTIFACT="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry-real-backend.json"
mkdir -p "$(dirname "$ARTIFACT")"

# denylist of durable Confluence fixtures this scenario must NEVER edit
# (space-padded for a substring-safe `case ... *" $id "*` membership test).
PROTECTED_IDS=" 7766017 7798785 "

STATUS="FAIL"
REASON_FIELD=""
ASSERTS_PASSED_JSON='[]'
ASSERTS_FAILED_JSON='[]'
PASSED_LABELS=()

# --- artifact writer (fires on EVERY exit path via the EXIT trap) ----------
finalize_artifact() {
  local ec=$?
  if [[ "$ec" -eq 0 && "$STATUS" != "NOT-VERIFIED" ]]; then
    STATUS="PASS"
    ASSERTS_PASSED_JSON="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "${PASSED_LABELS[@]}")"
  fi
  local reason_json="null"
  if [[ -n "$REASON_FIELD" ]]; then
    reason_json="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1]))' "$REASON_FIELD")"
  fi
  python3 - "$ARTIFACT" "$ROW_ID" "$ec" "$STATUS" "$ASSERTS_PASSED_JSON" "$ASSERTS_FAILED_JSON" "$reason_json" <<'PY'
import json, sys
from datetime import datetime, timezone

path, row_id, ec, status, passed_json, failed_json, reason_json = sys.argv[1:8]
data = {
    "ts": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "row_id": row_id,
    "exit_code": int(ec),
    "status": status,
    "asserts_passed": json.loads(passed_json),
    "asserts_failed": json.loads(failed_json),
}
reason = json.loads(reason_json)
if reason is not None:
    data["reason"] = reason
    if status == "NOT-VERIFIED":
        data["skip_reason"] = "env-missing" if "unset" in reason or "credential" in reason else "precondition-not-met"
with open(path, "w", encoding="utf-8") as fh:
    json.dump(data, fh, indent=2)
    fh.write("\n")
PY
  exit "$ec"
}
trap finalize_artifact EXIT

note_pass() {
  PASSED_LABELS+=("$1")
  echo "PASS: $1" >&2
}

not_verified_exit() {  # not_verified_exit <reason> <asserts_failed...>
  local reason="$1"; shift
  STATUS="NOT-VERIFIED"
  REASON_FIELD="$reason"
  ASSERTS_FAILED_JSON="$(python3 -c 'import json,sys; print(json.dumps(sys.argv[1:]))' "$@")"
  echo "NOT-VERIFIED: ${reason}" >&2
  echo "  env-gate: exit 75 -> runner maps to NOT-VERIFIED (never skip-as-pass, OD-2)" >&2
  exit 75
}

hard_fail_exit() {  # hard_fail_exit <label> [<detail>]
  local label="$1" detail="${2:-}"
  STATUS="FAIL"
  if [[ -n "$detail" ]]; then
    echo "FAIL: ${label}: ${detail}" >&2
  else
    echo "FAIL: ${label}" >&2
  fi
  ASSERTS_FAILED_JSON="$(python3 -c 'import json,sys; print(json.dumps([sys.argv[1]]))' "$label")"
  exit 1
}

# --- 1. ENV-GATE FIRST (before any cargo/init -- the hermetic property) ---
missing=()
for v in ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT; do
  [ -z "${!v:-}" ] && missing+=("$v")
done
[ -z "${REPOSIX_ALLOWED_ORIGINS:-}" ] && missing+=("REPOSIX_ALLOWED_ORIGINS")
if [ "${#missing[@]}" -gt 0 ]; then
  not_verified_exit "real-backend creds/allowlist unset: ${missing[*]}" "${missing[@]}"
fi

# --- 2. Sanctioned-target guard (OD-2 hard-FAIL, NOT 75) -------------------
# This row MUTATES the backend (appends to a real page), so ONLY TokenWorld
# is sanctioned for it -- unlike the read-only attach-sync-real-backend row,
# which also permits the REPOSIX durable-fixture space alias.
SPACE="${REPOSIX_CONFLUENCE_SPACE:-TokenWorld}"
case "$SPACE" in
  TokenWorld) ;;
  *) hard_fail_exit "non-sanctioned Confluence space '${SPACE}' -- only TokenWorld is sanctioned for this mutating row (docs/reference/testing-targets.md)" ;;
esac
TENANT="${REPOSIX_CONFLUENCE_TENANT}"
if [ "$TENANT" != "reuben-john" ]; then
  hard_fail_exit "non-sanctioned Confluence tenant '${TENANT}' (expected reuben-john)"
fi

# --- 3. git >= 2.34 precondition (verbatim convention: sim-arm sibling) ----
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
  not_verified_exit "git ${GIT_VERSION} < 2.34 -- the two-writer conflict + refetch scenario requires the stateless-connect partial-clone fetch path (CLAUDE.md Tech stack) -- upgrade git and re-run" \
    "git ${GIT_VERSION} < 2.34"
fi

# --- 4. Build binaries once (ONE cargo invocation, never --workspace) ------
cargo build -p reposix-cli -p reposix-remote --bin reposix --bin git-remote-reposix >&2 \
  || hard_fail_exit "cargo build of reposix + git-remote-reposix failed"
BIN_DIR="${WORKSPACE_ROOT}/target/debug"
export PATH="${BIN_DIR}:${PATH}"   # git-remote-reposix must be resolvable by `git push`

# --- 5. Two-cache scenario in a /tmp run dir (leaf isolation) --------------
RUN_DIR="/tmp/t4-conflict-rebase-ancestry-real-backend-$$"
mkdir -p "$RUN_DIR"
A="${RUN_DIR}/A"
B="${RUN_DIR}/B"
CACHE_A="${RUN_DIR}/cacheA"
CACHE_B="${RUN_DIR}/cacheB"

echo "t4-conflict-rebase-ancestry-real-backend: reposix init A (own cache) against confluence::${SPACE}" >&2
REPOSIX_CACHE_DIR="$CACHE_A" "${BIN_DIR}/reposix" init "confluence::${SPACE}" "$A" \
  || hard_fail_exit "reposix init A (confluence::${SPACE}) failed"
git -C "$A" config user.email "writer-a-confluence@example.invalid"
git -C "$A" config user.name "writer-A-confluence"

echo "t4-conflict-rebase-ancestry-real-backend: reposix init B (own cache) against confluence::${SPACE}" >&2
REPOSIX_CACHE_DIR="$CACHE_B" "${BIN_DIR}/reposix" init "confluence::${SPACE}" "$B" \
  || hard_fail_exit "reposix init B (confluence::${SPACE}) failed"
git -C "$B" config user.email "writer-b-confluence@example.invalid"
git -C "$B" config user.name "writer-B-confluence"

git -C "$A" checkout -B main refs/reposix/origin/main \
  || hard_fail_exit "git checkout -B main (A) failed" "requires git >= 2.34 stateless-connect fetch"
git -C "$B" checkout -B main refs/reposix/origin/main \
  || hard_fail_exit "git checkout -B main (B) failed" "requires git >= 2.34 stateless-connect fetch"
note_pass "two independent working trees against TokenWorld: A and B each bootstrapped via reposix init confluence::${SPACE} with SEPARATE REPOSIX_CACHE_DIR caches (cacheA/cacheB)"

# ROOT_B captured right after checkout, BEFORE any edits -- this is the
# ancestry baseline the refetch below must not disturb.
ROOT_B="$(git -C "$B" rev-list --max-parents=0 refs/heads/main | tail -1)"

# --- 6. Bucket-aware record path: confluence == pages/, NOT issues/ --------
# Honor the protected-fixture denylist DURING selection (mirrors
# lib/litmus-flow.sh's target-picking loop), never as a post-hoc check only.
PAGE_FILE_A=""
for md in "$A"/pages/*.md; do
  [ -e "$md" ] || continue
  id="$(basename "$md" .md)"
  case "$PROTECTED_IDS" in *" $id "*) continue ;; esac
  PAGE_FILE_A="$md"
  break
done
[ -n "$PAGE_FILE_A" ] || hard_fail_exit "no editable non-protected pages/<id>.md record found in A's checkout" "$A/pages (bucket_for_backend(confluence)==pages, never issues)"
PAGE_BASENAME="$(basename "$PAGE_FILE_A")"
PAGE_FILE_B="${B}/pages/${PAGE_BASENAME}"

# --- MASS-DELETE GUARD: refuse any delete-shaped diff before EITHER push ---
assert_safe_push_diff() {
  local tree="$1" label="$2" diff
  diff="$(git -C "$tree" diff --name-status HEAD~1 HEAD 2>&1)" \
    || hard_fail_exit "${label}: could not compute the pending-push diff" "$diff"
  echo "${label} pending-push diff (HEAD~1..HEAD):" >&2
  printf '%s\n' "$diff" | sed 's/^/  /' >&2
  if printf '%s\n' "$diff" | grep -qE '^D'; then
    hard_fail_exit "MASS-DELETE GUARD: ${label} push would delete file(s) -- refusing to push a delete-shaped diff (confluence pages/ bucket safety, per milestone-close-vision-litmus.sh)" "$diff"
  fi
  while IFS=$'\t' read -r _status fname; do
    [ -z "$fname" ] && continue
    local base id
    base="$(basename "$fname")"
    id="${base%.md}"
    case "$PROTECTED_IDS" in
      *" $id "*) hard_fail_exit "MASS-DELETE GUARD: ${label} diff touches PROTECTED fixture id ${id}" "${fname} is on the never-edit denylist (${PROTECTED_IDS})" ;;
    esac
  done <<< "$diff"
  note_pass "${label} pending-push diff is a safe single-file in-place edit (no deletions, no protected fixture ids touched)"
}

# The backend's `updated_at` cursor comparison needs A's edit to land in a
# strictly later wall-clock second than B's `last_fetched_at` cursor, or the
# conflict-detection precheck can collide at whatever timestamp precision
# the backend stores (verbatim rationale + fix from the sim-arm sibling).
sleep 2

# --- 7. Scenario: A baseline push, B stale-base push (must be rejected) ---
echo "t4-conflict-rebase-ancestry-real-backend: A edits + pushes (baseline)" >&2
{ echo ""; echo "A-edit-real-backend-$(date -u +%s)"; } >> "$PAGE_FILE_A"
git -C "$A" add "pages/${PAGE_BASENAME}"
git -C "$A" commit --quiet -m "A: edit ${PAGE_BASENAME} (real-backend conflict/ancestry probe)"
assert_safe_push_diff "$A" "A baseline"
REPOSIX_CACHE_DIR="$CACHE_A" git -C "$A" push origin main \
  || hard_fail_exit "A's baseline push failed" "expected a clean single-writer push to succeed against real TokenWorld"
note_pass "A's baseline push (no conflict) against real TokenWorld succeeded"

echo "t4-conflict-rebase-ancestry-real-backend: B edits the SAME record (stale base) + pushes (expect rejection)" >&2
{ echo ""; echo "B-edit-real-backend-$(date -u +%s)"; } >> "$PAGE_FILE_B"
git -C "$B" add "pages/${PAGE_BASENAME}"
git -C "$B" commit --quiet -m "B: edit ${PAGE_BASENAME} (real-backend conflict/ancestry probe)"
assert_safe_push_diff "$B" "B stale-base"

B_PUSH1_LOG="${RUN_DIR}/b-push1.log"
set +e
REPOSIX_CACHE_DIR="$CACHE_B" git -C "$B" push origin main > "$B_PUSH1_LOG" 2>&1
B_PUSH1_EXIT=$?
set -e
if [[ "$B_PUSH1_EXIT" -eq 0 ]]; then
  hard_fail_exit "B's stale-base push should have been REJECTED but exited 0" "$(cat "$B_PUSH1_LOG")"
fi
grep -qE 'version mismatch|fetch first' "$B_PUSH1_LOG" \
  || hard_fail_exit "B's rejected push did not name the expected conflict (version mismatch / fetch first)" "$(cat "$B_PUSH1_LOG")"
note_pass "two independent working trees against TokenWorld: B's stale-base push against the SAME record A just pushed was correctly rejected (version mismatch / fetch first) -- conflict reject proven"

echo "t4-conflict-rebase-ancestry-real-backend: B recovers via git fetch origin" >&2
REPOSIX_CACHE_DIR="$CACHE_B" git -C "$B" fetch origin \
  || hard_fail_exit "B's recovery git fetch origin failed" "this is the exact HIGH-1 regression path -- a refetch after a rejected push must succeed"

NEW_ROOT_B="$(git -C "$B" rev-list --max-parents=0 refs/reposix/origin/main | tail -1)"
[[ "$NEW_ROOT_B" == "$ROOT_B" ]] \
  || hard_fail_exit "HIGH-1 REGRESSED: refetch produced a NEW root commit (no ancestry to the prior tip)" "ROOT_B(before)=$ROOT_B ROOT_B(after)=$NEW_ROOT_B"
note_pass "B's recovery refetch produced no fresh root on refetch -- refs/reposix/origin/main's root commit stayed IDENTICAL before and after the refetch against real TokenWorld (no fresh disconnected root, HIGH-1 stays fixed)"

COMMIT_COUNT_AFTER="$(git -C "$B" rev-list --count refs/reposix/origin/main)"
[[ "$COMMIT_COUNT_AFTER" -gt 1 ]] \
  || hard_fail_exit "refetch did not advance refs/reposix/origin/main at all" "commit count = $COMMIT_COUNT_AFTER"
note_pass "refs/reposix/origin/main genuinely advanced past the shared root (commit count=${COMMIT_COUNT_AFTER}) -- the ancestry assertion above is not vacuous, matching the sim arm's non-vacuous check"

# --- 8. Cleanup: this flow only edits an existing matched page in place ----
rm -rf "$RUN_DIR"

echo "T4-CONFLICT-REBASE-ANCESTRY-REAL-BACKEND COMPLETE -- two-writer conflict correctly rejected against real TokenWorld; recovery refetch preserves ancestry (no fresh root)." >&2
exit 0
