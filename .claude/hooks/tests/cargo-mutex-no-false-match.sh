#!/usr/bin/env bash
# Regression guard for .claude/hooks/cargo-mutex.sh (P120 CLOSE Wave A).
#
# BUG (fixed): the prior liveness check used `pgrep -f 'cargo (…)|rustc '`,
# which matches the FULL command line of ANY process. A shell wait-loop, an
# editor, or a `pgrep`/`grep` whose argv merely CONTAINED `rustc …/target/debug`
# (or `cargo build`) FALSE-MATCHED — the hook exit-2'd and BLOCKED every
# subsequent cargo/rustc Bash machine-wide until that unrelated process died.
# That actually deadlocked a live session for ~180k tokens (executed evidence,
# SURPRISES-INTAKE). The fix matches on the process COMM (`pgrep -x`), which a
# shell/editor/grep can never satisfy.
#
# THIS TEST spawns exactly such a decoy — a `bash` process whose command line
# contains the poison `rustc /home/x/target/debug/foo` — and asserts the hook
# now ALLOWS (exit 0 + allow-decision JSON) a genuine `cargo` command. It fails
# LOUD if the hook regresses to argv-matching (it would exit 2 / BLOCK).
#
# A positive "a real SECOND build BLOCKS" test is intentionally OMITTED: comm
# cannot be spoofed without spawning a real `cargo`/`rustc`, which would itself
# trip this repo's machine-wide one-cargo rule. The block path is covered by the
# OOM doctrine (crates/CLAUDE.md § Build memory budget) + the unchanged exit-2
# contract the hook still enforces.
set -eu

HOOK="$(cd "$(dirname "$0")/.." && pwd)/cargo-mutex.sh"
[ -x "$HOOK" ] || { echo "FAIL: hook not found or not executable at $HOOK"; exit 1; }

# If a REAL cargo/rustc build is already running (e.g. a background
# rust-analyzer), the hook is SUPPOSED to block — the no-false-match assertion
# can't be isolated. Skip rather than flake. In a clean CI window this runs
# fully.
if pgrep -x 'cargo|cargo-nextest|rustc|cross|clippy-driver' >/dev/null 2>&1; then
  echo "SKIP: a real cargo/rustc build is running machine-wide; rerun this test in an idle window."
  exit 0
fi

POISON='rustc /home/x/target/debug/foo'
# The decoy's COMM is `bash`; its FULL command line contains the exact poison
# shape the OLD `pgrep -f` false-matched. `-x` (comm match) must ignore it.
# NB: the `; :` compound command is load-bearing — a single simple `-c` command
# is exec-optimized by bash into `sleep` (dropping this argv, and the poison with
# it); a compound command keeps `bash` resident with its full argv intact.
bash -c "sleep 30 ; : # $POISON" &
DECOY_PID=$!
cleanup() { kill "$DECOY_PID" 2>/dev/null || true; }
trap cleanup EXIT

# Wait for the decoy to appear in the process table (its argv carries the poison).
for _ in 1 2 3 4 5 6 7 8 9 10; do
  pgrep -f "$POISON" >/dev/null 2>&1 && break
  sleep 0.1
done

# Sanity: the poison IS live in some process's argv — i.e. the exact input the
# OLD pattern would have false-matched. If this fails, the test itself is broken.
if ! pgrep -f "$POISON" >/dev/null 2>&1; then
  echo "FAIL: decoy poison never became visible in the process table — test is broken."
  exit 1
fi

# Drive the hook with a genuine cargo command payload; it must ALLOW (exit 0).
payload='{"tool_input":{"command":"cargo check -p reposix-core"}}'
errfile="$(mktemp)"
set +e
out="$(printf '%s' "$payload" | "$HOOK" 2>"$errfile")"
rc=$?
set -e
err="$(cat "$errfile")"; rm -f "$errfile"

if [ "$rc" -ne 0 ]; then
  echo "FAIL: hook exit $rc (expected 0/ALLOW). A decoy whose COMM is 'bash' — not a"
  echo "      build tool — must NOT block. The hook appears to have regressed to argv"
  echo "      matching (pgrep -f). stderr: $err"
  exit 1
fi
if ! printf '%s' "$out" | grep -q '"permissionDecision":"allow"'; then
  echo "FAIL: hook exited 0 but did not emit the allow-decision JSON on the"
  echo "      no-concurrent-build path. stdout: $out"
  exit 1
fi

echo "PASS: cargo-mutex ALLOWS a real 'cargo check' while a decoy 'bash' process's argv"
echo "      mentions 'rustc /home/x/target/debug/foo' — no argv false-match (pgrep -x comm)."
