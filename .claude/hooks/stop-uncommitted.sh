#!/usr/bin/env bash
# .claude/hooks/stop-uncommitted.sh — Stop, advisory: warn on dirty/ahead tree.
# no canonical home under quality/gates/ — wired via .claude/settings.json
# ADVISORY ONLY (owner decision Q2, 2026-07-04): ALWAYS exits 0 — never blocks
# the turn. Emits a systemMessage when the tree is dirty or ahead of origin.
set -eu
payload=$(cat)
active=$(printf '%s' "$payload" | python3 -c 'import sys,json;print(json.load(sys.stdin).get("stop_hook_active",False))' 2>/dev/null || printf 'True')
[ "$active" = "True" ] && exit 0   # loop guard — cheap early exit, don't re-emit mid-loop
cd "${CLAUDE_PROJECT_DIR:-.}" 2>/dev/null || exit 0
dirty=$(git status --porcelain 2>/dev/null || true)
ahead=$(git rev-list --count '@{u}..HEAD' 2>/dev/null || printf '0')
if [ -n "$dirty" ] || [ "${ahead:-0}" -gt 0 ]; then
  msg="ADVISORY: tree dirty/ahead (${ahead} unpushed) — uncommitted work does not survive a crash; commit+push before ending. Durable-state rule: uncommitted = didn't happen."
  printf '%s' "$msg" | python3 -c 'import sys,json;print(json.dumps({"systemMessage":sys.stdin.read()}))'
fi
exit 0
