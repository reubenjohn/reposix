#!/usr/bin/env bash
# quality/gates/agent-ux/attach-sync-real-backend.sh — RBF-A-04 verifier
# Grades catalog row: agent-ux/attach-sync-real-backend (minted NOT-VERIFIED
# in 91-01; substrate wired in 91-03).
#
# Drives REAL `reposix attach` + `sync --reconcile` against a SANCTIONED
# real backend (Confluence TokenWorld by default) via the #[ignore]
# attach_real_confluence / sync_real_confluence smokes in
# crates/reposix-cli/tests/agent_flow_real.rs, and emits a shell-subprocess
# transcript via lib/transcript.sh (RBF-FW-02).
#
# Env-gated per the pre-release-real-backend cadence (PROTOCOL.md OD-2):
#   - creds unset             -> exit 75  (NOT-VERIFIED; honest env-gate,
#                                 runner maps 75 -> NOT-VERIFIED)
#   - creds set, sanctioned   -> run the smokes; PASS iff they pass
#   - non-sanctioned target   -> hard-FAIL exit 1 (never 75)
#   - creds set, unreachable  -> hard-FAIL exit 1 (the smokes fail)
#
# The coordinator runs this and flips the catalog row status; the verifier
# itself never edits the catalog.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

# shellcheck source=quality/gates/agent-ux/lib/transcript.sh
source "${SCRIPT_DIR}/lib/transcript.sh"

SLUG="attach-sync-real-backend"

# --- Env gate (pre-release-real-backend cadence) ---------------------------
# Confluence TokenWorld is this row's default sanctioned target.
missing=()
for v in ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT; do
  if [ -z "${!v:-}" ]; then missing+=("$v"); fi
done
if [ "${#missing[@]}" -gt 0 ]; then
  echo "NOT-VERIFIED: real-backend creds unset: ${missing[*]}" >&2
  echo "  env-gate: exit 75 -> runner maps to NOT-VERIFIED (never skip-as-pass)" >&2
  exit 75
fi
if [ -z "${REPOSIX_ALLOWED_ORIGINS:-}" ]; then
  echo "NOT-VERIFIED: REPOSIX_ALLOWED_ORIGINS unset (egress allowlist required)" >&2
  echo "  env-gate: exit 75 -> runner maps to NOT-VERIFIED" >&2
  exit 75
fi

# --- Sanctioned-target assertion (OD-2 hard-FAIL, NOT 75) ------------------
# Two owner-owned spaces in the tenant are sanctioned: TokenWorld (the
# "go crazy, it's safe" mutation space, docs/reference/testing-targets.md)
# and REPOSIX (the durable-fixture space, id 360450 — D91-08 / the
# durable-fixture SURPRISES-INTAKE entry, and the space the owner's own
# .env points REPOSIX_CONFLUENCE_SPACE at). This smoke is read-only
# (attach + sync issue `list_records` only), so listing either is safe.
# Any OTHER space is a hard FAIL — we never silently probe unknown real
# state.
SPACE="${REPOSIX_CONFLUENCE_SPACE:-TokenWorld}"
case "$SPACE" in
  TokenWorld | REPOSIX) ;;
  *)
    echo "FAIL: non-sanctioned Confluence space '$SPACE' — only TokenWorld and REPOSIX are sanctioned for this row (docs/reference/testing-targets.md + durable-fixture intake)" >&2
    exit 1
    ;;
esac

# --- Build the binary the smokes shell out to (one cargo invocation) -------
cargo build -p reposix-cli --bin reposix >&2

# --- Drive the real attach + sync smokes, capturing a transcript -----------
# write_transcript_and_artifact runs the command, writes the transcript +
# JSON artifact (quality/reports/{transcripts,verifications}/…), and returns
# the command's exit code.
set +e
write_transcript_and_artifact "$SLUG" \
  cargo test -p reposix-cli --test agent_flow_real -- --ignored \
  attach_real_confluence sync_real_confluence
rc=$?
set -e

if [ "$rc" -ne 0 ]; then
  echo "FAIL: real attach/sync smokes failed (creds present but backend unreachable, or dispatch/audit regressed) — inspect the transcript" >&2
  exit 1
fi

echo "PASS: real attach + sync --reconcile round-trip against Confluence TokenWorld (transcript emitted)"
