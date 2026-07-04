#!/usr/bin/env bash
# quality/gates/agent-ux/milestone-close-vision-litmus.sh — RBF-FW-03 SLOT
#
# The milestone-close 9th probe (RBF-FW-03, blast_radius P0). Drives the
# dark-factory vision litmus VERBATIM against a SANCTIONED real backend:
# a fresh-agent vanilla-clone + `reposix attach` + edit + `git push`
# round-trip, then asserts the dual audit tables + mirror-lag refs, per
# D91-06 (the 8-step body) and D90-06 (the sanctioned-target proof
# obligation lives HERE, in the verifier body — not as a second allowlist
# in _realbackend). The multi-step round-trip + artifact patch live in the
# sourced lib/litmus-flow.sh (10k .sh file-size budget factoring).
#
# Exit-code discipline (framework A(c) / _realbackend.map_exit_code_to_status):
#   0  -> PASS      (full round-trip + dual audit + refs/mirrors advanced)
#   2  -> PARTIAL
#   75 -> NOT-VERIFIED  (env-unset / creds-absent — the HONEST env-gate;
#         runner maps 75 -> NOT-VERIFIED. NEVER exit 1 for a missing
#         precondition — that would overwrite the honest deferral signal.)
#   1  -> FAIL      (OD-2 hard-RED: sanctioned-target violation, OR the
#         documented happy path DISAGREES with binary behaviour, OR
#         creds-present-yet-substrate-cannot-execute — NOT downgraded to 75.)
#
# NEVER add a `waiver` block to the catalog row (anti-C7 / OD-2 —
# _audit_field.py SystemExits if a pre-release-real-backend row is waived).
# This verifier NEVER edits the catalog row's status — the runner grades it.
#
# SAFETY (learned the hard way — see 91-05-SUMMARY.md incident + intake):
# the confluence export/push diff only recognises `issues/<id>.md`; a working
# tree whose records live under any OTHER bucket (e.g. `pages/`, which
# `reposix refresh` currently writes for confluence) makes EVERY cached
# `issues/<id>.md` look DELETED, so a naive push MASS-DELETES the backend
# space. GUARD A + GUARD B (in lib/litmus-flow.sh) REFUSE to push a
# delete-shaped diff and hard-FAIL instead; protected durable fixtures
# (7766017/7798785, docs/reference/testing-targets.md) are on a never-edit
# denylist. Env unset -> exit 75 stays legitimate.
set -uo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

# shellcheck source=quality/gates/agent-ux/lib/transcript.sh
source "${SCRIPT_DIR}/lib/transcript.sh"

SLUG="milestone-close-vision-litmus-real-backend"
export PROTECTED_IDS=" 7766017 7798785 "   # durable fixtures — NEVER edit (D91-08)
STATE_FILE="$(mktemp)"; : > "$STATE_FILE"  # flow appends PASS:/FAIL: assert lines
export STATE_FILE REPO_ROOT
pass() { echo "PASS: $1"; echo "PASS:$1" >> "$STATE_FILE"; }
fail() { echo "FAIL: $1" >&2; echo "FAIL:$1" >> "$STATE_FILE"; }

# --- STEP 1 (part): resolve target. Default Confluence space TokenWorld
# (== REPOSIX; both keys resolve to space id 360450, the owner's demo space).
SPACE="${REPOSIX_CONFLUENCE_SPACE:-TokenWorld}"; export SPACE

# --- Env gate (pre-release-real-backend cadence) --------------------------
missing=()
for v in ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT; do
  [ -z "${!v:-}" ] && missing+=("$v")
done
if [ "${#missing[@]}" -gt 0 ]; then
  echo "NOT-VERIFIED: real-backend creds unset: ${missing[*]}" >&2
  echo "  env-gate: exit 75 -> runner maps to NOT-VERIFIED (never skip-as-pass)" >&2
  exit 75
fi
if [ -z "${REPOSIX_ALLOWED_ORIGINS:-}" ]; then
  echo "NOT-VERIFIED: REPOSIX_ALLOWED_ORIGINS unset (egress allowlist required)" >&2
  exit 75
fi

# --- STEP 1: sanctioned-target assertion (D90-06, hard-FAIL not 75) --------
# The env-gate only checks non-loopback + cred-completeness; sanctioned-target
# membership is proven HERE (D90-06). Sanctioned three: Confluence
# {TokenWorld,REPOSIX}@reuben-john / GitHub reubenjohn/reposix / JIRA
# ${JIRA_TEST_PROJECT:-TEST} (docs/reference/testing-targets.md).
case "$SPACE" in
  TokenWorld | REPOSIX) pass "sanctioned Confluence space '$SPACE' (D90-06 in-body assertion)" ;;
  *)
    echo "FAIL: non-sanctioned Confluence space '$SPACE' — only TokenWorld/REPOSIX are sanctioned (docs/reference/testing-targets.md). Refusing to touch unknown real state (D90-06)." >&2
    exit 1 ;;
esac
TENANT="${REPOSIX_CONFLUENCE_TENANT}"
if [ "$TENANT" != "reuben-john" ]; then
  echo "FAIL: non-sanctioned Confluence tenant '$TENANT' (expected reuben-john) — hard-FAIL per D90-06." >&2
  exit 1
fi

# --- STEP 2: preflight -----------------------------------------------------
# preflight-real-backends.sh is all-or-nothing across all three backends; an
# unrelated JIRA/GitHub gap must NOT sink a confluence litmus, so we run it
# for the record then GATE on a confluence-SPECIFIC reachability probe. OD-2:
# creds-present-yet-confluence-unreachable is hard-FAIL ("substrate exists,
# cannot execute"), NOT a legitimate 75.
bash scripts/preflight-real-backends.sh >&2 || echo "  (preflight non-zero — evaluating confluence-specific reachability below)" >&2
probe_code="$(curl -sS -o /dev/null -w '%{http_code}' -u "${ATLASSIAN_EMAIL}:${ATLASSIAN_API_KEY}" \
  "https://${TENANT}.atlassian.net/wiki/api/v2/spaces?keys=${SPACE}" 2>/dev/null || echo 000)"
if [ "$probe_code" != "200" ]; then
  echo "FAIL: Confluence ${SPACE} unreachable with creds set (HTTP ${probe_code}). OD-2: substrate exists but cannot execute -> hard RED (not 75)." >&2
  exit 1
fi
pass "preflight: Confluence ${SPACE} reachable (HTTP 200)"

# --- Build the CLI + git remote helper (one cargo invocation) --------------
cargo build -p reposix-cli -p reposix-remote --bin reposix --bin git-remote-reposix >&2 || {
  echo "FAIL: cargo build of reposix + git-remote-reposix failed" >&2; exit 1; }
export PATH="${REPO_ROOT}/target/debug:${PATH}"   # git-remote-reposix must be on PATH
export BIN="${REPO_ROOT}/target/debug/reposix"
export MIRROR_URL="${REPOSIX_LITMUS_MIRROR:-git@github.com:reubenjohn/reposix-tokenworld-mirror.git}"

# shellcheck source=quality/gates/agent-ux/lib/litmus-flow.sh
source "${SCRIPT_DIR}/lib/litmus-flow.sh"

# --- STEP 3-7: run the round-trip under the transcript wrapper (RBF-FW-02).
# NOTE: the script top is `set -uo pipefail` (NO errexit) on purpose — the
# flow controls its own exit via explicit returns, and we must NOT let a
# post-flow hiccup mask the flow's honest rc. The transcript lib leaves
# errexit ON internally, so `|| rc=$?` captures the flow's non-zero exit
# WITHOUT the bare-command errexit aborting the script; `set +e` then
# restores our no-errexit contract for the patch + final echo.
rc=0
write_transcript_and_artifact "$SLUG" _litmus_flow || rc=$?
set +e

# --- STEP 7 (patch): asserts_passed/failed for grade-time congruence (F-K4b).
patch_litmus_artifact "${REPO_ROOT}/quality/reports/verifications/agent-ux/${SLUG}.json" "$STATE_FILE" "$rc" || true
rm -f "$STATE_FILE"

if [ "$rc" -eq 0 ]; then
  echo "PASS: vision litmus round-trip against Confluence ${SPACE} (transcript emitted)"
elif [ "$rc" -eq 75 ]; then
  echo "NOT-VERIFIED: env/precondition gap (exit 75)" >&2
else
  echo "FAIL (exit ${rc}): vision litmus did not complete — inspect the transcript + asserts_failed. Substrate not milestone-close-ready." >&2
fi
exit "$rc"
