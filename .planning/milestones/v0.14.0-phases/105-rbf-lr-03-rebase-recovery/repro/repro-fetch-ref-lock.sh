#!/usr/bin/env bash
# RBF-LR-03 SECOND BUG reproduction — the fetch-time double-ref-write ref-lock
# that the 90ddaff parent-chaining fix EXPOSED (it eliminated the `fatal: error
# while running fast-import` abort but uncovered a `cannot lock ref` abort one
# layer up).
#
# ROOT CAUSE: the `import` helper's fast-import stream writes
# `commit refs/reposix/origin/main` DIRECTLY (crates/reposix-remote/src/fast_import.rs:165),
# while the helper ALSO advertises `refspec refs/heads/*:refs/reposix/origin/*`
# (crates/reposix-remote/src/main.rs:193) — so git ALSO applies its own refspec
# update to refs/reposix/origin/main after import. On drift the fast-import
# stream fast-forwards the ref underneath git, then git's post-import ref
# transaction (old-value = the PRE-fetch tip) conflicts with the already-moved
# ref → `error: cannot lock ref 'refs/reposix/origin/main': is at <X> but
# expected <Y>` → `git fetch` (hence `git pull --rebase`) exits non-zero →
# the documented `git pull --rebase && git push` recovery SHORT-CIRCUITS at the
# `&&` and never pushes. A SECOND `git pull --rebase` succeeds (the ref is now
# settled and the no-op guard makes the re-fetch a clean fast-forward), so the
# recovery is possible but is NOT the single documented command.
#
# Filed: .planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md (P105, HIGH).
# Run from anywhere; uses an isolated /tmp workspace and an isolated sim port.
set -uo pipefail
ROOT=/home/reuben/workspace/reposix
BIN="$ROOT/target/debug"
PORT=7987
WORK=$(mktemp -d /tmp/rbf-lr-03-reflock.XXXXXX)
export REPOSIX_SIM_ORIGIN="http://127.0.0.1:$PORT"
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:$PORT"
export PATH="$BIN:$PATH"
export GIT_AUTHOR_NAME="RBF Repro" GIT_AUTHOR_EMAIL="repro@rbf.invalid"
export GIT_COMMITTER_NAME="RBF Repro" GIT_COMMITTER_EMAIL="repro@rbf.invalid"
cd "$WORK"
CACHE_A="$WORK/cacheA"; CACHE_B="$WORK/cacheB"
"$BIN/reposix-sim" --bind "127.0.0.1:$PORT" --db "$WORK/sim.db" >"$WORK/sim.log" 2>&1 &
SIM_PID=$!
trap 'kill $SIM_PID 2>/dev/null' EXIT
for _ in $(seq 1 50); do curl -sf "http://127.0.0.1:$PORT/healthz" >/dev/null 2>&1 && break; sleep 0.2; done

REPOSIX_CACHE_DIR="$CACHE_A" "$BIN/reposix" init sim::demo "$WORK/A" >"$WORK/initA.log" 2>&1
REPOSIX_CACHE_DIR="$CACHE_B" "$BIN/reposix" init sim::demo "$WORK/B" >"$WORK/initB.log" 2>&1
( cd "$WORK/A" && git checkout -q -B main refs/reposix/origin/main )
( cd "$WORK/B" && git checkout -q -B main refs/reposix/origin/main )

echo "### A edits issue1 + pushes (drift source)"
( cd "$WORK/A" && printf '\nEdit by A\n' >> issues/1.md && git add -A && git commit -q -m "A edits issue1" \
  && REPOSIX_CACHE_DIR="$CACHE_A" git push -q origin main 2>/dev/null )
sleep 2

echo "### B has an unpushed local commit on a DIFFERENT record (issue2)"
( cd "$WORK/B" && printf '\nEdit by B\n' >> issues/2.md && git add -A && git commit -q -m "B edits issue2" )

echo "### B runs the DOCUMENTED single-command recovery: git pull --rebase && git push"
( cd "$WORK/B" && REPOSIX_CACHE_DIR="$CACHE_B" git pull --rebase origin main \
    && REPOSIX_CACHE_DIR="$CACHE_B" git push origin main ) > "$WORK/recovery.log" 2>&1
echo "DOCUMENTED_RECOVERY_EXIT=$?  (expected 0; observed 1 — the bug)"
echo "--- recovery.log (notable lines) ---"
grep -iE 'fatal: error while running fast-import|does not contain|cannot lock ref|main -> main' "$WORK/recovery.log" || echo "(none)"
echo "### B's edit landed in the SoT?  (issue2 version stays 1 == LOST until a 2nd pull)"
curl -s "http://127.0.0.1:$PORT/projects/demo/issues/2" \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print('issue2 version =', d['version'])"

echo
echo "### PROOF the recovery IS achievable with a SECOND (undocumented) pull --rebase"
( cd "$WORK/B" && REPOSIX_CACHE_DIR="$CACHE_B" git pull --rebase origin main > "$WORK/pull2.log" 2>&1; echo "SECOND_PULL_EXIT=$?" )
( cd "$WORK/B" && REPOSIX_CACHE_DIR="$CACHE_B" git push origin main > "$WORK/push2.log" 2>&1; echo "PUSH_AFTER_2ND_PULL_EXIT=$?" )
curl -s "http://127.0.0.1:$PORT/projects/demo/issues/2" \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print('issue2 version after 2nd pull =', d['version'], '(2 == B converged)')"
echo "DONE. WORK=$WORK"
