#!/usr/bin/env bash
# 10-hooks.sh — drive .claude/hooks/*.sh through their JSON-stdin contract in a
# hermetic sandbox (private TMPDIR + throwaway git repo, no network). Also runs
# the committed hook-throttle regression harness, which itself exercises
# throttle.sh + dispatch-doctrine.sh + post-dispatch-relay.sh.
#
# Harness contract: exit 0 regardless of target exit codes (a coverage harness
# is not a gate). See 00-probe.sh header.
set -eu

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
HOOKS="$REPO_ROOT/.claude/hooks"

SANDBOX="$(mktemp -d)"
export TMPDIR="$SANDBOX"
trap 'rm -rf "$SANDBOX"' EXIT

# A throwaway git repo for hooks that inspect working-tree state.
REPO_A="$SANDBOX/repo"
mkdir -p "$REPO_A"
git -C "$REPO_A" init -q
git -C "$REPO_A" config user.email t@t && git -C "$REPO_A" config user.name t
printf 'seed\n' > "$REPO_A/seed.txt"
git -C "$REPO_A" add -A && git -C "$REPO_A" -c commit.gpgsign=false commit -qm seed

feed() { printf '%s' "$1" | "$2" >/dev/null 2>&1 || true; }

# cargo-mutex: non-cargo command (early exit 0) + a cargo command (allow branch,
# assuming no build is running — which the caller guarantees for coverage runs).
feed '{"tool_input":{"command":"ls -la"}}'       "$HOOKS/cargo-mutex.sh"
feed '{"tool_input":{"command":"cargo check -p x"}}' "$HOOKS/cargo-mutex.sh"
feed 'not json'                                   "$HOOKS/cargo-mutex.sh"

# stop-uncommitted: clean tree (no message) + dirty tree (message) + loop-guard.
CLAUDE_PROJECT_DIR="$REPO_A" feed '{"stop_hook_active":false}' "$HOOKS/stop-uncommitted.sh"
printf 'dirty\n' > "$REPO_A/dirty.txt"
CLAUDE_PROJECT_DIR="$REPO_A" feed '{"stop_hook_active":false}' "$HOOKS/stop-uncommitted.sh"
CLAUDE_PROJECT_DIR="$REPO_A" feed '{"stop_hook_active":true}'  "$HOOKS/stop-uncommitted.sh"

# precompact-persist: always emits.
feed '{}' "$HOOKS/precompact-persist.sh"

# session-start-brief: emits cursor+doctrine digest against the sandbox repo.
CLAUDE_PROJECT_DIR="$REPO_A" feed '{}' "$HOOKS/session-start-brief.sh"

# throttle + dispatch-doctrine + post-dispatch-relay: the committed regression
# harness drives all three across emit/suppress/stale/fail-open branches.
"$REPO_ROOT/quality/gates/agent-ux/hook-throttle.sh" >/dev/null 2>&1 || true

exit 0
