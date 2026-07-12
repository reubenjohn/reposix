#!/usr/bin/env bash
# quality/gates/agent-ux/fleet-safety-tat-identity-reject.sh
# Catalog row: agent-ux/fleet-safety-tat-identity-reject (kind: shell-subprocess).
#
# Drives .claude/hooks/leaf-isolation-guard.sh (guard A) + .githooks/pre-commit (backstop)
# as REAL subprocesses with crafted PreToolUse JSON payloads and asserts:
#   1. fixture-identity commit vs the SHARED tree  -> hook exit 2 (BLOCK) + teaching stderr
#   2. .githooks/pre-commit refuses a `t <t@t>` commit in a throwaway /tmp clone (non-zero)
#   3. a real-identity commit is NOT blocked (hook exit 0; /tmp-clone commit succeeds)
# Emits a transcript via the shared kind:shell-subprocess helper. Exits 0 iff ALL pass.
#
# LEAF-ISOLATION: every real git write below runs inside a /tmp clone cd-ed in the SAME
# process — the verifier never mutates the shared repo. The hook BLOCKS pre-execution, so
# the crafted `git commit` payloads never actually run.
set -uo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"
HOOK="${repo_root}/.claude/hooks/leaf-isolation-guard.sh"
PRECOMMIT="${repo_root}/.githooks/pre-commit"
# shellcheck source=quality/gates/agent-ux/lib/transcript.sh
. "${repo_root}/quality/gates/agent-ux/lib/transcript.sh"

# drive_hook <command> <cwd> : run the hook with a crafted payload; sets HOOK_RC + HOOK_ERR.
drive_hook() {
  local cmd="$1" cwd="$2" payload
  payload=$(python3 -c 'import json,sys; print(json.dumps({"tool_input":{"command":sys.argv[1]},"cwd":sys.argv[2]}))' "$cmd" "$cwd")
  HOOK_ERR="$(printf '%s' "$payload" | bash "$HOOK" 2>&1 >/dev/null)"; HOOK_RC=$?
  return 0
}

scenario() {
  local fails=0 shared="$repo_root"

  # --- Case 1: fixture-identity commit vs SHARED tree -> BLOCK (exit 2) --------
  drive_hook 'git -c user.email=t@t commit -m x' "$shared"
  printf 'CASE 1 (fixture commit vs shared): argv=[git -c user.email=t@t commit -m x] cwd=[%s] hook_exit=%s\n' "$shared" "$HOOK_RC"
  printf '  teaching_stderr: %s\n' "$HOOK_ERR"
  if [ "$HOOK_RC" = 2 ]; then echo "  ASSERT exit==2 BLOCK: PASS"; else echo "  ASSERT exit==2 BLOCK: FAIL"; fails=$((fails+1)); fi
  if printf '%s' "$HOOK_ERR" | grep -qi 'fixture' && printf '%s' "$HOOK_ERR" | grep -q 't@t' && printf '%s' "$HOOK_ERR" | grep -q 'RECOVERY'; then
    echo "  ASSERT stderr teaches rule+why+recovery: PASS"; else echo "  ASSERT stderr teaches rule+why+recovery: FAIL"; fails=$((fails+1)); fi

  # --- Case 2: .githooks/pre-commit backstop in a throwaway /tmp clone ----------
  local T="/tmp/fleet-tat-verify-$$-$RANDOM"
  rm -rf "$T"; mkdir -p "$T/.githooks" "$T/quality/runners"
  cp "$PRECOMMIT" "$T/.githooks/pre-commit"; chmod +x "$T/.githooks/pre-commit"
  printf '#!/usr/bin/env python3\nimport sys\nsys.exit(0)\n' > "$T/quality/runners/run.py"
  (
    cd "$T" || exit 3
    export HOME="$T/home"; mkdir -p "$HOME"
    git init -q; git config core.hooksPath .githooks
    echo seed > f.txt; git add f.txt
    git -c user.name=t -c user.email=t@t commit -q -m fixture 2>fx.err; echo "FIX_RC=$?"
    git -c user.name=Dev -c user.email=dev@example.com commit -q -m real 2>rl.err; echo "REAL_RC=$?"
  ) > "$T/out.txt" 2>&1
  local fix_rc real_rc
  fix_rc=$(grep -o 'FIX_RC=[0-9]*' "$T/out.txt" | cut -d= -f2)
  real_rc=$(grep -o 'REAL_RC=[0-9]*' "$T/out.txt" | cut -d= -f2)
  printf 'CASE 2 (pre-commit backstop in /tmp clone %s): fixture_commit_rc=%s real_commit_rc=%s\n' "$T" "$fix_rc" "$real_rc"
  printf '  pre-commit_reject_stderr: %s\n' "$(grep -m1 BLOCKED "$T/fx.err" 2>/dev/null || true)"
  if [ "${fix_rc:-0}" != 0 ]; then echo "  ASSERT fixture commit REJECTED (non-zero): PASS"; else echo "  ASSERT fixture commit REJECTED: FAIL"; fails=$((fails+1)); fi
  if [ "${real_rc:-1}" = 0 ]; then echo "  ASSERT real-identity commit ALLOWED (0): PASS"; else echo "  ASSERT real-identity commit ALLOWED: FAIL"; fails=$((fails+1)); fi
  rm -rf "$T"

  # --- Case 3: control — real-identity commit payload NOT blocked (exit 0) ------
  drive_hook 'git -c user.email=dev@example.com commit -m x' "$shared"
  printf 'CASE 3 (real-identity commit vs shared, no false-positive): argv=[git -c user.email=dev@example.com commit -m x] cwd=[%s] hook_exit=%s\n' "$shared" "$HOOK_RC"
  if [ "$HOOK_RC" = 0 ]; then echo "  ASSERT exit==0 ALLOW: PASS"; else echo "  ASSERT exit==0 ALLOW: FAIL"; fails=$((fails+1)); fi

  echo "----"
  if [ "$fails" = 0 ]; then echo "ALL ASSERTS PASSED"; return 0; else echo "ASSERTS FAILED: $fails"; return 1; fi
}

write_transcript_and_artifact "fleet-safety-tat-identity-reject" scenario
exit $?
