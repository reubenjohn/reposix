#!/usr/bin/env bash
# Probe: does B's stale-base git push get rejected end-to-end, or lost-update?
set -uo pipefail
ROOT=/home/reuben/workspace/reposix; BIN="$ROOT/target/debug"; PORT=7985
WORK=$(mktemp -d /tmp/rbf-lr-03.XXXXXX)
export REPOSIX_SIM_ORIGIN="http://127.0.0.1:$PORT" REPOSIX_CACHE_DIR="$WORK/cache"
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:$PORT" PATH="$BIN:$PATH"
export GIT_AUTHOR_NAME="R" GIT_AUTHOR_EMAIL="r@rbf.test" GIT_COMMITTER_NAME="R" GIT_COMMITTER_EMAIL="r@rbf.test"
mkdir -p "$WORK/cache"; echo "WORK=$WORK"
"$BIN/reposix-sim" --bind "127.0.0.1:$PORT" --db "$WORK/sim.db" >"$WORK/sim.log" 2>&1 &
SIM_PID=$!; trap 'kill $SIM_PID 2>/dev/null' EXIT
for i in $(seq 1 50); do curl -sf "http://127.0.0.1:$PORT/healthz" >/dev/null 2>&1 && break; sleep 0.2; done
"$BIN/reposix" init sim::demo "$WORK/A" >/dev/null 2>&1
"$BIN/reposix" init sim::demo "$WORK/B" >/dev/null 2>&1
( cd "$WORK/A" && git checkout -q -B main refs/reposix/origin/main )
( cd "$WORK/B" && git checkout -q -B main refs/reposix/origin/main )
echo "SoT issue1 version BEFORE:"; curl -s "http://127.0.0.1:$PORT/projects/demo/issues/1" | grep -o '"version":[0-9]*'
( cd "$WORK/A" && sed -i 's/^title: .*/title: A-CHANGED-TITLE/' issues/1.md && git commit -qam "A edit title" && git push origin main >/dev/null 2>&1 && echo "A pushed OK" )
echo "SoT after A: "; curl -s "http://127.0.0.1:$PORT/projects/demo/issues/1" | grep -o '"title":"[^"]*"\|"version":[0-9]*'
echo "--- B (stale base) edits body, pushes; watch for 'fetch first' ---"
( cd "$WORK/B" && sed -i 's/^title: .*/title: B-CHANGED-TITLE/' issues/1.md && git commit -qam "B edit title" && git push origin main 2>&1 | grep -iE "fetch first|main ->|reject|error" | sed 's/^/B> /' )
echo "SoT FINAL (whose title won? A lost update if B):"; curl -s "http://127.0.0.1:$PORT/projects/demo/issues/1" | grep -o '"title":"[^"]*"\|"version":[0-9]*'
echo "DONE"
