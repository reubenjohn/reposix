#!/usr/bin/env bash
# .claude/hooks/post-dispatch-relay.sh — PostToolUse/Task|Agent: relay/re-run guard.
# no canonical home under quality/gates/ — wired via .claude/settings.json
set -eu
cat >/dev/null
printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"Before re-running this lane: check git log --oneline -5 and git status. Subagent replies can mis-route (cross-session addressing); the work may already be committed. Relay a mis-routed report inline rather than re-dispatching."}}'
exit 0
