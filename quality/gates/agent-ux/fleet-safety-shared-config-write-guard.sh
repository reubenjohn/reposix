#!/usr/bin/env bash
# quality/gates/agent-ux/fleet-safety-shared-config-write-guard.sh
# Catalog row: agent-ux/fleet-safety-shared-config-write-guard (kind: shell-subprocess).
#
# Drives .claude/hooks/leaf-isolation-guard.sh (guard C) with crafted PreToolUse payloads:
#   1. `git config core.bare true` / `git config user.email t@t` cwd=SHARED -> exit 2 BLOCK
#   2. the REAL shared .git/config sha256 is byte-identical before/after the blocked
#      attempt — the write never executed (PreToolUse blocks pre-execution)
#   3. `git config --file /tmp/<clone>/.git/config core.bare true` -> exit 0 ALLOW
#   4. stderr names core.bare/user.email as forbidden shared-config writes + recovery
# Emits a transcript via the shared helper. Exits 0 iff ALL asserts pass. The verifier
# NEVER executes the blocked `git config` — it only drives the hook, so the shared config
# is untouched by construction; the sha256 check proves it.
set -uo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"
HOOK="${repo_root}/.claude/hooks/leaf-isolation-guard.sh"
SHARED_CONFIG="${repo_root}/.git/config"
# shellcheck source=quality/gates/agent-ux/lib/transcript.sh
. "${repo_root}/quality/gates/agent-ux/lib/transcript.sh"

drive_hook() {
  local cmd="$1" cwd="$2" payload
  payload=$(python3 -c 'import json,sys; print(json.dumps({"tool_input":{"command":sys.argv[1]},"cwd":sys.argv[2]}))' "$cmd" "$cwd")
  HOOK_ERR="$(cd "$repo_root" && printf '%s' "$payload" | bash "$HOOK" 2>&1 >/dev/null)"; HOOK_RC=$?
  return 0
}

scenario() {
  local fails=0 shared="$repo_root"

  # --- Case 2 prep: sha256 of the REAL shared .git/config BEFORE any attempt ----
  local sha_before sha_after
  sha_before="$(sha256sum "$SHARED_CONFIG" | cut -d' ' -f1)"
  printf 'BASELINE: sha256(%s) BEFORE = %s\n' "$SHARED_CONFIG" "$sha_before"

  # --- Case 1a: `git config core.bare true` vs SHARED -> BLOCK (exit 2) ---------
  drive_hook 'git config core.bare true' "$shared"
  printf 'CASE 1a (git config core.bare true vs shared): argv=[git config core.bare true] cwd=[%s] hook_exit=%s\n' "$shared" "$HOOK_RC"
  printf '  teaching_stderr: %s\n' "$HOOK_ERR"
  if [ "$HOOK_RC" = 2 ]; then echo "  ASSERT exit==2 BLOCK: PASS"; else echo "  ASSERT exit==2 BLOCK: FAIL"; fails=$((fails+1)); fi
  if printf '%s' "$HOOK_ERR" | grep -q 'core.bare' && printf '%s' "$HOOK_ERR" | grep -q 'RECOVERY'; then
    echo "  ASSERT stderr names core.bare + recovery: PASS"; else echo "  ASSERT stderr names core.bare + recovery: FAIL"; fails=$((fails+1)); fi

  # --- Case 1b: `git config user.email t@t` vs SHARED -> BLOCK (exit 2) ---------
  drive_hook 'git config user.email t@t' "$shared"
  printf 'CASE 1b (git config user.email t@t vs shared): argv=[git config user.email t@t] cwd=[%s] hook_exit=%s\n' "$shared" "$HOOK_RC"
  if [ "$HOOK_RC" = 2 ]; then echo "  ASSERT exit==2 BLOCK: PASS"; else echo "  ASSERT exit==2 BLOCK: FAIL"; fails=$((fails+1)); fi

  # --- Case 2: shared .git/config byte-unchanged after the blocked attempts -----
  sha_after="$(sha256sum "$SHARED_CONFIG" | cut -d' ' -f1)"
  printf 'CASE 2 (config byte-unchanged): sha256 AFTER = %s\n' "$sha_after"
  if [ "$sha_before" = "$sha_after" ]; then echo "  ASSERT shared .git/config byte-identical before==after: PASS"; else echo "  ASSERT config byte-unchanged: FAIL"; fails=$((fails+1)); fi

  # --- Case 3: `git config --file /tmp/<clone>/.git/config …` -> ALLOW (exit 0) -
  drive_hook 'git config --file /tmp/leaf-clone-xyz/.git/config core.bare true' "$shared"
  printf 'CASE 3 (git config --file /tmp/... allowed): argv=[git config --file /tmp/leaf-clone-xyz/.git/config core.bare true] cwd=[%s] hook_exit=%s\n' "$shared" "$HOOK_RC"
  if [ "$HOOK_RC" = 0 ]; then echo "  ASSERT exit==0 ALLOW (--file /tmp redirect): PASS"; else echo "  ASSERT exit==0 ALLOW: FAIL"; fails=$((fails+1)); fi

  # --- Case 4 (P102 hardening): cd-back evasion must BLOCK ----------------------
  # `cd /tmp/x && cd <shared> && git config core.bare true` lands back in the shared tree.
  drive_hook "cd /tmp/x && cd $shared && git config core.bare true" "$shared"
  printf 'CASE 4 (cd /tmp then cd back, config write): hook_exit=%s\n' "$HOOK_RC"
  if [ "$HOOK_RC" = 2 ]; then echo "  ASSERT cd-back config write exit==2 BLOCK: PASS"; else echo "  ASSERT cd-back BLOCK: FAIL"; fails=$((fails+1)); fi

  # --- Case 5 (P102 hardening): `-f /tmp` short flag must ALLOW (was false-pos) --
  # git config `-f` is the short form of `--file`; a /tmp target must NOT over-block.
  drive_hook 'git config -f /tmp/leaf-clone-xyz/.git/config core.bare true' "$shared"
  printf 'CASE 5 (git config -f /tmp/... allowed): hook_exit=%s\n' "$HOOK_RC"
  if [ "$HOOK_RC" = 0 ]; then echo "  ASSERT exit==0 ALLOW (-f /tmp redirect): PASS"; else echo "  ASSERT -f /tmp ALLOW: FAIL"; fails=$((fails+1)); fi

  # --- Case 6 (P102 hardening): a READ (--get) must ALLOW -----------------------
  drive_hook 'git config --get core.bare' "$shared"
  printf 'CASE 6 (git config --get core.bare read): hook_exit=%s\n' "$HOOK_RC"
  if [ "$HOOK_RC" = 0 ]; then echo "  ASSERT exit==0 ALLOW (read not write): PASS"; else echo "  ASSERT read ALLOW: FAIL"; fails=$((fails+1)); fi

  echo "----"
  if [ "$fails" = 0 ]; then echo "ALL ASSERTS PASSED"; return 0; else echo "ASSERTS FAILED: $fails"; return 1; fi
}

write_transcript_and_artifact "fleet-safety-shared-config-write-guard" scenario
exit $?
