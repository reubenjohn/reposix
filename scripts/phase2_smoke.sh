#!/usr/bin/env bash
# Phase-2 live smoke test: boots the sim against fixtures/seed.json on a
# fixed loopback port, drives the three ROADMAP Phase-2 success criteria
# that don't require the audit middleware (SC1/SC2/SC3), and returns
# non-zero on any failure. Plan 02-02 adds a parallel script that also
# covers SC4/SC5 (audit + trigger).
#
# Usage:
#   scripts/phase2_smoke.sh            # uses port 17878
#   PORT=18080 scripts/phase2_smoke.sh # override port
#
# Expects the `reposix-sim` binary to already be built (cargo build
# -p reposix-sim or --release first).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

PORT="${PORT:-17878}"
BIN="${BIN:-./target/debug/reposix-sim}"
SEED="crates/reposix-sim/fixtures/seed.json"
LOG="/tmp/reposix-sim-smoke.log"

if [[ ! -x "$BIN" ]]; then
    echo "binary not found at $BIN — run 'cargo build -p reposix-sim' first" >&2
    exit 2
fi
if [[ ! -f "$SEED" ]]; then
    echo "seed file missing at $SEED" >&2
    exit 2
fi

"$BIN" --bind "127.0.0.1:${PORT}" --ephemeral --seed-file "$SEED" \
    >"$LOG" 2>&1 &
SIM_PID=$!

cleanup() {
    if kill -0 "$SIM_PID" 2>/dev/null; then
        kill "$SIM_PID" 2>/dev/null || true
        wait "$SIM_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Give the server a moment to bind.
for _ in 1 2 3 4 5 6 7 8 9 10; do
    if curl -sf "http://127.0.0.1:${PORT}/healthz" >/dev/null 2>&1; then
        break
    fi
    sleep 0.1
done

# SC1: list returns >=3
LIST_LEN="$(curl -sf "http://127.0.0.1:${PORT}/projects/demo/issues" \
    | python3 -c 'import sys,json;print(len(json.load(sys.stdin)))')"
if [[ "$LIST_LEN" -lt 3 ]]; then
    echo "SC1 FAIL: list length=$LIST_LEN (expected >=3)" >&2
    exit 1
fi
echo "SC1 PASS: list length=$LIST_LEN"

# SC2: GET /projects/demo/issues/1 returns 200 + id + version
GET_CODE="$(curl -s -o /tmp/reposix-sim-get.json -w '%{http_code}' \
    "http://127.0.0.1:${PORT}/projects/demo/issues/1")"
if [[ "$GET_CODE" != "200" ]]; then
    echo "SC2 FAIL: GET status=$GET_CODE (expected 200)" >&2
    exit 1
fi
python3 -c '
import json, sys
d = json.load(open("/tmp/reposix-sim-get.json"))
assert d["id"] == 1, d
assert isinstance(d["version"], int) and d["version"] >= 1, d
'
echo "SC2 PASS: GET status=200, id=1, version>=1"

# SC3: PATCH with bogus If-Match returns 409
PATCH_CODE="$(curl -s -o /tmp/reposix-sim-patch.json -w '%{http_code}' \
    -X PATCH -H 'If-Match: "bogus"' -H 'Content-Type: application/json' \
    -d '{"status":"done"}' \
    "http://127.0.0.1:${PORT}/projects/demo/issues/1")"
if [[ "$PATCH_CODE" != "409" ]]; then
    echo "SC3 FAIL: PATCH status=$PATCH_CODE (expected 409)" >&2
    cat /tmp/reposix-sim-patch.json >&2
    exit 1
fi
python3 -c '
import json
d = json.load(open("/tmp/reposix-sim-patch.json"))
assert d["error"] == "version_mismatch", d
assert d["current"] == 1, d
assert d["sent"] == "bogus", d
'
echo "SC3 PASS: PATCH status=409 body={error:version_mismatch,current:1,sent:bogus}"

echo "PHASE-2 SC1/SC2/SC3 PASS"
