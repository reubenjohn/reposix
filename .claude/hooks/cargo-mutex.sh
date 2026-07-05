#!/usr/bin/env bash
# .claude/hooks/cargo-mutex.sh — PreToolUse/Bash: one cargo invocation machine-wide.
# no canonical home under quality/gates/ — wired via .claude/settings.json
set -eu
payload=$(cat)
cmd=$(printf '%s' "$payload" | python3 -c 'import sys,json;print(json.load(sys.stdin).get("tool_input",{}).get("command",""))' 2>/dev/null || true)
case "$cmd" in
  *cargo\ *|*"cargo"|*cross\ *|*rustc\ *) : ;;   # gate build tools only
  *) exit 0 ;;
esac
# Robust liveness: is a cargo/rustc BUILD already running? (crash-safe vs a stale PID lock)
# Machine-wide check (pgrep, not repo-scoped) — the OOM risk spans any repo on this VM.
if pgrep -f 'cargo (check|build|test|clippy|nextest|run)|rustc ' >/dev/null 2>&1; then
  echo "BLOCKED: a cargo/rustc build is already running machine-wide. reposix Build memory budget (crates/CLAUDE.md): exactly ONE cargo invocation at a time — the VM OOM-crashed 3x from parallel workspace builds. Wait for the running build to finish, then retry." >&2
  exit 2
fi
printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","additionalContext":"cargo OK (no concurrent build). Reminder: prefer -p <crate> over --workspace; CARGO_BUILD_JOBS=2; cargo nextest for full-workspace tests."}}'
exit 0
