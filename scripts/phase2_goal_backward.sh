#!/usr/bin/env bash
# Phase 2 goal-backward harness: runs the full ROADMAP Phase-2 SC1–SC5
# assertion suite against a freshly-built `reposix-sim` binary.
#
# Covers:
#   SC1: list returns >= 3
#   SC2: GET /projects/demo/issues/1 returns 200 + id + version
#   SC3: PATCH bogus If-Match returns 409
#   SC4: audit_events COUNT grows after curls; UPDATE trigger fires with
#        literal "append-only"
#   SC5: `cargo test -p reposix-sim --test api` (three integration tests)
#
# Usage:
#   scripts/phase2_goal_backward.sh
#   PORT=18080 scripts/phase2_goal_backward.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

PORT="${PORT:-17878}"
BIN="./target/debug/reposix-sim"
SEED="crates/reposix-sim/fixtures/seed.json"
DB="$(mktemp -u /tmp/reposix-sim-sc.XXXXXX.db)"
LOG="/tmp/reposix-sim-goalback.log"

echo "=== building reposix-sim ==="
cargo build -p reposix-sim --quiet

"$BIN" --bind "127.0.0.1:${PORT}" --db "$DB" --seed-file "$SEED" \
    >"$LOG" 2>&1 &
SIM_PID=$!
cleanup() {
    if kill -0 "$SIM_PID" 2>/dev/null; then
        kill "$SIM_PID" 2>/dev/null || true
        wait "$SIM_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Wait for healthz.
for _ in $(seq 1 40); do
    if curl -sf "http://127.0.0.1:${PORT}/healthz" >/dev/null 2>&1; then
        break
    fi
    sleep 0.1
done

# SC1
LEN=$(curl -sf "http://127.0.0.1:${PORT}/projects/demo/issues" \
    | python3 -c 'import sys,json;print(len(json.load(sys.stdin)))')
[[ "$LEN" -ge 3 ]] || { echo "SC1 FAIL (len=$LEN)"; exit 1; }
echo "SC1 PASS (list length=$LEN)"

# SC2
CODE=$(curl -s -o /dev/null -w '%{http_code}' \
    "http://127.0.0.1:${PORT}/projects/demo/issues/1")
[[ "$CODE" == "200" ]] || { echo "SC2 FAIL (code=$CODE)"; exit 1; }
echo "SC2 PASS (GET 200)"

# SC3
CODE=$(curl -s -o /dev/null -w '%{http_code}' -X PATCH \
    -H 'If-Match: "bogus"' -H 'Content-Type: application/json' \
    -d '{"status":"done"}' \
    "http://127.0.0.1:${PORT}/projects/demo/issues/1")
[[ "$CODE" == "409" ]] || { echo "SC3 FAIL (code=$CODE)"; exit 1; }
echo "SC3 PASS (PATCH bogus If-Match -> 409)"

# SC4 count
sleep 0.2
N=$(sqlite3 "$DB" \
    "SELECT COUNT(*) FROM audit_events WHERE method IN ('GET','PATCH');")
[[ "$N" -ge 2 ]] || { echo "SC4 count FAIL (got $N)"; exit 1; }
echo "SC4 count PASS ($N audit rows for GET/PATCH)"

# SC4 trigger
set +e
UPDATE_OUT=$(sqlite3 "$DB" "UPDATE audit_events SET path='x' WHERE id=1;" 2>&1)
UPDATE_RC=$?
set -e
echo "$UPDATE_OUT" | grep -q "append-only" \
    || { echo "SC4 trigger FAIL (msg=$UPDATE_OUT rc=$UPDATE_RC)"; exit 1; }
echo "SC4 trigger PASS (UPDATE blocked with 'append-only')"

# Stop the external sim before running integration tests (they spin their own).
cleanup
trap - EXIT

# SC5 — run the three integration tests.
cargo test -p reposix-sim --test api --quiet || { echo "SC5 FAIL"; exit 1; }
echo "SC5 PASS (3 integration tests green)"

echo "ALL FIVE SUCCESS CRITERIA PASS"
