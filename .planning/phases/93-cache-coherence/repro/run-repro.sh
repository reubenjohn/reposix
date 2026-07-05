#!/usr/bin/env bash
# Self-contained D-P92-03 reproduction driver. Builds the three reposix binaries
# from the workspace, starts a fresh seeded sim on the HOST, then runs the
# git-2.54 container litmus (litmus.sh) for the requested MODE. The host box is
# git 2.25.1 (below the >=2.34 helper floor), so the helper/push/pull litmus MUST
# run in the container; the sim is reached via `--network host`.
#
#   MODE=pin-cursor  : deterministic same-second cursor pin  -> expect `not our ref` (exit 128)
#   MODE=same-second : natural tight init+push race          -> expect `not our ref` (exit 128)
#   MODE=gap2s       : 2s sleep before A's push (later second)-> expect ordinary CONFLICT (exit 1)
#
# Usage: run-repro.sh <pin-cursor|same-second|gap2s>
set -euo pipefail
MODE="${1:?usage: run-repro.sh <pin-cursor|same-second|gap2s>}"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../../../.." && pwd)"        # -> workspace root
IMG=reposix-repro:git254
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

echo "== building binaries (CARGO_BUILD_JOBS=2, one cargo invocation) =="
( cd "$REPO" && CARGO_BUILD_JOBS=2 cargo build --release \
    -p reposix-sim -p reposix-cli -p reposix-remote )
mkdir -p "$STAGE/bin"
cp "$REPO/target/release/reposix" "$REPO/target/release/reposix-sim" \
   "$REPO/target/release/git-remote-reposix" "$STAGE/bin/"

echo "== ensuring container image ($IMG) =="
docker image inspect "$IMG" >/dev/null 2>&1 || docker build -t "$IMG" "$HERE"

echo "== fresh seeded sim on 127.0.0.1:7878 =="
pkill -f 'target/release/reposix-sim' 2>/dev/null || true
sleep 1
"$REPO/target/release/reposix-sim" --ephemeral --bind 127.0.0.1:7878 \
  --seed-file "$REPO/crates/reposix-sim/fixtures/seed.json" >"$STAGE/sim.log" 2>&1 &
SIM_PID=$!
trap 'kill $SIM_PID 2>/dev/null; rm -rf "$STAGE"' EXIT
for _ in $(seq 1 30); do
  curl -sf http://127.0.0.1:7878/projects/demo/issues/1 >/dev/null 2>&1 && break
  sleep 0.3
done
curl -sf http://127.0.0.1:7878/projects/demo/issues/1 >/dev/null || { echo FATAL: sim down; cat "$STAGE/sim.log"; exit 2; }

echo "== litmus MODE=$MODE in git-2.54 container =="
docker run --rm --network host \
  -v "$STAGE/bin:/bin-ro:ro" \
  -v "$HERE/litmus.sh:/litmus.sh:ro" \
  "$IMG" bash /litmus.sh "$MODE"
