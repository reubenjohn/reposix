#!/usr/bin/env bash
# 04-conflict-resolve -- two agents touch the same issue. Second sees
# `[remote rejected] main -> main (fetch first)` and recovers by
# rebasing onto the new tip and re-pushing.
set -euo pipefail

WORK_A="${WORK_A:-/tmp/reposix-example-04-A}"
WORK_B="${WORK_B:-/tmp/reposix-example-04-B}"
SIM_URL="${SIM_URL:-http://127.0.0.1:7878}"
export REPOSIX_ALLOWED_ORIGINS="${REPOSIX_ALLOWED_ORIGINS:-${SIM_URL}}"

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
    local dir="$1"
    rm -rf "$dir"
    reposix init sim::demo "$dir" >/dev/null
    git -C "$dir" fetch origin 2>/dev/null || true
    git -C "$dir" checkout -q -B main refs/reposix/origin/main
    git -C "$dir" config user.email "$2@reposix.dev"
    git -C "$dir" config user.name "$2"
}

# Pick a known-existing issue. The first .md file at the root of either
# working tree works; we list one tree to find a target id.
echo '[1/5] bootstrap two working trees'
bootstrap "$WORK_A" agent-A
bootstrap "$WORK_B" agent-B
target="$(ls "$WORK_A"/*.md | sort | head -1)"
target_name="$(basename "$target")"
echo "  target: $target_name"

echo
echo '[2/5] agent A: append a note + push'
cat >>"$WORK_A/$target_name" <<EOF

## note from agent-A
appended at $(date -Iseconds)
EOF
git -C "$WORK_A" add "$target_name"
git -C "$WORK_A" commit -q -m 'A: append note'
git -C "$WORK_A" push origin main

echo
echo '[3/5] agent B (stale base): edit a different line + push -- expect rejection'
# B is unaware of A's push. B edits the title line on its still-stale tree.
sed -i 's/^title: \(.*\)$/title: \1 [B]/' "$WORK_B/$target_name"
git -C "$WORK_B" add "$target_name"
git -C "$WORK_B" commit -q -m 'B: tag title'
set +e
push_out="$(git -C "$WORK_B" push origin main 2>&1)"
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
# v0.9.0 detail: each `git fetch` produces a fresh fast-import root commit
# (the helper does not maintain a parent chain across fetches), so a plain
# `git pull --rebase` fails with "new tip does not contain <old>". The
# manual recovery is: drop the stale tracking ref, re-fetch, rebase onto
# the new tip. In v0.10+ the tutorial-friendly `git pull --rebase` will
# Just Work; for now the recovery is three commands.
git -C "$WORK_B" update-ref -d refs/reposix/origin/main
git -C "$WORK_B" fetch origin 2>/dev/null || true
git -C "$WORK_B" rebase --onto refs/reposix/origin/main HEAD~1 2>&1 | tail -5

echo
echo '[5/5] agent B retries the push'
git -C "$WORK_B" push origin main

echo
echo 'Done. Inspect the rejection + accept rows:'
echo '  sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \'
echo '    "SELECT id, ts, op, reason FROM audit_events_cache ORDER BY id"'
