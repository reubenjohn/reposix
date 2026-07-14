#!/usr/bin/env bash
# quality/gates/agent-ux/p94-git243-fallback-sentinel.sh
# Verifier for catalog row agent-ux/p94-git243-fallback-sentinel (P94 D2).
#
# Closes the git-2.43 single-backend-push HIGH carry-forward
# (SURPRISES-INTAKE.md L602-610). Three arms:
#
#   Arm 1 (source, always runs) — assert the two production reply changes
#     live in the tree:
#       * crates/reposix-remote/src/stateless_connect.rs replies the literal
#         `fallback` sentinel for a non-upload-pack `stateless-connect`
#         service (git-remote-helpers(7) spec-compliance; catalog assert a)
#         and NO LONGER emits the bug-preserving `unsupported service:` line;
#       * crates/reposix-remote/src/main.rs answers `option object-format`
#         with `ok` (the REAL, DP-2-traced git-2.43 push blocker — see the
#         REPRO-NOTES; the intake's fallback-only diagnosis was incomplete).
#
#   Arm 2 (cargo, always runs) — the flipped e2e assertion + the option
#     regression tests pass (catalog assert b, plus the object-format guard).
#
#   Arm 3 (container, docker-gated) — a stock ubuntu:24.04 (git 2.43.0)
#     drives a REAL single-backend `git push` against a sim backend and exits
#     0 (catalog assert c). Delegates to the committed repro driver
#     .planning/milestones/v0.13.0-phases/94-real-backend-frictions/94-D2-git243-repro.sh.
#     When docker is ABSENT the row is env-gated to NOT-VERIFIED (exit 75,
#     fail-closed, never skip-as-pass — OD-2 / catalog assert d). Arms 1+2
#     still gate the source+cargo claims in that case.
#
# Exit-code convention: quality/PROTOCOL.md "Verifier exit-code conventions"
# — the runner maps exit 75 -> NOT-VERIFIED. clean 0 -> PASS, 1 -> FAIL.
#
# Usage: p94-git243-fallback-sentinel.sh [--row-id <id>]
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

ROW_ID="agent-ux/p94-git243-fallback-sentinel"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  ROW_ID="$2"
fi

ARTIFACT="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux/p94-git243-fallback-sentinel.json"
mkdir -p "$(dirname "$ARTIFACT")"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

SRC="${WORKSPACE_ROOT}/crates/reposix-remote/src/stateless_connect.rs"
MAIN="${WORKSPACE_ROOT}/crates/reposix-remote/src/main.rs"

PASSED=()
fail() {
  local desc="$1" detail="${2:-}"
  echo "FAIL: ${desc}${detail:+: ${detail}}" >&2
  local pj; pj="$(printf '%s\n' "${PASSED[@]:-}" | python3 -c 'import json,sys; print(json.dumps([l for l in sys.stdin.read().splitlines() if l]))')"
  cat > "$ARTIFACT" <<EOF
{
  "ts": "$TS", "row_id": "$ROW_ID", "exit_code": 1, "status": "FAIL",
  "asserts_passed": ${pj},
  "asserts_failed": ["${desc}${detail:+ — ${detail}}"]
}
EOF
  exit 1
}
pass() { echo "  PASS: $1" >&2; PASSED+=("$1"); }

# ---- Arm 1: source asserts -------------------------------------------------
grep -Eq 'send_line\("fallback"\)' "$SRC" \
  || fail "stateless_connect.rs must reply the literal \`fallback\` sentinel" "send_line(\"fallback\") not found"
# The bug-preserving custom reply must be gone: no `send_line` may emit an
# `unsupported service` string (a comment mentioning the old behaviour is fine).
if grep -Eq 'send_line\([^)]*unsupported service' "$SRC"; then
  fail "stateless_connect.rs still emits the bug-preserving \`unsupported service:\` reply line"
fi
pass "stateless_connect.rs replies \`fallback\` (not \`unsupported service:\`) for non-upload-pack service"

grep -Eq 'object-format' "$MAIN" && grep -Eq 'send_line\("ok"\)' "$MAIN" \
  || fail "main.rs must answer \`option object-format\` with \`ok\` (the real git-2.43 push blocker)"
pass "main.rs answers \`option object-format\` with \`ok\` (sha1-only accept; non-sha1 rejected)"

# ---- Arm 2: cargo asserts --------------------------------------------------
# Run the two directly-affected integration files in full (12 tests): the
# flipped e2e assertion lives in stateless_connect_e2e, the option
# object-format regressions live in protocol. `cargo test` accepts only ONE
# name filter, so gate on the whole-file exit status (every test in both
# files must pass) and confirm the three P94-D2 names ran GREEN.
echo "p94-d2: cargo test (flipped e2e + option object-format regression)…" >&2
CARGO_LOG="$(mktemp)"
if ! ( cd "$WORKSPACE_ROOT" && CARGO_BUILD_JOBS=2 cargo test -p reposix-remote \
        --test stateless_connect_e2e --test protocol 2>&1 ) > "$CARGO_LOG"; then
  # Preserve the full log for post-mortem BEFORE printing a truncated tail —
  # a bare `tail -25` here previously discarded the actual panic/assertion
  # message (only the tail end of a long backtrace survived), leaving CI
  # failures undiagnosable after the fact (found investigating a p94
  # release-gate flake, 2026-07-06: SURPRISES-INTAKE.md).
  FAIL_LOG_DIR="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux"
  mkdir -p "$FAIL_LOG_DIR"
  FAIL_LOG_ARCHIVE="${FAIL_LOG_DIR}/p94-git243-fallback-sentinel-cargo-test-failure.log"
  cp "$CARGO_LOG" "$FAIL_LOG_ARCHIVE" 2>/dev/null || true
  echo "--- full cargo test log archived to ${FAIL_LOG_ARCHIVE} ---" >&2
  # Print any panic/assertion context (not just the trailing backtrace lines)
  # plus the final summary so the actual failure reason is visible in CI logs.
  grep -n -B2 -A15 'panicked at\|^failures:\|assertion.*failed' "$CARGO_LOG" >&2 || true
  echo "--- tail ---" >&2
  tail -60 "$CARGO_LOG" >&2
  rm -f "$CARGO_LOG"
  fail "cargo test arm failed (flipped e2e / option object-format regression)"
fi
for t in stateless_connect_replies_fallback_for_non_upload_pack_service \
         option_object_format_sha1_replies_ok \
         option_object_format_non_sha1_replies_error; do
  grep -Eq "test ${t} \.\.\. ok" "$CARGO_LOG" \
    || { tail -25 "$CARGO_LOG" >&2; rm -f "$CARGO_LOG"; fail "expected test did not run GREEN: ${t}"; }
done
rm -f "$CARGO_LOG"
pass "flipped e2e (\`fallback\`) + option object-format regression tests pass"

# ---- Arm 3: container arm (docker-gated) -----------------------------------
if ! command -v docker >/dev/null 2>&1; then
  echo "NOT-VERIFIED: docker absent — container git-2.43 push arm cannot run (exit 75, fail-closed per OD-2). Arms 1+2 passed." >&2
  PJ="$(printf '%s\n' "${PASSED[@]}" | python3 -c 'import json,sys; print(json.dumps([l for l in sys.stdin.read().splitlines() if l]))')"
  cat > "$ARTIFACT" <<EOF
{
  "ts": "$TS", "row_id": "$ROW_ID", "exit_code": 75, "status": "NOT-VERIFIED",
  "reason": "docker-absent", "skip_reason": "env-missing",
  "asserts_passed": ${PJ},
  "asserts_failed": ["container git-2.43 push arm requires docker — env-gated NOT-VERIFIED (never skip-as-pass)"]
}
EOF
  exit 75
fi

echo "p94-d2: building bins for the container arm…" >&2
( cd "$WORKSPACE_ROOT" && CARGO_BUILD_JOBS=2 cargo build -p reposix-cli -p reposix-sim -p reposix-remote --bins -q ) \
  || fail "cargo build (container-arm bins) failed"
BIN_DIR="${WORKSPACE_ROOT}/target/debug"

echo "p94-d2: running git-2.43 container push repro…" >&2
if BIN_DIR="$BIN_DIR" bash "${WORKSPACE_ROOT}/.planning/milestones/v0.13.0-phases/94-real-backend-frictions/94-D2-git243-repro.sh"; then
  pass "stock ubuntu:24.04 (git 2.43.0) real single-backend \`git push\` exits 0 (was 128 pre-fix)"
else
  fail "git-2.43 container push did NOT exit 0 — the version-windowed regression is not closed"
fi

PJ="$(printf '%s\n' "${PASSED[@]}" | python3 -c 'import json,sys; print(json.dumps([l for l in sys.stdin.read().splitlines() if l]))')"
cat > "$ARTIFACT" <<EOF
{
  "ts": "$TS", "row_id": "$ROW_ID", "exit_code": 0, "status": "PASS",
  "git_container": "ubuntu:24.04 (git 2.43.0)",
  "asserts_passed": ${PJ},
  "asserts_failed": []
}
EOF
echo "P94-D2 COMPLETE — fallback sentinel + option object-format fix; git-2.43 container push exits 0." >&2
exit 0
