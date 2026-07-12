#!/usr/bin/env bash
# P111 ci-wait helper presence + shape verifier (TINY, static grep).
#
# Milestone-close hygiene item 1: promote the ad-hoc background
# `gh run watch` (which HANGS on already-concluded GREEN runs — evidence
# hang IDs `bulqmsyrv`, `biy9yxt33`) into a committed, bounded-poll
# helper `scripts/ci-wait.sh` (CLAUDE.md OP-4: promote ad-hoc bash).
#
# Asserts (all must hold):
#  1. scripts/ci-wait.sh exists.
#  2. scripts/ci-wait.sh is executable (chmod +x).
#  3. It drives `gh run` (view or list) — the CI query surface.
#  4. It has a hard TIMEOUT *and* a poll INTERVAL knob
#     (CI_WAIT_TIMEOUT + CI_WAIT_INTERVAL).
#  5. It exits non-zero on failure AND uses a distinct non-zero code
#     for the hard-timeout path (exit 1 + exit 2 both present).
#  6. It handles an already-`completed`/concluded run without looping
#     (the exact bug that hung `gh run watch`) — the concluded fast-path
#     references the `completed` status.
#
# Owner-hint on RED: author/finish scripts/ci-wait.sh per the P111
# spec — bounded poll of `gh run view <id> --json status,conclusion`,
# immediate return when the first query shows status `completed`,
# exit 0 only on conclusion `success`, exit 2 on hard-timeout.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

HELPER="scripts/ci-wait.sh"

if [[ ! -f "${HELPER}" ]]; then
  echo "FAIL: ${HELPER} not found — P111 hygiene item 1 (ci-wait helper) not landed" >&2
  exit 1
fi

if [[ ! -x "${HELPER}" ]]; then
  echo "FAIL: ${HELPER} exists but is not executable — run: chmod +x ${HELPER}" >&2
  exit 1
fi

if ! grep -qE 'gh[[:space:]]+run[[:space:]]+(view|list)' "${HELPER}"; then
  echo "FAIL: ${HELPER} does not drive 'gh run view' or 'gh run list' — no CI query surface" >&2
  exit 1
fi

if ! grep -qF 'CI_WAIT_TIMEOUT' "${HELPER}"; then
  echo "FAIL: ${HELPER} has no hard TIMEOUT knob (expected env CI_WAIT_TIMEOUT)" >&2
  exit 1
fi

if ! grep -qF 'CI_WAIT_INTERVAL' "${HELPER}"; then
  echo "FAIL: ${HELPER} has no poll INTERVAL knob (expected env CI_WAIT_INTERVAL)" >&2
  exit 1
fi

if ! grep -qE 'exit[[:space:]]+1' "${HELPER}"; then
  echo "FAIL: ${HELPER} never exits non-zero on failure (expected 'exit 1')" >&2
  exit 1
fi

if ! grep -qE 'exit[[:space:]]+2' "${HELPER}"; then
  echo "FAIL: ${HELPER} lacks a distinct hard-timeout exit code (expected 'exit 2')" >&2
  exit 1
fi

# The concluded fast-path must key off the `completed` status BEFORE the
# poll loop — this is the exact bug (`gh run watch` hangs on a concluded
# run) the helper exists to kill.
if ! grep -qF 'completed' "${HELPER}"; then
  echo "FAIL: ${HELPER} has no already-'completed' fast-path — would risk the gh-run-watch hang it replaces" >&2
  exit 1
fi

echo "PASS: ${HELPER} present, executable, bounded-poll (gh run + timeout/interval + concluded fast-path)"
exit 0
