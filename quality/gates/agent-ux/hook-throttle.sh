#!/usr/bin/env bash
# quality/gates/agent-ux/hook-throttle.sh — regression test for the per-agent
# JIT-context emission throttle (.claude/hooks/throttle.sh) and its two wired
# consumers (dispatch-doctrine.sh PreToolUse, post-dispatch-relay.sh PostToolUse).
#
# Contract under test:
#   - first hook invocation for a given transcript_path EMITS the full nudge;
#   - repeats within the cooldown are SUPPRESSED (PreToolUse suppressed output
#     must STILL be valid JSON carrying permissionDecision=allow; PostToolUse
#     suppressed output is empty, exit 0);
#   - a stale last-emission (older than cooldown) re-emits;
#   - all failure paths (empty stdin, invalid JSON, missing transcript file)
#     FAIL OPEN to pre-throttle emit-always behavior, exit 0;
#   - state is isolated per transcript_path and per hook name.
#
# Hermetic: runs against a private TMPDIR so it never touches live throttle
# state of running sessions. Exit 0 = all asserts pass; exit 1 = regression.
set -eu
cd "$(dirname "$0")/../../.."
HOOKS=.claude/hooks
SANDBOX=$(mktemp -d)
export TMPDIR="$SANDBOX"
trap 'rm -rf "$SANDBOX"' EXIT

fail=0
say() { printf '%s\n' "$*"; }
assert() { # name cond
  if [ "$2" -eq 0 ]; then say "PASS $1"; else say "FAIL $1"; fail=1; fi
}
json_ok() { printf '%s' "$1" | python3 -m json.tool >/dev/null 2>&1; }

TP="$SANDBOX/agent-a.jsonl"; printf '{}' > "$TP"
TP2="$SANDBOX/agent-b.jsonl"; printf '{}' > "$TP2"
IN='{"transcript_path":"'"$TP"'","tool_name":"Agent","tool_input":{}}'
IN2='{"transcript_path":"'"$TP2"'","tool_name":"Agent","tool_input":{}}'

# 1. dispatch-doctrine: first call emits full checklist
out=$(printf '%s' "$IN" | bash $HOOKS/dispatch-doctrine.sh)
json_ok "$out" && case "$out" in *additionalContext*) c=0;; *) c=1;; esac || c=1
assert "dd first call emits valid JSON with additionalContext" "$c"

# 2. immediate repeat suppressed but still valid allow JSON
out=$(printf '%s' "$IN" | bash $HOOKS/dispatch-doctrine.sh)
json_ok "$out" || { assert "dd suppressed output valid JSON" 1; out=''; }
case "$out" in
  *additionalContext*) assert "dd repeat suppressed (no additionalContext)" 1 ;;
  *'"permissionDecision":"allow"'*) assert "dd repeat suppressed, allow preserved" 0 ;;
  *) assert "dd suppressed output kept permissionDecision=allow" 1 ;;
esac

# 3. different agent (transcript path) unaffected by agent-a's state
out=$(printf '%s' "$IN2" | bash $HOOKS/dispatch-doctrine.sh)
case "$out" in *additionalContext*) c=0;; *) c=1;; esac
assert "dd second agent gets its own first emission" "$c"

# 4. stale state (older than cooldown) re-emits
key=$(printf '%s' "$TP" | sha256sum | cut -c1-16)
touch -d '-10 minutes' "$SANDBOX/claude-hook-throttle/dispatch-doctrine.$key"
out=$(printf '%s' "$IN" | bash $HOOKS/dispatch-doctrine.sh)
case "$out" in *additionalContext*) c=0;; *) c=1;; esac
assert "dd stale (>cooldown) state re-emits" "$c"

# 5. post-dispatch-relay: first emits valid JSON, repeat is silent exit 0
out=$(printf '%s' "$IN" | bash $HOOKS/post-dispatch-relay.sh)
json_ok "$out" && case "$out" in *additionalContext*) c=0;; *) c=1;; esac || c=1
assert "relay first call emits valid JSON" "$c"
out=$(printf '%s' "$IN" | bash $HOOKS/post-dispatch-relay.sh); rc=$?
[ $rc -eq 0 ] && [ -z "$out" ] && c=0 || c=1
assert "relay repeat silent, exit 0" "$c"

# 6. relay state independent of dispatch-doctrine state (same path, hook-keyed)
[ -f "$SANDBOX/claude-hook-throttle/post-dispatch-relay.$key" ] && c=0 || c=1
assert "throttle state keyed per hook name" "$c"

# 7. fail-open: empty stdin, invalid JSON stdin, nonexistent transcript file
for bad in '' 'not json' '{"transcript_path":"'"$SANDBOX/nope.jsonl"'"}'; do
  out=$(printf '%s' "$bad" | bash $HOOKS/dispatch-doctrine.sh); rc=$?
  json_ok "$out" && [ $rc -eq 0 ] && c=0 || c=1
  assert "dd fail-open on stdin [${bad:-empty}] (exit 0, valid JSON)" "$c"
done

# 8. throttle.sh direct: explicit cooldown honored
bash $HOOKS/throttle.sh unit-test "$TP" 1 && c=0 || c=1   # fresh key -> emit
assert "throttle first call emits" "$c"
bash $HOOKS/throttle.sh unit-test "$TP" 60 && c=1 || c=0  # within cooldown -> suppress
assert "throttle within cooldown suppresses" "$c"
sleep 1.2
bash $HOOKS/throttle.sh unit-test "$TP" 1 && c=0 || c=1   # cooldown 1s elapsed -> emit
assert "throttle explicit cooldown elapsed re-emits" "$c"

# 9. rapid-fire: 10 calls, exactly 1 emission
n=0
for _ in 1 2 3 4 5 6 7 8 9 10; do
  out=$(printf '%s' "$IN2" | bash $HOOKS/post-dispatch-relay.sh)
  [ -n "$out" ] && n=$((n+1))
done
[ "$n" -eq 1 ] && c=0 || c=1
assert "relay 10 rapid calls -> exactly 1 emission (got $n)" "$c"

exit "$fail"
