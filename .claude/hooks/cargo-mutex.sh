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
# Liveness check: is a cargo/rustc BUILD already running machine-wide? Match on
# the process EXECUTABLE NAME (comm, via `pgrep -x`), NEVER full argv -- a prior
# `pgrep -f` false-matched any shell/editor/grep whose command TEXT merely
# mentioned "cargo"/"rustc", deadlocking a live session for ~180k tokens. Full
# incident + why `-x` is exhaustive against the OOM contract without weakening
# it: .claude/hooks/CLAUDE.md § "cargo-mutex.sh: pgrep -x vs -f",
# SURPRISES-INTAKE.md. Excludes the hook's own PID/PPID belt-and-suspenders
# (comm can't spoof regardless).
exclude="^($$|${PPID:-0})$"
if pgrep -x 'cargo|cargo-nextest|rustc|cross|clippy-driver' 2>/dev/null \
     | grep -Ev "$exclude" | grep -q .; then
  echo "BLOCKED: a cargo/rustc build is already running machine-wide. reposix Build memory budget (crates/CLAUDE.md): exactly ONE cargo invocation at a time — the VM OOM-crashed 3x from parallel workspace builds. Wait for the running build to finish, then retry." >&2
  exit 2
fi
printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","additionalContext":"cargo OK (no concurrent build). Reminder: prefer -p <crate> over --workspace; CARGO_BUILD_JOBS=2; cargo nextest for full-workspace tests."}}'
exit 0
