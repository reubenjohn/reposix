#!/usr/bin/env bash
# .claude/hooks/throttle.sh — shared per-agent emission throttle for JIT-context hooks.
# no canonical home under quality/gates/ — called by sibling hooks, not wired directly.
#
# Problem: PreToolUse/PostToolUse hooks that emit a static additionalContext nudge
# re-inject the same tokens on EVERY matching tool call (e.g. dispatch-doctrine.sh on
# every Agent dispatch — ~175 tokens x N dispatches of pure duplication).
#
# Usage: throttle.sh <hook-name> <transcript_path> [cooldown_seconds=300]
#   exit 0 = EMIT      (first call for this agent, or last emission to this agent is
#                       older than the cooldown, or any error — fail-open preserves
#                       pre-throttle behavior)
#   exit 1 = SUPPRESS  (this hook already emitted to this agent within the cooldown)
#
# Keyed on the agent's own transcript JSONL path (hook stdin's transcript_path):
# every fresh agent/subagent has a fresh transcript, so it ALWAYS gets its first
# emission — dedupe never starves a cold context that actually needs the nudge.
# The state file's mtime IS the last-emission record; nothing is parsed or sized.
# State: ${TMPDIR:-/tmp}/claude-hook-throttle/<hook>.<sha256-16-of-path> (reboot-cleaned).
# Fail-open is deliberate throughout: any breakage (unwritable tmp, missing tools)
# degrades to pre-throttle behavior (emit every time), never to a lost nudge.
set -eu
hook=${1:?usage: throttle.sh <hook-name> <transcript_path> [cooldown_seconds]}
tp=${2:-}
cooldown=${3:-300}
[ -n "$tp" ] || exit 0
dir="${TMPDIR:-/tmp}/claude-hook-throttle"
mkdir -p "$dir" 2>/dev/null || exit 0
key=$(printf '%s' "$tp" | sha256sum | cut -c1-16) || exit 0
state="$dir/$hook.$key"
if [ -f "$state" ]; then
  now=$(date +%s) || exit 0
  last=$(stat -c %Y "$state" 2>/dev/null) || exit 0
  if [ $((now - last)) -lt "$cooldown" ]; then
    exit 1
  fi
fi
touch "$state" 2>/dev/null || true
exit 0
