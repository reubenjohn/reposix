#!/usr/bin/env bash
# quality/gates/agent-ux/p93-partial-failure-recovery-real-confluence.sh --
# agent-ux/p93-partial-failure-recovery-real-confluence verifier
# (RBF-LR-03 real-backend arm, catalog row minted 2026-07-05T10:30:00Z,
# coverage_kind: real-backend, cadence: pre-release-real-backend).
#
# Drives the REAL `partial_failure_recovery_real_confluence` #[ignore] smoke
# in crates/reposix-cli/tests/agent_flow_real.rs against the sanctioned
# TokenWorld Confluence space, and emits a shell-subprocess transcript via
# lib/transcript.sh (RBF-FW-02). Mirrors agent-ux/attach-sync-real-backend's
# env-gate / sanctioned-target / transcript convention verbatim.
#
# Env-gated per OD-2 (PROTOCOL.md): creds/allowlist unset -> exit 75
# (NOT-VERIFIED, fail-closed, NEVER skip-as-pass). Non-sanctioned target ->
# hard FAIL exit 1 (never 75). Creds set + smoke fails -> hard FAIL exit 1.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

# shellcheck source=quality/gates/agent-ux/lib/transcript.sh
source "${SCRIPT_DIR}/lib/transcript.sh"

SLUG="p93-partial-failure-recovery-real-confluence"

# --- Env gate (pre-release-real-backend cadence, OD-2 fail-closed) --------
missing=()
for v in ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT; do
  if [ -z "${!v:-}" ]; then missing+=("$v"); fi
done
if [ "${#missing[@]}" -gt 0 ]; then
  echo "NOT-VERIFIED: real-backend creds unset: ${missing[*]}" >&2
  echo "  env-gate: exit 75 -> runner maps to NOT-VERIFIED (never skip-as-pass, OD-2)" >&2
  exit 75
fi
if [ -z "${REPOSIX_ALLOWED_ORIGINS:-}" ]; then
  echo "NOT-VERIFIED: REPOSIX_ALLOWED_ORIGINS unset (egress allowlist required)" >&2
  echo "  env-gate: exit 75 -> runner maps to NOT-VERIFIED" >&2
  exit 75
fi

# --- Sanctioned-target assertion (OD-2 hard-FAIL, NOT 75) ------------------
# Only TokenWorld (the "go crazy, it's safe" mutation space) is sanctioned
# for this row -- it creates + mutates pages (a bad-parent Create + a
# content-equivalent retry), unlike the read-only attach-sync-real-backend
# smoke which also permits the durable-fixture REPOSIX space.
SPACE="${REPOSIX_CONFLUENCE_SPACE:-TokenWorld}"
case "$SPACE" in
  TokenWorld) ;;
  *)
    echo "FAIL: non-sanctioned Confluence space '$SPACE' -- only TokenWorld is sanctioned for this mutating row (docs/reference/testing-targets.md)" >&2
    exit 1
    ;;
esac

# --- Build the binaries the smoke shells out to (one cargo invocation) ----
cargo build -p reposix-cli --bin reposix >&2

# --- Drive the real partial_failure_recovery_real_confluence smoke --------
set +e
write_transcript_and_artifact "$SLUG" \
  cargo test -p reposix-cli --test agent_flow_real \
  partial_failure_recovery_real_confluence -- --ignored --exact
rc=$?
set -e

if [ "$rc" -ne 0 ]; then
  echo "FAIL: partial_failure_recovery_real_confluence smoke failed (creds present but backend unreachable, PRECHECK B replan regressed, or SotPartialFail recovery broke) -- inspect the transcript" >&2
  exit 1
fi

echo "PASS: real SoT-success + mirror-fail partial-failure recovery round-trip against Confluence TokenWorld (next push read new SoT via PRECHECK B and replanned; transcript emitted)"
