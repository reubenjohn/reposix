#!/usr/bin/env bash
# .claude/hooks/dispatch-doctrine.sh — PreToolUse/Task|Agent: JIT dispatch doctrine.
# no canonical home under quality/gates/ — wired via .claude/settings.json
set -eu
cat >/dev/null
printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","additionalContext":"Dispatch check — (a) model tier: never fable at a leaf; opus complex / sonnet default / haiku mechanical. (b) Ownership charter embedded verbatim? (c) Lane sliced <100 tool calls. (d) Report <=400 words, evidence committed not chatted. (e) Read >100 lines -> reader-digester. (f) Past ~50% context at this wave boundary? -> relief-handover-writer."}}'
exit 0
