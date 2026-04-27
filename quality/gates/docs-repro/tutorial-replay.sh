#!/usr/bin/env bash
# quality/gates/docs-repro/tutorial-replay.sh -- post-release tutorial-replay.
# DOCS-REPRO-03 + SIMPLIFY-06. Runs the 7-step quickstart from
# docs/tutorials/first-run.md against the local simulator. <=150 lines.
# Lineage: ports scripts/repro-quickstart.sh predecessor verbatim per
# SIMPLIFY-06 (canonical home rewritten alongside the migrated path).
# Exits 0 (PASS) iff all steps assert successfully; 2 (PARTIAL) if cache DB
# inspection cannot run; 1 (FAIL) on step regression.
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ARTIFACT="$REPO_ROOT/quality/reports/verifications/docs-repro/tutorial-replay.json"
mkdir -p "$(dirname "$ARTIFACT")"

PASSED=()
FAILED=()
EXIT_CODE=0
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)

REPO=$(mktemp -d /tmp/tutorial-replay-XXXX)
PORT=7780
SIM_PID=""

cleanup() {
  if [[ -n "$SIM_PID" ]]; then
    kill "$SIM_PID" 2>/dev/null || true
    wait "$SIM_PID" 2>/dev/null || true
  fi
  rm -rf "$REPO"
}
trap cleanup EXIT

# Step 1: cargo build (assume host has it; tutorial-replay reuses host binaries).
echo "[1/7] cargo build (cli + sim + remote)..." >&2
if ! (cd "$REPO_ROOT" && cargo build -p reposix-cli -p reposix-sim -p reposix-remote 2>&1 | tail -5); then
    FAILED+=("step 1: cargo build failed")
    EXIT_CODE=1
fi
export PATH="$REPO_ROOT/target/debug:$PATH"
command -v reposix >/dev/null || { FAILED+=("step 1: reposix not on PATH"); EXIT_CODE=1; }
command -v git-remote-reposix >/dev/null || { FAILED+=("step 1: git-remote-reposix not on PATH"); EXIT_CODE=1; }
[[ ${#FAILED[@]} -eq 0 ]] && PASSED+=("step 1: cargo build + binaries on PATH")

# Step 2: start simulator.
if [[ $EXIT_CODE -eq 0 ]]; then
    echo "[2/7] start simulator on 127.0.0.1:$PORT..." >&2
    reposix sim --bind "127.0.0.1:$PORT" --ephemeral \
        --seed-file "$REPO_ROOT/crates/reposix-sim/fixtures/seed.json" \
        > "$REPO/sim.log" 2>&1 &
    SIM_PID=$!
    sleep 2
    if kill -0 "$SIM_PID" 2>/dev/null && curl -fsS "http://127.0.0.1:$PORT/projects/demo/issues" >/dev/null 2>&1; then
        PASSED+=("step 2: simulator listens on 127.0.0.1:$PORT")
    else
        FAILED+=("step 2: simulator did not start; see $REPO/sim.log")
        EXIT_CODE=1
    fi
fi

# Step 3: reposix init.
if [[ $EXIT_CODE -eq 0 ]]; then
    echo "[3/7] reposix init sim::demo $REPO/clone..." >&2
    if reposix init sim::demo "$REPO/clone" 2>>"$REPO/init.err"; then
        git -C "$REPO/clone" config remote.origin.url "reposix::http://127.0.0.1:$PORT/projects/demo"
        PASSED+=("step 3: reposix init sim::demo /tmp/repo configured partial-clone")
    else
        FAILED+=("step 3: reposix init failed; see $REPO/init.err")
        EXIT_CODE=1
    fi
fi

# Step 4: git checkout main.
if [[ $EXIT_CODE -eq 0 ]]; then
    echo "[4/7] git fetch + checkout main..." >&2
    git -C "$REPO/clone" fetch --filter=blob:none origin 2>"$REPO/fetch.err" || true
    if git -C "$REPO/clone" rev-parse --verify refs/reposix/origin/main >/dev/null 2>&1 && \
       git -C "$REPO/clone" checkout -B main refs/reposix/origin/main 2>>"$REPO/fetch.err"; then
        PASSED+=("step 4: git checkout -B main refs/reposix/origin/main exited 0")
    else
        FAILED+=("step 4: checkout failed; see $REPO/fetch.err")
        EXIT_CODE=1
    fi
fi

# Step 5: inspect seeded record.
if [[ $EXIT_CODE -eq 0 ]]; then
    echo "[5/7] inspect seeded record..." >&2
    if [[ -f "$REPO/clone/0001.md" ]] && grep -q '^title:' "$REPO/clone/0001.md"; then
        PASSED+=("step 5: 0001.md exists with title: frontmatter")
    else
        FAILED+=("step 5: 0001.md missing or has no title: frontmatter")
        EXIT_CODE=1
    fi
fi

# Step 6 + 7: edit, commit, push.
if [[ $EXIT_CODE -eq 0 ]]; then
    echo "[6-7/7] edit + commit + push..." >&2
    cd "$REPO/clone"
    printf '\n## tutorial-replay\nReproducibility regression -- the quickstart still works.\n' >> 0001.md
    sed -i 's/^status: .*/status: in_progress/' 0001.md
    git add 0001.md
    git -c user.email=tutorial@local -c user.name=tutorial \
        commit -m "tutorial: in_progress + comment" -q
    if git push origin main 2>"$REPO/push.err" | tee "$REPO/push.out" | grep -qE 'main -> main'; then
        PASSED+=("step 7: git push reports main -> main")
    else
        FAILED+=("step 7: push did not match main -> main; see $REPO/push.err")
        EXIT_CODE=1
    fi
    cd "$REPO_ROOT"
fi

# Step 8: verify audit row.
if [[ $EXIT_CODE -eq 0 ]]; then
    echo "[8/8] verify audit row..." >&2
    DB="$HOME/.cache/reposix/sim-demo.git/cache.db"
    if [[ -f "$DB" ]] && command -v sqlite3 >/dev/null 2>&1; then
        ROW=$(sqlite3 "$DB" "SELECT op FROM audit_events_cache WHERE op LIKE 'helper_push%' ORDER BY ts DESC LIMIT 1" 2>/dev/null)
        if [[ -n "$ROW" ]]; then
            PASSED+=("step 8: audit_events_cache has helper_push_% row (op=$ROW)")
        else
            FAILED+=("step 8: no helper_push_* audit row found in $DB")
            EXIT_CODE=1
        fi
    else
        FAILED+=("step 8: cache DB or sqlite3 missing -- skipped audit assertion")
        # Non-fatal -- mark PARTIAL if everything else passed
        EXIT_CODE=2
    fi
fi

python3 - "$ARTIFACT" "$EXIT_CODE" "$TS" "${PASSED[@]:-}" -- "${FAILED[@]:-}" <<'PY'
import json, sys
artifact, exit_code, ts = sys.argv[1:4]
rest = sys.argv[4:]
sep = rest.index("--")
passed = [s for s in rest[:sep] if s]
failed = [s for s in rest[sep + 1:] if s]
open(artifact, "w").write(json.dumps({
    "ts": ts, "row_id": "docs-repro/tutorial-replay", "exit_code": int(exit_code),
    "asserts_passed": passed, "asserts_failed": failed,
}, indent=2) + "\n")
PY

exit "$EXIT_CODE"
