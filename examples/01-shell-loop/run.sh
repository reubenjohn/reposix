#!/usr/bin/env bash
# 01-shell-loop -- find an open issue, append a review note, push.
# Pure dark-factory: only `reposix init` is reposix-specific. Everything
# else is git/POSIX an agent already knows.
set -euo pipefail

WORK="${WORK:-/tmp/reposix-example-01}"
SIM_URL="${SIM_URL:-http://127.0.0.1:7878}"
export REPOSIX_ALLOWED_ORIGINS="${REPOSIX_ALLOWED_ORIGINS:-${SIM_URL}}"

# Allow running with target/debug binaries that are not on PATH.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
if [[ -x "${WORKSPACE_ROOT}/target/debug/reposix" ]]; then
    export PATH="${WORKSPACE_ROOT}/target/debug:${PATH}"
fi

# Sanity check: the sim must be reachable.
if ! curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null; then
    echo "FAIL: sim not reachable at ${SIM_URL}." >&2
    echo "Start it: reposix-sim --bind 127.0.0.1:7878 --seed-file crates/reposix-sim/fixtures/seed.json --ephemeral" >&2
    exit 1
fi

rm -rf "$WORK"
mkdir -p "$(dirname "$WORK")"

# 1. Bootstrap the partial-clone working tree.
reposix init sim::demo "$WORK"

# 2. From here on it is pure git/POSIX. The helper writes refs to
#    `refs/reposix/origin/*`; we check that out into a local `main`.
cd "$WORK"
git fetch origin 2>&1 || true   # writes refs/reposix/origin/main; trailing fatal is harmless
git checkout -q -B main refs/reposix/origin/main
git -c user.email=example@reposix.dev -c user.name='reposix-example' \
    commit --allow-empty -m 'baseline' >/dev/null 2>&1 || true

# 3. Triage: pick the first open issue. Substitute any `grep -r` predicate
#    here -- this is the loop an agent runs over a full backlog.
issue="$(grep -lr '^status: open' . --include='*.md' | sort | head -1)"
if [[ -z "$issue" ]]; then
    echo 'no open issues found in seed -- nothing to do' >&2
    exit 0
fi
echo "triaging: $issue"

# 4. Append a review-comment block.
cat >>"$issue" <<EOF

## Comment from shell-loop example
Reviewed by reposix shell-loop example at $(date -Iseconds).
EOF

# 5. Stage, commit, push.
git add "$issue"
git -c user.email=example@reposix.dev -c user.name='reposix-example' \
    commit -m "review: $(basename "$issue")"
git push origin main

echo
echo 'Done. Inspect the audit log with:'
echo '  sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \'
echo '    "SELECT ts, op, reason FROM audit_events_cache ORDER BY id DESC LIMIT 5"'
