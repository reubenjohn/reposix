#!/usr/bin/env bash
# quality/gates/agent-ux/dark-factory/dvcs-third-arm.sh -- v0.13.0 P86
# DVCS third-arm. Invoked via `dark-factory.sh dvcs-third-arm`.
#
# CATALOG ROW: agent-ux/dvcs-third-arm
# INVARIANT: a fresh agent given only a bus URL + goal completes
# vanilla-clone + reposix attach + edit via teaching strings the helper
# emits over stderr / `--help`. Verifies attach + agent-UX teaching-string
# surface + reconciliation (RBF-A-05); does NOT drive `git push reposix
# main` (wire-path round-trip: bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok
# -- driving the helper as a git subprocess in shell is brittle, init.rs:198+).
# No real LLM in CI; kind is mechanical (P90 90-03). TokenWorld real-backend
# leg (REPOSIX_DARK_FACTORY_REAL_TOKENWORLD=1) substrate-gap-deferred until
# v0.13.x ships to crates.io with non-yanked gix + binstall (SURPRISES-INTAKE
# P84). Fixture-writer + assert helpers (write_issue, populate_reconciliation_fixture,
# assert_reconciliation_counts_honest, assert_duplicate_id_hard_aborts) live in
# the sourced reconciliation-fixture.sh (file-size-limits factoring, RBF-A-05).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"

# Port 7878: reposix attach bakes DEFAULT_SIM_ORIGIN=127.0.0.1:7878 into the
# bus URL; collision surfaces at lib.sh's curl probe loop.
SIM_BIND="127.0.0.1:7878"
RUN_DIR="/tmp/dark-factory-third-$$"
ARTIFACT="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux/dark-factory-dvcs-third-arm.json"
ROW_ID="agent-ux/dvcs-third-arm"
SIM_URL="http://${SIM_BIND}"
SIM_DB="${RUN_DIR}/sim.db"
mkdir -p "$RUN_DIR"

export REPOSIX_ALLOWED_ORIGINS="${SIM_URL}"

# shellcheck disable=SC1091
source "${SCRIPT_DIR}/lib.sh"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/reconciliation-fixture.sh"

build_and_resolve_bins
spawn_sim seeded

echo "dark-factory: third-arm scenario starting..." >&2

# Per-arm cache dir + isolated REPOSIX_CACHE_DIR for a clean cache.db.
THIRD_ARM_CACHE_DIR="$(mktemp -d -t dark-factory-third-cache.XXXXXX)"
export THIRD_ARM_CACHE_DIR
export REPOSIX_CACHE_DIR="${THIRD_ARM_CACHE_DIR}"
export REPOSIX_SIM_ORIGIN="${SIM_URL}"

ASSERT_LOG=()

# Step 1 — Static teaching-string asserts (assert_grep in lib.sh).
echo "dark-factory: third-arm static teaching-string asserts" >&2
SRC="${WORKSPACE_ROOT}/crates/reposix-remote/src"

assert_grep \
    "?mirror= canonical bus URL form taught (Q3.3) in bus_url.rs reject hints" \
    'reposix::<sot-spec>\?mirror=<mirror-url>' "${SRC}/bus_url.rs"
assert_grep \
    "refs/mirrors/<sot>-synced-at ref namespace cited in bus_handler reject hint" \
    'refs/mirrors/(\{sot\}|<sot>)-synced-at' "${SRC}/bus_handler.rs"
assert_grep \
    "Q3.5 'configure the mirror remote first: git remote add' hint emitted by helper" \
    'configure the mirror remote first: .git remote add' "${SRC}/bus_handler.rs"
# v0.9.0-arm conflict-recovery teaching strings carry forward.
assert_grep \
    "blob-limit teaching string ('git sparse-checkout') present in stateless_connect.rs" \
    'git sparse-checkout' "${SRC}/stateless_connect.rs"
assert_grep \
    "conflict-rebase teaching string ('git pull --rebase') present in helper" \
    'git pull --rebase' "${SRC}/write_loop.rs"

# Step 2 — Dynamic teaching-string asserts (`--help` output greps).
echo "dark-factory: third-arm dynamic --help asserts" >&2

HELP_TOP=$("${BIN_DIR}/reposix" --help 2>&1)
if echo "$HELP_TOP" | grep -qE '^\s+attach\s'; then
    echo "  PASS: reposix --help lists 'attach' subcommand" >&2
    ASSERT_LOG+=("reposix --help lists attach subcommand (recoverable spelling)")
else
    fail_with "reposix --help missing attach subcommand listing"
fi

# `reposix attach --help` documents --orphan-policy with all 3 enum values.
HELP_ATTACH=$("${BIN_DIR}/reposix" attach --help 2>&1)
for tok in 'orphan-policy' 'delete-local' 'fork-as-new' 'abort'; do
    if echo "$HELP_ATTACH" | grep -q "$tok"; then
        echo "  PASS: reposix attach --help documents '$tok'" >&2
        ASSERT_LOG+=("reposix attach --help documents ${tok} (recoverable from --help)")
    else
        fail_with "reposix attach --help missing $tok"
    fi
done

# Step 3 — End-to-end attach + bus URL composition (UX surface only).
WORK_REPO="${RUN_DIR}/work-repo"
MIRROR_BARE="${RUN_DIR}/mirror.git"

# 3.1-3.2 NON-EMPTY work tree + populated bare "GH mirror" remote
# (post-vanilla-clone). RBF-A-05 (p86 F13): an empty tree can only ever
# report an all-zeros reconciliation summary, which is vacuous — seed
# real content shaped to the 5 reconciliation.rs cases (module doc :4-15).
git init --quiet "$WORK_REPO"
git -C "$WORK_REPO" config user.email "p86@example.invalid"
git -C "$WORK_REPO" config user.name "P86 Third Arm"
# Repoint the still-unborn HEAD to refs/heads/main (no commits exist yet,
# so this is a safe rename regardless of init.defaultBranch / git version).
git -C "$WORK_REPO" symbolic-ref HEAD refs/heads/main
populate_reconciliation_fixture "$WORK_REPO" "$SIM_URL"

git init --bare --quiet "$MIRROR_BARE"
git -C "$WORK_REPO" remote add origin "file://${MIRROR_BARE}"
# Vanilla-clone baseline: push the populated tree into the bare mirror.
git -C "$WORK_REPO" push --quiet origin main

# 3.3 Run reposix attach — folds origin URL into bus URL `?mirror=...`.
echo "dark-factory: reposix attach sim::demo --remote-name reposix --mirror-name origin" >&2
ATTACH_OUT=$("${BIN_DIR}/reposix" attach "sim::demo" "$WORK_REPO" \
    --remote-name reposix --mirror-name origin 2>&1)
echo "$ATTACH_OUT" | head -6 >&2

# 3.4 Cache at $REPOSIX_CACHE_DIR/reposix/sim-demo.git per resolve_cache_path.
CACHE_BARE="${THIRD_ARM_CACHE_DIR}/reposix/sim-demo.git"
test -d "$CACHE_BARE" || fail_with "cache bare repo missing after reposix attach" "$CACHE_BARE"
test -f "${CACHE_BARE}/cache.db" || fail_with "cache.db missing post-attach" "$CACHE_BARE"
ASSERT_LOG+=("reposix attach builds cache at REPOSIX_CACHE_DIR/reposix/sim-demo.git")

# 3.5 extensions.partialClone=reposix (NOT origin; per architecture-sketch step 5).
PCLONE=$(git -C "$WORK_REPO" config --get extensions.partialClone || true)
[[ "$PCLONE" == "reposix" ]] || fail_with "extensions.partialClone post-attach != reposix" "got '$PCLONE'"
ASSERT_LOG+=("post-attach extensions.partialClone == reposix (NOT origin; per architecture-sketch step 5)")

# 3.6 remote.reposix.url has reposix:: prefix AND ?mirror= form (Q3.3).
REPOSIX_URL=$(git -C "$WORK_REPO" config --get remote.reposix.url || true)
[[ "$REPOSIX_URL" == reposix::* ]] || fail_with "remote.reposix.url missing reposix:: prefix" "got '$REPOSIX_URL'"
case "$REPOSIX_URL" in
    *'?mirror='*) ASSERT_LOG+=("remote.reposix.url contains ?mirror= bus URL form (Q3.3 round-trip)") ;;
    *) fail_with "remote.reposix.url missing ?mirror= form" "got '$REPOSIX_URL'" ;;
esac

# 3.7 origin remote unchanged (plain-git semantics; attach.rs invariant).
ORIGIN_URL=$(git -C "$WORK_REPO" config --get remote.origin.url || true)
[[ "$ORIGIN_URL" == "file://${MIRROR_BARE}" ]] \
    || fail_with "origin remote URL mutated by attach (must be unchanged per attach.rs invariant)" "got '$ORIGIN_URL'"
ASSERT_LOG+=("origin remote URL unchanged by attach (plain-git semantics preserved)")

# 3.8 Reconciliation report + audit_events_cache attach_walk row (OP-3 per
# attach.rs:215). RBF-A-05: no longer a shape-only regex — the populated
# tree (3.1-3.2) makes an all-zeros report impossible; the helper requires
# matched>=1 AND >=3 of 4 non-zero, sets MATCHED/etc for the summary below.
assert_reconciliation_counts_honest "$ATTACH_OUT"

ATTACH_WALK_COUNT=$(sqlite3 "${CACHE_BARE}/cache.db" \
    "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'attach_walk';" 2>/dev/null || echo "0")
if [[ "$ATTACH_WALK_COUNT" -ge 1 ]]; then
    ASSERT_LOG+=("audit_events_cache contains attach_walk row (OP-3 unconditional; count=${ATTACH_WALK_COUNT})")
else
    fail_with "audit_events_cache missing attach_walk row"
fi

# 3.9 Wire-path coverage citation.
WIRE_TEST="${WORKSPACE_ROOT}/crates/reposix-remote/tests/bus_write_happy.rs"
test -f "$WIRE_TEST" || fail_with "wire-path coverage test bus_write_happy.rs missing" "$WIRE_TEST"
grep -q 'happy_path_writes_both_refs_and_acks_ok' "$WIRE_TEST" \
    || fail_with "bus_write_happy.rs missing happy_path test fn"
ASSERT_LOG+=("wire-path round-trip covered by bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok (cargo test layer)")

# 3.10 Duplicate-id (case 4) hard-abort — isolated sub-invocation so a
# deliberately-aborting attach doesn't corrupt the case-1/2/3/5 report above
# (reconciliation.rs :136-144 detects duplicates before any backend/SQLite work).
echo "dark-factory: third-arm duplicate-id (case 4) hard-abort sub-invocation" >&2
assert_duplicate_id_hard_aborts "$RUN_DIR" "$SIM_URL" "$BIN_DIR"

# Step 4 — TokenWorld substrate-gap deferral notice.
echo "" >&2
echo "dark-factory: TokenWorld arm SUBSTRATE-GAP-DEFERRED (P84; non-yanked gix + binstall pending v0.13.x crates.io). Set REPOSIX_DARK_FACTORY_REAL_TOKENWORLD=1 + creds after v0.13.x ships." >&2

ASSERTS_PASSED=$(python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" "${ASSERT_LOG[@]}")

echo "" >&2
echo "DARK-FACTORY THIRD ARM COMPLETE -- DVCS thesis: pure-git agent UX." >&2
echo "  - teaching strings recoverable from source / --help (?mirror=, attach, --orphan-policy, refs/mirrors, git remote add)" >&2
echo "  - reposix attach binds ?mirror=file:// into remote.reposix.url; extensions.partialClone=reposix; origin unchanged" >&2
echo "  - cache materialized at REPOSIX_CACHE_DIR; audit_events_cache has attach_walk row" >&2
echo "  - reconciliation NON-VACUOUS: matched=${MATCHED} no_id=${NO_ID} backend_deleted=${BACKEND_DELETED} mirror_lag=${MIRROR_LAG}; duplicate-id hard-aborts (RBF-A-05, closes p86 F13)" >&2
echo "  - wire-path covered by bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok; TokenWorld leg substrate-gap-deferred (P84)" >&2

exit 0
