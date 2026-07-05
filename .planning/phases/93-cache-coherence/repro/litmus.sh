#!/usr/bin/env bash
# Runs INSIDE the git-2.54 container. Reproduces the DP-2 / D-P92-03 delta-sync
# suspicion: two-writer conflict, writer B `git pull --rebase` triggers
#   fatal: git upload-pack: not our ref <oid>
#   could not fetch <oid> from promisor remote
#
# MODE selects the timing/cursor variation:
#   gap2s      — 2s sleep between B-init and A-push (cursor second < write second) → expect CLEAN
#   same-second— tight back-to-back (init+push likely share a wall-clock second) → race, may bug
#   pin-cursor — deterministically place B's cursor in A's write-second → expect BUG every time
#
# Binaries are bind-mounted read-only at /bin-ro. The sim runs on the HOST
# (127.0.0.1:7878) reached via `--network host`.
set -u

MODE="${1:?usage: litmus.sh <gap2s|same-second|pin-cursor>}"
export PATH="/bin-ro:$PATH"                 # git finds `git-remote-reposix` here
export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,http://localhost:*"

REPOSIX=/bin-ro/reposix
CACHE_A=/work/cacheA
CACHE_B=/work/cacheB
TREE_A=/work/A
TREE_B=/work/B
B_DB="$CACHE_B/reposix/sim-demo.git/cache.db"

rm -rf "$CACHE_A" "$CACHE_B" "$TREE_A" "$TREE_B"
git config --global user.email w@example.com
git config --global user.name  "Writer"
git config --global init.defaultBranch main
git config --global advice.detachedHead false

banner() { echo; echo "############### $* ###############"; }
issue1() { curl -s "http://127.0.0.1:7878/projects/demo/issues/1"; }
cursorB() { sqlite3 "$B_DB" "SELECT value FROM meta WHERE key='last_fetched_at'" 2>/dev/null; }

echo "MODE=$MODE   git=$(git --version)   date=$(date -u +%Y-%m-%dT%H:%M:%S.%NZ)"

banner "reposix init A (cache A) + checkout"
REPOSIX_CACHE_DIR="$CACHE_A" "$REPOSIX" init sim::demo "$TREE_A" 2>&1 | sed 's/^/[initA] /'
( cd "$TREE_A" && REPOSIX_CACHE_DIR="$CACHE_A" git checkout -B main refs/reposix/origin/main 2>&1 | sed 's/^/[coA] /' )

banner "reposix init B (cache B) + checkout"
REPOSIX_CACHE_DIR="$CACHE_B" "$REPOSIX" init sim::demo "$TREE_B" 2>&1 | sed 's/^/[initB] /'
( cd "$TREE_B" && REPOSIX_CACHE_DIR="$CACHE_B" git checkout -B main refs/reposix/origin/main 2>&1 | sed 's/^/[coB] /' )
echo "B cursor right after init: $(cursorB)"

if [ "$MODE" = "gap2s" ]; then
  banner "sleep 2s (push lands in a LATER wall-clock second than B's cursor)"
  sleep 2
fi

banner "A edits issues/1.md, commit, git push origin main  (expect exit 0)"
( cd "$TREE_A"
  printf '\nEdited by writer A at %s\n' "$(date -u +%H:%M:%S.%N)" >> issues/1.md
  git add issues/1.md
  git commit -q -m "A edits issue 1"
  REPOSIX_CACHE_DIR="$CACHE_A" git push origin main 2>&1 | sed 's/^/[pushA] /'
  echo "==> A push exit: ${PIPESTATUS[0]}"
)

banner "sim state after A push"
echo "issue 1 (backend): $(issue1 | sed -E 's/("body":")[^"]*/\1.../')"
A_UPD=$(issue1 | sed -n 's/.*"updated_at":"\([^"]*\)".*/\1/p')
echo "issue 1 updated_at = $A_UPD"
echo "B cursor (before any pin) = $(cursorB)"

if [ "$MODE" = "pin-cursor" ]; then
  banner "PIN B cursor into A's write-second (deterministic trigger)"
  # A_UPD is seconds-precision RFC3339 (e.g. 2026-07-05T12:34:56Z). Place B's
  # cursor at the SAME second, .999999999 — exactly what happens naturally when
  # init+push race inside one second. sim truncates the cursor to seconds and
  # uses strict `updated_at > cursor`, so A's write (updated_at == that second)
  # is dropped from list_changed_since. Guard: never write an invalid cursor.
  case "$A_UPD" in
    ????-??-??T??:??:??Z)
      PIN="${A_UPD%Z}.999999999+00:00"
      sqlite3 "$B_DB" "UPDATE meta SET value='$PIN' WHERE key='last_fetched_at'"
      echo "B cursor (pinned)  = $(cursorB)"
      echo "A write updated_at = $A_UPD   (sim truncates cursor to seconds: ${A_UPD})"
      echo "predicate: updated_at('$A_UPD') > trunc(cursor)='$A_UPD'  =>  FALSE  =>  list_changed_since drops it"
      ;;
    *) echo "SKIP pin: A_UPD not seconds-RFC3339: '$A_UPD'"; exit 3 ;;
  esac
fi

banner "B edits issues/1.md from STALE base, commit, git push origin main  (expect exit 1 conflict)"
( cd "$TREE_B"
  printf '\nEdited by writer B at %s\n' "$(date -u +%H:%M:%S.%N)" >> issues/1.md
  git add issues/1.md
  git commit -q -m "B edits issue 1"
  REPOSIX_CACHE_DIR="$CACHE_B" git push origin main 2>&1 | sed 's/^/[pushB] /'
  echo "==> B push exit: ${PIPESTATUS[0]}"
)

banner "B: git pull --rebase origin main   <<< THE MOMENT — watch for 'not our ref' >>>"
( cd "$TREE_B"
  REPOSIX_CACHE_DIR="$CACHE_B" git pull --rebase origin main 2>&1 | sed 's/^/[pullB] /'
  echo "==> B pull --rebase exit: ${PIPESTATUS[0]}"
)

banner "post-pull B cursor + delta-sync commit message (shows 'N changed (of M)')"
echo "B cursor (after pull) = $(cursorB)"
echo "B cache HEAD log:"
git --git-dir="$CACHE_B/reposix/sim-demo.git" log --oneline -3 2>&1 | sed 's/^/[cacheBlog] /'

banner "END MODE=$MODE"
