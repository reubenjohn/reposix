#!/usr/bin/env bash
# 11-fleet-safety-guards.sh — drive the fleet-safety leaf-isolation surface through
# kcov: the four agent-ux fleet-safety-*.sh gates AND their shared target,
# .claude/hooks/leaf-isolation-guard.sh. The four gates are REAL behavioral tests of
# the guard (allow / block / fail-closed / quoting / canonical-form / cd-back-traversal /
# config-read-vs-write branches); running them exercises the guard's branches honestly —
# this is coverage of genuine behavior, not line-touching for its own sake. kcov traces
# child bash subprocesses via ptrace, so invoking each gate ALSO captures the guard it
# spawns (`bash "$HOOK"`) plus the shared transcript helper.
#
# Harness contract (see 00-probe.sh / 10-hooks.sh headers): hermetic sandbox (private
# TMPDIR + throwaway git repo, no network), drive each target via its shebang, and exit 0
# regardless of target exit codes — a coverage harness is NOT a gate. A gate's own non-zero
# exit is reported by the gate that owns it, not by this harness.
#
# LEAF-ISOLATION (doubly load-bearing here — this harness tests the leaf-isolation guard
# itself): every git/reposix write lands inside the mktemp -d sandbox with `cd` in the SAME
# invocation. The direct guard drives below only feed crafted JSON on stdin; the guard
# BLOCKs pre-execution, so no guarded command ever runs against the shared repo.
set -eu

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
AGENT_UX="$REPO_ROOT/quality/gates/agent-ux"
HOOK="$REPO_ROOT/.claude/hooks/leaf-isolation-guard.sh"

SANDBOX="$(mktemp -d)"
export TMPDIR="$SANDBOX"
trap 'rm -rf "$SANDBOX"' EXIT

# --- 1) Drive the four fleet-safety gates ------------------------------------------------
# Each builds its own sandboxes and drives the guard exhaustively. Swallow non-zero exit.
for gate in fleet-safety-tat-identity-reject \
            fleet-safety-leaf-isolation-enforce \
            fleet-safety-shared-config-write-guard \
            fleet-safety-evasion-probe; do
  "$AGENT_UX/$gate.sh" >/dev/null 2>&1 || true
done

# --- 2) Directly drive leaf-isolation-guard.sh across its branches -----------------------
# Belt-and-suspenders on top of the gates: feed the guard's JSON-stdin PreToolUse contract
# multiple input variants so every guard (A/B/C), the allow path, and both fail-closed
# stages are hit even if a gate is later refactored. Payload shape mirrors the gates'
# {"tool_input":{"command":..},"cwd":..} contract (parsed by the guard's python3 stage).
SHARED="$REPO_ROOT"

feed_json() {  # feed_json <command> <cwd>
  python3 -c 'import json,sys;print(json.dumps({"tool_input":{"command":sys.argv[1]},"cwd":sys.argv[2]}))' \
    "$1" "$2" | bash "$HOOK" >/dev/null 2>&1 || true
}
feed_json_nocwd() {  # feed_json_nocwd <command>  (undeterminable-cwd fail-closed path)
  python3 -c 'import json,sys;print(json.dumps({"tool_input":{"command":sys.argv[1]}}))' \
    "$1" | bash "$HOOK" >/dev/null 2>&1 || true
}
feed_raw() {  # feed_raw <raw-stdin>
  printf '%s' "$1" | bash "$HOOK" >/dev/null 2>&1 || true
}

# Allow path: a benign non-git command, and a real identity (Guard A no-false-positive).
feed_json 'ls -la && grep -r TODO .'                 "$SHARED"
feed_json 'git -c user.email=dev@example.com commit -m x' "$SHARED"
# Guard A: fixture identity (bare + quoted) vs shared -> block; real substring -> allow.
feed_json 'git -c user.email=t@t commit -m x'        "$SHARED"
feed_json "git -c user.email='t@t' commit -m x"      "$SHARED"
feed_json 'env GIT_AUTHOR_EMAIL=t@t git commit -m x' "$SHARED"
feed_json 'git -c user.email=scott@things.io commit -m x' "$SHARED"
feed_json 'git commit -m "documents the t@t fixture"' "$SHARED"
# Guard B: leaf-setup verbs in shared -> block; /tmp redirect + sim-server -> allow.
feed_json 'reposix init sim::demo .'                 "$SHARED"
feed_json '/usr/bin/reposix init sim::demo .'        "$SHARED"
feed_json 'cargo run -p reposix-cli -- init sim::demo .' "$SHARED"
feed_json 'reposix attach sim::demo .'               "$SHARED"
feed_json 'reposix sync --reconcile'                 "$SHARED"
feed_json './target/debug/reposix-sim seed issues'   "$SHARED"
feed_json 'cargo run -p reposix-sim -- seed issues'  "$SHARED"
feed_json 'git init --bare'                          "$SHARED"
feed_json 'cd /tmp/leaf-clone-xyz && reposix init sim::demo .' "$SHARED"
feed_json 'cargo run -p reposix-sim'                 "$SHARED"          # sim-server: allow
feed_json "cd /tmp/x && cd $SHARED && reposix init sim::demo ." "$SHARED"  # cd-back: block
# Guard C: config write -> block; read + trailing token -> allow; /tmp --file -> allow.
feed_json 'git config core.bare true'                "$SHARED"
feed_json 'git config user.email t@t'                "$SHARED"
feed_json 'git config --get core.bare 2>/dev/null'   "$SHARED"
feed_json 'git config --get user.email 2>/dev/null || echo x' "$SHARED"
feed_json 'git config --file /tmp/leaf/.git/config core.bare true' "$SHARED"
feed_json 'git config -f /tmp/leaf/.git/config core.bare true' "$SHARED"
feed_json 'git config --global user.email x@y'       "$SHARED"          # --global: out of scope
# Fail-closed + empty branches of the parse stage.
feed_json_nocwd 'reposix init sim::demo .'           # undeterminable cwd -> block
feed_raw 'not json at all'                           # non-empty unparseable -> block
feed_raw '{"tool_input": {'                          # truncated json -> block
feed_raw '[]'                                        # non-object json -> block
feed_raw ''                                          # empty payload -> allow
feed_raw '{}'                                        # empty obj, no command -> allow

exit 0
