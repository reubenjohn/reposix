#!/usr/bin/env bash
# .claude/hooks/cargo-mutex.sh — PreToolUse/Bash: one cargo invocation machine-wide.
# no canonical home under quality/gates/ — wired via .claude/settings.json
set -eu
payload=$(cat)
cmd=$(printf '%s' "$payload" | python3 -c 'import sys,json;print(json.load(sys.stdin).get("tool_input",{}).get("command",""))' 2>/dev/null || true)
case "$cmd" in
  *cargo\ *|*"cargo"|*cross\ *|*rustc\ *) : ;;   # gate build tools only
  *) exit 0 ;;
esac
# Robust liveness: is a cargo/rustc BUILD already running? (crash-safe vs a stale PID lock)
# Machine-wide check (pgrep, not repo-scoped) — the OOM risk spans any repo on this VM.
#
# Match on the process EXECUTABLE NAME (comm, via `pgrep -x`), NEVER the full argv.
# The prior `pgrep -f 'cargo (…)|rustc '` matched the FULL command line of ANY
# process, so a shell wait-loop, an editor, or a `pgrep`/`grep`/`rg` whose argv
# merely CONTAINED `cargo build` or `rustc …/target/debug` FALSE-MATCHED — the
# hook then exit-2'd and BLOCKED every subsequent cargo/rustc Bash machine-wide
# until that unrelated process died. That actually deadlocked a live session for
# ~180k tokens (see .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md,
# executed evidence). `-x` (exact whole-comm match) structurally cannot false-
# match a shell/editor/grep: their comm is `bash`/`zsh`/`nvim`/`pgrep`, never a
# build tool, regardless of what their argv mentions.
#
# The comm alternation covers every build-phase process, so the OOM protection
# (Hard rule 1, crates/CLAUDE.md § Build memory budget — VM OOM-crashed 3x from
# parallel workspace builds) is NOT weakened: a genuine second concurrent build
# always shows one of these comms — `cargo` (check/build/test/run + `cargo
# clippy`, which execs `clippy-driver`), `cargo-nextest`, `rustc`, `cross`,
# `clippy-driver`. comm cannot be spoofed without spawning a real build process,
# so no decoy can slip a second build past this gate. (A positive "second build
# blocks" regression test is impractical to fake for exactly that reason — the
# no-false-match test in tests/ covers the fixed bug; this OOM contract is the
# doctrine that keeps the alternation exhaustive.)
#
# Belt-and-suspenders: exclude the hook's own PID and its parent from the match,
# even though a bash/sh/node comm can never satisfy the alternation above.
exclude="^($$|${PPID:-0})$"
if pgrep -x 'cargo|cargo-nextest|rustc|cross|clippy-driver' 2>/dev/null \
     | grep -Ev "$exclude" | grep -q .; then
  echo "BLOCKED: a cargo/rustc build is already running machine-wide. reposix Build memory budget (crates/CLAUDE.md): exactly ONE cargo invocation at a time — the VM OOM-crashed 3x from parallel workspace builds. Wait for the running build to finish, then retry." >&2
  exit 2
fi
printf '%s\n' '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","additionalContext":"cargo OK (no concurrent build). Reminder: prefer -p <crate> over --workspace; CARGO_BUILD_JOBS=2; cargo nextest for full-workspace tests."}}'
exit 0
