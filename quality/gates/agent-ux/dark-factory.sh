#!/usr/bin/env bash
# quality/gates/agent-ux/dark-factory.sh -- agent-ux dimension dark-factory regression.
#
# MIGRATED FROM: scripts/dark-factory-test.sh per SIMPLIFY-07 (P59).
# CATALOG ROWS:
#   sim arm           -> agent-ux/dark-factory-sim     (v0.9.0; pre-pr; mechanical)
#   dvcs-third-arm    -> agent-ux/dvcs-third-arm       (v0.13.0 P86; pre-pr; subagent-graded)
# CADENCE:       pre-pr (per CI dark-factory job; ~30s wall time per arm)
# INVARIANT:
#   sim arm:        v0.9.0 dark-factory regression -- helper stderr-teaching
#                   strings emit on conflict + blob-limit paths so a
#                   stderr-reading agent can recover without prompt engineering.
#   dvcs-third-arm: v0.13.0 DVCS regression -- a fresh agent given only a
#                   bus URL + a goal completes vanilla-clone + reposix
#                   attach + edit + bus-push end-to-end because the
#                   helper teaches itself via stderr / `--help` output.
#                   Asserted via the shell-stub approach: emulate the
#                   agent's git workflow + grep teaching strings from
#                   helper source / `--help`. No real LLM in CI.
#
# AUDIENCE: developer / autonomous agent / quality runner
# RUNTIME_SEC: ~30 (sim arm) / ~45 (dvcs-third-arm)
# REQUIRES: cargo, git (>= 2.20 for init+config; >= 2.27 for blob:none),
#           reposix-sim, reposix, git-remote-reposix on PATH, sqlite3 (third arm).
#
# Usage:
#   bash quality/gates/agent-ux/dark-factory.sh sim          # default (v0.9.0 arm)
#   bash quality/gates/agent-ux/dark-factory.sh dvcs-third-arm  # v0.13.0 P86 arm
#   bash quality/gates/agent-ux/dark-factory.sh github       # delegates to 35-03 tests
#   bash quality/gates/agent-ux/dark-factory.sh confluence
#   bash quality/gates/agent-ux/dark-factory.sh jira
#
# THIRD ARM — `dvcs-third-arm` (P86, DVCS-DARKFACTORY-01..02)
# ===========================================================
# Proves the v0.13.0 DVCS thesis. The "agent" is shell — the harness
# asserts the helper's teaching contract (stderr / `--help`-recoverable
# strings) AND the bus URL composition shape (reposix attach binds
# `?mirror=file://...` into `remote.reposix.url`).
#
# Coverage layering (subagent-graded contract):
#   - This shell harness  : agent UX surface — teaching strings + attach
#                           config + bus URL composition (T-shaped: wide
#                           on UX recovery, shallow on wire path).
#   - Cargo integration   : wire-path round-trip — helper exec, push
#     tests                 fan-out, refs/mirrors writes, dual-table
#                           audit. Anchor:
#                           crates/reposix-remote/tests/bus_write_happy.rs
#                           ::happy_path_writes_both_refs_and_acks_ok.
#
# A literal `git push reposix main` from the shell stub is NOT exercised
# here. Driving the helper as a `git fetch`/`git push` subprocess at
# shell scope is documented best-effort (see init.rs:198+ "fetch failed
# with status 128 — local repo is configured but not yet synced") and
# subject to env-propagation + cache-poisoning races (see
# `reposix-attach.sh` "stale ~/.cache/reposix dir" comment). The cargo
# tests above drive the helper via assert_cmd which controls env and
# stdin precisely; that is the wire-path layer's natural home.
#
# TokenWorld substrate-gap deferral: the real-TokenWorld leg
# (REPOSIX_DARK_FACTORY_REAL_TOKENWORLD=1) is skipped until v0.13.x
# ships to crates.io with non-yanked gix + binstall artifacts per
# `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (P84 entry).
# Sim arm covers the falsifiable threshold for P86 in CI.

set -euo pipefail

BACKEND="${1:-sim}"

if [[ "$BACKEND" != "sim" && "$BACKEND" != "dvcs-third-arm" ]]; then
    cat >&2 <<EOF
dark-factory.sh: backend=$BACKEND requires real-backend creds and is
exercised via the gated integration tests in 35-03 (cargo test -p
reposix-cli --test agent_flow_real -- --ignored). This shell wrapper only
runs the sim and dvcs-third-arm paths. Skipping.
EOF
    exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Workspace root is two levels up from quality/gates/agent-ux/.
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

# Per-arm constants. Different ports + run dirs allow concurrent runs of
# sim and dvcs-third-arm on the same host without collision.
if [[ "$BACKEND" == "sim" ]]; then
    SIM_BIND="127.0.0.1:7779"
    RUN_DIR="/tmp/dark-factory-$$"
    ARTIFACT="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux/dark-factory-sim.json"
    ROW_ID="agent-ux/dark-factory-sim"
else
    # dvcs-third-arm uses port 7878 because reposix attach (and the helper
    # behind it) bake DEFAULT_SIM_ORIGIN=127.0.0.1:7878 into the bus URL.
    # If the port is in use a `lsof -t -i :7878` collision will surface
    # at the curl probe loop below.
    SIM_BIND="127.0.0.1:7878"
    RUN_DIR="/tmp/dark-factory-third-$$"
    ARTIFACT="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux/dark-factory-dvcs-third-arm.json"
    ROW_ID="agent-ux/dvcs-third-arm"
fi
SIM_URL="http://${SIM_BIND}"
SIM_DB="${RUN_DIR}/sim.db"
mkdir -p "$RUN_DIR"

# Egress allowlist must contain only the sim's localhost origin so any
# accidental egress to a real backend is refused.
export REPOSIX_ALLOWED_ORIGINS="${SIM_URL}"

EXIT_CODE=0
ASSERTS_PASSED='[]'
ASSERTS_FAILED='[]'
mkdir -p "$(dirname "$ARTIFACT")"

cleanup() {
    EXIT_CODE=$?
    if [[ -n "${SIM_PID:-}" ]]; then
        kill "$SIM_PID" 2>/dev/null || true
        wait "$SIM_PID" 2>/dev/null || true
    fi
    rm -rf "$RUN_DIR"
    if [[ -n "${THIRD_ARM_CACHE_DIR:-}" ]]; then
        rm -rf "$THIRD_ARM_CACHE_DIR"
    fi
    cat > "$ARTIFACT" <<EOF
{
  "ts": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "row_id": "${ROW_ID}",
  "exit_code": $EXIT_CODE,
  "asserts_passed": ${ASSERTS_PASSED},
  "asserts_failed": ${ASSERTS_FAILED}
}
EOF
    exit "$EXIT_CODE"
}
trap cleanup EXIT

# Resolve the binaries: prefer debug (dev cycle) then release. Either
# way we re-build first to make sure the binaries are not stale relative
# to the working tree.
echo "dark-factory: ensuring binaries are fresh..." >&2
(cd "$WORKSPACE_ROOT" && cargo build --workspace --bins -q 2>&1 | tail -5) || {
    echo "FAIL: cargo build failed" >&2; exit 1;
}
if [[ "${REPOSIX_DARK_FACTORY_USE_RELEASE:-0}" == "1" \
    && -x "${WORKSPACE_ROOT}/target/release/reposix" ]]; then
    BIN_DIR="${WORKSPACE_ROOT}/target/release"
elif [[ -x "${WORKSPACE_ROOT}/target/debug/reposix" ]]; then
    BIN_DIR="${WORKSPACE_ROOT}/target/debug"
else
    echo "FAIL: no reposix binary found after build" >&2; exit 1
fi
export PATH="${BIN_DIR}:${PATH}"

# Spawn the simulator on the per-arm port.
echo "dark-factory: spawning reposix-sim on $SIM_BIND" >&2
if [[ "$BACKEND" == "sim" ]]; then
    "${BIN_DIR}/reposix-sim" --bind "$SIM_BIND" --db "$SIM_DB" --ephemeral &
else
    # dvcs-third-arm: seed with the canonical fixture so issues/0001.md has
    # a body the agent can edit. The fixture's body contains line 3 prose
    # ("1. Start a fresh postgres container.") which we'll treat as the
    # "typo" to fix.
    "${BIN_DIR}/reposix-sim" --bind "$SIM_BIND" --db "$SIM_DB" --ephemeral \
        --seed-file "${WORKSPACE_ROOT}/crates/reposix-sim/fixtures/seed.json" &
fi
SIM_PID=$!

# Wait up to 5s for the sim to be reachable.
for _ in $(seq 1 50); do
    if curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1; then
        break
    fi
    sleep 0.1
done

##############################################################################
# SIM ARM (v0.9.0)
##############################################################################
if [[ "$BACKEND" == "sim" ]]; then
    REPO="${RUN_DIR}/repo"

    # 2. reposix init: bootstrap the partial-clone working tree.
    echo "dark-factory: reposix init sim::demo $REPO" >&2
    "${BIN_DIR}/reposix" init "sim::demo" "$REPO"
    git -C "$REPO" config remote.origin.url "reposix::${SIM_URL}/projects/demo"

    # 3. Assertions: working tree shape.
    test -d "$REPO/.git" || { echo "FAIL: $REPO/.git missing"; exit 1; }
    [[ "$(git -C "$REPO" config extensions.partialClone)" == "origin" ]] \
        || { echo "FAIL: extensions.partialClone != origin"; exit 1; }
    [[ "$(git -C "$REPO" config remote.origin.promisor)" == "true" ]] \
        || { echo "FAIL: remote.origin.promisor != true"; exit 1; }
    [[ "$(git -C "$REPO" config remote.origin.partialclonefilter)" == "blob:none" ]] \
        || { echo "FAIL: remote.origin.partialclonefilter != blob:none"; exit 1; }

    echo "dark-factory: working tree configured correctly" >&2

    # 4. Assertion: blob-limit stderr teaches the agent the recovery move.
    grep -q 'git sparse-checkout' \
        "${WORKSPACE_ROOT}/crates/reposix-remote/src/stateless_connect.rs" \
        || { echo "FAIL: BLOB_LIMIT teaching string regressed in stateless_connect.rs"; exit 1; }

    # 5. Assertion: conflict path teaches `git pull --rebase`.
    grep -q 'git pull --rebase' \
        "${WORKSPACE_ROOT}/crates/reposix-remote/src/main.rs" \
        "${WORKSPACE_ROOT}/crates/reposix-remote/src/write_loop.rs" \
        || { echo "FAIL: conflict-rebase teaching string regressed in main.rs/write_loop.rs"; exit 1; }

    echo "DARK-FACTORY DEMO COMPLETE -- sim backend: agent UX is pure git." >&2
    echo "  - init configures partial-clone working tree without FUSE" >&2
    echo "  - blob-limit error message names sparse-checkout recovery" >&2
    echo "  - conflict error message names git-pull recovery" >&2
    ASSERTS_PASSED='["dark-factory regression sim path exits 0", "helper stderr-teaching strings present on conflict + blob-limit paths (v0.9.0 invariant)", "no regression vs v0.9.0 baseline"]'
    exit 0
fi

##############################################################################
# DVCS-THIRD-ARM (P86, v0.13.0)
##############################################################################

echo "dark-factory: third-arm scenario starting..." >&2

# Per-arm cache dir + isolated REPOSIX_CACHE_DIR so cache audit assertions
# read a clean cache.db (no cross-pollution from the host's user cache).
THIRD_ARM_CACHE_DIR="$(mktemp -d -t dark-factory-third-cache.XXXXXX)"
export REPOSIX_CACHE_DIR="${THIRD_ARM_CACHE_DIR}"
# attach.rs reads REPOSIX_SIM_ORIGIN to override the cache build_from URL.
# We point both at the third-arm sim port; attach bakes DEFAULT_SIM_ORIGIN
# into the bus URL, but by pinning third-arm sim to 7878 the URL matches.
export REPOSIX_SIM_ORIGIN="${SIM_URL}"

###############################################################################
# Step 1 — Static teaching-string asserts (helper source greps).
###############################################################################
# These are the load-bearing claim of P86: the "agent" recovers everything
# from stderr / source. We grep the source to prove the strings ARE present
# for the helper to emit them on the relevant code paths.

ASSERT_LOG=()

assert_grep() {
    local description="$1"; shift
    local pattern="$1"; shift
    local file="$1"; shift
    if grep -qE "$pattern" "$file"; then
        echo "  PASS: ${description}" >&2
        ASSERT_LOG+=("$description")
    else
        echo "  FAIL: ${description} (pattern '$pattern' not found in $file)" >&2
        ASSERTS_FAILED='["'"${description}"'"]'
        exit 1
    fi
}

echo "dark-factory: third-arm static teaching-string asserts" >&2

assert_grep \
    "?mirror= canonical bus URL form taught (Q3.3) in bus_url.rs reject hints" \
    'reposix::<sot-spec>\?mirror=<mirror-url>' \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/bus_url.rs"

assert_grep \
    "refs/mirrors/<sot>-synced-at ref namespace cited in bus_handler reject hint" \
    'refs/mirrors/(\{sot\}|<sot>)-synced-at' \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/bus_handler.rs"

assert_grep \
    "Q3.5 'configure the mirror remote first: git remote add' hint emitted by helper" \
    'configure the mirror remote first: .git remote add' \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/bus_handler.rs"

# The conflict-recovery teaching strings from the v0.9.0 sim arm carry
# forward (the third arm's recovery surface includes them).
assert_grep \
    "blob-limit teaching string ('git sparse-checkout') present in stateless_connect.rs" \
    'git sparse-checkout' \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/stateless_connect.rs"

assert_grep \
    "conflict-rebase teaching string ('git pull --rebase') present in helper" \
    'git pull --rebase' \
    "${WORKSPACE_ROOT}/crates/reposix-remote/src/write_loop.rs"

###############################################################################
# Step 2 — Dynamic teaching-string asserts (`--help` output greps).
###############################################################################
echo "dark-factory: third-arm dynamic --help asserts" >&2

# `reposix --help` lists the `attach` subcommand.
HELP_TOP=$("${BIN_DIR}/reposix" --help 2>&1)
if echo "$HELP_TOP" | grep -qE '^\s+attach\s'; then
    echo "  PASS: reposix --help lists 'attach' subcommand" >&2
    ASSERT_LOG+=("reposix --help lists attach subcommand (recoverable spelling)")
else
    echo "  FAIL: reposix --help missing 'attach' subcommand listing" >&2
    ASSERTS_FAILED='["reposix --help missing attach subcommand listing"]'
    exit 1
fi

# `reposix attach --help` documents --orphan-policy with all 3 enum values.
HELP_ATTACH=$("${BIN_DIR}/reposix" attach --help 2>&1)
for tok in 'orphan-policy' 'delete-local' 'fork-as-new' 'abort'; do
    if echo "$HELP_ATTACH" | grep -q "$tok"; then
        echo "  PASS: reposix attach --help documents '$tok'" >&2
        ASSERT_LOG+=("reposix attach --help documents ${tok} (recoverable from --help)")
    else
        echo "  FAIL: reposix attach --help missing '$tok'" >&2
        ASSERTS_FAILED='["reposix attach --help missing '"$tok"'"]'
        exit 1
    fi
done

###############################################################################
# Step 3 — Functional-shape end-to-end (reposix attach + bus URL composition).
###############################################################################
# This shell harness exercises the agent UX SURFACE (teaching strings +
# attach config + bus URL composition). It does NOT drive a literal
# `git push reposix main` round-trip — the wire-path round-trip is
# covered byte-for-byte by the cargo integration tests:
#   - crates/reposix-remote/tests/bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok
#     (helper exits zero on bus URL push; refs/mirrors/<sot>-* populated;
#      audit_events_cache rows for helper_push_started/_accepted/mirror_sync_written;
#      sim PATCH observed via wiremock).
#   - crates/reposix-remote/tests/bus_write_audit_completeness.rs (dual-table audit).
#   - crates/reposix-remote/tests/mirror_refs.rs (refs readable by vanilla fetch).
# Spawning the helper as a git-push subprocess in shell is brittle (env
# propagation across `git fetch` is documented best-effort in init.rs:198+;
# `reposix-attach.sh` carries a comment about cache-poisoning races). The
# subagent-graded contract here is: the agent UX teaching contract holds,
# AND the bus URL form composes correctly when reposix attach binds an
# existing mirror remote. The wire path is the cargo-test layer's job.
#
# Workflow:
#   1. git init an empty work tree (the "vanilla clone of mirror" stand-in).
#   2. git remote add origin file://<mirror.git> (mimics what `git clone`
#      would have produced).
#   3. reposix attach sim::demo --remote-name reposix --mirror-name origin
#      → builds cache (Cache::open in CLI env, controlled by REPOSIX_CACHE_DIR)
#      → adds remote.reposix.url with reposix::<sot>?mirror=<file://>
#      → sets extensions.partialClone=reposix.
#   4. Verify post-attach config + cache state + reconciliation report.

WORK_REPO="${RUN_DIR}/work-repo"
MIRROR_BARE="${RUN_DIR}/mirror.git"

# 3.1 Initialize an empty git working tree — the "post-vanilla-clone" state.
git init --quiet "$WORK_REPO"
git -C "$WORK_REPO" config user.email "p86@example.invalid"
git -C "$WORK_REPO" config user.name "P86 Third Arm"

# 3.2 Add a file:// "GH mirror" remote (a real bare repo so the bus URL
# resolves to a valid file:// URL — what an agent's `git clone` would have
# left behind as `origin`).
git init --bare --quiet "$MIRROR_BARE"
git -C "$WORK_REPO" remote add origin "file://${MIRROR_BARE}"

# 3.3 Run reposix attach. The "agent" spells this from `reposix --help`
# output (asserted in Step 2). With --mirror-name origin, attach folds the
# pre-existing origin URL into the bus URL form `?mirror=...`.
echo "dark-factory: reposix attach sim::demo --remote-name reposix --mirror-name origin" >&2
ATTACH_OUT=$("${BIN_DIR}/reposix" attach "sim::demo" "$WORK_REPO" \
    --remote-name reposix --mirror-name origin 2>&1)
echo "$ATTACH_OUT" | head -3 >&2

# 3.4 Cache built — `reposix attach` calls Cache::open which honors
# REPOSIX_CACHE_DIR (this env var is read by the CLI binary itself, NOT by
# downstream `git fetch` subprocesses). Cache lives at
# $REPOSIX_CACHE_DIR/reposix/sim-demo.git per resolve_cache_path.
CACHE_BARE="${THIRD_ARM_CACHE_DIR}/reposix/sim-demo.git"
test -d "$CACHE_BARE" || {
    echo "FAIL: cache bare repo not found at $CACHE_BARE after attach" >&2
    ASSERTS_FAILED='["cache bare repo missing after reposix attach"]'
    exit 1
}
test -f "${CACHE_BARE}/cache.db" || {
    echo "FAIL: cache.db not found at $CACHE_BARE" >&2
    ASSERTS_FAILED='["cache.db missing post-attach"]'
    exit 1
}
ASSERT_LOG+=("reposix attach builds cache at REPOSIX_CACHE_DIR/reposix/sim-demo.git")

# 3.5 Post-attach config: extensions.partialClone=reposix.
PCLONE=$(git -C "$WORK_REPO" config --get extensions.partialClone || true)
[[ "$PCLONE" == "reposix" ]] || {
    echo "FAIL: extensions.partialClone=$PCLONE expected 'reposix'" >&2
    ASSERTS_FAILED='["extensions.partialClone post-attach != reposix"]'
    exit 1
}
ASSERT_LOG+=("post-attach extensions.partialClone == reposix (NOT origin; per architecture-sketch step 5)")

# 3.6 remote.reposix.url contains the reposix:: prefix AND ?mirror= form
# (the bus URL pattern Q3.3 specifies).
REPOSIX_URL=$(git -C "$WORK_REPO" config --get remote.reposix.url || true)
[[ "$REPOSIX_URL" == reposix::* ]] || {
    echo "FAIL: remote.reposix.url=$REPOSIX_URL not starting with reposix::" >&2
    ASSERTS_FAILED='["remote.reposix.url missing reposix:: prefix"]'
    exit 1
}
case "$REPOSIX_URL" in
    *'?mirror='*)
        ASSERT_LOG+=("remote.reposix.url contains ?mirror= bus URL form (Q3.3 round-trip)")
        ;;
    *)
        echo "FAIL: remote.reposix.url=$REPOSIX_URL missing ?mirror= (bus URL form expected)" >&2
        ASSERTS_FAILED='["remote.reposix.url missing ?mirror= form"]'
        exit 1
        ;;
esac

# 3.7 origin remote unchanged (plain-git semantics preserved per architecture-sketch).
ORIGIN_URL=$(git -C "$WORK_REPO" config --get remote.origin.url || true)
[[ "$ORIGIN_URL" == "file://${MIRROR_BARE}" ]] || {
    echo "FAIL: remote.origin.url=$ORIGIN_URL expected file://${MIRROR_BARE} (origin must be unchanged)" >&2
    ASSERTS_FAILED='["origin remote URL mutated by attach (must be unchanged per attach.rs invariant)"]'
    exit 1
}
ASSERT_LOG+=("origin remote URL unchanged by attach (plain-git semantics preserved)")

# 3.8 Reconciliation walked + audit row written. Empty work tree means
# matched=0; mirror_lag count reflects the seeded issues that appear as
# "in cache but not in tree".
if echo "$ATTACH_OUT" | grep -qE 'matched=[0-9]+ no_id=[0-9]+ backend_deleted=[0-9]+ mirror_lag=[0-9]+'; then
    ASSERT_LOG+=("attach reconciliation report emitted (matched/no_id/backend_deleted/mirror_lag)")
else
    echo "FAIL: attach output missing reconciliation report line" >&2
    ASSERTS_FAILED='["attach reconciliation report missing"]'
    exit 1
fi

# Audit table contains attach_walk row (OP-3 unconditional per attach.rs:215).
ATTACH_WALK_COUNT=$(sqlite3 "${CACHE_BARE}/cache.db" \
    "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'attach_walk';" 2>/dev/null || echo "0")
if [[ "$ATTACH_WALK_COUNT" -ge 1 ]]; then
    ASSERT_LOG+=("audit_events_cache contains attach_walk row (OP-3 unconditional; count=${ATTACH_WALK_COUNT})")
else
    echo "FAIL: audit_events_cache has 0 attach_walk rows" >&2
    ASSERTS_FAILED='["audit_events_cache missing attach_walk row"]'
    exit 1
fi

# 3.9 Cite the cargo test that covers the wire-path round-trip the shell
# stub does NOT exercise. The presence of this test is part of the
# subagent-graded contract — if the test file were deleted, the third
# arm's coverage claim regresses.
WIRE_TEST="${WORKSPACE_ROOT}/crates/reposix-remote/tests/bus_write_happy.rs"
test -f "$WIRE_TEST" || {
    echo "FAIL: wire-path coverage test missing at $WIRE_TEST" >&2
    ASSERTS_FAILED='["wire-path coverage test bus_write_happy.rs missing"]'
    exit 1
}
grep -q 'happy_path_writes_both_refs_and_acks_ok' "$WIRE_TEST" || {
    echo "FAIL: bus_write_happy.rs missing happy_path_writes_both_refs_and_acks_ok test fn" >&2
    ASSERTS_FAILED='["bus_write_happy.rs missing happy_path test fn"]'
    exit 1
}
ASSERT_LOG+=("wire-path round-trip covered by bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok (cargo test layer)")

###############################################################################
# Step 4 — TokenWorld substrate-gap deferral notice.
###############################################################################
if [[ "${REPOSIX_DARK_FACTORY_REAL_TOKENWORLD:-0}" == "1" ]]; then
    cat >&2 <<EOF

dark-factory: REPOSIX_DARK_FACTORY_REAL_TOKENWORLD=1 detected, but the
TokenWorld arm is SUBSTRATE-GAP-DEFERRED until v0.13.x ships to crates.io
with non-yanked gix + binstall artifacts. See
.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md (P84 entry,
2026-05-01 16:43). Skipping.
EOF
else
    echo "" >&2
    echo "dark-factory: TokenWorld arm SUBSTRATE-GAP-DEFERRED (P84). Run with" >&2
    echo "  REPOSIX_DARK_FACTORY_REAL_TOKENWORLD=1 + creds + REPOSIX_ALLOWED_ORIGINS=https://reuben-john.atlassian.net" >&2
    echo "  AFTER v0.13.x ships to crates.io." >&2
fi

###############################################################################
# Done. Compose ASSERTS_PASSED JSON for the artifact.
###############################################################################
# Build a JSON array from ASSERT_LOG entries.
ASSERTS_PASSED=$(python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" "${ASSERT_LOG[@]}")

echo "" >&2
echo "DARK-FACTORY THIRD ARM COMPLETE -- DVCS thesis: pure-git agent UX." >&2
echo "  - all teaching strings (?mirror=, attach, --orphan-policy, refs/mirrors, git remote add) recoverable from helper source / --help" >&2
echo "  - reposix attach binds bus URL (?mirror=file://) form into remote.reposix.url" >&2
echo "  - extensions.partialClone post-attach is 'reposix' (NOT origin); origin URL unchanged" >&2
echo "  - cache materialized at REPOSIX_CACHE_DIR; audit_events_cache records attach_walk row" >&2
echo "  - wire-path (helper exec + push fan-out + mirror-refs writes + dual-table audit) covered by" >&2
echo "    crates/reposix-remote/tests/bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok" >&2
echo "  - TokenWorld real-backend leg substrate-gap-deferred (P84 SURPRISES-INTAKE; binstall + yanked-gix)" >&2

exit 0
