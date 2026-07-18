#!/usr/bin/env bash
# quality/gates/docs-repro/container-rehearse-sigkill-safe.sh
#
# P124/SC2 (DRAIN-23, closes the b773c04 orphan). Proves container-rehearse.sh's
# ephemeral-sim teardown SURVIVES the runner's outer SIGKILL and REFUSES a stale orphan
# on 7878 fail-loud. Lightweight: NO live docker container is spun here (mirrors W1a's
# container-congruence-earned gate); the container end-to-end proof is the post-release
# example rows. Tests THE HARNESS docker-free via its own --selftest-* hooks (lib/sim-
# lifecycle.sh), not a copy. Two legs:
#   STATIC (always) -- greps harness + lib for: internal `timeout` wrapping `docker run`
#     (< timeout_s); own-pgroup start + GROUP teardown; the pre-run fail-loud port gate
#     called before the sim starts.
#   DYNAMIC (needs reposix bin + lsof|ss + setsid + timeout + python3 + a FREE 7878) --
#     (b) plants a listener on 7878; the pre-run gate must FAIL LOUD (exit!=0 + teaching
#         recovery), never silent reuse.
#     (a) runs the harness under a run.py-EQUIVALENT outer SIGKILL (python
#         subprocess.run(timeout=) -- the exact b773c04 kill path): inner<outer -> harness
#         self-terminates, sim reaped, 7878 FREE; inner>=outer -> harness SIGKILLed, sim
#         orphaned, 7878 BUSY (rubber-stamp CONTROL: the test can detect the bug), then a
#         GROUP-kill of the orphan frees 7878 (the group teardown reaps).
# Degrades to NOT-VERIFIED (exit 75, never false PASS -- OP-2) when substrate is absent.
# ALWAYS leaves 127.0.0.1:7878 free. Exit: 0 PASS, 1 FAIL, 75 NOT-VERIFIED.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
HARNESS="$REPO_ROOT/quality/gates/docs-repro/container-rehearse.sh"
LIB="$REPO_ROOT/quality/gates/docs-repro/lib/sim-lifecycle.sh"
SIM_BIN="$REPO_ROOT/target/debug/reposix"
ARTIFACT="${CRSS_ARTIFACT_PATH:-$REPO_ROOT/quality/reports/verifications/docs-repro/container-rehearse-sigkill-safe.json}"
mkdir -p "$(dirname "$ARTIFACT")"
EXIT_NOT_VERIFIED=75

now_iso() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

PASSED=()
FAILED=()
SKIPPED=()

# ---- port helpers (independent of the harness so a harness bug can't hide here).
port_bound() {
    if command -v lsof >/dev/null 2>&1; then
        [[ -n "$(lsof -ti:7878 -sTCP:LISTEN 2>/dev/null)" ]]; return
    fi
    if command -v ss >/dev/null 2>&1; then
        ss -H -ltn 'sport = :7878' 2>/dev/null | grep -q .; return
    fi
    (exec 3<>/dev/tcp/127.0.0.1/7878) 2>/dev/null && { exec 3>&- 3<&-; return 0; }
    return 1
}
port_pids() { lsof -ti:7878 2>/dev/null | tr '\n' ' '; }
# Sweep any listener WE started off 7878 (safe: STATIC precondition asserts 7878 was
# free at entry, so anything here is ours). Also runs on EXIT so the port never leaks.
sweep_7878() {
    local p
    for p in $(port_pids); do kill -KILL "$p" 2>/dev/null || true; done
    # give the socket a moment to release
    local i; for i in 1 2 3 4 5; do port_bound || return 0; sleep 0.2; done
}
trap 'sweep_7878' EXIT

# wait_port_free <secs>: returns 0 as soon as 7878 is free, else 1 after the budget.
wait_port_free() {
    local budget="$1" i n
    n=$(( budget * 5 ))
    for ((i=0; i<n; i++)); do port_bound || return 0; sleep 0.2; done
    return 1
}
# wait_port_bound <secs>: returns 0 as soon as 7878 is bound, else 1.
wait_port_bound() {
    local budget="$1" i n
    n=$(( budget * 5 ))
    for ((i=0; i<n; i++)); do port_bound && return 0; sleep 0.2; done
    return 1
}

# ======================================================================= STATIC
if [[ ! -f "$HARNESS" ]]; then
    FAILED+=("STATIC: harness not found at $HARNESS")
elif [[ ! -f "$LIB" ]]; then
    FAILED+=("STATIC: sim-lifecycle lib not found at $LIB")
else
    # (S1) internal `timeout` wraps `docker run`, bounded by the row's timeout_s.
    if grep -Eq 'timeout .*("\$DOCKER_TIMEOUT"|\$DOCKER_TIMEOUT)' "$HARNESS" \
       && grep -Eq '^[[:space:]]*docker run' "$HARNESS" \
       && grep -q 'DOCKER_TIMEOUT=' "$HARNESS" \
       && grep -q 'ROW_TIMEOUT_S' "$HARNESS"; then
        PASSED+=("container-rehearse.sh wraps docker run in an internal timeout strictly shorter than the row's catalog timeout_s (DOCKER_TIMEOUT derived from ROW_TIMEOUT_S), reaping children before an outer SIGKILL")
    else
        FAILED+=("STATIC S1: container-rehearse.sh does not wrap 'docker run' in an internal timeout derived from the row timeout_s")
    fi
    # (S2) sim started in its own process group (setsid) + GROUP teardown (kill -- -PGID).
    if grep -q 'setsid ' "$LIB" \
       && grep -Eq 'kill -(TERM|KILL) -- "-\$\{?SIM_PGID' "$LIB"; then
        PASSED+=("the ephemeral sim runs in its own process group (setsid) and teardown kills the GROUP (kill -- -SIM_PGID), not just the leader pid")
    else
        FAILED+=("STATIC S2: lib/sim-lifecycle.sh does not start the sim via setsid and group-kill it (kill -- -\$SIM_PGID) on teardown")
    fi
    # (S3) the pre-run fail-loud port gate exists AND the main flow calls it before the sim.
    if grep -q 'assert_port_7878_free()' "$LIB" \
       && grep -q 'exit 75' "$LIB" \
       && grep -q '^assert_port_7878_free$' "$HARNESS"; then
        # calls-before-sim ordering: assert_port_7878_free must precede start_sim_in_own_pgroup.
        gate_ln=$(grep -n '^assert_port_7878_free$' "$HARNESS" | head -1 | cut -d: -f1)
        start_ln=$(grep -n '^start_sim_in_own_pgroup ' "$HARNESS" | head -1 | cut -d: -f1)
        if [[ -n "$gate_ln" && -n "$start_ln" && "$gate_ln" -lt "$start_ln" ]]; then
            PASSED+=("the pre-docker-run gate FAILS LOUD (exit 75 NOT-VERIFIED + teaching recovery) when port 7878 is already occupied by a stale sim, and the harness calls it before starting its own sim -- never silent reuse")
        else
            FAILED+=("STATIC S3: assert_port_7878_free is not called BEFORE start_sim_in_own_pgroup in the harness main flow")
        fi
    else
        FAILED+=("STATIC S3: the fail-loud pre-run port gate (assert_port_7878_free + exit 75) is missing from lib/harness")
    fi
fi

# ====================================================================== DYNAMIC
DYN_OK=1
DYN_REASON=""
if [[ ! -x "$SIM_BIN" ]]; then
    DYN_OK=0; DYN_REASON="target/debug/reposix not built (run 'cargo build -p reposix-cli')"
elif ! command -v setsid >/dev/null 2>&1 || ! command -v timeout >/dev/null 2>&1; then
    DYN_OK=0; DYN_REASON="setsid/timeout unavailable"
elif ! command -v lsof >/dev/null 2>&1 && ! command -v ss >/dev/null 2>&1; then
    DYN_OK=0; DYN_REASON="neither lsof nor ss available to observe port 7878"
elif ! command -v python3 >/dev/null 2>&1; then
    DYN_OK=0; DYN_REASON="python3 unavailable (needed to mimic run.py's subprocess SIGKILL)"
elif port_bound; then
    DYN_OK=0; DYN_REASON="127.0.0.1:7878 already in use at gate entry (pid(s): $(port_pids)) -- refusing to collide"
fi

# run.py-EQUIVALENT outer SIGKILL: subprocess.run([bash,harness,...], timeout=OUTER).
# On timeout Python SIGKILLs ONLY the child pid (no start_new_session) -- the exact
# b773c04 path. Prints the harness stdout so we can recover SIM_PGID if it orphans.
run_under_outer_sigkill() {
    local outer="$1"; shift
    OUTER="$outer" HARNESS="$HARNESS" python3 - "$@" <<'PY'
import os, subprocess, sys
outer = float(os.environ["OUTER"])
cmd = ["bash", os.environ["HARNESS"], *sys.argv[1:]]
try:
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=outer)
    sys.stdout.write(r.stdout)
    print("HARNESS_EXIT=%d" % r.returncode)
except subprocess.TimeoutExpired as e:
    out = e.stdout.decode() if isinstance(e.stdout, (bytes, bytearray)) else (e.stdout or "")
    sys.stdout.write(out)
    print("HARNESS_EXIT=TIMEOUT_SIGKILLED")
PY
}

if [[ "$DYN_OK" -ne 1 ]]; then
    SKIPPED+=("DYNAMIC legs skipped ($DYN_REASON) -- NOT-VERIFIED per OP-2, static mechanism still graded")
else
    # ---- DYN-B: fail-loud on a pre-bound 7878 (leg b). Plant a real sim, then invoke
    #      the harness's docker-free --selftest-port-gate and assert it refuses fail-loud.
    setsid "$SIM_BIN" sim --bind 127.0.0.1:7878 --ephemeral >/tmp/crss-planted-sim.$$.log 2>&1 &
    PLANT_PID=$!
    PLANT_PGID="$(ps -o pgid= -p "$PLANT_PID" 2>/dev/null | tr -d ' ')"
    if wait_port_bound 10; then
        GATE_ERR="$(bash "$HARNESS" --selftest-port-gate 2>&1 >/dev/null)"
        GATE_RC=$?
        if [[ "$GATE_RC" -ne 0 ]] \
           && grep -qi 'FAIL-LOUD' <<<"$GATE_ERR" \
           && grep -q '7878' <<<"$GATE_ERR" \
           && grep -qi 'kill' <<<"$GATE_ERR"; then
            PASSED+=("the pre-docker-run gate FAILED LOUD (exit $GATE_RC, teaching recovery naming port 7878 + the kill command) against a planted stale sim on 7878 -- proven at runtime, not silent reuse")
        else
            FAILED+=("DYN-B: --selftest-port-gate did NOT fail loud against a planted 7878 listener (rc=$GATE_RC); stderr head: $(head -1 <<<"$GATE_ERR")")
        fi
    else
        SKIPPED+=("DYN-B: planted sim never bound 7878 within 10s -- fail-loud leg NOT-VERIFIED")
        DYN_OK=0; DYN_REASON="planted sim failed to bind"
    fi
    [[ -n "$PLANT_PGID" ]] && kill -KILL -- "-${PLANT_PGID}" 2>/dev/null || true
    kill -KILL "$PLANT_PID" 2>/dev/null || true
    wait_port_free 6 || sweep_7878

    # ---- DYN-A: SIGKILL-then-port-free (leg a), only if 7878 is clean again.
    if [[ "$DYN_OK" -eq 1 ]] && wait_port_free 6; then
        # FIX case: inner(2) < outer(8). The harness self-terminates when its internal
        # timeout fires, the EXIT trap group-reaps the sim -> 7878 free. subprocess never
        # reaches its outer SIGKILL.
        FIX_OUT="$(run_under_outer_sigkill 8 --selftest-sigkill-lifecycle 2 30)"
        if grep -q 'HARNESS_EXIT=[0-9]' <<<"$FIX_OUT" && wait_port_free 8; then
            PASSED+=("the selftest ran the harness under a run.py-equivalent outer SIGKILL (subprocess.run timeout); the internal timeout fired first, the harness self-terminated and its group teardown left no listener on 127.0.0.1:7878 afterward")
        else
            FAILED+=("DYN-A(fix): after an inner<outer run the harness did NOT self-terminate and free 7878 (harness_exit line: $(grep -o 'HARNESS_EXIT=[A-Z0-9_]*' <<<"$FIX_OUT" | head -1); port_bound=$(port_bound && echo yes || echo no))")
        fi
        sweep_7878

        # CONTROL case (rubber-stamp guard): inner(30) >= outer(4). The outer SIGKILL
        # lands first -> harness SIGKILLed, EXIT trap skipped -> sim ORPHANS -> 7878 busy.
        # Proves the test can DETECT the bug; then a GROUP-kill of the orphan frees 7878.
        if wait_port_free 4; then
            CTL_OUT="$(run_under_outer_sigkill 4 --selftest-sigkill-lifecycle 30 30)"
            CTL_PGID="$(grep -oE 'SIM_PGID=[0-9]+' <<<"$CTL_OUT" | head -1 | cut -d= -f2)"
            if grep -q 'HARNESS_EXIT=TIMEOUT_SIGKILLED' <<<"$CTL_OUT" && wait_port_bound 3; then
                # The bug is reproducible (orphan present) -> the FIX-case "free" is meaningful.
                # Now group-kill the orphan and assert 7878 frees (the literal process-group SIGKILL).
                [[ -n "$CTL_PGID" ]] && kill -KILL -- "-${CTL_PGID}" 2>/dev/null || true
                sweep_7878
                if wait_port_free 6; then
                    PASSED+=("control: an unbounded (inner>=outer) run WAS orphaned by the outer SIGKILL (7878 busy -> the DYN-A free result is not a rubber stamp), and a GROUP-kill of the orphan's process group then freed 127.0.0.1:7878")
                else
                    FAILED+=("DYN-A(control): group-kill of the orphaned sim's process group did NOT free 7878")
                fi
            else
                # No orphan appeared -- the control could not reproduce the bug. Do not
                # claim the FIX result is meaningful; degrade rather than false-PASS.
                SKIPPED+=("DYN-A(control): could not reproduce the orphan (harness_exit=$(grep -o 'HARNESS_EXIT=[A-Z0-9_]*' <<<"$CTL_OUT" | head -1)) -- rubber-stamp guard NOT-VERIFIED")
                DYN_OK=0; DYN_REASON="control could not reproduce the orphan"
            fi
        else
            SKIPPED+=("DYN-A(control): 7878 not free before control run -- NOT-VERIFIED")
            DYN_OK=0; DYN_REASON="7878 not free before control"
        fi
    else
        SKIPPED+=("DYN-A: 7878 not free after the fail-loud leg -- SIGKILL leg NOT-VERIFIED")
        DYN_OK=0; DYN_REASON="7878 not free before DYN-A"
    fi
fi

# ============================================================= verdict + artifact
# Final safety: never leave 7878 bound.
sweep_7878 || true

STATUS_EXIT=0
if [[ ${#FAILED[@]} -gt 0 ]]; then
    STATUS_EXIT=1
elif [[ "$DYN_OK" -ne 1 ]]; then
    # Static mechanism present but a load-bearing dynamic leg could not run: NOT-VERIFIED,
    # not PASS (OP-2: a skipped runtime proof is never a green).
    STATUS_EXIT="$EXIT_NOT_VERIFIED"
fi

python3 - "$ARTIFACT" "$(now_iso)" "$STATUS_EXIT" "${PASSED[@]:-}" "--SEP--" "${FAILED[@]:-}" "--SEP--" "${SKIPPED[@]:-}" <<'PY'
import json, sys
artifact, ts, exit_code = sys.argv[1:4]
rest = sys.argv[4:]
i1 = rest.index("--SEP--")
i2 = rest.index("--SEP--", i1 + 1)
passed = [s for s in rest[:i1] if s]
failed = [s for s in rest[i1 + 1:i2] if s]
skipped = [s for s in rest[i2 + 1:] if s]
open(artifact, "w").write(json.dumps({
    "ts": ts,
    "row_id": "docs-repro/container-rehearse-sigkill-safe",
    "exit_code": int(exit_code),
    "asserts_passed": passed,
    "asserts_failed": failed,
    "skipped": skipped,
}, indent=2) + "\n")
PY

if [[ "$STATUS_EXIT" == "1" ]]; then
    printf 'container-rehearse-sigkill-safe FAILED:\n' >&2
    printf '  - %s\n' "${FAILED[@]}" >&2
elif [[ "$STATUS_EXIT" == "$EXIT_NOT_VERIFIED" ]]; then
    printf 'container-rehearse-sigkill-safe NOT-VERIFIED (static passed, dynamic skipped):\n' >&2
    printf '  - %s\n' "${SKIPPED[@]:-}" >&2
fi
exit "$STATUS_EXIT"
