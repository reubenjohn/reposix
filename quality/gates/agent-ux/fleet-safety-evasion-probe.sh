#!/usr/bin/env bash
# quality/gates/agent-ux/fleet-safety-evasion-probe.sh
# Adversarial evasion probe against .claude/hooks/leaf-isolation-guard.sh (P102 fix lane).
#
# Durable, RE-RUNNABLE proof: this script IS the committed artifact (per the kind:
# shell-subprocess philosophy — proof is the script + hook, not a frozen transcript). It
# self-checks: every vector carries its expected exit code; the script exits non-zero if any
# vector deviates, so re-running regenerates the proof. Drove live by an APPROVE-WITH-NITS
# code-review that found the original guards' evasion vectors; each `*** WRONG ***` line here
# would have marked a live bypass before the P102 hardening.
#
# LEAF-ISOLATION: READ-ONLY vs the shared repo — it only feeds the hook crafted JSON on
# stdin (the hook exits pre-execution, nothing mutates). The single real symlink it creates
# lives under /tmp only. Safe to run from the shared repo.
set -uo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)"
HOOK="${repo_root}/.claude/hooks/leaf-isolation-guard.sh"
SHARED="$repo_root"
PASS=0; FAIL=0

# drive <cmd> <cwd|__OMIT__> <want_rc> <label>
drive() {
  local cmd="$1" cwd="$2" want="$3" label="$4" p rc
  if [ "$cwd" = "__OMIT__" ]; then
    p=$(python3 -c 'import json,sys;print(json.dumps({"tool_input":{"command":sys.argv[1]}}))' "$cmd")
  else
    p=$(python3 -c 'import json,sys;print(json.dumps({"tool_input":{"command":sys.argv[1]},"cwd":sys.argv[2]}))' "$cmd" "$cwd")
  fi
  printf '%s' "$p" | bash "$HOOK" >/dev/null 2>&1; rc=$?
  local verdict="OK"
  if [ "$rc" != "$want" ]; then verdict="*** WRONG (want $want) ***"; FAIL=$((FAIL+1)); else PASS=$((PASS+1)); fi
  printf 'rc=%s want=%s %-8s [%s] cwd=[%s]  %s\n' "$rc" "$want" "$verdict" "$cmd" "$cwd" "$label"
}
raw() { # <raw-stdin> <want_rc> <label>
  local rc; printf '%s' "$1" | bash "$HOOK" >/dev/null 2>&1; rc=$?
  local verdict="OK"; if [ "$rc" != "$2" ]; then verdict="*** WRONG (want $2) ***"; FAIL=$((FAIL+1)); else PASS=$((PASS+1)); fi
  printf 'rc=%s want=%s %-8s raw=[%s]  %s\n' "$rc" "$2" "$verdict" "$1" "$3"
}

echo "=== Guard A quoting evasion (must BLOCK rc=2) ==="
drive "git -c user.email='t@t' commit -m x" "$SHARED" 2 "single-quoted fixture email"
drive 'git -c user.email="t@t" commit -m x' "$SHARED" 2 "double-quoted fixture email"
drive 'env GIT_AUTHOR_EMAIL=t@t git commit -m x' "$SHARED" 2 "env GIT_AUTHOR_EMAIL"
drive "git -c user.name='t' -c user.email='t@t' commit -m x" "$SHARED" 2 "quoted name+email"
echo "=== Guard A must NOT false-positive on a real address (rc=0) ==="
drive "git -c user.email=scott@things.io commit -m x" "$SHARED" 0 "real email contains t@t substring"
drive 'git commit -m "documents the t@t fixture"' "$SHARED" 0 "t@t only in commit message"

echo "=== Guard B cd-back / traversal evasion (must BLOCK rc=2) ==="
drive "cd /tmp/x && cd $SHARED && reposix init sim::demo ." "$SHARED" 2 "cd /tmp then cd back to shared"
drive "cd /tmp/..$SHARED && reposix init sim::demo ." "$SHARED" 2 "/tmp/../ traversal to shared"
echo "=== Guard B canonical forms (must BLOCK rc=2) ==="
drive "/usr/bin/reposix init sim::demo ." "$SHARED" 2 "absolute-path reposix"
drive "./target/debug/reposix init sim::demo ." "$SHARED" 2 "relative-path reposix"
drive "cargo run -p reposix-cli -- init sim::demo ." "$SHARED" 2 "cargo run -- init"
drive "cargo run -p reposix-cli -- attach sim::demo ." "$SHARED" 2 "cargo run -- attach"
drive "reposix init sim::demo ." "$SHARED" 2 "bare reposix init (control)"
drive "reposix attach sim::demo ." "$SHARED" 2 "bare reposix attach"
drive "reposix sync --reconcile" "$SHARED" 2 "bare reposix sync"
echo "=== Guard B legit /tmp forms (must ALLOW rc=0 — no over-block regression) ==="
drive "cd /tmp/leaf-clone-xyz && reposix init sim::demo ." "$SHARED" 0 "cd /tmp then init"
drive "cd /tmp/leaf-$$ && cargo run -p reposix-cli -- init sim::demo ." "$SHARED" 0 "cd /tmp then cargo init"
drive "reposix init sim::demo ." "/tmp/leaf-clone" 0 "cwd already /tmp"

echo "=== Guard C cd-back evasion (must BLOCK rc=2) ==="
drive "cd /tmp/x && cd $SHARED && git config core.bare true" "$SHARED" 2 "cd /tmp then cd back, config write"
drive "git config --local core.bare true" "$SHARED" 2 "--local core.bare"
drive "git config user.email t@t" "$SHARED" 2 "config user.email write"
echo "=== Guard C legit /tmp-target writes (must ALLOW rc=0) ==="
drive "git config -f /tmp/clone/.git/config core.bare true" "$SHARED" 0 "-f short flag /tmp (was false-positive)"
drive "git config --file /tmp/clone/.git/config core.bare true" "$SHARED" 0 "--file /tmp"
drive "git config --get core.bare" "$SHARED" 0 "read is not a write"

echo "=== symlink-to-shared cwd (real symlink under /tmp, must BLOCK rc=2) ==="
LINK="/tmp/leaf-iso-symlink-$$"; ln -sfn "$SHARED" "$LINK"
drive "reposix init sim::demo ." "$LINK" 2 "cwd is /tmp symlink -> shared"
drive "git config core.bare true" "$LINK" 2 "config write, cwd /tmp symlink -> shared"
rm -f "$LINK"
echo "=== nonexistent /tmp path stays safe (rc=0) ==="
drive "reposix init sim::demo ." "/tmp/does-not-exist-$$" 0 "genuine throwaway /tmp path"

echo "=== fail-closed parse ==="
raw 'not json at all' 2 "non-empty unparseable -> BLOCK"
raw '{"tool_input": {' 2 "truncated json -> BLOCK"
raw '' 0 "empty payload -> allow"
raw '{}' 0 "empty obj, no command -> allow"
raw '[]' 2 "non-object json -> BLOCK"

echo "======================================"
printf 'PROBE RESULT: PASS=%s FAIL=%s\n' "$PASS" "$FAIL"
[ "$FAIL" = 0 ]
