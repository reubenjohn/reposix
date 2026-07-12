#!/usr/bin/env bash
# 04-conflict-resolve -- two agents touch the same issue. Second sees
# `[remote rejected] main -> main (fetch first)` and recovers by
# rebasing onto the new tip and re-pushing.
set -euo pipefail

WORK_A="${WORK_A:-/tmp/reposix-example-04-A}"
WORK_B="${WORK_B:-/tmp/reposix-example-04-B}"
SIM_URL="${SIM_URL:-http://127.0.0.1:7878}"
export REPOSIX_ALLOWED_ORIGINS="${REPOSIX_ALLOWED_ORIGINS:-${SIM_URL}}"

# Two INDEPENDENT agents must have INDEPENDENT reposix caches. The cache holds
# the per-record base version the helper compares against the backend at push
# time; if A and B shared one cache, A's push would advance the shared base
# and B's stale push would (wrongly) look up-to-date -- masking the very
# conflict this example exists to demonstrate. Separate caches model two
# agents on two machines. (Cache root override: REPOSIX_CACHE_DIR.)
CACHE_A="${CACHE_A:-${WORK_A}.cache}"
CACHE_B="${CACHE_B:-${WORK_B}.cache}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
if [[ -x "${WORKSPACE_ROOT}/target/debug/reposix" ]]; then
    export PATH="${WORKSPACE_ROOT}/target/debug:${PATH}"
fi

if ! curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null; then
    echo "FAIL: sim not reachable at ${SIM_URL}." >&2
    exit 1
fi

bootstrap() {
    local dir="$1" name="$2" cache="$3"
    rm -rf "$dir" "$cache"
    REPOSIX_CACHE_DIR="$cache" reposix init sim::demo "$dir" >/dev/null
    REPOSIX_CACHE_DIR="$cache" git -C "$dir" fetch origin 2>/dev/null || true
    REPOSIX_CACHE_DIR="$cache" git -C "$dir" checkout -q -B main refs/reposix/origin/main
    git -C "$dir" config user.email "$name@reposix.dev"
    git -C "$dir" config user.name "$name"
}

# Pick a known-existing issue. Records live under the canonical `issues/`
# bucket (QL-001: issues/<id>.md); we list one tree to find a target id.
echo '[1/5] bootstrap two working trees'
bootstrap "$WORK_A" agent-A "$CACHE_A"
bootstrap "$WORK_B" agent-B "$CACHE_B"
target="$(ls "$WORK_A"/issues/*.md | sort | head -1)"
target_name="issues/$(basename "$target")"
echo "  target: $target_name"

echo
echo '[2/5] agent A: append a note + push'
cat >>"$WORK_A/$target_name" <<EOF

## note from agent-A
appended at $(date -Iseconds)
EOF
git -C "$WORK_A" add "$target_name"
git -C "$WORK_A" commit -q -m 'A: append note'
REPOSIX_CACHE_DIR="$CACHE_A" git -C "$WORK_A" push origin main

echo
echo '[3/5] agent B (stale base): edit a different line + push -- expect rejection'
# B is unaware of A's push. B edits the title line on its still-stale tree.
sed -i 's/^title: \(.*\)$/title: \1 [B]/' "$WORK_B/$target_name"
git -C "$WORK_B" add "$target_name"
git -C "$WORK_B" commit -q -m 'B: tag title'
set +e
push_out="$(REPOSIX_CACHE_DIR="$CACHE_B" git -C "$WORK_B" push origin main 2>&1)"
push_rc=$?
set -e
echo "$push_out"
if [[ $push_rc -eq 0 ]]; then
    echo "FAIL: expected B's push to be rejected with 'fetch first'" >&2
    exit 1
fi
echo "  (push exited non-zero; agent reads the stderr)"

echo
echo '[4/5] agent B reads stderr -> rebase onto the new tip'
# Each `git fetch` produces a fresh fast-import root commit (the helper does
# not maintain a parent chain across fetches), so the re-fetch is a
# non-fast-forward on the helper's fast-import staging ref
# `refs/reposix-import/main` -- git refuses it with "new tip does not contain
# <old>" and fast-import aborts. The manual recovery is: drop BOTH the staging
# ref (`refs/reposix-import/main`) AND the tracking ref
# (`refs/reposix/origin/main`), re-fetch cleanly, then rebase onto the new tip.
# In a future release the tutorial-friendly `git pull --rebase` will Just Work.
git -C "$WORK_B" update-ref -d refs/reposix-import/main 2>/dev/null || true
git -C "$WORK_B" update-ref -d refs/reposix/origin/main 2>/dev/null || true
REPOSIX_CACHE_DIR="$CACHE_B" git -C "$WORK_B" fetch origin 2>/dev/null || true
REPOSIX_CACHE_DIR="$CACHE_B" git -C "$WORK_B" rebase --onto refs/reposix/origin/main HEAD~1 2>&1 | tail -5

echo
echo '[5/5] agent B retries the push'
REPOSIX_CACHE_DIR="$CACHE_B" git -C "$WORK_B" push origin main

echo
echo 'Done. Inspect the rejection + accept rows in agent-B'\''s cache:'
echo "  sqlite3 ${CACHE_B}/reposix/sim-demo.git/cache.db \\"
echo '    "SELECT id, ts, op, reason FROM audit_events_cache ORDER BY id"'
