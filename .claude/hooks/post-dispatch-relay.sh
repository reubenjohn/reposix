#!/usr/bin/env bash
# .claude/hooks/post-dispatch-relay.sh — PostToolUse/Task|Agent: relay/re-run guard.
# no canonical home under quality/gates/ — wired via .claude/settings.json
# Throttled per-agent via throttle.sh (same rationale as dispatch-doctrine.sh):
# first dispatch-return always gets the guard, repeats only if >5min since the last.
set -eu
input=$(cat)
tp=$(printf '%s' "$input" | python3 -c 'import sys,json;print(json.load(sys.stdin).get("transcript_path",""))' 2>/dev/null || true)
if ! bash "$(dirname "$0")/throttle.sh" post-dispatch-relay "$tp" 300; then
  exit 0
fi
printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"Before re-running this lane: check git log --oneline -5 and git status. Subagent replies can mis-route (cross-session addressing); the work may already be committed. Relay a mis-routed report inline rather than re-dispatching."}}'
exit 0
