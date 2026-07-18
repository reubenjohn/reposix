#!/usr/bin/env bash
# quality/gates/docs-repro/lib/sim-lifecycle.sh
#
# DRAIN-23 (P124 Wave 2): SIGKILL-proof ephemeral-sim lifecycle for the
# container-rehearse harness. SOURCED by container-rehearse.sh; the docker-free
# selftest hooks are exercised directly by container-rehearse-sigkill-safe.sh so the
# SC2 verifier tests THIS code, not a copy.
#
# Why: container-rehearse.sh backgrounds a sim on 127.0.0.1:7878 for the containerised
# examples to reach (--network host). The runner grades the harness via
# subprocess.run([bash,harness],timeout=timeout_s) with NO start_new_session, so on
# timeout it SIGKILLs ONLY the harness PID -- a bare-backgrounded sim then orphans on
# 7878 and the harness EXIT trap never fires (the b773c04 bug). Two complementary cuts:
#   (1) start the sim in its OWN process group (setsid) and tear the GROUP down
#       (kill -- -$SIM_PGID) so the sim + any grandchildren die together -- a bare
#       `kill $SIM_PID` would leave grandchildren.  [reaping mechanism -- HERE]
#   (2) container-rehearse.sh wraps `docker run` in an internal `timeout` STRICTLY
#       shorter than the row's timeout_s so the harness always reaches teardown BEFORE
#       the outer SIGKILL fires.  [the wrapper lives in container-rehearse.sh]
# Plus a pre-run port-7878-free gate that FAILS LOUD on a stale orphan (never reuses
# whatever answers -- the old :135-141 silent-reuse miss).
#
# REQUIRED from the sourcing script (dynamic-scope, referenced at call time):
#   SIM_BIN, ARTIFACT_ABS, and the functions write_skip_artifact + now_iso.
# SETS/USES: SIM_PID, SIM_PGID. cleanup() also reaps ${STDOUT_TMP,STDERR_TMP,HARVEST_TMP}
# when the sourcing script has defined them.

# 0 if something is LISTENing on 127.0.0.1:7878, else 1.
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

# Space-separated listener pids on 7878 (best-effort; "" if none/unknown).
find_port_pids() {
    if command -v lsof >/dev/null 2>&1; then
        lsof -ti:7878 -sTCP:LISTEN 2>/dev/null | tr '\n' ' '
    elif command -v ss >/dev/null 2>&1; then
        ss -H -ltnp 'sport = :7878' 2>/dev/null | grep -oE 'pid=[0-9]+' | grep -oE '[0-9]+' | tr '\n' ' '
    fi
}

# FAIL LOUD (teaching error + NOT-VERIFIED artifact + exit 75) if a stale sim/orphan
# already holds 7878. NEVER silently reuse it (the old :135-141 miss).
assert_port_7878_free() {
    port_bound || return 0
    local pids; pids="$(find_port_pids)"; pids="${pids:-unknown}"
    {
        echo "container-rehearse: FAIL-LOUD (DRAIN-23) -- 127.0.0.1:7878 is ALREADY bound"
        echo "  by a stale sim/orphan (pid(s): ${pids}) BEFORE this harness started its own"
        echo "  ephemeral sim. A prior run's sim was likely orphaned when the runner SIGKILLed"
        echo "  the harness before its teardown ran (the b773c04 bug). Refusing to SILENTLY"
        echo "  REUSE it: a stale sim can hold pre-DRAIN-22 seed data and would make this grade"
        echo "  meaningless."
        echo "  Recover:    kill \$(lsof -ti:7878) 2>/dev/null || fuser -k 7878/tcp"
        echo "  Then re-run: bash quality/gates/docs-repro/container-rehearse.sh <catalog-row-id>"
    } >&2
    write_skip_artifact "stale sim/orphan already bound on 127.0.0.1:7878 (pid(s): ${pids}); refusing silent reuse (DRAIN-23) -- free it with 'kill \$(lsof -ti:7878)' and re-run"
    exit 75
}

# Launch the sim as leader of a NEW session/pgroup (setsid) -> its PGID == its PID,
# so teardown can group-kill it without touching this harness. Sets SIM_PID/SIM_PGID.
start_sim_in_own_pgroup() {
    local log="$1"
    setsid "$SIM_BIN" sim --bind 127.0.0.1:7878 --ephemeral > "$log" 2>&1 &
    SIM_PID=$!
    SIM_PGID="$(ps -o pgid= -p "$SIM_PID" 2>/dev/null | tr -d ' ')"
    [[ -z "$SIM_PGID" ]] && SIM_PGID="$SIM_PID"
}

# Reap the sim's WHOLE process group (SIGTERM, then SIGKILL any survivor). A bare
# `kill $SIM_PID` would orphan grandchildren. Idempotent; safe when SIM_PGID unset.
teardown_sim() {
    [[ -z "${SIM_PGID:-}" ]] && return 0
    kill -TERM -- "-${SIM_PGID}" 2>/dev/null || true
    local i
    for i in 1 2 3 4 5 6 7 8 9 10; do
        kill -0 -- "-${SIM_PGID}" 2>/dev/null || { SIM_PGID=""; return 0; }
        sleep 0.2
    done
    kill -KILL -- "-${SIM_PGID}" 2>/dev/null || true
    SIM_PGID=""
}

# EXIT trap: reap tempfiles + the sim group. Fires on normal/TERM/INT/HUP exit but NOT
# on SIGKILL -- which is why cut (2)'s internal timeout keeps the harness alive to here.
cleanup() {
    [[ -n "${STDOUT_TMP:-}" ]] && rm -f "$STDOUT_TMP" "${STDERR_TMP:-}" "${HARVEST_TMP:-}" 2>/dev/null
    teardown_sim
}

# Docker-free selftest hooks (exercise the REAL lifecycle above so the SC2 verifier
# tests THIS code, not a copy). Invoked as `container-rehearse.sh --selftest-* [args]`.
# When $1 is not a selftest hook this returns 0 and the caller continues normal flow.
sim_lifecycle_selftest_dispatch() {
    case "${1:-}" in
      --selftest-port-gate)
        # Leg (b): run ONLY the pre-run port-free gate (selftest plants a listener first).
        assert_port_7878_free
        echo "container-rehearse: --selftest-port-gate -- 127.0.0.1:7878 is free" >&2
        exit 0
        ;;
      --selftest-sim-lifecycle)
        # Leg (a) process-group: start the real sim, announce SIM_PGID, sleep; the selftest
        # group-kills SIM_PGID and asserts 7878 frees (proves the teardown group-kill reaps it).
        [[ -x "$SIM_BIN" ]] || { echo "selftest: SIM_BIN not built at $SIM_BIN" >&2; exit 3; }
        trap cleanup EXIT
        start_sim_in_own_pgroup "/tmp/reposix-sigkill-selftest-sim.$$.log"
        echo "SIM_PGID=$SIM_PGID SIM_PID=$SIM_PID"
        sleep "${2:-30}"
        exit 0
        ;;
      --selftest-sigkill-lifecycle)
        # Leg (a) internal-timeout: `timeout <inner> sleep <busy>` stands in for the
        # timeout-wrapped `docker run`. Under an OUTER SIGKILL: inner<outer -> inner fires,
        # EXIT trap reaps -> 7878 free (FIX); inner>=outer -> SIGKILL first -> orphan (BUG/control).
        [[ -x "$SIM_BIN" ]] || { echo "selftest: SIM_BIN not built at $SIM_BIN" >&2; exit 3; }
        trap cleanup EXIT
        start_sim_in_own_pgroup "/tmp/reposix-sigkill-selftest-sim.$$.log"
        echo "SIM_PGID=$SIM_PGID SIM_PID=$SIM_PID"
        timeout "${2:-2}" sleep "${3:-30}" || true
        exit 0
        ;;
    esac
    return 0
}
