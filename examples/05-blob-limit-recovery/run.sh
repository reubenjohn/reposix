#!/usr/bin/env bash
# 05-blob-limit-recovery -- the dark-factory teaching mechanism for the
# blob-limit guardrail. See RUN.md for the v0.9.0 caveat: the literal
# stderr trigger does not fire on the canonical fast-import flow today;
# this script demonstrates the recovery move (`git sparse-checkout set`)
# pre-emptively to show what an agent would do once it reads the
# teaching message.
set -euo pipefail

WORK="${WORK:-/tmp/reposix-example-05}"
SIM_URL="${SIM_URL:-http://127.0.0.1:7878}"
export REPOSIX_ALLOWED_ORIGINS="${REPOSIX_ALLOWED_ORIGINS:-${SIM_URL}}"
export REPOSIX_BLOB_LIMIT=3

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
if [[ -x "${WORKSPACE_ROOT}/target/debug/reposix" ]]; then
    export PATH="${WORKSPACE_ROOT}/target/debug:${PATH}"
fi

if ! curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null; then
    echo "FAIL: sim not reachable at ${SIM_URL}." >&2
    exit 1
fi

echo '[1/4] the literal teaching string an agent would read on stderr:'
echo
grep -A1 'BLOB_LIMIT_EXCEEDED_FMT: &str =' \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/stateless_connect.rs" \
    | sed -n 's/^[[:space:]]*"\(.*\)";$/    \1/p'
echo
echo '    -- The literal `git sparse-checkout` token is the contract.'

echo
echo '[2/4] bootstrap with REPOSIX_BLOB_LIMIT=3'
rm -rf "$WORK"
reposix init sim::demo "$WORK" >/dev/null
git -C "$WORK" fetch origin 2>/dev/null || true

echo
echo '[3/4] dark-factory recovery: narrow scope with git sparse-checkout, then check out'
# The demo seed has 6 issues. Without sparse-checkout, materialising the
# full tree on a real fetch with limit=3 would refuse. The recovery is to
# only materialize a subset.
git -C "$WORK" sparse-checkout init --no-cone
git -C "$WORK" sparse-checkout set '0001.md' '0002.md' '0003.md'
git -C "$WORK" checkout -q -B main refs/reposix/origin/main

echo "Checked out $(ls "$WORK"/*.md 2>/dev/null | wc -l) of 6 issue files (sparse-checkout matched only 3 paths):"
ls "$WORK"/*.md

echo
echo '[4/4] widening scope is the same recovery, repeated'
git -C "$WORK" sparse-checkout set '0001.md' '0002.md' '0003.md' '0004.md'
git -C "$WORK" checkout -q -B main refs/reposix/origin/main
echo "After widening to 4 paths:"
ls "$WORK"/*.md

echo
echo 'Done. Inspect blob-limit hits with:'
echo '  sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \'
echo '    "SELECT id, ts, op, bytes, reason FROM audit_events_cache \'
echo '     WHERE op = '\''blob_limit_exceeded'\'' ORDER BY id DESC"'
