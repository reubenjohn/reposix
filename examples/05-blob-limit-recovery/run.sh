#!/usr/bin/env bash
# 05-blob-limit-recovery -- the dark-factory teaching mechanism for the
# blob-limit guardrail, driven END TO END against the real helper.
#
# An agent checks out a backlog larger than its REPOSIX_BLOB_LIMIT. The
# helper REFUSES the blob-materializing fetch with a self-teaching stderr
# error that names the recovery move (`git sparse-checkout set <pathspec>`)
# verbatim, tagged [RPX-0503]. The agent reads that stderr, narrows its
# scope with sparse-checkout, retries, and succeeds -- no prompt
# engineering, no reposix-specific knowledge beyond `reposix init`.
#
# This is the REAL observe-then-recover cycle (P124/SC1, DRAIN-22): the
# script drives the actual BLOB_LIMIT_EXCEEDED_FMT refusal through the
# modern-git (2.34+) stateless-connect protocol-v2 lazy-blob fetch that a
# `git checkout` of refs/reposix/origin/main issues, then recovers. The
# fast-import path bypasses the per-RPC blob-limit check, so it is the
# CHECKOUT's lazy fetch -- not the initial filtered fetch -- that fires it.
set -euo pipefail

WORK="${WORK:-/tmp/reposix-example-05}"
SIM_URL="${SIM_URL:-http://127.0.0.1:7878}"
export REPOSIX_ALLOWED_ORIGINS="${REPOSIX_ALLOWED_ORIGINS:-${SIM_URL}}"
# The demo seed has 6 issues -> 6 blobs. A limit of 3 refuses a full-tree
# checkout and forces the sparse-checkout recovery.
export REPOSIX_BLOB_LIMIT=3

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
if [[ -x "${WORKSPACE_ROOT}/target/debug/reposix" ]]; then
    export PATH="${WORKSPACE_ROOT}/target/debug:${PATH}"
fi

# Sanity check: the sim must be reachable.
if ! curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null; then
    echo "FAIL: sim not reachable at ${SIM_URL}." >&2
    echo "Start it: reposix sim --bind 127.0.0.1:7878 --ephemeral" >&2
    exit 1
fi

CHECKOUT_LOG="$(mktemp "${TMPDIR:-/tmp}/reposix-example-05-checkout.XXXXXX")"
cleanup() { rm -f "$CHECKOUT_LOG"; }
trap cleanup EXIT

echo '[1/4] bootstrap the partial-clone working tree with REPOSIX_BLOB_LIMIT=3'
rm -rf "$WORK"
mkdir -p "$(dirname "$WORK")"
# init runs `git fetch --filter=blob:none origin`: it brings back the tree +
# commit but NO blobs (want_count stays small), so init itself is under the
# limit and succeeds. The blobs stay unmaterialized until a checkout needs them.
reposix init sim::demo "$WORK" >/dev/null

echo
echo '[2/4] check out the full backlog WITHOUT narrowing (drives the real refusal)'
# `git checkout` of refs/reposix/origin/main lazy-fetches all 6 issue blobs in
# ONE protocol-v2 `command=fetch` RPC (want_count=6 > limit=3). The helper
# refuses on stderr BEFORE materializing anything -- this is the runtime error
# an agent actually reads. We EXPECT this to fail; capturing it in an `if`
# condition keeps `set -e` from aborting on the intended non-zero exit.
if git -C "$WORK" checkout -B main refs/reposix/origin/main >"$CHECKOUT_LOG" 2>&1; then
    echo "FAIL: the no-narrow checkout unexpectedly SUCCEEDED -- the blob-limit" >&2
    echo "      guardrail did not fire (REPOSIX_BLOB_LIMIT=${REPOSIX_BLOB_LIMIT})." >&2
    sed 's/^/  captured: /' "$CHECKOUT_LOG" >&2
    exit 1
fi
# Fail loud unless it failed for the RIGHT reason: the self-teaching refusal
# that names the recovery move AND carries its stable [RPX-0503] code.
if ! grep -q 'git sparse-checkout' "$CHECKOUT_LOG" \
   || ! grep -q '\[RPX-0503\]' "$CHECKOUT_LOG"; then
    echo "FAIL: checkout failed but NOT with the blob-limit refusal (no" >&2
    echo "      \`git sparse-checkout\` / [RPX-0503] token in captured stderr):" >&2
    sed 's/^/  captured: /' "$CHECKOUT_LOG" >&2
    exit 1
fi
echo '    the helper refused on stderr (this is the message an agent reads):'
grep -m1 'refusing to fetch' "$CHECKOUT_LOG" | sed 's/^/    /'
# Earned-congruence marker (harvested by container-rehearse.sh; DRAIN-22): the
# REAL runtime refusal was observed with its RPX code + sparse-checkout guidance.
echo "ASSERT-PASS: the helper emitted the BLOB_LIMIT_EXCEEDED_FMT stderr refusal naming git sparse-checkout set with [RPX-0503] on the no-narrow checkout of refs/reposix/origin/main under REPOSIX_BLOB_LIMIT"

echo
echo '[3/4] recover exactly as the stderr taught: narrow scope, then retry'
# The agent read `git sparse-checkout set <pathspec>` off stderr and runs it.
git -C "$WORK" sparse-checkout init --no-cone
# Canonical record paths are issues/<id>.md (QL-001), unpadded + bucketed. The
# leading slash anchors each pattern to the repo root -- the git-recommended
# non-cone form (git warns "pass a leading slash" without it). Three paths ->
# three blobs -> at the limit (3 is NOT > 3), so the retry is allowed.
git -C "$WORK" sparse-checkout set '/issues/1.md' '/issues/2.md' '/issues/3.md'
git -C "$WORK" checkout -B main refs/reposix/origin/main
# Count via a glob array (SC2012-safe: never parse `ls` output). The array holds
# the literal pattern when nothing matches, so guard on the first element existing.
narrowed_files=("$WORK"/issues/*.md)
narrowed_count=0
[[ -e "${narrowed_files[0]}" ]] && narrowed_count=${#narrowed_files[@]}
if [[ "$narrowed_count" -ne 3 ]]; then
    echo "FAIL: expected 3 narrowed issue files after recovery, got ${narrowed_count}" >&2
    exit 1
fi
echo "    retry succeeded -- materialized ${narrowed_count} of 6 issue files (sparse-checkout narrowed the scope):"
ls "$WORK"/issues/*.md | sed 's/^/    /'
# Earned-congruence marker (DRAIN-22): recovery via sparse-checkout succeeded.
echo "ASSERT-PASS: after git sparse-checkout set narrowed the scope, the retried checkout materialized the narrowed issues/*.md record set and completed 0 -- the observe-error-then-recover cycle"

echo
echo '[4/4] widening scope is the same recovery, repeated'
git -C "$WORK" sparse-checkout set '/issues/1.md' '/issues/2.md' '/issues/3.md' '/issues/4.md'
git -C "$WORK" checkout -B main refs/reposix/origin/main
echo "    after widening to 4 paths:"
ls "$WORK"/issues/*.md | sed 's/^/    /'

# Earned-congruence marker (DRAIN-22): the whole observe-then-recover cycle
# completed under `set -euo pipefail`, so the script exits 0.
echo "ASSERT-PASS: bash examples/05-blob-limit-recovery/run.sh drove a REAL runtime blob-limit-exceeded refusal under REPOSIX_BLOB_LIMIT, recovered via sparse-checkout, and exits 0"

echo
echo 'Done. The refusal wrote a blob_limit_exceeded audit row -- inspect it with:'
echo '  sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \'
echo '    "SELECT id, ts, op, reason FROM audit_events_cache \'
echo '     WHERE op = '\''blob_limit_exceeded'\'' ORDER BY id DESC"'
