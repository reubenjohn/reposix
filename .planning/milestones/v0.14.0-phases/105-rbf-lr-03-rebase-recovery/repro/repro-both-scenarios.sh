#!/usr/bin/env bash
# RBF-LR-03 empirical reproduction — full two-scenario flow.
set -uo pipefail
ROOT=/home/reuben/workspace/reposix
BIN="$ROOT/target/debug"
PORT=7981
WORK=$(mktemp -d /tmp/rbf-lr-03.XXXXXX)
export REPOSIX_SIM_ORIGIN="http://127.0.0.1:$PORT"
export REPOSIX_CACHE_DIR="$WORK/cache"
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:$PORT"
export PATH="$BIN:$PATH"
export GIT_AUTHOR_NAME="RBF Repro" GIT_AUTHOR_EMAIL="repro@rbf.test"
export GIT_COMMITTER_NAME="RBF Repro" GIT_COMMITTER_EMAIL="repro@rbf.test"
mkdir -p "$WORK/cache"
echo "WORK=$WORK"

"$BIN/reposix-sim" --bind "127.0.0.1:$PORT" --db "$WORK/sim.db" >"$WORK/sim.log" 2>&1 &
SIM_PID=$!
trap 'kill $SIM_PID 2>/dev/null' EXIT
for i in $(seq 1 50); do curl -sf "http://127.0.0.1:$PORT/healthz" >/dev/null 2>&1 && break; sleep 0.2; done

"$BIN/reposix" init sim::demo "$WORK/A" >"$WORK/initA.log" 2>&1
"$BIN/reposix" init sim::demo "$WORK/B" >"$WORK/initB.log" 2>&1
( cd "$WORK/A" && git checkout -q -B main refs/reposix/origin/main )
( cd "$WORK/B" && git checkout -q -B main refs/reposix/origin/main )
echo "A issues:"; ls "$WORK/A/issues"
FILE="$WORK/A/issues/1.md"; BFILE="$WORK/B/issues/1.md"
[ -f "$FILE" ] || FILE=$(ls "$WORK"/A/issues/*.md | head -1)
BASENAME=$(basename "$FILE"); BFILE="$WORK/B/issues/$BASENAME"

echo "############ SCENARIO A: two git clients drift (push-conflict) ############"
# A edits + push
( cd "$WORK/A" && printf '\nEdit by A\n' >> "issues/$BASENAME" && git add -A && git commit -q -m "A edits $BASENAME" && echo "--- A push ---" && git push origin main 2>&1 | sed 's/^/A> /' )
# B edits SAME issue (stale base) + push -> expect reject
( cd "$WORK/B" && printf '\nEdit by B\n' >> "issues/$BASENAME" && git add -A && git commit -q -m "B edits $BASENAME" && echo "--- B push (expect fetch first) ---" && git push origin main 2>&1 | sed 's/^/B> /' )
# B recovery
echo "--- B: git pull --rebase (recovery step 1) ---"
( cd "$WORK/B" && git pull --rebase origin main 2>&1 | sed 's/^/B> /' )
echo "--- B: git push (recovery step 2) ---"
( cd "$WORK/B" && git push origin main 2>&1 | sed 's/^/B> /' )
echo "SCENARIO A exit summary: B HEAD log:"; git -C "$WORK/B" log --oneline -5 2>&1 | sed 's/^/B> /'

echo "############ SCENARIO B: EXTERNAL REST write moves SoT ############"
# fresh clone C at current state
"$BIN/reposix" init sim::demo "$WORK/C" >"$WORK/initC.log" 2>&1
( cd "$WORK/C" && git checkout -q -B main refs/reposix/origin/main )
# local commit on C
( cd "$WORK/C" && printf '\nEdit by C local\n' >> "issues/$BASENAME" && git add -A && git commit -q -m "C local edit" )
# external REST PATCH (not a git push) moves the SoT
ID="${BASENAME%.md}"
echo "--- external PATCH issue $ID ---"
curl -s -X PATCH "http://127.0.0.1:$PORT/projects/demo/issues/$ID" \
  -H 'content-type: application/json' \
  -d '{"body":"EXTERNALLY EDITED VIA REST\n"}' | head -c 200; echo
# C tries recovery
echo "--- C: git pull --rebase (external-write recovery) ---"
( cd "$WORK/C" && git pull --rebase origin main 2>&1 | sed 's/^/C> /' )
echo "--- C: git push ---"
( cd "$WORK/C" && git push origin main 2>&1 | sed 's/^/C> /' )
echo "SCENARIO B exit summary: C log:"; git -C "$WORK/C" log --oneline -5 2>&1 | sed 's/^/C> /'
echo "DONE. WORK=$WORK"
