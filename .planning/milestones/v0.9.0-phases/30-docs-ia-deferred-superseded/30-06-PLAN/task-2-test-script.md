← [back to index](./index.md)

<task type="auto">
  <name>Task 2: Write scripts/test_phase_30_tutorial.sh — end-to-end runner with cleanup trap</name>
  <files>scripts/test_phase_30_tutorial.sh</files>
  <read_first>
    - `scripts/demos/full.sh` (canonical demo — find the simulator-start and reposix-mount invocations)
    - `scripts/hooks/test-pre-push.sh` (cleanup-trap + run_and_check pattern)
    - `docs/tutorial.md` (authored in Task 1 — the script runs the EXACT commands in this file)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §scripts/test_phase_30_tutorial.sh
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md` §"Wave 0 Gaps" (line 1099 — "Promote the ad-hoc bash per OP #4")
  </read_first>
  <action>
    Create `scripts/test_phase_30_tutorial.sh` with EXACT content:

```bash
#!/usr/bin/env bash
#
# End-to-end tutorial runner — proves docs/tutorial.md is accurate.
#
# Spawns reposix-sim, mounts via reposix CLI, executes each tutorial step,
# asserts the version bump, and tears down on any exit path.
#
# Run from the repository root:
#   cargo build --release --workspace --bins
#   bash scripts/test_phase_30_tutorial.sh
#
# CI: this script runs on ubuntu-latest after `cargo build --release`.

set -euo pipefail

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m'

readonly repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly SIM_BIN="${repo_root}/target/release/reposix-sim"
readonly CLI_BIN="${repo_root}/target/release/reposix"
readonly SEED_FILE="${repo_root}/crates/reposix-sim/fixtures/seed.json"
readonly MNT="/tmp/test-phase-30-tutorial-mnt"
readonly DB="/tmp/test-phase-30-tutorial-sim.db"
readonly PORT=7878
readonly BASE_URL="http://127.0.0.1:${PORT}"

log() { printf '%b\n' "$*" >&2; }

cleanup() {
  log "${YELLOW}==>${NC} cleanup"
  if mountpoint -q "$MNT" 2>/dev/null; then
    fusermount3 -u "$MNT" 2>/dev/null || fusermount -u "$MNT" 2>/dev/null || true
  fi
  pkill -f "reposix-sim.*${PORT}" 2>/dev/null || true
  sleep 0.5
  rm -rf "$MNT" "$DB"
}
trap cleanup EXIT

# --- Preflight ---

for bin in "$SIM_BIN" "$CLI_BIN"; do
  if [[ ! -x "$bin" ]]; then
    log "${RED}error:${NC} $bin not found. Run 'cargo build --release --workspace --bins' first."
    exit 1
  fi
done

if [[ ! -f "$SEED_FILE" ]]; then
  log "${RED}error:${NC} $SEED_FILE not found."
  exit 1
fi

# Defense in depth: prior run left stuff behind?
cleanup

mkdir -p "$MNT"

# --- Step 1: start simulator ---

log "${GREEN}==>${NC} Step 1 — start simulator"
"$SIM_BIN" \
    --bind "127.0.0.1:${PORT}" \
    --db "$DB" \
    --seed-file "$SEED_FILE" &
SIM_PID=$!

for _ in $(seq 1 30); do
  if curl -sf "${BASE_URL}/healthz" >/dev/null 2>&1; then break; fi
  sleep 0.2
done

if ! curl -sf "${BASE_URL}/healthz" >/dev/null; then
  log "${RED}✖${NC} simulator failed to become ready"
  exit 1
fi

SEED_COUNT=$(curl -s "${BASE_URL}/projects/demo/issues" | jq 'length')
if [[ "$SEED_COUNT" != "6" ]]; then
  log "${RED}✖${NC} expected 6 seeded issues, got ${SEED_COUNT}"
  exit 1
fi

log "${GREEN}✓${NC} simulator up with ${SEED_COUNT} issues"

# --- Step 2: mount ---

log "${GREEN}==>${NC} Step 2 — mount"
"$CLI_BIN" mount "$MNT" \
    --backend "${BASE_URL}/projects/demo" &
MOUNT_PID=$!

for _ in $(seq 1 30); do
  if ls "${MNT}/issues/" 2>/dev/null | grep -q '^00000000001.md$'; then break; fi
  sleep 0.2
done

if ! ls "${MNT}/issues/" 2>/dev/null | grep -q '^00000000001.md$'; then
  log "${RED}✖${NC} mount did not expose issues/"
  exit 1
fi

log "${GREEN}✓${NC} mount listing 00000000001.md"

# --- Step 3: edit ---

log "${GREEN}==>${NC} Step 3 — edit issue 1"
NEW="$(sed 's/^status: open$/status: in_progress/' "${MNT}/issues/00000000001.md")"
printf '%s\n' "$NEW" > "${MNT}/issues/00000000001.md"

if ! grep -q '^status: in_progress$' "${MNT}/issues/00000000001.md"; then
  log "${RED}✖${NC} status edit did not persist in working tree"
  exit 1
fi

log "${GREEN}✓${NC} issue 1 status is now in_progress"

# --- Step 4: git push + verify version bump ---

log "${GREEN}==>${NC} Step 4 — git push + verify version bump"

VERSION_BEFORE=$(curl -s "${BASE_URL}/projects/demo/issues/1" | jq -r '.version')
if [[ "$VERSION_BEFORE" != "1" ]]; then
  log "${RED}✖${NC} expected version=1 before push, got ${VERSION_BEFORE}"
  exit 1
fi

pushd "$MNT" > /dev/null
git -c user.email=tutorial-test@example.invalid \
    -c user.name=tutorial-test \
    init -q
git remote add origin "reposix::${BASE_URL}/projects/demo"
git add -A
git -c user.email=tutorial-test@example.invalid \
    -c user.name=tutorial-test \
    commit -q -m "start issue 1"
git push -q origin HEAD:main
popd > /dev/null

VERSION_AFTER=$(curl -s "${BASE_URL}/projects/demo/issues/1" | jq -r '.version')
if [[ "$VERSION_AFTER" != "2" ]]; then
  log "${RED}✖${NC} expected version=2 after push, got ${VERSION_AFTER}"
  exit 1
fi

log "${GREEN}✓${NC} version bumped 1 → 2 server-side — aha confirmed"

# Cleanup runs via trap.
log "${GREEN}✓ tutorial end-to-end green${NC}"
```

Make the script executable:

```bash
chmod +x scripts/test_phase_30_tutorial.sh
```

Run it to confirm it works (this requires `cargo build --release --workspace --bins` to have been run at least once on the host):

```bash
bash scripts/test_phase_30_tutorial.sh
# expected: all 4 step headers, all 4 ✓ lines, "tutorial end-to-end green".
```

Optionally: wire the script into CI (`.github/workflows/ci.yml`) as a follow-up job that runs after the build step. That wiring is NOT part of this plan — it is a deferred follow-up captured in the phase SUMMARY. The script exists executable now and can be invoked manually.
  </action>
  <verify>
    <automated>test -x scripts/test_phase_30_tutorial.sh && grep -q 'trap cleanup EXIT' scripts/test_phase_30_tutorial.sh && grep -q 'fusermount3 -u' scripts/test_phase_30_tutorial.sh && grep -q 'pkill -f "reposix-sim' scripts/test_phase_30_tutorial.sh && grep -q 'VERSION_AFTER.*!= "2"' scripts/test_phase_30_tutorial.sh && bash -n scripts/test_phase_30_tutorial.sh</automated>
  </verify>
  <acceptance_criteria>
    - `test -x scripts/test_phase_30_tutorial.sh` returns 0.
    - `bash -n scripts/test_phase_30_tutorial.sh` returns 0 (no shell syntax errors).
    - `grep -c 'trap cleanup EXIT' scripts/test_phase_30_tutorial.sh` returns `1` (cleanup trap present).
    - `grep -c 'fusermount3 -u' scripts/test_phase_30_tutorial.sh` returns `>= 1` (unmount on cleanup).
    - `grep -c 'VERSION_BEFORE' scripts/test_phase_30_tutorial.sh` returns `>= 1` and `VERSION_AFTER` returns `>= 1` (aha assertion).
    - `grep -c 'Step [1-4]' scripts/test_phase_30_tutorial.sh` returns `>= 4` (all four tutorial steps present).
    - When run with `cargo build --release` artifacts present: `bash scripts/test_phase_30_tutorial.sh` exits 0.
  </acceptance_criteria>
  <done>
    Test script committed executable. Wave 4 verification plan 30-09 invokes it. Tutorial is now self-testing per CLAUDE.md OP #4 (committed-script over ad-hoc bash).
  </done>
</task>

<verification>
1. `~/.local/bin/vale --config=.vale.ini docs/tutorial.md` exits 0.
2. `mkdocs build --strict` exits 0.
3. `bash -n scripts/test_phase_30_tutorial.sh` exits 0.
4. Run on a host with `cargo build --release` artifacts: `bash scripts/test_phase_30_tutorial.sh` exits 0 and prints "aha confirmed".
</verification>

<success_criteria>
- docs/tutorial.md is a runnable 4-step tutorial with the aha moment in step 4.
- scripts/test_phase_30_tutorial.sh runs the tutorial end-to-end and asserts the server-side version bump.
- Vale clean.
- All tutorial commands target localhost simulator (zero credentials, zero remote calls).
</success_criteria>

<output>
After completion, create `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-06-SUMMARY.md` documenting:
- The 4 steps + timing (how long does the test script run?)
- Confirmation Vale passes
- Note about CI-wiring being deferred (phase decides in SUMMARY whether to add the job to `.github/workflows/ci.yml` now or leave for follow-up)
</output>
