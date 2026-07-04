#!/usr/bin/env bash
# quality/gates/agent-ux/lib/transcript.sh — RBF-FW-02 shared helper
# source this file from a kind: shell-subprocess verifier; call:
#   write_transcript_and_artifact <row-slug> <argv...>
# Writes BOTH a transcript at quality/reports/transcripts/<row-slug>-<RFC3339>.txt
# AND a JSON artifact at quality/reports/verifications/agent-ux/<row-slug>.json
# with a transcript_path field referencing the transcript.
#
# Mirrors the quality/gates/agent-ux/dark-factory/lib.sh factoring precedent:
# a sourced-only helper (not directly invokable) that centralizes the
# artifact-write boilerplate the per-verifier scripts would otherwise inline.

write_transcript_and_artifact() {
  local slug="$1"; shift
  local repo_root
  # NOTE: this helper lives at quality/gates/agent-ux/lib/transcript.sh,
  # so repo root is FOUR levels up from the helper file (../../../..).
  repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../../.." &> /dev/null && pwd)"
  local ts ts_file
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  ts_file="$(date -u +%Y-%m-%dT%H-%M-%SZ)"
  local transcript="${repo_root}/quality/reports/transcripts/${slug}-${ts_file}.txt"
  local artifact="${repo_root}/quality/reports/verifications/agent-ux/${slug}.json"
  mkdir -p "$(dirname "$transcript")" "$(dirname "$artifact")"

  local stdout_file stderr_file env_keys exit_code
  # SECURITY (CLAUDE.md threat-model, exfiltration leg): emit variable NAMES
  # only, NEVER values. The `cut -d= -f1` is load-bearing — do not "improve"
  # this to include values.
  env_keys=$(env | cut -d= -f1 | sort | tr '\n' ',' | sed 's/,$//')
  stdout_file=$(mktemp); stderr_file=$(mktemp)
  set +e
  "$@" >"$stdout_file" 2>"$stderr_file"
  exit_code=$?
  set -e

  {
    printf 'argv: %s\n' "$*"
    printf 'env_keys: %s\n' "$env_keys"
    printf 'cwd: %s\n' "$(pwd)"
    printf 'exit_code: %s\n' "$exit_code"
    printf -- '--- STDOUT ---\n'
    cat "$stdout_file"
    printf -- '--- STDERR ---\n'
    cat "$stderr_file"
  } > "$transcript"

  # transcript_path is the load-bearing field — verifier subagent dereferences it.
  # Path is REPO-RELATIVE so it survives runner cwd changes.
  local rel_transcript="${transcript#"${repo_root}"/}"
  cat > "$artifact" <<EOF
{"ts":"${ts}","row_id":"agent-ux/${slug}","exit_code":${exit_code},"transcript_path":"${rel_transcript}","asserts_passed":[],"asserts_failed":[]}
EOF
  rm -f "$stdout_file" "$stderr_file"
  return "$exit_code"
}
