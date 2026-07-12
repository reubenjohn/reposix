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
# HARDENING (v0.14.0 P102 adversarial fix lane — closes the evasion vectors an
# APPROVE-WITH-NITS code-review drove live):
#   * Guard B canonical-form coverage: catches the CLAUDE.md-documented dev forms too —
#     bare `reposix init`, path-suffixed `/usr/bin/reposix init` / `./target/…/reposix init`,
#     and `cargo run … -- (init|attach|sync)`, plus path-suffixed sim-seed. A leaf that
#     "forgot to cd" via ANY of these canonical spellings now BLOCKs at the shared tree.
#   * realpath-based effective-location: the /tmp-means-safe decision resolves the EFFECTIVE
#     target (last `cd`, or a git path-flag target, else the payload cwd) and
#     realpath-canonicalizes it, so `cd /tmp/x && cd <shared>`, `/tmp/../<shared>`, and a
#     `/tmp/symlink→<shared>` cwd all canonicalize back to the shared tree and BLOCK.
#   * Guard A quoting tolerance: `-c user.email='t@t'` / `"t@t"` are caught (optional
#     surrounding quotes), while a real address (`scott@things.io`) still does NOT match.
#   * Fail-CLOSED parse: a non-empty but UNPARSEABLE tool payload BLOCKs (exit 2) rather
#     than falling through to allow. An empty payload (nothing to inspect) still passes.
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

# Parse the payload ONCE, fail-closed. python3 emits STATUS + base64(cmd) + base64(cwd):
#   empty        — payload is whitespace-only / absent → nothing to inspect → allow.
#   parse_error  — payload is NON-EMPTY but not parseable JSON → fail CLOSED (block).
#   ok           — extracted the Bash command + effective cwd.
# base64 round-trips a command that may contain quotes/newlines without shell re-parsing.
parse_out=$(printf '%s' "$payload" | python3 -c '
import sys, json, base64
raw = sys.stdin.read()
def emit(status, cmd="", cwd=""):
    print("STATUS=" + status)
    print("CMD=" + base64.b64encode(cmd.encode()).decode())
    print("CWD=" + base64.b64encode(cwd.encode()).decode())
if raw.strip() == "":
    emit("empty"); sys.exit(0)
try:
    d = json.loads(raw)
except Exception:
    emit("parse_error"); sys.exit(0)
if not isinstance(d, dict):
    emit("parse_error"); sys.exit(0)
ti = d.get("tool_input") or {}
if not isinstance(ti, dict):
    ti = {}
cmd = ti.get("command", "")
if not isinstance(cmd, str):
    cmd = ""
cwd = ti.get("cwd") or d.get("cwd") or ""
if not isinstance(cwd, str):
    cwd = ""
emit("ok", cmd, cwd)
' 2>/dev/null || true)

# --- helpers (defined before dispatch so the fail-closed paths can call block()) ---------

block() {
  # $1 = teaching message (rule + why + recovery). Emit to stderr, exit 2 = BLOCK.
  printf '%s\n' "$1" >&2
  exit 2
}

emit_allow() {
  printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","additionalContext":"leaf-isolation OK (no fixture identity / shared-tree setup verb / shared-config write). Reminder: run reposix/sim/git test setup in a /tmp clone, cd in the SAME Bash invocation."}}'
  exit 0
}

# Fail-closed parse dispatch. If python emitted nothing at all (interpreter missing/crashed)
# but the payload is non-empty, BLOCK rather than run blind.
FAILCLOSED_MSG='BLOCKED (leaf-isolation guard — fail-closed parse): the tool payload was NON-EMPTY but the command could not be extracted (unparseable JSON or unavailable interpreter). A fail-closed security guard blocks rather than let an un-inspected command run.
WHY: leaf-isolation enforcement (S-260707-pr-08) must default-DENY on ambiguous input — an unparseable payload could be a shared-tree mutation the guard simply failed to read.
RECOVERY: re-issue the command as a well-formed Bash tool call; if this misfires on a legitimate payload, inspect .claude/hooks/leaf-isolation-guard.sh parse stage.'

status=$(printf '%s\n' "$parse_out" | sed -n 's/^STATUS=//p')
if [ -z "$status" ]; then
  # python produced no STATUS line. Fail closed on any non-empty payload; allow if empty.
  if [ -n "$(printf '%s' "$payload" | tr -d '[:space:]')" ]; then
    block "$FAILCLOSED_MSG"
  fi
  emit_allow
fi
case "$status" in
  empty)       emit_allow ;;
  parse_error) block "$FAILCLOSED_MSG" ;;
  ok)          : ;;
  *)           block "$FAILCLOSED_MSG" ;;
esac

cmd=$(printf '%s\n' "$parse_out" | sed -n 's/^CMD=//p' | base64 -d 2>/dev/null || true)
eff_cwd=$(printf '%s\n' "$parse_out" | sed -n 's/^CWD=//p' | base64 -d 2>/dev/null || true)
[ -z "$eff_cwd" ] && eff_cwd="${PWD:-}"

# Canonical shared-repo root. settings.json interpolates ${CLAUDE_PROJECT_DIR}; when the
# hook is driven directly (verifier subprocess) it may be unset, so derive from the
# script's own location (.claude/hooks/ → repo root is two levels up).
if [ -n "${CLAUDE_PROJECT_DIR:-}" ]; then
  SHARED_ROOT="$CLAUDE_PROJECT_DIR"
else
  SHARED_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
fi

# --- effective-location resolution (realpath-sound; the /tmp-means-safe linchpin) --------
# effective_target: the path the command actually operates against, resolved soundly so a
# cd-back / `..`-traversal / symlink cannot smuggle a shared-tree op past the /tmp check.
# Priority (last-wins within each tier):
#   1. an explicit git path-target flag (-C, -f, --git-dir, --file, --work-tree) — these
#      override cwd for that git invocation, so a `--file /tmp/…` legitimately targets /tmp.
#   2. the LAST `cd <path>` at a command position — the effective cwd after chained cds
#      (`cd /tmp/x && cd <shared>` ends at <shared>, NOT /tmp).
#   3. the payload's effective cwd (fallback: $PWD).
effective_target() {
  local t
  t=$(printf '%s' "$cmd" | grep -oE -- '(-C|-f|--git-dir|--file|--work-tree)([= ])[^[:space:];&|]+' | tail -1 | sed -E 's/^(-C|-f|--git-dir|--file|--work-tree)[= ]//')
  if [ -n "$t" ]; then printf '%s' "$t"; return 0; fi
  t=$(printf '%s' "$cmd" | grep -oE '(^|[;&|(]|&&|\|\|)[[:space:]]*cd[[:space:]]+[^[:space:];&|]+' | tail -1 | sed -E 's/.*cd[[:space:]]+//')
  if [ -n "$t" ]; then printf '%s' "$t"; return 0; fi
  printf '%s' "$eff_cwd"
}

# is_safe: the operation's EFFECTIVE target realpath-canonicalizes under /tmp (a throwaway
# clone) → allowed. Anything else — shared tree, elsewhere, symlink/`..` that lands back in
# shared, or an undeterminable target — is UNSAFE (fail-closed / default-deny). Only a
# path that truly resolves under /tmp is the sanctioned safe zone.
is_safe() {
  local tgt canon
  tgt=$(effective_target)
  [ -z "$tgt" ] && return 1                       # undeterminable → fail closed
  canon=$(realpath -m -- "$tgt" 2>/dev/null || true)
  [ -z "$canon" ] && return 1                     # realpath unavailable/failed → fail closed
  case "$canon" in
    /tmp|/tmp/*|/private/tmp|/private/tmp/*) return 0 ;;
  esac
  return 1
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
  # BEFORE it runs. Quoting tolerance: optional `'`/`"` around the fixture email is allowed
  # (`-c user.email='t@t'`), while the delimiter boundary still keeps `scott@things.io`
  # from matching. Assignment contexts:
  #   -c user.email=<fx> | -c user.name=t | GIT_{AUTHOR,COMMITTER}_{EMAIL,NAME}=… | --author=…<fx>
  local fx="$FIXTURE_EMAIL_ALT"
  printf '%s' "$cmd" | grep -Eq -- \
    "(-c[[:space:]]+user\.email=['\"]?(${fx})['\"]?([^A-Za-z0-9._%+-]|$))|(GIT_(AUTHOR|COMMITTER)_EMAIL=['\"]?(${fx})['\"]?([^A-Za-z0-9._%+-]|$))|(--author[=[:space:]][^;&|]*<(${fx})>)|(-c[[:space:]]+user\.name=['\"]?t['\"]?([^A-Za-z0-9._%+-]|$))|(GIT_(AUTHOR|COMMITTER)_NAME=['\"]?t['\"]?([^A-Za-z0-9._%+-]|$))" \
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
#
# Canonical-form coverage (P102 hardening): the verb is caught whether spelled bare
# (`reposix init`), path-suffixed (`/usr/bin/reposix init`, `./target/debug/reposix init`),
# or via cargo (`cargo run -p reposix-cli -- init`). The `([^[:space:]]*/)?` optional path
# prefix absorbs an absolute/relative binary path; the cargo branch matches
# `cargo run … -- <verb>`. sim-seed is caught bare or path-suffixed.
guard_leaf_setup_location() {
  local verb=""
  if   at_command_position '([^[:space:]]*/)?reposix[[:space:]]+init';   then verb="reposix init"
  elif at_command_position '([^[:space:]]*/)?reposix[[:space:]]+attach'; then verb="reposix attach"
  elif at_command_position '([^[:space:]]*/)?reposix[[:space:]]+sync';   then verb="reposix sync"
  elif at_command_position 'cargo[[:space:]]+run[^;&|]*--[[:space:]]+init';   then verb="cargo run -- init"
  elif at_command_position 'cargo[[:space:]]+run[^;&|]*--[[:space:]]+attach'; then verb="cargo run -- attach"
  elif at_command_position 'cargo[[:space:]]+run[^;&|]*--[[:space:]]+sync';   then verb="cargo run -- sync"
  elif at_command_position '([^[:space:]]*/)?reposix-sim';               then verb="sim-seed (reposix-sim)"
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
emit_allow
