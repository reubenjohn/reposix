#!/usr/bin/env bash
# .claude/hooks/leaf-isolation-guard.sh — PreToolUse/Bash: leaf-isolation safety substrate.
# no canonical home under quality/gates/ — wired via .claude/settings.json (sibling to
# cargo-mutex.sh). Mechanical enforcement of the ORCHESTRATION.md § "Leaf isolation for
# reposix/sim/git test setup (HARD STOP)" doctrine. Founding incident: S-260707-pr-08 — a
# sim-seed leaf that "forgot to cd" into /tmp committed under fixture identity `t <t@t>`
# and flipped core.bare=true, corrupting the shared checkout for every concurrent agent.
#
# THREE fail-closed guards, first-match-blocks (exit 2 = BLOCK, matching cargo-mutex.sh):
#   A guard_fixture_identity   — block a fixture-identity (`t@t`) git commit/-c/env against
#                                the SHARED tree; allow it in a /tmp clone.
#   B guard_leaf_setup_location — block reposix init|attach|sync / sim-seed against the
#                                SHARED tree with no same-invocation /tmp redirect.
#   C guard_shared_config_write — block a `git config` WRITE of core.bare|user.email|
#                                user.name targeting the SHARED .git/config.
#
# COVERAGE BOUNDARY (documented honestly, per D2 raise-list): this PreToolUse hook fires
# ONLY on the Claude Code Bash *tool*. A git/reposix write spawned by a subprocess or a
# shell script (not the tool directly) BYPASSES it. The `.githooks/pre-commit` git-native
# backstop catches fixture-identity COMMITS in the shared repo even on that bypass path,
# but NOT `reposix init` / `git config` non-commit writes. A binary-side / git-alias
# non-tool backstop for guards B/C is filed as a GOOD-TO-HAVE (v0.14.0), not built here.
#
# HARD CONSTRAINT (ROADMAP SC2 / T-102-04): this mechanism invokes `git worktree remove
# --force` NOWHERE — that command is itself a corruption vector.

set -eu

# --- fixture identity list (configurable; email tokens the throwaway test identity uses) ---
# The canonical fixture is `t <t@t>`; extend this alternation as new throwaway identities
# appear (regex-escaped, `|`-separated email locals). Matched with delimiter boundaries so
# a real address that merely CONTAINS the token as a substring (e.g. `scott@things.io`
# contains `t@t`) does NOT false-positive. The email `t@t` is the reliable signal; the
# bare name `t` is too generic to match safely, so identity is keyed on the email.
FIXTURE_EMAIL_ALT='t@t'

payload=$(cat)

# Extract the bash command string (may be multi-line; case-globs handle that fine).
cmd=$(printf '%s' "$payload" | python3 -c 'import sys,json;print(json.load(sys.stdin).get("tool_input",{}).get("command",""))' 2>/dev/null || true)

# Extract the effective cwd the runtime carries. Live-verified payload shape (D2 cwd
# linchpin): Claude Code sends a top-level "cwd"; tool_input MAY also carry one. Prefer
# tool_input.cwd, then top-level cwd. If BOTH are absent → empty string → fail-closed
# (treated as shared/unsafe below), with a $PWD fallback as a last resort.
eff_cwd=$(printf '%s' "$payload" | python3 -c 'import sys,json
d=json.load(sys.stdin)
print(d.get("tool_input",{}).get("cwd") or d.get("cwd") or "")' 2>/dev/null || true)
[ -z "$eff_cwd" ] && eff_cwd="${PWD:-}"

# Canonical shared-repo root. settings.json interpolates ${CLAUDE_PROJECT_DIR}; when the
# hook is driven directly (verifier subprocess) it may be unset, so derive from the
# script's own location (.claude/hooks/ → repo root is two levels up).
if [ -n "${CLAUDE_PROJECT_DIR:-}" ]; then
  SHARED_ROOT="$CLAUDE_PROJECT_DIR"
else
  SHARED_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
fi

# --- helpers -----------------------------------------------------------------

# has_tmp_redirect: does the command carry a same-invocation redirect INTO /tmp?
# Recognised shapes: `cd /tmp…`, `-C /tmp…`, `--git-dir=/tmp…`, `--file /tmp…`,
# `--file=/tmp…`, `--work-tree=/tmp…`. Any one makes the effective target /tmp → SAFE.
has_tmp_redirect() {
  case " $cmd " in
    *"cd /tmp"*|*"-C /tmp"*|*"--git-dir=/tmp"*|*"--git-dir /tmp"*|\
    *"--file /tmp"*|*"--file=/tmp"*|*"--work-tree=/tmp"*|*"--work-tree /tmp"*)
      return 0 ;;
  esac
  return 1
}

# cwd_under_tmp: does the effective cwd itself resolve under /tmp (a throwaway clone)?
cwd_under_tmp() {
  case "$eff_cwd" in
    /tmp|/tmp/*|/private/tmp|/private/tmp/*) return 0 ;;
  esac
  return 1
}

# is_safe: the operation targets a throwaway /tmp location (redirect OR cwd) → allowed.
# Anything else — cwd in the shared tree, cwd elsewhere, or cwd undeterminable — is
# UNSAFE (fail-closed / default-deny). Only /tmp is the sanctioned safe zone.
is_safe() {
  if has_tmp_redirect; then return 0; fi
  if cwd_under_tmp; then return 0; fi
  return 1
}

block() {
  # $1 = teaching message (rule + why + recovery). Emit to stderr, exit 2 = BLOCK.
  printf '%s\n' "$1" >&2
  exit 2
}

# Command-position prefix (load-bearing): a guarded verb must appear where a command
# STARTS — at string start, or after a shell separator (`;`, `&&`, `||`, `|`, `&`, `(`,
# newline), optionally after leading env-assignments (`VAR=val `). This prevents a commit
# whose -m MESSAGE merely MENTIONS `reposix init` / `git config core.bare` (the command
# string includes the quoted message) from false-positive-blocking a legitimate commit.
# A real invocation is at a command position; a quoted argument is not.
CMD_POS_PREFIX='(^|[|&;(]|&&|\|\|)[[:space:]]*([A-Za-z_][A-Za-z0-9_]*=[^[:space:]]*[[:space:]]+)*'
at_command_position() {  # $1 = verb regex (ERE); true if it appears at a command position
  printf '%s' "$cmd" | grep -Eq -- "${CMD_POS_PREFIX}$1"
}

RECOVERY_HINT='RECOVERY: run this inside a throwaway /tmp clone in the SAME Bash invocation, under a REAL identity, e.g.:
  git clone "'"$SHARED_ROOT"'" /tmp/leaf-$$ && cd /tmp/leaf-$$ && <your command>
Doctrine: ORCHESTRATION.md § "Leaf isolation for reposix/sim/git test setup (HARD STOP)".'

# --- Guard A: fixture-identity reject ----------------------------------------
# BLOCK a git operation that carries a fixture-identity token (`t@t`, …) — a
# `git commit`, `git -c user.email=…`, or `GIT_AUTHOR_EMAIL=…`/`GIT_COMMITTER_EMAIL=…`
# prefix — when the effective location is the shared tree (unsafe). (D2-TAT-IDENTITY-HOOK-01)
guard_fixture_identity() {
  case "$cmd" in *git*) : ;; *) return 0 ;; esac
  # Match the fixture identity ONLY in an identity-ASSIGNMENT context, never a bare token
  # anywhere in the command. A commit message that merely MENTIONS `t@t` / `<t@t>` (e.g.
  # `git commit -m "documents the t@t fixture"`) must NOT be blocked — the command string
  # includes the -m message, so a naive substring scan false-positives on prose. The
  # reliable identity enforcement is the .githooks/pre-commit backstop (git var
  # GIT_AUTHOR_IDENT), immune to message contents; this guard catches the exact
  # S-260707-pr-08 command SHAPE (`-c user.email=…` / `GIT_*_EMAIL=…` / `--author=…<…>`)
  # BEFORE it runs. Assignment contexts:
  #   -c user.email=<fx> | -c user.name=t | GIT_{AUTHOR,COMMITTER}_{EMAIL,NAME}=… | --author=…<fx>
  local fx="$FIXTURE_EMAIL_ALT"
  printf '%s' "$cmd" | grep -Eq -- \
    "(-c[[:space:]]+user\.email=(${fx})([^A-Za-z0-9._%+-]|$))|(GIT_(AUTHOR|COMMITTER)_EMAIL=(${fx})([^A-Za-z0-9._%+-]|$))|(--author[=[:space:]][^;&|]*<(${fx})>)|(-c[[:space:]]+user\.name=t([^A-Za-z0-9._%+-]|$))|(GIT_(AUTHOR|COMMITTER)_NAME=t([^A-Za-z0-9._%+-]|$))" \
    || return 0
  local hit="$fx"
  is_safe && return 0
  block "BLOCKED (leaf-isolation guard A — fixture-identity reject): this git command carries the throwaway fixture identity '$hit' and would run against the SHARED repo at $SHARED_ROOT.
WHY: the fixture identity 't <t@t>' must NEVER reach shared history or origin — S-260707-pr-08: a fixture-identity leaf corrupted the shared repo (flipped core.bare=true) for every concurrent agent.
$RECOVERY_HINT"
}

# --- Guard B: leaf-setup location --------------------------------------------
# BLOCK a leaf-SETUP verb (`reposix init|attach|sync`, sim-seed / reposix-sim) when the
# effective location is the shared tree with no same-invocation /tmp redirect. The setup
# command NEVER runs — PreToolUse blocks pre-execution. (D2-LEAF-ISOLATION-01)
guard_leaf_setup_location() {
  # Only trigger when the setup verb is at a COMMAND POSITION (a real invocation), not
  # when it is mentioned inside a quoted argument (e.g. a commit message).
  local verb=""
  if   at_command_position 'reposix[[:space:]]+init';   then verb="reposix init"
  elif at_command_position 'reposix[[:space:]]+attach'; then verb="reposix attach"
  elif at_command_position 'reposix[[:space:]]+sync';   then verb="reposix sync"
  elif at_command_position 'reposix-sim';               then verb="sim-seed (reposix-sim)"
  fi
  [ -z "$verb" ] && return 0
  is_safe && return 0
  block "BLOCKED (leaf-isolation guard B — leaf setup in shared tree): '$verb' would run with effective cwd '$eff_cwd' inside the SHARED repo ($SHARED_ROOT) with NO same-invocation /tmp redirect.
WHY: agent worktrees SHARE the coordinator's .git/config + object store (they are NOT isolated) and cwd resets between Bash calls — a setup verb that did not 'cd /tmp' in the SAME invocation mutates the real shared repo. Founding incident: S-260707-pr-08.
$RECOVERY_HINT"
}

# --- Guard C: shared-config write --------------------------------------------
# BLOCK a `git config` WRITE of core.bare|user.email|user.name whose target is the shared
# .git/config (unsafe location, not --global/--system, no --file/--git-dir /tmp redirect).
# Reads (--get/--list/--get-all/--get-regexp) and core.hooksPath writes are NOT blocked.
# Because PreToolUse blocks BEFORE the tool runs, the shared .git/config is byte-unchanged.
# (D2-SHARED-CONFIG-GUARD-01)
guard_shared_config_write() {
  # Only trigger when `git config` is at a COMMAND POSITION (a real invocation), not when
  # it is mentioned inside a quoted argument (e.g. a commit message discussing the guard).
  at_command_position 'git[[:space:]]+config' || return 0
  # --global/--system do not touch this repo's .git/config — out of scope.
  case "$cmd" in *"--global"*|*"--system"*) return 0 ;; esac
  local key=""
  case "$cmd" in
    *core.bare*)   key="core.bare" ;;
    *user.email*)  key="user.email" ;;
    *user.name*)   key="user.name" ;;
  esac
  [ -z "$key" ] && return 0
  # READ vs WRITE discrimination (load-bearing — a bare `git config <key>` is a READ that
  # PRINTS the value, not a mutation, and must NOT be blocked). A WRITE is either:
  #   (a) an explicit write flag (--unset/--unset-all/--add/--replace-all) on the key, OR
  #   (b) the key IMMEDIATELY followed by a value token (whitespace then a non-flag char).
  # `git config user.email` (read) → key at segment end, no value → NOT a write → allow.
  # `git config user.email t@t` (write) → key + space + 't' → write → candidate for block.
  local is_write=1
  printf '%s' "$cmd" | grep -Eq -- "git config[^;&|]*--(unset|unset-all|add|replace-all)[^;&|]*(core\.bare|user\.email|user\.name)" && is_write=0
  printf '%s' "$cmd" | grep -Eq -- "(core\.bare|user\.email|user\.name)[[:space:]]+[^[:space:]-]" && is_write=0
  [ "$is_write" = 0 ] || return 0
  is_safe && return 0
  block "BLOCKED (leaf-isolation guard C — shared-config write): a 'git config' WRITE of '$key' targets the SHARED .git/config ($SHARED_ROOT/.git/config).
WHY: writing core.bare / user.email / user.name into the shared config corrupts the checkout for every concurrent agent — this is the exact S-260707-pr-08 corruption (core.bare=true). The write was blocked pre-execution, so the shared .git/config is byte-unchanged.
$RECOVERY_HINT"
}

# --- dispatch: first-match-blocks --------------------------------------------
guard_fixture_identity
guard_leaf_setup_location
guard_shared_config_write

# No guard tripped — allow. Mirror cargo-mutex.sh allow contract.
printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","additionalContext":"leaf-isolation OK (no fixture identity / shared-tree setup verb / shared-config write). Reminder: run reposix/sim/git test setup in a /tmp clone, cd in the SAME Bash invocation."}}'
exit 0
