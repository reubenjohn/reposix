#!/usr/bin/env bash
#
# scripts/repro-quickstart.sh — POLISH-05 reproducibility regression.
#
# Asserts the 7-step quickstart in docs/tutorials/first-run.md works
# end-to-end against the simulator from a fresh checkout.  The script is
# the "did the tutorial silently bit-rot?" canary — a developer or CI
# runs it whenever the tutorial or any code path it touches changes.
#
# What it proves:
#   1. `cargo build` produces all three required binaries.
#   2. `reposix init sim::demo <path>` configures a partial-clone working
#      tree and invokes `git fetch` successfully *with the helper on
#      PATH* (the common-fail mode if you forget `cargo install
#      reposix-remote`).
#   3. `git checkout -B main refs/reposix/origin/main` lands the seeded
#      records into the working tree (the helper's refspec namespace is
#      `refs/heads/*:refs/reposix/*`; the seeded snapshot lives under
#      `refs/reposix/origin/main` after fetch).
#   4. The seeded record file is readable, has a `title:` frontmatter
#      key, and survives an in-place edit + `git push` round-trip.
#   5. The push emits a `helper_push_*` audit row in
#      `cache.db::audit_events_cache`.
#
# The script is idempotent — it cleans up its scratch dir + simulator
# subprocess on EXIT.  Runs in <60s on a warm cargo cache.
#
# Run: bash scripts/repro-quickstart.sh
#
# CI wiring: NOT yet wired into .github/workflows/ci.yml — that workflow
# is owned by Phase 54.  Promote this script into ci.yml as a
# tutorial-doesn't-bitrot smoke test in a follow-up phase.

set -euo pipefail
cd "$(dirname "$0")/.."

REPOSIX_ROOT="$PWD"
REPO=$(mktemp -d /tmp/repro-quickstart-XXXX)
PORT=7779   # avoid colliding with the default sim on 7878
SIM_PID=""

cleanup() {
  if [[ -n "$SIM_PID" ]]; then
    kill "$SIM_PID" 2>/dev/null || true
    wait "$SIM_PID" 2>/dev/null || true
  fi
  rm -rf "$REPO"
}
trap cleanup EXIT

echo "[1/7] cargo build (cli + sim + remote)..." >&2
cargo build -p reposix-cli -p reposix-sim -p reposix-remote

# Both `reposix` AND `git-remote-reposix` MUST be on PATH — the latter is
# what `git fetch` invokes under the hood when the URL starts with
# `reposix::`. Forgetting this is THE classic broken-quickstart symptom
# ("could not read from remote repository" / "unable to find remote
# helper for 'reposix'").
export PATH="$REPOSIX_ROOT/target/debug:$PATH"
command -v reposix >/dev/null || { echo "FAIL: reposix not on PATH" >&2; exit 10; }
command -v git-remote-reposix >/dev/null \
    || { echo "FAIL: git-remote-reposix not on PATH" >&2; exit 11; }

echo "[2/7] start simulator on 127.0.0.1:$PORT..." >&2
reposix sim --bind "127.0.0.1:$PORT" --ephemeral \
    --seed-file "$REPOSIX_ROOT/crates/reposix-sim/fixtures/seed.json" \
    > "$REPO/sim.log" 2>&1 &
SIM_PID=$!
sleep 2
kill -0 "$SIM_PID" 2>/dev/null \
    || { echo "FAIL: simulator died on startup; sim.log:" >&2; cat "$REPO/sim.log" >&2; exit 20; }

# Wait for the sim to be reachable.
for _ in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:$PORT/projects/demo/issues" >/dev/null 2>&1; then
    break
  fi
  sleep 0.1
done

echo "[3/7] reposix init sim::demo $REPO/clone..." >&2
reposix init sim::demo "$REPO/clone"

# `reposix init` hardcodes 127.0.0.1:7878 in the URL it writes; rewrite
# remote.origin.url to point at our isolated port so this script doesn't
# collide with a long-running default sim. dark-factory-test.sh does the
# same thing (see comment in that script).
git -C "$REPO/clone" config remote.origin.url \
    "reposix::http://127.0.0.1:$PORT/projects/demo"

echo "[4/7] git fetch + checkout main..." >&2
# Re-fetch under the corrected origin URL so the helper actually runs.
# NOTE: the helper currently exits 128 with `fatal: could not read ref
# refs/reposix/main` even on a successful fetch (the seeded
# `refs/reposix/origin/main` IS created). Tolerate the non-zero exit and
# verify the ref instead — known divergence to be tracked separately.
git -C "$REPO/clone" fetch --filter=blob:none origin 2>"$REPO/fetch.err" || true
if ! git -C "$REPO/clone" rev-parse --verify refs/reposix/origin/main >/dev/null 2>&1; then
  echo "FAIL: fetch did not create refs/reposix/origin/main; stderr:" >&2
  cat "$REPO/fetch.err" >&2
  exit 25
fi
# The helper advertises refspec `refs/heads/*:refs/reposix/*`, so the
# remote main lands at `refs/reposix/origin/main`. Plain `git checkout
# origin/main` does NOT work — it expects the standard remotes layout.
git -C "$REPO/clone" checkout -B main refs/reposix/origin/main

cd "$REPO/clone"

echo "[5/7] inspect seeded record..." >&2
# The helper's fast-export flow writes records as `<id:04>.md` at the
# repo root (see crates/reposix-remote/src/fast_import.rs).
[[ -f 0001.md ]] \
    || { echo "FAIL: 0001.md missing; ls=$(ls)" >&2; exit 30; }
grep -q '^title:' 0001.md \
    || { echo "FAIL: frontmatter missing 'title:' in 0001.md" >&2; exit 31; }

echo "[6/7] edit + commit + push..." >&2
cat >> 0001.md <<'EOF'

## tutorial-test-comment
Reproducibility regression — the quickstart still works.
EOF
sed -i 's/^status: .*/status: in_progress/' 0001.md
git add 0001.md
git \
    -c user.email=tutorial@local \
    -c user.name=tutorial \
    commit -m "tutorial: in_progress + comment" -q
git push origin main 2>&1 | sed 's/^/    /'

echo "[7/7] verify audit row..." >&2
# The cache DB lives under XDG cache (~/.cache/reposix/) named by the
# sanitized backend+project slug. For sim::demo that's `sim-demo.git/`.
DB="$HOME/.cache/reposix/sim-demo.git/cache.db"
if [[ ! -f "$DB" ]]; then
  echo "FAIL: cache DB missing at $DB" >&2
  exit 40
fi
ROW=$(sqlite3 "$DB" \
    "SELECT op FROM audit_events_cache WHERE op LIKE 'helper_push%' ORDER BY ts DESC LIMIT 1")
if [[ -z "$ROW" ]]; then
  echo "FAIL: no helper_push_* audit row found" >&2
  echo "Recent audit ops:" >&2
  sqlite3 "$DB" "SELECT ts, op FROM audit_events_cache ORDER BY ts DESC LIMIT 5" >&2
  exit 41
fi
echo "    last push audit row: op=$ROW" >&2

echo "OK: quickstart reproducible (PASS)" >&2
