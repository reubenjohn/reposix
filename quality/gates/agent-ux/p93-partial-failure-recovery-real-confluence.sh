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
# (NOT-VERIFIED, fail-closed, NEVER skip-as-pass). The target space is PINNED
# to the sanctioned TokenWorld (see the pin block below) -- the smoke can only
# ever mutate the sanctioned space. Creds set + smoke fails -> hard FAIL exit 1.
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

# --- Pin the sanctioned mutation target (fix-twice) ------------------------
# This row CREATES + MUTATES Confluence pages (a bad-parent Create + a
# content-equivalent retry), so TokenWorld -- the owner-sanctioned "go crazy,
# it's safe" scratch space -- is the ONLY sanctioned target for it (unlike the
# read-only attach-sync-real-backend smoke, which also permits the durable
# REPOSIX fixture space). The ambient `.env` points REPOSIX_CONFLUENCE_SPACE at
# the READ-ONLY REPOSIX fixture; the earlier version READ that var and then
# hard-FAILed because REPOSIX != TokenWorld -- self-rejecting under normal
# creds. PINNING to TokenWorld is strictly SAFER than rejecting: the smoke can
# then ONLY ever mutate the sanctioned space, whatever `.env` points at. The
# underlying `partial_failure_recovery_real_confluence` smoke resolves its
# space from this exact env var (crates/reposix-cli/tests/agent_flow_real.rs
# confluence_test_space(); default TokenWorld), so exporting it here binds the
# real backend target. (docs/reference/testing-targets.md)
export REPOSIX_CONFLUENCE_SPACE=TokenWorld

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
