#!/usr/bin/env bash
# quality/gates/agent-ux/p94-pagination-prune-completeness-gate.sh
#   -- agent-ux/p94-pagination-prune-completeness-gate
#
# P94 D1 -- pagination-truncation prune-safety (Fork A + Fork B, RATIFIED at
# .planning/CONSULT-DECISIONS.md 2026-07-05 [FABLE] pagination-truncation
# prune-safety fork; NO revert of 272882c). Grades the row's expected.asserts:
#
#   1. A capped-mock BackendConnector returning is_complete=false with records
#      truncated at a cap drives Cache::sync/build_from; after sync, oid_map
#      rows for records BEYOND the cap are NOT deleted (prune-SKIP branch pinned)
#      -- proven by executed regression, not code-read.
#   2. BackendConnector exposes a completeness signal (Listing { records,
#      is_complete } via list_records_complete); BOTH prune_oid_map call sites
#      (builder.rs full-rebuild AND delta) are gated on is_complete == true --
#      neither prunes unconditionally.
#   3. The sim connector reports is_complete=true (default impl), so a COMPLETE
#      listing STILL prunes a genuinely-absent oid_map row (no functional
#      regression of the legitimate prune).
#   4. Fork B: a delete against an already-absent record is an idempotent
#      success (not Err) at the write boundary (unit test).
#   5. 272882c is NOT reverted -- ADR-010 coherence enforcement retained
#      (prune_oid_map DELETE still present; upsert-before-prune in builder.rs).
#
# ONE cargo invocation at a time (crates/CLAUDE.md build-memory budget) -- the
# two `cargo test` calls below run SEQUENTIALLY, never concurrently.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"

ROW_ID="agent-ux/p94-pagination-prune-completeness-gate"
BUILDER="crates/reposix-cache/src/builder.rs"
BACKEND="crates/reposix-core/src/backend.rs"
META="crates/reposix-cache/src/meta.rs"
MAIN="crates/reposix-remote/src/main.rs"
TEST="crates/reposix-cache/tests/pagination_prune_safety.rs"

fail() {
  echo "FAIL (${ROW_ID}): $1" >&2
  exit 1
}

# --- Assert 2a: completeness signal exists on the trait ---------------------
grep -qE 'struct +Listing' "${BACKEND}" \
  || fail "reposix-core backend.rs has no Listing struct (completeness signal)"
grep -qE 'fn +list_records_complete' "${BACKEND}" \
  || fail "BackendConnector has no list_records_complete method"

# --- Assert 3 (part): default impl / sim reports is_complete = true ----------
# The default list_records_complete wraps list_records with is_complete: true,
# and the sim does NOT override it -- so every sim-backed gate keeps pruning.
grep -qE 'is_complete: *true' "${BACKEND}" \
  || fail "backend.rs default list_records_complete does not report is_complete: true"
if grep -qE 'fn +list_records_complete' crates/reposix-core/src/backend/sim.rs; then
  fail "sim overrides list_records_complete -- it must inherit the is_complete=true default"
fi

# --- Assert 2b: BOTH prune_oid_map call sites are GATED, none unconditional --
PRUNE_CALLS=$(grep -cE 'meta::prune_oid_map\(' "${BUILDER}" || true)
[[ "${PRUNE_CALLS}" -eq 2 ]] \
  || fail "expected exactly 2 meta::prune_oid_map call sites in builder.rs, found ${PRUNE_CALLS}"
# Each prune must sit under an is_complete gate. Assert the two gate conditions
# are present (build_from uses `is_complete`, delta uses `all_is_complete`).
grep -qE 'if +is_complete +\{' "${BUILDER}" \
  || fail "build_from prune site is not gated on `if is_complete {`"
grep -qE 'if +all_is_complete +\{' "${BUILDER}" \
  || fail "delta-sync prune site is not gated on `if all_is_complete {`"
# Belt-and-suspenders: no prune_oid_map may be called via the completeness-blind
# list_records; build_from + delta must both source their listing from
# list_records_complete.
LRC_CALLS=$(grep -cE 'list_records_complete\(' "${BUILDER}" || true)
[[ "${LRC_CALLS}" -ge 2 ]] \
  || fail "builder.rs must call list_records_complete at BOTH prune-bearing sites (found ${LRC_CALLS})"

# --- Assert 5: 272882c NOT reverted -----------------------------------------
grep -qE 'DELETE FROM oid_map' "${META}" \
  || fail "meta.rs prune_oid_map DELETE is gone -- 272882c appears reverted (ADR-010 coherence lost)"
# upsert-before-prune: put_oid_mapping must run before prune in build_from so a
# retained superset can never dangle a tree reference.
grep -qE 'put_oid_mapping' "${BUILDER}" \
  || fail "builder.rs no longer upserts oid_map before pruning (ADR-010 invariant-at-source lost)"

# --- Assert 4: Fork B reclassification present -------------------------------
grep -qE 'is_delete_notfound' "${MAIN}" \
  || fail "reposix-remote main.rs has no delete-NotFound reclassification (Fork B)"

# --- Assert 1 + 3 (executed): Fork A capped-mock regression, exit 0 ----------
[[ -f "${TEST}" ]] || fail "${TEST} (capped-mock regression) does not exist"
echo "running: cargo test -p reposix-cache --test pagination_prune_safety" >&2
cargo test -p reposix-cache --test pagination_prune_safety \
  || fail "cargo test -p reposix-cache --test pagination_prune_safety did not exit 0"

# --- Assert 4 (executed): Fork B unit test, exit 0 ---------------------------
echo "running: cargo test -p reposix-remote --bin git-remote-reposix fork_b" >&2
cargo test -p reposix-remote --bin git-remote-reposix fork_b \
  || fail "Fork B unit tests (fork_b_tests) did not exit 0"

echo "PASS (${ROW_ID}): completeness gate on both prune sites (Fork A) + idempotent delete-NotFound (Fork B); 272882c retained; capped-mock regression GREEN"
exit 0
