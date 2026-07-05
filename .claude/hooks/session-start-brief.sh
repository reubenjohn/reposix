#!/usr/bin/env bash
# .claude/hooks/session-start-brief.sh — SessionStart: cursor + doctrine digest.
# no canonical home under quality/gates/ — wired via .claude/settings.json
set -eu
cd "${CLAUDE_PROJECT_DIR:-.}" 2>/dev/null || exit 0
cursor=$(sed -n '1,24p' .planning/STATE.md 2>/dev/null || printf '(STATE.md unreadable)')
log=$(git log --oneline -5 2>/dev/null || true)
status=$(git status -s 2>/dev/null || true)
dirty=""
[ -n "$status" ] && dirty="DIRTY TREE — uncommitted work exists (durable-state rule: uncommitted = didn't happen)."
brief=$(printf '%s\n' \
 "reposix orchestration brief" \
 "== STATE.md cursor ==" "$cursor" \
 "== last 5 commits ==" "$log" \
 "== working tree ==" "${status:-clean}" "$dirty" \
 "== doctrine digest (full: .planning/ORCHESTRATION.md — READ before dispatching agents) ==" \
 "1 Fable top-level: delegate ONLY to fable coordinators. No-fable top-level (opus or below): five-tier recursion per ORCHESTRATION 11. Either way: opus complex / sonnet default / haiku mechanical; never fable at a leaf." \
 "2 Coordinators route, don't work: Agent dispatches + 1-line git checks + short reads only." \
 "3 Scope every charter to reach end-state by ~10% of your OWN context (rest is correction margin, never planned workload); write+commit a handover past ~50% at a wave boundary." \
 "4 ONE cargo invocation machine-wide (VM OOM history); prefer -p <crate>, jobs=2." \
 "5 Understand intention over faithful plan execution; tangential tooling is a deliverable." \
 "6 Uncommitted = didn't happen; /tmp does not survive a crash." \
 "7 External mutations need owner-named-target approval." \
 "8 Quality upkeep is a standing parallel op; no dispatch over undrained BLOCKERs.")
printf '%s' "$brief" | python3 -c 'import sys,json;print(json.dumps({"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":sys.stdin.read()}}))'
exit 0
