#!/usr/bin/env bash
# quality/gates/agent-ux/fleet-safety-leaf-isolation-enforce.sh
# Catalog row: agent-ux/fleet-safety-leaf-isolation-enforce (kind: shell-subprocess).
#
# Drives .claude/hooks/leaf-isolation-guard.sh (guard B) with crafted PreToolUse payloads:
#   1. `reposix init sim::demo .` cwd=SHARED, no cd /tmp   -> exit 2 (BLOCK), never runs
#   2. same command prefixed `cd /tmp/<clone> && …`        -> exit 0 (ALLOW; /tmp sanctioned)
#   3. fail-closed: undeterminable effective cwd            -> exit 2 (default-deny)
#   4. stderr teaches the /tmp-clone rule + cites ORCHESTRATION.md § Leaf isolation
#   5. the hook mechanism invokes `git worktree remove --force` NOWHERE (comment-filtered)
# Emits a transcript via the shared helper. Exits 0 iff ALL asserts pass. Drives the hook
# only (no shared-repo mutation; the setup verb never executes because the hook blocks it).
set -uo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"
HOOK="${repo_root}/.claude/hooks/leaf-isolation-guard.sh"
# shellcheck source=quality/gates/agent-ux/lib/transcript.sh
. "${repo_root}/quality/gates/agent-ux/lib/transcript.sh"

# drive_hook <command> <cwd|__OMIT__> : run the hook; sets HOOK_RC + HOOK_ERR.
# __OMIT__ builds a payload with NO cwd key at all (undeterminable-cwd fail-closed test).
drive_hook() {
  local cmd="$1" cwd="$2" payload
  if [ "$cwd" = "__OMIT__" ]; then
    payload=$(python3 -c 'import json,sys; print(json.dumps({"tool_input":{"command":sys.argv[1]}}))' "$cmd")
  else
    payload=$(python3 -c 'import json,sys; print(json.dumps({"tool_input":{"command":sys.argv[1]},"cwd":sys.argv[2]}))' "$cmd" "$cwd")
  fi
  # Run from a NON-/tmp cwd so the $PWD fail-closed fallback resolves to a shared-like
  # location (proving default-deny), matching the live runtime where $PWD is the repo root.
  HOOK_ERR="$(cd "$repo_root" && printf '%s' "$payload" | bash "$HOOK" 2>&1 >/dev/null)"; HOOK_RC=$?
  return 0
}

scenario() {
  local fails=0 shared="$repo_root"

  # --- Case 1: leaf-setup verb vs SHARED tree, no cd /tmp -> BLOCK (exit 2) -----
  drive_hook 'reposix init sim::demo .' "$shared"
  printf 'CASE 1 (reposix init vs shared, no cd): argv=[reposix init sim::demo .] cwd=[%s] hook_exit=%s\n' "$shared" "$HOOK_RC"
  printf '  teaching_stderr: %s\n' "$HOOK_ERR"
  if [ "$HOOK_RC" = 2 ]; then echo "  ASSERT exit==2 BLOCK: PASS"; else echo "  ASSERT exit==2 BLOCK: FAIL"; fails=$((fails+1)); fi
  if printf '%s' "$HOOK_ERR" | grep -q 'ORCHESTRATION.md' && printf '%s' "$HOOK_ERR" | grep -qi '/tmp' && printf '%s' "$HOOK_ERR" | grep -qi 'Leaf isolation'; then
    echo "  ASSERT stderr teaches /tmp-clone rule + cites ORCHESTRATION.md: PASS"; else echo "  ASSERT stderr teaches rule + cites ORCHESTRATION.md: FAIL"; fails=$((fails+1)); fi

  # --- Case 2: same verb prefixed `cd /tmp/<clone> && …` -> ALLOW (exit 0) ------
  drive_hook 'cd /tmp/leaf-clone-xyz && reposix init sim::demo .' "$shared"
  printf 'CASE 2 (reposix init WITH cd /tmp redirect): argv=[cd /tmp/leaf-clone-xyz && reposix init sim::demo .] cwd=[%s] hook_exit=%s\n' "$shared" "$HOOK_RC"
  if [ "$HOOK_RC" = 0 ]; then echo "  ASSERT exit==0 ALLOW (redirect sanctioned): PASS"; else echo "  ASSERT exit==0 ALLOW: FAIL"; fails=$((fails+1)); fi

  # --- Case 3: fail-closed — undeterminable effective cwd -> BLOCK (exit 2) -----
  drive_hook 'reposix init sim::demo .' "__OMIT__"
  printf 'CASE 3 (reposix init, cwd OMITTED from payload -> fail-closed): argv=[reposix init sim::demo .] cwd=[<omitted>] hook_exit=%s\n' "$HOOK_RC"
  if [ "$HOOK_RC" = 2 ]; then echo "  ASSERT exit==2 default-deny on undeterminable cwd: PASS"; else echo "  ASSERT exit==2 fail-closed: FAIL"; fails=$((fails+1)); fi

  # --- Case 4: guard must not be built on `git worktree remove --force` ---------
  local wt_count
  wt_count=$(grep -v '^[[:space:]]*#' "$HOOK" | grep -c 'worktree remove --force' || true)
  printf 'CASE 4 (no corruption-vector): grep `git worktree remove --force` (comment-filtered) count=%s\n' "$wt_count"
  if [ "$wt_count" = 0 ]; then echo "  ASSERT zero worktree-remove-force occurrences: PASS"; else echo "  ASSERT zero worktree-remove-force: FAIL"; fails=$((fails+1)); fi

  echo "----"
  if [ "$fails" = 0 ]; then echo "ALL ASSERTS PASSED"; return 0; else echo "ASSERTS FAILED: $fails"; return 1; fi
}

write_transcript_and_artifact "fleet-safety-leaf-isolation-enforce" scenario
exit $?
