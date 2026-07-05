#!/usr/bin/env bash
# .claude/hooks/precompact-persist.sh — PreCompact: persist-state reminder (exit 0).
# no canonical home under quality/gates/ — wired via .claude/settings.json
# Best-effort advisory; ALWAYS exits 0.
set -eu
cat >/dev/null
msg="PreCompact: persist session state to COMMITTED artifacts NOW — /tmp and full context will not survive intact. If you are a coordinator, dispatch relief-handover-writer to write+commit a handover per .planning/ORCHESTRATION.md (ground-truth git -> wave state -> binding constraints -> numbered next steps -> close checklist). Uncommitted = didn't happen."
printf '%s' "$msg" | python3 -c 'import sys,json; m=sys.stdin.read(); print(json.dumps({"systemMessage":m,"hookSpecificOutput":{"hookEventName":"PreCompact","additionalContext":m}}))'
exit 0
