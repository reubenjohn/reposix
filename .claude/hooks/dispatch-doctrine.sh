#!/usr/bin/env bash
# .claude/hooks/dispatch-doctrine.sh — PreToolUse/Task|Agent: JIT dispatch doctrine.
# no canonical home under quality/gates/ — wired via .claude/settings.json
# Throttled per-agent via throttle.sh: the full checklist lands on an agent's FIRST
# dispatch, then repeats only if the agent's last nudge is >5min old — a coordinator
# dispatching 30 lanes back-to-back gets it once, not 30 times. transcript_path
# extraction fails open (incl. python3 missing): worst case = pre-throttle behavior.
set -eu
input=$(cat)
tp=$(printf '%s' "$input" | python3 -c 'import sys,json;print(json.load(sys.stdin).get("transcript_path",""))' 2>/dev/null || true)
if ! bash "$(dirname "$0")/throttle.sh" dispatch-doctrine "$tp" 300; then
  printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow"}}'
  exit 0
fi
printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","additionalContext":"Dispatch check — (a) model tier: never fable at a leaf; opus complex / sonnet default / haiku mechanical. (b) Ownership charter embedded verbatim? (c) Lane sliced <100 tool calls. (d) Report <=400 words, evidence committed not chatted. (e) Read >100 lines -> reader-digester. (f) Past ~100k tokens (hard stop ~150k; absolute, not %) at this wave boundary? -> relief-handover-writer. (g) Is this dispatch fixing a BLOCKER, implementing a sketched design, reordering the plan, re-dispatching after apparent stall, or covering out-of-charter work? -> load the decision-procedures skill FIRST, before dispatching, not after."}}'
exit 0
