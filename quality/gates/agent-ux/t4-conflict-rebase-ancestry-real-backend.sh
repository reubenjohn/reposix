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

# --- 5-8. Two-cache scenario + mass-delete guard (factored under the 10k
# .sh file-size budget, quality/CLAUDE.md "File-size limits" -- mirrors the
# dark-factory/lib.sh + lib/litmus-flow.sh sourced-helper precedent). The lib
# below needs BIN_DIR, SPACE, PROTECTED_IDS, hard_fail_exit(), note_pass() --
# all already in scope from steps 1-4 above.
# shellcheck source=quality/gates/agent-ux/lib/t4-real-backend-flow.sh
source "${SCRIPT_DIR}/lib/t4-real-backend-flow.sh"
_t4_real_backend_flow

exit 0
