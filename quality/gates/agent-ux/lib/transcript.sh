#!/usr/bin/env bash
# quality/gates/agent-ux/lib/transcript.sh — RBF-FW-02 shared helper
# source this file from a kind: shell-subprocess verifier; call:
#   write_transcript_and_artifact <row-slug> <argv...>
# Writes BOTH a transcript at the STABLE path
# quality/reports/transcripts/<row-slug>.txt (no RFC3339 stamp -- see the
# D-P96-01 rationale in the function body below) AND a JSON artifact at
# quality/reports/verifications/agent-ux/<row-slug>.json with a
# transcript_path field referencing the transcript.
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
  # STABLE transcript filename (D-P96-01, extended). The transcript is a
  # gitignored per-run snapshot (quality/reports/transcripts/*.txt) that is
  # OVERWRITTEN each run — it must NOT carry a per-run RFC3339 stamp, because
  # the committed verdict references it and a volatile filename would re-dirty
  # the tracked JSON on every grade run (the exact stop-on-dirty hazard this
  # helper caused). The per-run timestamp lives INSIDE the transcript body.
  local ts transcript artifact rel_transcript
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  transcript="${repo_root}/quality/reports/transcripts/${slug}.txt"
  artifact="${repo_root}/quality/reports/verifications/agent-ux/${slug}.json"
  rel_transcript="quality/reports/transcripts/${slug}.txt"
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
    printf 'ts: %s\n' "$ts"
    printf 'env_keys: %s\n' "$env_keys"
    printf 'cwd: %s\n' "$(pwd)"
    printf 'exit_code: %s\n' "$exit_code"
    printf -- '--- STDOUT ---\n'
    cat "$stdout_file"
    printf -- '--- STDERR ---\n'
    cat "$stderr_file"
  } > "$transcript"

  # HONEST asserts (was hardcoded []): parse the scenario's own
  # "ASSERT <label>: PASS|FAIL" report lines into TAB-separated PASS/FAIL
  # records so the committed verdict records EXACTLY the conditions the
  # fleet-safety scenario evaluated. Deterministic — a given outcome emits a
  # fixed label set; `awk '!seen'` dedupes loop-repeated labels while
  # preserving first-seen order. The greedy `(.*)` binds the label up to the
  # FINAL ": PASS"/": FAIL", so labels that themselves contain ": " survive.
  # BUGFIX (D2, v0.14.0 tag remediation): this helper is SOURCED into a
  # caller running `set -euo pipefail`. `grep` exits 1 on "zero matches" --
  # which is the NORMAL, expected outcome for any `kind: shell-subprocess`
  # scenario whose underlying command is a plain `cargo test` invocation
  # (the "ASSERT <label>: PASS|FAIL" convention only applies to bash
  # scenario scripts; `cargo test` output never emits it). Under inherited
  # `pipefail`, that zero-match grep silently aborted this WHOLE function
  # via `errexit` -- masking a genuine exit-0 PASS as a bogus non-zero
  # script failure and skipping the canonical verdict artifact write below
  # (see `_shell_verdict.py`'s own docstring: "an empty scenario report is
  # a clean ... verdict, not a crash" -- the intended behavior this grep
  # violated). `|| true` on the whole pipeline restores that intent: zero
  # ASSERT lines -> assert_records is empty -> a clean asserts_passed=[]
  # verdict, never a masked script abort.
  local assert_records
  assert_records="$(
    grep -E '^[[:space:]]*ASSERT .*: (PASS|FAIL)$' "$stdout_file" \
      | sed -E 's/^[[:space:]]*ASSERT (.*): (PASS|FAIL)$/\2\t\1/' \
      | awk '!seen[$0]++'
  )" || true

  # DETERMINISTIC committed verdict via the shared canonical serializer
  # (quality/runners/_shell_verdict.py) — SAME schema + byte formatting as
  # run.py's write-back, so the two producers never fight. No `ts`, stable
  # transcript_path: a re-run rewrites the tracked JSON ONLY if the graded
  # result (exit_code / asserts) changed.
  printf '%s\n' "$assert_records" | python3 "${repo_root}/quality/runners/_shell_verdict.py" \
    "$artifact" "agent-ux/${slug}" "$exit_code" "$rel_transcript"

  rm -f "$stdout_file" "$stderr_file"
  return "$exit_code"
}
