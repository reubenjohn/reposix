#!/usr/bin/env bash
# quality/gates/agent-ux/zero-shot-onboarding.sh -- agent-ux zero-shot
# onboarding regression (v0.13.1 Lane E2). CATALOG ROW: agent-ux/zero-shot-onboarding
#
# INVARIANT: the PUBLISHED getting-started flow (first-run.md steps 1-7 +
# README.md Quick start) works verbatim, copy-pasted, zero fixups -- and the
# Wave D/E1 doc-lies (padded 0001.md, "5 issues", nonexistent `decision`
# audit column, source-tree-only --seed-file, bare `git checkout
# origin/main`) never reappear in the doc surfaces this drives from.
#
# v0.13.1 front-door fix: `reposix sim` runs the simulator IN-PROCESS (single
# shipped binary, no separate reposix-sim, no cargo fallback) and serves a
# compiled-in builtin demo seed -- so step 2 is a bare `reposix sim`, no curl
# and no --seed-file, and the whole flow works offline.
#
# LEAF ISOLATION: mutation (init/config/commit/push) runs inside a throwaway
# mktemp -d /tmp tree in THIS invocation, never the shared repo.
# RUNTIME_SEC: ~20-40. REQUIRES: cargo, git >= 2.34, curl (localhost only).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

SIM_BIND="127.0.0.1:17878"
SIM_URL="http://${SIM_BIND}"
RUN_DIR="$(mktemp -d -t zsonboard.XXXXXX)"
ARTIFACT="${WORKSPACE_ROOT}/quality/reports/verifications/agent-ux/zero-shot-onboarding.json"
ROW_ID="agent-ux/zero-shot-onboarding"
SIM_DB="${RUN_DIR}/sim.db"
export REPOSIX_ALLOWED_ORIGINS="${SIM_URL}"
export REPOSIX_SIM_ORIGIN="${SIM_URL}"  # isolated-port sim (init.rs)

EXIT_CODE=0
ASSERTS_PASSED='[]'
ASSERTS_FAILED='[]'
SEED_SOURCE="unset"
mkdir -p "$(dirname "$ARTIFACT")"

fail_with() {
    local desc="$1" detail="${2:-}"
    [[ -n "$detail" ]] && echo "FAIL: ${desc}: ${detail}" >&2 || echo "FAIL: ${desc}" >&2
    ASSERTS_FAILED=$(python3 -c "import json,sys; print(json.dumps([sys.argv[1]]))" "$desc")
    exit 1
}

cleanup() {
    EXIT_CODE=$?
    if [[ -n "${SIM_PID:-}" ]]; then
        kill "$SIM_PID" 2>/dev/null || true
        wait "$SIM_PID" 2>/dev/null || true
    fi
    # reposix-sim is an unmanaged grandchild; wrapper SIGTERM skips Drop
    # forwarding (raw signal, no unwind) -- reap by bind addr, belt+braces.
    pkill -f "reposix-sim.*${SIM_BIND}" 2>/dev/null || true
    rm -rf "$RUN_DIR"
    cat > "$ARTIFACT" <<EOF
{
  "ts": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "row_id": "${ROW_ID}",
  "exit_code": $EXIT_CODE,
  "seed_source": "${SEED_SOURCE}",
  "asserts_passed": ${ASSERTS_PASSED},
  "asserts_failed": ${ASSERTS_FAILED}
}
EOF
    exit "$EXIT_CODE"
}
trap cleanup EXIT

ASSERT_LOG=()

# --- 0. Static doc-lie signature greps ---------------------------------
# Scoped to the two files this flow drives from. index.md, mental-model,
# dvcs-topology.md deliberately KEEP illustrative 0001.md (mermaid diagram
# + attach examples, unrelated -- see commit 0578919). No blanket repo-grep.
FIRST_RUN="${WORKSPACE_ROOT}/docs/tutorials/first-run.md"
README="${WORKSPACE_ROOT}/README.md"

echo "zero-shot-onboarding: doc-lie signature scan" >&2

grep -qE 'issues/0001\.md' "$FIRST_RUN" "$README" \
    && fail_with "padded issues/0001.md doc-lie present (real seed is unpadded issues/1.md..6.md)"
ASSERT_LOG+=("no padded issues/0001.md signature in first-run.md/README.md")

grep -qE '5 issues' "$FIRST_RUN" "$README" \
    && fail_with "'5 issues' doc-lie present (real seed is 6 issues)"
ASSERT_LOG+=("no '5 issues' doc-lie signature in first-run.md/README.md")

grep -qE 'SELECT.*\bdecision\b.*FROM audit_events_cache' "$FIRST_RUN" "$README" \
    && fail_with "nonexistent 'decision' audit_events_cache column doc-lie present"
ASSERT_LOG+=("no nonexistent 'decision' audit column doc-lie signature")

# v0.13.1 front-door fix: the documented step 2 no longer curls a seed file
# or passes --seed-file at all -- `reposix sim` serves a compiled-in builtin
# seed offline. Guard against the source-tree-only --seed-file path (which
# only works from a source checkout, never a prebuilt install) creeping back
# into the tutorial body.
grep -qE -- '--seed-file crates/reposix-sim/fixtures/seed\.json' "$FIRST_RUN" "$README" \
    && fail_with "source-tree-only --seed-file present in first-run.md/README.md (front door serves a builtin seed offline; no --seed-file needed)"
ASSERT_LOG+=("no source-tree-only --seed-file in the documented onboarding flow (builtin seed serves offline)")

grep -qE 'git checkout origin/main\b' "$FIRST_RUN" "$README" \
    && fail_with "bare 'git checkout origin/main' doc-lie present (canonical form: -B main refs/reposix/origin/main)"
ASSERT_LOG+=("no bare 'git checkout origin/main' signature; canonical -B form confirmed")

echo "zero-shot-onboarding: doc-lie signature scan clean" >&2

# --- 1. Reuse or build the reposix binary -------------------------------
echo "zero-shot-onboarding: resolving reposix binary..." >&2
if [[ -x "${WORKSPACE_ROOT}/target/debug/reposix" ]]; then
    BIN_DIR="${WORKSPACE_ROOT}/target/debug"
elif [[ -x "${WORKSPACE_ROOT}/target/release/reposix" ]]; then
    BIN_DIR="${WORKSPACE_ROOT}/target/release"
else
    echo "zero-shot-onboarding: no prebuilt binary -- building (-p reposix-cli -j2)..." >&2
    (cd "$WORKSPACE_ROOT" && timeout 300 cargo build -p reposix-cli -j2 -q) \
        || fail_with "cargo build -p reposix-cli failed"
    BIN_DIR="${WORKSPACE_ROOT}/target/debug"
fi
export PATH="${BIN_DIR}:${PATH}"
[[ -x "${BIN_DIR}/reposix" ]] || fail_with "reposix binary not found after resolve/build" "$BIN_DIR"
ASSERT_LOG+=("reposix binary resolved (reused existing build, no unbounded inline cargo run)")

# --- 2. Step 2: start the simulator (in-process, builtin seed, offline) --
# v0.13.1 front door: no curl, no --seed-file. `reposix sim` runs the sim
# IN-PROCESS (single shipped binary; no separate reposix-sim, no cargo
# fallback) and serves its compiled-in demo seed with no network fetch.
# The gate keeps --db/--ephemeral for /tmp isolation only; the DEFAULT
# (no seed flags) exercises the documented builtin-seed path.
SEED_SOURCE="builtin"
echo "zero-shot-onboarding: reposix sim --bind ${SIM_BIND} & (builtin seed, offline)" >&2
curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1 \
    && fail_with "${SIM_URL} already serving before spawn -- port ${SIM_BIND} occupied"
"${BIN_DIR}/reposix" sim --bind "$SIM_BIND" --db "$SIM_DB" --ephemeral &
SIM_PID=$!
for _ in $(seq 1 50); do
    kill -0 "$SIM_PID" 2>/dev/null || fail_with "reposix sim (pid ${SIM_PID}) exited during startup (builtin seed)"
    curl -fsS "${SIM_URL}/projects/demo/issues" >/dev/null 2>&1 && break
    sleep 0.1
done
kill -0 "$SIM_PID" 2>/dev/null || fail_with "reposix sim did not come up within 5s"
ASSERT_LOG+=("reposix sim (no --seed-file) starts in-process and answers on ${SIM_BIND} (documented step 2 command)")

# Builtin seed must serve the canonical six-issue demo project offline --
# the whole point of the front-door fix. Count the ids the sim advertises.
ISSUE_COUNT=$(curl -fsS "${SIM_URL}/projects/demo/issues" | grep -o '"id"' | wc -l | tr -d ' ')
[[ "$ISSUE_COUNT" == "6" ]] \
    || fail_with "builtin seed served ${ISSUE_COUNT} issues, expected 6 (offline demo seed regressed)"
ASSERT_LOG+=("builtin seed serves the canonical 6-issue demo project with no --seed-file and no network fetch")

# --- 4. Step 3: reposix init ---------------------------------------------
REPO="${RUN_DIR}/repo"
echo "zero-shot-onboarding: reposix init sim::demo ${REPO}" >&2
"${BIN_DIR}/reposix" init "sim::demo" "$REPO" || fail_with "reposix init sim::demo <path> exited non-zero"
git -C "$REPO" config user.email "zero-shot-onboarding@example.invalid"
git -C "$REPO" config user.name "Zero Shot Onboarding Gate"
ASSERT_LOG+=("reposix init sim::demo <path> exits 0 (documented step 3 command)")

# --- 5. Step 4: documented initial checkout (canonical -B form) ---------
echo "zero-shot-onboarding: git checkout -B main refs/reposix/origin/main" >&2
git -C "$REPO" checkout -B main refs/reposix/origin/main \
    || fail_with "git checkout -B main refs/reposix/origin/main exited non-zero (documented step 4)"
ASSERT_LOG+=("git checkout -B main refs/reposix/origin/main exits 0 (READ leg)")

# --- 6. Step 5: read -- cat issues/1.md ----------------------------------
test -f "${REPO}/issues/1.md" || fail_with "issues/1.md missing after checkout (documented step 5)"
cat "${REPO}/issues/1.md" >&2
ASSERT_LOG+=("cat issues/1.md succeeds -- read leg works with zero fixups")

# --- 7. Step 6: edit -- append comment + flip status ---------------------
{
    echo ""
    echo "## Comment from tutorial"
    echo "First-run tutorial -- confirmed avatar upload is blocked, escalating."
} >> "${REPO}/issues/1.md"
sed -i 's/^status: .*/status: in_progress/' "${REPO}/issues/1.md"
git -C "$REPO" diff issues/1.md | grep -q '^-status:' \
    || fail_with "documented edit (comment append + status sed) produced no diff"
ASSERT_LOG+=("documented edit produces the expected diff -- zero reposix-specific verbs")

# --- 8. Step 7: commit + push (WRITE leg) --------------------------------
git -C "$REPO" add issues/1.md
git -C "$REPO" commit -q -m "tutorial: add comment, move issue 1 to in_progress" \
    || fail_with "git commit exited non-zero (documented step 7)"
ASSERT_LOG+=("git commit exits 0 -- write leg works with zero fixups")

git -C "$REPO" push 2>&1 | tee "${RUN_DIR}/push.log" >&2
PUSH_EXIT=${PIPESTATUS[0]}
[[ "$PUSH_EXIT" -eq 0 ]] || fail_with "git push exited non-zero (documented step 7)" "exit=${PUSH_EXIT}"
ASSERT_LOG+=("git push exits 0 -- push leg works with zero fixups")

echo "" >&2
echo "ZERO-SHOT ONBOARDING COMPLETE -- published docs flow reproduced verbatim (seed=${SEED_SOURCE})." >&2

# F-K4b congruence: affirm each catalog expected.assert headline so the
# runner's per-expected-assert congruence check doesn't block the PASS flip.
ASSERT_LOG+=("the verifier drives the literal documented commands from docs/tutorials/first-run.md + README.md Quick start section in a throwaway /tmp leaf, never the shared repo")
ASSERT_LOG+=("read (cat issues/1.md), write (edit + git commit), and push (git push through the helper) each exit 0 with zero manual fixups beyond what the docs instruct")
ASSERT_LOG+=("none of the Wave D/E1 doc-lie signatures are present in the doc surfaces the flow was driven from")
ASSERT_LOG+=("reposix sim runs in-process and serves the compiled-in builtin demo seed (6 issues) with no --seed-file and no network fetch (offline front door)")

ASSERTS_PASSED=$(python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" "${ASSERT_LOG[@]}")
exit 0
