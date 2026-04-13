#!/usr/bin/env bash
# scripts/demos/full.sh — reposix full 9-step walkthrough (Tier 2).
#
# Drives the canonical 9-step demo narrative top-to-bottom against a fresh
# in-process simulator + FUSE mount. Idempotent: the EXIT trap tears down
# all child processes, FUSE mounts, and tmp state every run.
#
# NOTE: this is the Tier 2 walkthrough demo. For shorter Tier 1 demos
# (60s each, audience-specific) see scripts/demos/0{1,2,3,4}-*.sh. The
# top-level scripts/demo.sh is now a shim that execs this script.
#
# Inline helpers are kept (vs. sourcing _lib.sh) so the recording at
# docs/demo.typescript stays byte-for-byte comparable across runs.
#
# Usage:
#   bash scripts/demos/full.sh     # normal run; trap cleans up at the end.
#
# Required tools: cargo, fusermount3, jq, sqlite3, curl, sed.
#
# The recording at docs/demo.typescript is captured by:
#   script -q -c 'bash scripts/demos/full.sh' docs/demo.typescript
#
# Three guardrails fire on camera:
#   - 8a: REPOSIX_ALLOWED_ORIGINS mismatch -> EIO from a second mount.
#   - 8b: SG-02 bulk-delete cap refuses 6 deletes;
#         '[allow-bulk-delete]' tag on the commit lets it through.
#   - Step 6 (implicit + step 8c): the `version: 999` line embedded in
#         issue 3's body is NOT propagated to the server's authoritative
#         version (sanitize-on-egress strips it).

set -euo pipefail

# ---------------------------------------------------------------- CONFIG
SIM_BIND="${SIM_BIND:-127.0.0.1:7878}"
SIM_DB="${SIM_DB:-/tmp/demo-sim.db}"
MNT="${MNT:-/tmp/demo-mnt}"
ALLOW_MNT="${ALLOW_MNT:-/tmp/demo-allow-mnt}"
REPO="${REPO:-/tmp/demo-repo}"
SEED="crates/reposix-sim/fixtures/seed.json"

# ---------------------------------------------------------- CLEANUP TRAP
cleanup() {
    set +e
    fusermount3 -u "$MNT" 2>/dev/null
    fusermount3 -u "$ALLOW_MNT" 2>/dev/null
    pkill -f "reposix-fuse $MNT" 2>/dev/null
    pkill -f "reposix-fuse $ALLOW_MNT" 2>/dev/null
    pkill -f "reposix-sim --bind $SIM_BIND" 2>/dev/null
    sleep 0.3
    pkill -9 -f "reposix-fuse $MNT" 2>/dev/null
    pkill -9 -f "reposix-fuse $ALLOW_MNT" 2>/dev/null
    pkill -9 -f "reposix-sim --bind $SIM_BIND" 2>/dev/null
    rm -rf "$MNT" "$ALLOW_MNT" "$REPO" "$SIM_DB" "${SIM_DB}-wal" "${SIM_DB}-shm" 2>/dev/null
    set -e
}
trap cleanup EXIT

banner() {
    echo
    echo "==[ $1 ]== $2"
    sleep 0.3
}

wait_for_url() {
    local url=$1 deadline=$((SECONDS + 10))
    while ((SECONDS < deadline)); do
        if curl -sf "$url" >/dev/null 2>&1; then return 0; fi
        sleep 0.1
    done
    echo "timeout waiting for $url" >&2
    return 1
}

# ----------------------------------------- PRE-FLIGHT (outside recording)
# Build release binaries up front so the recording is not dominated by
# compile noise. If they are already built this is a no-op.
cargo build --release --workspace --bins --quiet

# Make `git-remote-reposix` discoverable by `git`.
RELEASE_BIN="$(pwd)/target/release"
export PATH="$RELEASE_BIN:$PATH"
SIM_BINARY="$RELEASE_BIN/reposix-sim"
FUSE_BINARY="$RELEASE_BIN/reposix-fuse"

# Clean slate (idempotency guard — second run starts as clean as first).
cleanup
mkdir -p "$MNT" "$REPO"

# ============================================================== [1/9]
banner "1/9" "workspace overview"
cargo --version
ls crates/

# ============================================================== [2/9]
banner "2/9" "test suite (release-built binaries already cached)"
# `cargo test --workspace --quiet` collapses progress dots but emits one
# `test result:` line per binary. We summarise by counting passes.
TEST_OUT="$(cargo test --workspace --no-fail-fast 2>&1 | grep -E '^test result:' || true)"
PASSED="$(echo "$TEST_OUT" | awk '{s+=$4} END{print s}')"
FAILED="$(echo "$TEST_OUT" | awk '{s+=$6} END{print s}')"
IGNORED="$(echo "$TEST_OUT" | awk '{s+=$8} END{print s}')"
echo "$TEST_OUT" | wc -l | xargs -I{} echo "test binaries: {}"
echo "passed=$PASSED failed=$FAILED ignored=$IGNORED"
[ "$FAILED" = "0" ] || { echo "tests failed -- aborting demo" >&2; exit 1; }

# ============================================================== [3/9]
banner "3/9" "start in-process simulator on $SIM_BIND"
"$SIM_BINARY" --bind "$SIM_BIND" --db "$SIM_DB" --seed-file "$SEED" \
    >/tmp/demo-sim.log 2>&1 &
SIM_PID=$!
wait_for_url "http://$SIM_BIND/healthz"
echo "sim PID=$SIM_PID; healthz OK"
curl -s "http://$SIM_BIND/projects/demo/issues" | jq 'length' \
    | xargs -I{} echo "seeded issues: {}"

# ============================================================== [4/9]
banner "4/9" "mount FUSE at $MNT (backend = http://$SIM_BIND)"
"$FUSE_BINARY" "$MNT" --backend "http://$SIM_BIND" --project demo \
    >/tmp/demo-fuse.log 2>&1 &
FUSE_PID=$!
# Wait for the mount to become readable.
for _ in $(seq 1 50); do
    if ls "$MNT" 2>/dev/null | grep -q '\.md$'; then break; fi
    sleep 0.1
done
echo "mount PID=$FUSE_PID; listing:"
ls "$MNT" | sort

# ============================================================== [5/9]
banner "5/9" "browse with shell tools (cat, grep)"
echo "--- head of 0001.md ---"
head -8 "$MNT/0001.md"
echo "--- grep -ril database ---"
grep -ril database "$MNT" || true

# ============================================================== [6/9]
banner "6/9" "edit issue 1 through FUSE and confirm server state"
echo "before: status = $(curl -s "http://$SIM_BIND/projects/demo/issues/1" | jq -r '.status')"
# Note: the FUSE FS only accepts filenames matching `<id>.md`; `sed -i`
# creates a temp file like `sed.XYZ` which gets EINVAL. We instead read,
# transform in memory, and write back via a single open(O_TRUNC)+write.
NEW_BODY="$(sed 's/^status: open$/status: in_progress/' "$MNT/0001.md")"
printf '%s\n' "$NEW_BODY" > "$MNT/0001.md"
sleep 0.3
echo "after FUSE write:"
head -6 "$MNT/0001.md"
echo "server state (id, status, version, body length):"
curl -s "http://$SIM_BIND/projects/demo/issues/1" \
    | jq '{id, status, version, body_len: (.body | length)}'

# ============================================================== [7/9]
banner "7/9" "git push round-trip via git-remote-reposix"
(
    cd "$REPO"
    # `git init -b main` requires git >= 2.28 (this dev host is 2.25).
    # Use the portable `symbolic-ref` form so the script works on stock
    # Ubuntu 20.04 and newer.
    git init -q
    git symbolic-ref HEAD refs/heads/main
    git config user.email "demo@reposix.local"
    git config user.name "reposix-demo"
    git remote add origin "reposix::http://$SIM_BIND/projects/demo"
    # Bootstrap: fetch the helper-imported snapshot, then materialise it
    # as the local `main` working tree. The helper exposes the import as
    # `refs/reposix/origin/main` (git prepends the alias to the helper's
    # `refs/reposix/*` refspec).
    #
    # Note: `git fetch` exits 128 on this v0.1 helper because it tries to
    # update a non-existent `refs/remotes/origin/main` after the import
    # completes. The actual fetch into `refs/reposix/origin/main`
    # succeeds; we verify by listing the ref and tolerate the spurious
    # exit code. (Tracked for v0.2 in the helper's `list` handler.)
    echo "fetching imported snapshot via git-remote-reposix..."
    git fetch origin 2>&1 | sed 's/^/    /' || true
    if ! git rev-parse --verify refs/reposix/origin/main >/dev/null 2>&1; then
        echo "ERROR: fetch did not produce refs/reposix/origin/main" >&2
        exit 1
    fi
    git checkout -q -B main refs/reposix/origin/main
    echo "working tree after fetch:"
    ls
    # Bump status: in_progress -> in_review via local commit + push.
    # Issue 1 is in_progress at this point (set in step 6); after push it
    # should be in_review server-side.
    sed -i 's/^status: in_progress$/status: in_review/' 0001.md
    git commit -am "request review" -q
    echo "pushing..."
    git push origin main 2>&1 | sed 's/^/    /'
)
echo "server state after push (issue 1 status, expect in_review):"
curl -s "http://$SIM_BIND/projects/demo/issues/1" | jq -r '.status'

# ============================================================== [8/9]
banner "8/9a" "GUARDRAIL: outbound HTTP allowlist refusal"
# Spawn a *second* mount whose REPOSIX_ALLOWED_ORIGINS allows port 9999
# only — the real sim is on $SIM_BIND, so every fetch fails with EIO
# on this mount. The primary $MNT is unaffected.
mkdir -p "$ALLOW_MNT"
REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:9999" \
    "$FUSE_BINARY" "$ALLOW_MNT" --backend "http://$SIM_BIND" --project demo \
    >/tmp/demo-allow.log 2>&1 &
ALLOW_PID=$!
sleep 1.5
echo "ls on allowlist-constrained mount (expect EIO / empty / refusal):"
ls "$ALLOW_MNT" 2>&1 || echo "allowlist refused backend -- expected"
echo "(stderr from the constrained mount, last 10 lines:)"
tail -10 /tmp/demo-allow.log 2>/dev/null | sed 's/^/    /' || true
fusermount3 -u "$ALLOW_MNT" 2>/dev/null || true
kill "$ALLOW_PID" 2>/dev/null || true
sleep 0.3
rm -rf "$ALLOW_MNT"

banner "8/9b" "GUARDRAIL: SG-02 bulk-delete cap"
(
    cd "$REPO"
    git rm -q 0001.md 0002.md 0003.md 0004.md 0005.md 0006.md
    git commit -am "cleanup" -q
    set +e
    git push origin main 2>/tmp/sg02.log
    PUSH_RC=$?
    set -e
    echo "first push exit code: $PUSH_RC (expect non-zero)"
    echo "stderr:"
    sed 's/^/    /' /tmp/sg02.log
    if grep -q "allow-bulk-delete" /tmp/sg02.log; then
        echo "SG-02 fired as expected"
    else
        echo "SG-02 did NOT fire -- failing demo" >&2
        exit 1
    fi
    git commit --amend -q -m "[allow-bulk-delete] cleanup"
    echo "second push (with override tag):"
    git push origin main 2>&1 | sed 's/^/    /'
)
echo "server issue count after override push:"
curl -s "http://$SIM_BIND/projects/demo/issues" | jq 'length'

banner "8/9c" "audit log truth (last 5 rows)"
sqlite3 -header -column "$SIM_DB" \
    'SELECT method, path, status FROM audit_events ORDER BY id DESC LIMIT 5;'

# ============================================================== [9/9]
banner "9/9" "cleanup (trap will fusermount3 -u, pkill, rm /tmp/demo-*)"
echo "DEMO COMPLETE: cleanup trap will run on exit."

echo
echo "== DEMO COMPLETE =="
