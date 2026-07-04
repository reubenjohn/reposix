#!/usr/bin/env bash
# quality/gates/agent-ux/shell-subprocess-example.sh — RBF-FW-02 worked example
# Exercises the kind: shell-subprocess convention end-to-end against a real
# binary subprocess (no real-backend env required — works in CI on the
# simulator path).
#
# ─────────────────────────────────────────────────────────────────────────
# The bash --version FALLBACK is INTENTIONAL
# CI-portability behavior, NOT an oversight. The catalog row
# agent-ux/kind-shell-subprocess-worked-example (minted in 89-01) honestly
# names this in its `expected.asserts[0]`:
#
#   "exits 0 against a real binary subprocess (reposix preferred; bash --version
#    as CI fallback when cargo target absent)"
#
# The kind:shell-subprocess proof shape (real subprocess + transcript) is
# exercised regardless of which binary is invoked. The worked example is the
# KIND-DEMONSTRATION row, not a transport-claim row. Downstream consumers
# (P92 transport tests) MUST invoke a real reposix binary or a real-backend
# endpoint in their OWN catalog rows; do NOT remove the bash fallback here
# without also updating the catalog row's expected.asserts in 89-01.
# ─────────────────────────────────────────────────────────────────────────
#
# Asserts:
#   1. Subprocess invocation produces exit 0 (real binary, not Python inline)
#   2. Transcript file written at quality/reports/transcripts/<slug>-<ts>.txt
#   3. Transcript contains argv + env_keys + exit_code blocks
#   4. JSON artifact contains transcript_path field
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
# shell-subprocess-example.sh lives at quality/gates/agent-ux/, so repo
# root is ../../.. (THREE levels up). Compare with lib/transcript.sh
# which is one level deeper and uses ../../../.. .
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

# shellcheck disable=SC1091
source "${SCRIPT_DIR}/lib/transcript.sh"

SLUG="kind-shell-subprocess-worked-example"

# Resolve a real binary to invoke as a subprocess.
# Preference order: reposix on PATH > target/debug/reposix > bash (CI fallback).
if command -v reposix > /dev/null 2>&1; then
  REPOSIX="reposix"
  ARGS=("--version")
elif [[ -x "${REPO_ROOT}/target/debug/reposix" ]]; then
  REPOSIX="${REPO_ROOT}/target/debug/reposix"
  ARGS=("--version")
else
  # CI-portability fallback (per row's expected.asserts[0]):
  # The kind contract is "real subprocess + transcript". bash satisfies that
  # contract; reposix is preferred but not required for the worked-example.
  REPOSIX="bash"
  ARGS=("--version")
fi

# write_transcript_and_artifact returns the subprocess exit code. Under
# `set -e` a non-zero return would abort, so capture it explicitly.
exit_code=0
write_transcript_and_artifact "$SLUG" "$REPOSIX" "${ARGS[@]}" || exit_code=$?
if [[ "$exit_code" -ne 0 ]]; then
  echo "✖ subprocess exited non-zero: $REPOSIX ${ARGS[*]} (exit $exit_code)" >&2
  exit 1
fi

# Asserts on the produced artifacts
ARTIFACT="${REPO_ROOT}/quality/reports/verifications/agent-ux/${SLUG}.json"
[[ -f "$ARTIFACT" ]] || { echo "✖ artifact missing: $ARTIFACT" >&2; exit 1; }
TRANSCRIPT_REL=$(python3 -c "import json; print(json.load(open('$ARTIFACT'))['transcript_path'])")
TRANSCRIPT="${REPO_ROOT}/${TRANSCRIPT_REL}"
[[ -f "$TRANSCRIPT" ]] || { echo "✖ transcript missing: $TRANSCRIPT" >&2; exit 1; }
grep -q '^argv: ' "$TRANSCRIPT" || { echo "✖ transcript missing argv block" >&2; exit 1; }
grep -q '^env_keys: ' "$TRANSCRIPT" || { echo "✖ transcript missing env_keys block" >&2; exit 1; }
grep -q '^exit_code: ' "$TRANSCRIPT" || { echo "✖ transcript missing exit_code block" >&2; exit 1; }

# Security check: env_keys MUST be NAMES only, no `=value`
if grep -E '^env_keys:.*=' "$TRANSCRIPT" > /dev/null; then
  echo "✖ env_keys leaks values (security failure)" >&2
  exit 1
fi

echo "PASS: shell-subprocess kind worked example (4/4 asserts; subprocess: $REPOSIX ${ARGS[*]})"
