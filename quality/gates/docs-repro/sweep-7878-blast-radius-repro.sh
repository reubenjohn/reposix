#!/usr/bin/env bash
# quality/gates/docs-repro/sweep-7878-blast-radius-repro.sh
#
# P127 T1 (DP-2 PROVE-BEFORE-FIX) — executed repro for the suspected leaked
# process-kill blast radius in container-rehearse-sigkill-safe.sh (v0.15.0
# "Floor", surprises-intake row #1, HIGH).
#
# STATUS: INERT / RED. NOT wired into any catalog, cadence, or gate-discovery
# path — run.py discovers verifiers from catalog rows only and this file has no
# row, so it never runs at pre-commit / pre-push / pre-pr / CI. It is a *pending
# regression test*: RED today (the escape reproduces) and it flips GREEN by
# itself once the real gate scopes its port sweep to processes IT started,
# because it sources the SHIPPING functions, not a paraphrase of them.
#
# ── WHAT IT PROVES ────────────────────────────────────────────────────────────
# container-rehearse-sigkill-safe.sh frees port 7878 with sweep_7878():
#     for p in $(port_pids); do kill -KILL "$p" 2>/dev/null || true; done
# port_pids() returns EVERY pid LISTENing on 7878 (lsof -ti:7878 / ss), with ZERO
# check that the gate itself started it. The sweep runs UNCONDITIONALLY twice —
# `trap 'sweep_7878' EXIT` (line ~69) AND `sweep_7878 || true` in the verdict
# section (line ~242) — so it fires even on the "refusing to collide" path, where
# the gate DETECTS a foreign sim on 7878 at entry (line ~136:
# `elif port_bound; then DYN_OK=0 …refusing to collide`) and skips its own dynamic
# legs. Net effect: a pre-existing sim owned by ANOTHER session (or by a sibling
# gate under the same run.py) that merely holds 7878 is SIGKILLed by THIS gate — a
# blast-radius escape outside the gate's own child subtree. The comment at line
# ~61 ("safe: STATIC precondition asserts 7878 was free at entry, so anything here
# is ours") is FALSE: no precondition gates the sweep, and the very port-busy
# branch that "refuses to collide" still falls through to the unconditional sweep.
#
# ── HOW IT PROVES IT (fully sandboxed — never touches the real 7878) ──────────
#   1. Picks a FREE private high port P (never 7878), so the sweep can only reach
#      OUR sentinel — never a real sim or another session's process.
#   2. Plants a SENTINEL listener on P in its OWN session/process group (setsid) —
#      "a foreign sim owned by another session." Being its own group, the gate's
#      LEGITIMATE teardown (kill -- -$SIM_PGID, scoped to the sim IT started) could
#      never reach it; only the ownership-blind port sweep can.
#   3. Extracts the REAL port_pids/port_bound/sweep_7878 (awk slice, port-swapped)
#      and sources them — testing SHIPPING logic, auto-tracking any future fix.
#   4. Mirrors the "busy-at-entry -> refuse to collide -> skip dyn legs" flow, then
#      runs the sweep exactly as the verdict section does.
#   5. Asserts the sentinel SURVIVED. If it did not -> RED (escape CONFIRMED).
#
# EXIT: 1 = escape reproduced (sentinel killed).  0 = sentinel survived (fixed).
#       2 = could not set up the repro (substrate missing / no free port).
#
# RUN:  cd /tmp && bash /ABS/PATH/quality/gates/docs-repro/sweep-7878-blast-radius-repro.sh
set -uo pipefail

GATE="$(cd "$(dirname "$0")" && pwd)/container-rehearse-sigkill-safe.sh"
[[ -f "$GATE" ]] || { echo "repro-setup: gate not found at $GATE" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "repro-setup: python3 required" >&2; exit 2; }
command -v setsid  >/dev/null 2>&1 || { echo "repro-setup: setsid required"  >&2; exit 2; }
command -v lsof >/dev/null 2>&1 || command -v ss >/dev/null 2>&1 \
    || { echo "repro-setup: need lsof or ss to observe the port" >&2; exit 2; }

SANDBOX="$(mktemp -d "${TMPDIR:-/tmp}/sweep7878-repro.XXXXXX")"

# ── static tie-in: confirm the mechanism is REALLY in the shipping gate ───────
grep -q 'for p in \$(port_pids); do kill -KILL "\$p"' "$GATE" \
    || echo "repro-note: gate no longer sweeps by-port with 'kill -KILL' — extraction may be stale" >&2
grep -q "trap 'sweep_7878' EXIT" "$GATE" \
    || echo "repro-note: gate no longer wires sweep_7878 to EXIT" >&2

# ── pick a FREE private port (NEVER 7878) ─────────────────────────────────────
port_in_use() {
    local p="$1"
    if command -v lsof >/dev/null 2>&1; then
        [[ -n "$(lsof -ti:"$p" 2>/dev/null)" ]]
    else
        ss -H -ltn "sport = :$p" 2>/dev/null | grep -q .
    fi
}
PORT=""
for _ in 1 2 3 4 5 6 7 8; do
    cand=$(( (RANDOM % 20000) + 20000 ))   # 20000..39999
    [[ "$cand" == "7878" ]] && continue
    port_in_use "$cand" || { PORT="$cand"; break; }
done
[[ -n "$PORT" ]] || { echo "repro-setup: no free private port found" >&2; rm -rf "$SANDBOX"; exit 2; }

# ── plant the SENTINEL: a foreign listener on PORT, in its OWN process group ──
setsid python3 - "$PORT" >"$SANDBOX/sentinel.log" 2>&1 <<'PY' &
import socket, sys, time
p = int(sys.argv[1])
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", p))
s.listen(8)
print("sentinel-listening", p, flush=True)
time.sleep(120)
PY
SENTINEL_PID=$!
SENTINEL_PGID="$(ps -o pgid= -p "$SENTINEL_PID" 2>/dev/null | tr -d ' ')"
SELF_PGID="$(ps -o pgid= -p $$ 2>/dev/null | tr -d ' ')"

# Defensive cleanup independent of the gate's sweep — we NEVER leak the sentinel
# even if the escape does not reproduce. Idempotent; runs on every exit path.
cleanup() {
    [[ -n "${SENTINEL_PGID:-}" ]] && kill -KILL -- "-${SENTINEL_PGID}" 2>/dev/null || true
    kill -KILL "${SENTINEL_PID:-0}" 2>/dev/null || true
    rm -rf "$SANDBOX" 2>/dev/null || true
}
trap cleanup EXIT

# Wait for the sentinel to actually bind PORT.
for _ in $(seq 1 50); do port_in_use "$PORT" && break; sleep 0.1; done
port_in_use "$PORT" || { echo "repro-setup: sentinel never bound $PORT (log: $(cat "$SANDBOX/sentinel.log"))" >&2; exit 2; }
kill -0 "$SENTINEL_PID" 2>/dev/null || { echo "repro-setup: sentinel exited early" >&2; exit 2; }

# ── source the SHIPPING port_pids / port_bound / sweep_7878 (port-swapped) ────
# awk slice: from the port_bound() definition up to (but excluding) the EXIT-trap
# line — captures all three functions regardless of their exact line numbers.
# NB: swap only the PORT usages (`:7878` in lsof/ss, `/7878` in /dev/tcp) — NOT
# the function name `sweep_7878` (a bare `s/7878/…/g` would rename it too).
awk '/^port_bound\(\) \{/{f=1} /^trap /{f=0} f{print}' "$GATE" \
    | sed -e "s/:7878/:$PORT/g" -e "s#/7878#/$PORT#g" > "$SANDBOX/gate-funcs.sh"
# shellcheck disable=SC1090
source "$SANDBOX/gate-funcs.sh"

command -v sweep_7878 >/dev/null 2>&1 || { echo "repro-setup: could not source sweep_7878 from gate" >&2; exit 2; }
port_bound || { echo "repro-setup: sourced port_bound does not see the sentinel on $PORT" >&2; exit 2; }

echo "repro: sentinel pid=$SENTINEL_PID pgid=$SENTINEL_PGID on 127.0.0.1:$PORT (this harness pgid=$SELF_PGID)"
echo "repro: sentinel's process group ($SENTINEL_PGID) is DISTINCT from this harness ($SELF_PGID)"
echo "repro: -> the gate's legitimate group teardown (kill -- -\$SIM_PGID) could NOT reach it."

# ── mirror container-rehearse-sigkill-safe.sh's "port busy at entry" path ─────
DYN_OK=1
if port_bound; then
    DYN_OK=0
    DYN_REASON="127.0.0.1:$PORT already in use at gate entry — refusing to collide"
fi
echo "repro: gate entry check -> DYN_OK=$DYN_OK ($DYN_REASON)"
echo "repro: DYN_OK=0 means the gate SKIPS every dynamic leg (it 'refuses to collide')…"
echo "repro: …but control falls through to the verdict section, which ALWAYS sweeps (line ~242):"
echo "repro:     sweep_7878 || true"

sweep_7878 || true

# ── verdict ───────────────────────────────────────────────────────────────────
echo "----------------------------------------------------------------------------"
if kill -0 "$SENTINEL_PID" 2>/dev/null; then
    echo "REPRO RESULT: sentinel SURVIVED the sweep -> the port cleanup is ownership-scoped."
    echo "DP-2 VERDICT: NOT reproduced (bug appears fixed)."
    exit 0
else
    echo "REPRO RESULT: sentinel (pid $SENTINEL_PID, pgid $SENTINEL_PGID) was SIGKILLed by the gate's sweep_7878."
    echo "  It runs in its OWN process group ($SENTINEL_PGID != this harness's $SELF_PGID) and was NEVER"
    echo "  started by the gate; the gate DETECTED it at entry and 'refused to collide' (DYN_OK=0), yet its"
    echo "  ownership-blind port sweep (kill -KILL by lsof/ss port lookup) killed it anyway."
    echo "DP-2 VERDICT: CONFIRMED — leaked kill blast radius outside the gate's own child subtree."
    exit 1
fi
