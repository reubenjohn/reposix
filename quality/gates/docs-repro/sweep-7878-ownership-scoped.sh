#!/usr/bin/env bash
# quality/gates/docs-repro/sweep-7878-ownership-scoped.sh
#
# P127 T1 regression lock (surprises-intake #1, HIGH; RED repro b3b1b407 promoted here).
# container-rehearse-sigkill-safe.sh's sweep_7878() USED to free 7878 with an ownership-
# BLIND kill (`for p in $(port_pids); do kill -KILL "$p"; done`) fired UNCONDITIONALLY
# (EXIT trap + final `sweep_7878 || true`) even on the "refuse to collide" path -- so it
# SIGKILLed a foreign sim it never started. The FIX: sweep_7878() now kills ONLY the
# pids/process-groups registered via register_owned() (SWEEP_OWNED_*) as it spawns each.
#
# This gate sources the SHIPPING register_owned / port_bound / port_pids / sweep_7878
# (awk-sliced, port-swapped -- tracks any future edit, never a paraphrase) and proves
# BOTH directions against the real code:
#   (a) a FOREIGN listener the gate never registered SURVIVES the sweep, AND
#   (b) a listener the gate DID register as owned is REAPED (cleanup preserved, not
#       neutered -- the mechanism-not-symptom half).
# Each sentinel binds a FREE PRIVATE high port in its OWN process group and is reaped on
# every exit path, so this gate NEVER touches real 7878 or a process it did not start.
#
# EXIT: 0 PASS (both directions) | 1 FAIL (either wrong) | 75 NOT-VERIFIED (substrate/
#       port absent -- OP-2, never a false PASS).
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
GATE="$REPO_ROOT/quality/gates/docs-repro/container-rehearse-sigkill-safe.sh"
ARTIFACT="${SWEEP_ARTIFACT_PATH:-$REPO_ROOT/quality/reports/verifications/docs-repro/sweep-7878-ownership-scoped.json}"
mkdir -p "$(dirname "$ARTIFACT")"
EXIT_NOT_VERIFIED=75

now_iso() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }
PASSED=(); FAILED=(); SKIPPED=()
SENTINELS=()   # "pid pgid" pairs we planted -- reaped in cleanup no matter what

write_artifact() {  # <exit_code>
    python3 - "$ARTIFACT" "$(now_iso)" "$1" "${PASSED[@]:-}" "--SEP--" "${FAILED[@]:-}" "--SEP--" "${SKIPPED[@]:-}" <<'PY'
import json, sys
artifact, ts, exit_code = sys.argv[1:4]
rest = sys.argv[4:]
i1 = rest.index("--SEP--"); i2 = rest.index("--SEP--", i1 + 1)
passed  = [s for s in rest[:i1] if s]
failed  = [s for s in rest[i1 + 1:i2] if s]
skipped = [s for s in rest[i2 + 1:] if s]
open(artifact, "w").write(json.dumps({
    "ts": ts,
    "row_id": "docs-repro/sweep-7878-ownership-scoped",
    "exit_code": int(exit_code),
    "asserts_passed": passed,
    "asserts_failed": failed,
    "skipped": skipped,
}, indent=2) + "\n")
PY
}

finish() {  # <exit_code> : write artifact, print any failures, exit
    local rc="$1"
    write_artifact "$rc"
    if [[ "$rc" == "1" ]]; then
        printf 'sweep-7878-ownership-scoped FAILED:\n' >&2
        printf '  - %s\n' "${FAILED[@]}" >&2
    elif [[ "$rc" == "$EXIT_NOT_VERIFIED" ]]; then
        printf 'sweep-7878-ownership-scoped NOT-VERIFIED:\n' >&2
        printf '  - %s\n' "${SKIPPED[@]:-}" >&2
    fi
    exit "$rc"
}

# ---- substrate gate (NOT-VERIFIED, never a false PASS/RED) --------------------
missing=""
[[ -f "$GATE" ]] || missing="gate not found at $GATE"
command -v python3 >/dev/null 2>&1 || missing="${missing:-python3 required}"
command -v setsid  >/dev/null 2>&1 || missing="${missing:-setsid required}"
command -v lsof >/dev/null 2>&1 || command -v ss >/dev/null 2>&1 || missing="${missing:-need lsof or ss}"
if [[ -n "$missing" ]]; then
    SKIPPED+=("substrate missing ($missing) -- NOT-VERIFIED per OP-2")
    finish "$EXIT_NOT_VERIFIED"
fi

SANDBOX="$(mktemp -d "${TMPDIR:-/tmp}/sweep7878-ownership.XXXXXX")"
cleanup() {
    local pair pid pgid
    for pair in "${SENTINELS[@]:-}"; do
        [[ -n "$pair" ]] || continue
        read -r pid pgid <<<"$pair"
        [[ -n "${pgid:-}" ]] && kill -KILL -- "-${pgid}" 2>/dev/null || true
        [[ -n "${pid:-}"  ]] && kill -KILL "${pid}" 2>/dev/null || true
    done
    rm -rf "$SANDBOX" 2>/dev/null || true
}
trap cleanup EXIT

port_in_use() {
    local p="$1"
    if command -v lsof >/dev/null 2>&1; then [[ -n "$(lsof -ti:"$p" 2>/dev/null)" ]]
    else ss -H -ltn "sport = :$p" 2>/dev/null | grep -q .; fi
}
pick_free_port() {  # echoes a free private high port (never 7878), or returns 1
    local cand _
    for _ in 1 2 3 4 5 6 7 8; do
        cand=$(( (RANDOM % 20000) + 20000 ))   # 20000..39999
        [[ "$cand" == "7878" ]] && continue
        port_in_use "$cand" || { echo "$cand"; return 0; }
    done
    return 1
}
# Plant a sentinel on <port> in its OWN process group (setsid); wait for bind; echo
# "PID PGID". Own group => the gate's real teardown (kill -- -$SIM_PGID) cannot reach it.
plant_sentinel() {
    local port="$1" pid pgid i
    setsid python3 - "$port" >"$SANDBOX/sentinel.$port.log" 2>&1 <<'PY' &
import socket, sys, time
p = int(sys.argv[1])
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", p)); s.listen(8)
print("sentinel-listening", p, flush=True)
time.sleep(120)
PY
    pid=$!
    pgid="$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ')"
    for i in $(seq 1 50); do port_in_use "$port" && break; sleep 0.1; done
    echo "$pid $pgid"
}
# Source the SHIPPING helpers, port-swapped to <port>. awk slices from the
# SWEEP_OWNED_PIDS decl to (not incl.) the EXIT-trap line -- capturing register_owned +
# SWEEP_OWNED_* + port_bound + port_pids + sweep_7878. sed swaps ONLY the port literals
# (`:7878`, `/7878`), never the `sweep_7878` name.
source_gate_funcs() {
    awk '/^SWEEP_OWNED_PIDS=\(\)/{f=1} /^trap /{f=0} f{print}' "$GATE" \
        | sed -e "s/:7878/:$1/g" -e "s#/7878#/$1#g" > "$SANDBOX/gate-funcs.sh"
    # shellcheck disable=SC1090
    source "$SANDBOX/gate-funcs.sh"
}

# ---- static tie-in: the ownership mechanism is REALLY in the shipping gate --------
if grep -Eq 'for p in "\$\{SWEEP_OWNED_PGIDS\[@\]' "$GATE" \
   && grep -q 'register_owned "\$PLANT_PID"' "$GATE" \
   && ! grep -q 'for p in \$(port_pids); do kill -KILL "\$p"' "$GATE"; then
    PASSED+=("the shipping sweep_7878 kills only processes tracked as owned (register_owned / SWEEP_OWNED_*) and the ownership-blind port-lookup kill (kill -KILL of port_pids output) is ABSENT from container-rehearse-sigkill-safe.sh")
else
    FAILED+=("static: sweep_7878 no longer ownership-scoped OR the ownership-blind 'kill -KILL \$(port_pids)' sweep returned to the gate")
fi

# ---- direction (a): a FOREIGN (unregistered) listener SURVIVES the sweep ----------
# The foreign sentinel binds the SWEPT port, so the OLD ownership-blind sweep (kill by
# port_pids on the swapped port) WOULD have killed it -> this is a real regression.
A_PORT="$(pick_free_port)" || { SKIPPED+=("direction (a): no free private port"); finish "$EXIT_NOT_VERIFIED"; }
read -r FA_PID FA_PGID <<<"$(plant_sentinel "$A_PORT")"; SENTINELS+=("$FA_PID $FA_PGID")
if ! kill -0 "$FA_PID" 2>/dev/null || ! port_in_use "$A_PORT"; then
    SKIPPED+=("direction (a): foreign sentinel failed to bind $A_PORT (log: $(cat "$SANDBOX/sentinel.$A_PORT.log" 2>/dev/null))")
    finish "$EXIT_NOT_VERIFIED"
fi
source_gate_funcs "$A_PORT"          # arrays reset EMPTY -> the gate owns NOTHING here
if ! port_bound; then
    SKIPPED+=("direction (a): sourced port_bound cannot see the sentinel on $A_PORT")
    finish "$EXIT_NOT_VERIFIED"
fi
sweep_7878 || true                    # ownership-blind OLD code would SIGKILL FA here
if kill -0 "$FA_PID" 2>/dev/null; then
    PASSED+=("a FOREIGN listener the gate never started (its own process group $FA_PGID, private port $A_PORT, never registered as owned) SURVIVED sweep_7878 -- the sweep never kills a process it did not register")
else
    FAILED+=("direction (a): FOREIGN listener (pid $FA_PID pgroup $FA_PGID on $A_PORT, unregistered) SIGKILLed by sweep_7878 -- blast radius is back")
fi
[[ -n "$FA_PGID" ]] && kill -KILL -- "-${FA_PGID}" 2>/dev/null || true
kill -KILL "$FA_PID" 2>/dev/null || true

# ---- direction (b): a REGISTERED (owned) listener IS REAPED by the sweep ----------
B_PORT="$(pick_free_port)" || { SKIPPED+=("direction (b): no free private port"); finish "$EXIT_NOT_VERIFIED"; }
read -r OB_PID OB_PGID <<<"$(plant_sentinel "$B_PORT")"; SENTINELS+=("$OB_PID $OB_PGID")
if ! kill -0 "$OB_PID" 2>/dev/null || ! port_in_use "$B_PORT"; then
    SKIPPED+=("direction (b): owned sentinel failed to bind $B_PORT (log: $(cat "$SANDBOX/sentinel.$B_PORT.log" 2>/dev/null))")
    finish "$EXIT_NOT_VERIFIED"
fi
source_gate_funcs "$B_PORT"          # arrays reset EMPTY again...
register_owned "$OB_PID" "$OB_PGID"  # ...then the gate OWNS this one (own pgroup)
sweep_7878 || true
if ! kill -0 "$OB_PID" 2>/dev/null; then
    PASSED+=("a listener the gate registered as owned via register_owned (own process group $OB_PGID on private port $B_PORT) was REAPED by sweep_7878 -- legitimate orphan cleanup is preserved, not neutered")
else
    FAILED+=("direction (b): OWNED listener (pid $OB_PID, register_owned'd) SURVIVED sweep_7878 -- cleanup neutered, gate's own orphan leaks")
fi

# ---- verdict -------------------------------------------------------------------
STATUS_EXIT=0
[[ ${#FAILED[@]} -gt 0 ]] && STATUS_EXIT=1
finish "$STATUS_EXIT"
