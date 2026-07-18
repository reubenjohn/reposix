#!/usr/bin/env bash
# quality/gates/docs-repro/container-rehearse.sh -- generic container-rehearsal driver.
#
# DOCS-REPRO-02. Reads quality/catalogs/docs-reproducible.json by row id; runs
# row.command in row.verifier.container; writes artifact to row.artifact.
# Stdlib bash + python3 (catalog parsing); byte-ceiling governed by
# structure/file-size-limits (10000 for .sh), not a line cap.
#
# Usage:
#   quality/gates/docs-repro/container-rehearse.sh <catalog-row-id>
#
# Graceful skip: when docker is absent or daemon unreachable, write artifact
# with status=NOT-VERIFIED + exit 0 (non-fatal per quality/gates/docs-repro/README.md).
# The runner sees status=NOT-VERIFIED and either trips on P0+P1 (use a waiver
# per quality/PROTOCOL.md waiver protocol) or accepts on P2.
#
# P56 SIGPIPE lesson (quality/SURPRISES.md row 5): tempfile-then-grep, not
# pipe-into-head, when capturing docker stdout. We redirect stdout/stderr to
# tempfiles before reading.

set -uo pipefail  # NOT -e; we capture docker exit code without bash exiting

ROW_ID="${1:?usage: $0 <catalog-row-id>}"
REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
CATALOG="$REPO_ROOT/quality/catalogs/docs-reproducible.json"

now_iso() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

# Resolve artifact path from catalog (always; even on docker-absent we want
# an artifact). Falls back to a deterministic default if the row is missing.
ARTIFACT_PATH=$(python3 - "$CATALOG" "$ROW_ID" <<'PY'
import json, sys
catalog, rid = sys.argv[1], sys.argv[2]
try:
    rows = json.loads(open(catalog).read())["rows"]
except Exception:
    print("")
    sys.exit(0)
for r in rows:
    if r.get("id") == rid:
        print(r.get("artifact", ""))
        sys.exit(0)
print("")
PY
)

if [[ -z "$ARTIFACT_PATH" ]]; then
    ARTIFACT_PATH="quality/reports/verifications/docs-repro/${ROW_ID//\//-}.json"
fi
ARTIFACT_ABS="$REPO_ROOT/$ARTIFACT_PATH"
mkdir -p "$(dirname "$ARTIFACT_ABS")"

# Write a result artifact carrying an EXPLICIT exit_code + error message. Two named
# wrappers make the intent unambiguous at every call site:
#   write_skip_artifact (exit_code=0, NOT-VERIFIED) -- a substrate-absent SKIP (docker
#     missing/unreachable, target/debug/reposix not built, manual-kind row). Never a pass,
#     never a fail: the runner reads NOT-VERIFIED.
#   write_fail_artifact (exit_code=1) -- a REAL failure that must NOT be masked as success
#     (DRAIN-13 sim-readiness leg: the harness's own sim never became reachable).
write_result_artifact() {
    local code="$1"; local error="$2"
    cat > "$ARTIFACT_ABS" <<EOF
{
  "ts": "$(now_iso)",
  "row_id": "$ROW_ID",
  "exit_code": $code,
  "asserts_passed": [],
  "asserts_failed": [],
  "error": "$error"
}
EOF
    echo "container-rehearse: $error -- emitted artifact (exit_code=$code) at $ARTIFACT_PATH" >&2
}
write_skip_artifact() { write_result_artifact 0 "$1"; }
write_fail_artifact() { write_result_artifact 1 "$1"; }

# DRAIN-13 (SC4): the harness exit is derived STRICTLY from the persisted artifact
# exit_code. EVERY path that produced an artifact exits through HERE -- we re-read the
# exit_code we just wrote and exit with THAT, so a docker/timeout rc that DISAGREES with
# the recorded exit_code can never mask it (the exact W1a-reproduced rc=0-masks-exit_code=1
# gap). Fail-closed: an unreadable/missing/non-numeric artifact exits 1, never 0.
exit_from_artifact() {
    local code
    code=$(python3 - "$ARTIFACT_ABS" <<'PY'
import json, sys
try:
    print(int(json.load(open(sys.argv[1])).get("exit_code", 1)))
except Exception:
    print(1)
PY
)
    [[ "$code" =~ ^[0-9]+$ ]] || code=1
    exit "$code"
}

# DRAIN-13/SC4 docker-free selftest hook: given a PRE-WRITTEN artifact path, run the REAL
# exit-derivation path (exit_from_artifact) and exit with the code it reads. Lets
# container-rehearse-exit-from-artifact.sh prove `harness exit == persisted artifact
# exit_code` WITHOUT spinning a container -- it grades THIS code, not a copy.
if [[ "${1:-}" == "--selftest-exit-from-artifact" ]]; then
    ARTIFACT_ABS="${2:?usage: $0 --selftest-exit-from-artifact <artifact-path>}"
    exit_from_artifact
fi

# DRAIN-23 (SC2): SIGKILL-proof sim lifecycle + fail-loud stale-orphan gate. The
# reaping mechanism (own-process-group start + group teardown), the pre-run
# port-7878-free fail-loud gate, and the docker-free selftest hooks live in the sourced
# lib; cut (2) (the internal `timeout` wrapping `docker run`) lives in the main flow
# below. Full rationale: lib/sim-lifecycle.sh header + 124-PLAN.md § Wave 2. Selftest:
# container-rehearse-sigkill-safe.sh.
# shellcheck source=quality/gates/docs-repro/lib/sim-lifecycle.sh
source "$REPO_ROOT/quality/gates/docs-repro/lib/sim-lifecycle.sh"
SIM_BIN="$REPO_ROOT/target/debug/reposix"
SIM_PID=""
SIM_PGID=""
# Docker-free selftest entry points (no-op passthrough for a real row id).
sim_lifecycle_selftest_dispatch "$@"

# 1. Docker availability gate
if ! command -v docker >/dev/null 2>&1; then
    write_skip_artifact "docker not installed; rehearsal skipped (NOT-VERIFIED)"
    exit 0
fi
if ! timeout 5 docker info >/dev/null 2>&1; then
    write_skip_artifact "docker daemon unreachable; rehearsal skipped (NOT-VERIFIED)"
    exit 0
fi

# 2. Resolve command + container from catalog
ROW_JSON=$(python3 - "$CATALOG" "$ROW_ID" <<'PY'
import json, sys
catalog, rid = sys.argv[1], sys.argv[2]
rows = json.loads(open(catalog).read())["rows"]
match = next((r for r in rows if r.get("id") == rid), None)
if not match:
    sys.stderr.write(f"row not found: {rid}\n")
    sys.exit(2)
print(json.dumps(match))
PY
)
if [[ -z "$ROW_JSON" ]]; then
    write_skip_artifact "row not found: $ROW_ID"
    exit 1
fi

COMMAND=$(printf '%s' "$ROW_JSON" | python3 -c "import json,sys; r=json.load(sys.stdin); print(r.get('command','') or '')")
CONTAINER=$(printf '%s' "$ROW_JSON" | python3 -c "import json,sys; r=json.load(sys.stdin); print((r.get('verifier') or {}).get('container') or 'ubuntu:24.04')")
# DRAIN-23 cut (2): the row's OUTER grading budget (what the runner passes to
# subprocess.run(timeout=)). We wrap `docker run` in an internal `timeout` strictly
# shorter than this so the harness reaps its own sim before the outer SIGKILL fires.
ROW_TIMEOUT_S=$(printf '%s' "$ROW_JSON" | python3 -c "import json,sys; r=json.load(sys.stdin); print(int((r.get('verifier') or {}).get('timeout_s', 600) or 600))")
# Margin 60s (teardown + reap headroom); floor 30s so a tiny timeout_s still yields
# a positive internal bound. STRICTLY < ROW_TIMEOUT_S by construction.
if [[ "$ROW_TIMEOUT_S" -gt 90 ]]; then
    DOCKER_TIMEOUT=$(( ROW_TIMEOUT_S - 60 ))
else
    DOCKER_TIMEOUT=30
fi
# DRAIN-22 / F-K4b (P124 Wave 1a): congruence is EARNED, never emitted. This
# harness NO LONGER resolves row.expected.asserts to copy them verbatim into
# asserts_passed on a bare `exit 0` -- that made _audit_field.py::asserts_congruent
# a TAUTOLOGY (a no-op `exit 0` script passed identically to a real one). Instead
# each container example prints a machine-parseable `ASSERT-PASS: <text>` line
# AFTER the load-bearing step that establishes that specific assert, and we HARVEST
# those lines from the container's own stdout below (mirrors tutorial-replay.sh's
# per-step-earned pattern, adapted to the stdout-line protocol an isolated container
# requires). The `docs-repro/container-congruence-earned` meta-check gate proves a
# no-op fixture FAILS earned congruence while a real one passes.

if [[ -z "$COMMAND" ]]; then
    write_skip_artifact "row has no command (manual kind?); use manual-spec-check.sh instead"
    exit 0
fi

# 2b. Bring up an ephemeral simulator the containerised example can reach.
#     The example scripts assume a seeded sim on 127.0.0.1:7878. A default
#     `docker run` gives the container its own isolated loopback, so a host
#     sim on 127.0.0.1 is invisible inside. We start the sim on the host and
#     run the container with `--network host` so its 127.0.0.1:7878 IS the
#     host sim (Linux-only; on a host without host-networking the example's
#     own `sim not reachable` guard fails the row honestly rather than lying).
#     Starting the pre-built `reposix` binary is NOT a cargo invocation.
#     (SIM_BIN / SIM_PID / SIM_PGID + the process-group helpers are defined at the
#     top of this file alongside the DRAIN-23 selftest hooks.)
STDOUT_TMP=$(mktemp)
STDERR_TMP=$(mktemp)
HARVEST_TMP=$(mktemp)
# DRAIN-23 cut (1): the EXIT trap tears down the sim's whole PROCESS GROUP, not
# just the leader pid (cleanup -> teardown_sim). Reaps grandchildren too.
trap cleanup EXIT

if [[ ! -x "$SIM_BIN" ]]; then
    write_skip_artifact "target/debug/reposix not built; cannot start sim for rehearsal (NOT-VERIFIED) -- run 'cargo build -p reposix-cli' first"
    exit 0
fi
# DRAIN-23 pre-run gate: refuse to start over a stale orphan on 7878. FAILS LOUD
# (teaching error + NOT-VERIFIED artifact + exit 75) rather than silently reusing
# whatever answers -- the exact :135-141 miss the old readiness curl committed.
assert_port_7878_free
# DRAIN-23 cut (1): start the sim in its OWN process group so teardown can group-kill it.
# The per-run stdout/stderr log lives under quality/reports/verifications/docs-repro/ and
# is git-ignored (DRAIN-14: `.sim-*.log` pattern) -- transient runtime noise, not evidence.
SIM_LOG="$REPO_ROOT/quality/reports/verifications/docs-repro/.sim-${ROW_ID//\//-}.log"
start_sim_in_own_pgroup "$SIM_LOG"
# DRAIN-13 sim-REACHABILITY readiness gate: the pre-docker-run gate is not just port-free
# (assert_port_7878_free, above) -- it also requires the sim the harness STARTED to actually
# ANSWER a request. A bound-but-unresponsive sim (the readiness-race leg) is caught here.
SIM_READY=0
for _ in $(seq 1 30); do
    if curl -fsS "http://127.0.0.1:7878/projects/demo/issues" >/dev/null 2>&1; then
        SIM_READY=1; break
    fi
    if ! kill -0 "$SIM_PID" 2>/dev/null; then break; fi
    sleep 0.5
done
if [[ "$SIM_READY" -ne 1 ]]; then
    # DRAIN-13 sim-readiness leg: the harness STARTED its own sim (docker present, binary
    # present, port was free) but it never answered on 127.0.0.1:7878 within the readiness
    # budget. That is a REAL failure of the harness's own sim -- surface it NON-ZERO
    # (exit_code=1 via write_fail_artifact) so a sim-not-reachable flake is never MASKED as a
    # pass. Substrate-absent SKIPs (docker missing / binary not built) stayed NOT-VERIFIED /
    # exit 0 ABOVE; this is not one of those. Exit derives from the persisted artifact.
    write_fail_artifact "ephemeral sim the harness started never became reachable on 127.0.0.1:7878 within the readiness budget (DRAIN-13 sim-readiness leg) -- a broken or too-slow sim must not be masked as success; inspect the sim log at $SIM_LOG, and free any stale listener with 'kill \$(lsof -ti:7878)' before re-running"
    exit_from_artifact
fi

# 3. Run in container. Mount workspace read-only; mount target/ read-write so
# pre-built debug binaries on host PATH are visible inside. `--network host`
# makes the container's 127.0.0.1:7878 reach the host sim started above.
# Compiler toolchain (build-essential pkg-config libssl-dev) intentionally EXCLUDED
# (fix-it-twice, ruling b773c04): examples run the pre-built host-mounted target/debug/reposix
# on PATH -- there is NO in-container cargo build, so those compile-time deps were never
# exercised yet consumed the whole timeout budget via apt. Do NOT re-add build-essential.
SETUP="apt-get update -qq && apt-get install -y -qq curl ca-certificates python3 git sqlite3 >/dev/null 2>&1"

# DRAIN-23 cut (2): wrap `docker run` in an internal `timeout` STRICTLY shorter than
# the row's outer timeout_s (DOCKER_TIMEOUT computed above). If the container hangs,
# `timeout` SIGTERMs the `docker run` client (--rm tears the container down), then
# `--kill-after` SIGKILLs it -- and control returns HERE so the EXIT trap reaps the
# sim group. This guarantees the harness completes teardown BEFORE the runner's outer
# subprocess.run(timeout=timeout_s) SIGKILLs it (the b773c04 orphan path).
timeout --signal=TERM --kill-after=15 "$DOCKER_TIMEOUT" \
    docker run --rm \
    --network host \
    -v "$REPO_ROOT:/workspace:ro" \
    -v "$REPO_ROOT/target:/workspace/target:rw" \
    -w /workspace \
    "$CONTAINER" \
    sh -c "$SETUP && export PATH=/workspace/target/debug:\$PATH && $COMMAND" \
    > "$STDOUT_TMP" 2> "$STDERR_TMP"
EXIT_CODE=$?
# `timeout` exit 124 (TERM fired) / 137 (kill-after SIGKILL) => the container blew
# the internal budget. Surface it as a distinct diagnostic so a hung container is not
# mistaken for a plain assertion failure (Wave 4 derives the final exit from the
# persisted artifact; here we just annotate).
if [[ "$EXIT_CODE" -eq 124 || "$EXIT_CODE" -eq 137 ]]; then
    echo "container-rehearse: docker run exceeded internal timeout ${DOCKER_TIMEOUT}s (< row timeout_s ${ROW_TIMEOUT_S}s); container killed, sim group reaped on exit" >&2
fi

# 4. Tempfile-then-grep stdout/stderr (P56 SIGPIPE lesson: do NOT pipe-into-head).
STDOUT_TAIL=$(tail -c 4096 "$STDOUT_TMP")
STDERR_TAIL=$(tail -c 4096 "$STDERR_TMP")

# 4b. EARN congruence (DRAIN-22): harvest the container's own `ASSERT-PASS: <text>`
# lines from STDOUT ONLY. Each example emits one such line AFTER the load-bearing
# step under `set -euo pipefail`, so a harvested line is proof the step ran. STDERR
# is excluded so the [RPX-xxxx] teaching strings can never masquerade as an earned
# assert. Tempfile-then-grep (P56 SIGPIPE lesson), never copied from expected.asserts.
grep '^ASSERT-PASS: ' "$STDOUT_TMP" > "$HARVEST_TMP" 2>/dev/null || true

# 4c. Write the artifact with the AUTHORITATIVE (persisted) exit_code (DRAIN-13 / SC4).
# The harness exits with the PERSISTED exit_code -- NOT the raw docker/timeout rc (below,
# via exit_from_artifact). A clean docker exit (rc=0) is a PASS ONLY when every
# expected.assert is EARNED by a harvested ASSERT-PASS line (asserts_congruent); a rc=0
# with missing/uncongruent asserts is recorded exit_code=1 (a FAIL, never a silent pass --
# the rc-masks-artifact gap W1a reproduced). Any non-zero docker/timeout rc is exit_code=1.
# asserts_passed stays the HARVESTED set; the row's expected.asserts are read ONLY to grade
# coverage for the exit code, NEVER copied into asserts_passed (the F-K4b tautology stays
# closed -- the variable is named `expected`, not the verbatim-copy token the SC1 gate bans).
python3 - "$ARTIFACT_ABS" "$ROW_ID" "$CONTAINER" "$COMMAND" "$EXIT_CODE" "$STDOUT_TAIL" "$STDERR_TAIL" "$(now_iso)" "$HARVEST_TMP" "$CATALOG" "$REPO_ROOT" <<'PY'
import json, os, sys
artifact, rid, container, command, docker_rc, stdout, stderr, ts, harvest_path, catalog, repo_root = sys.argv[1:12]
sys.path.insert(0, os.path.join(repo_root, "quality", "runners"))
from _audit_field import asserts_congruent

PREFIX = "ASSERT-PASS: "
harvested = []
try:
    with open(harvest_path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            line = line.rstrip("\n")
            if line.startswith(PREFIX):
                text = line[len(PREFIX):].strip()
                if text:
                    harvested.append(text)
except FileNotFoundError:
    pass

# Read the row's expected.asserts ONLY to grade coverage for the exit code (never to
# populate asserts_passed -- that was the closed F-K4b tautology).
expected = []
try:
    rows = json.loads(open(catalog).read())["rows"]
    row = next((r for r in rows if r.get("id") == rid), None)
    if row:
        expected = (row.get("expected") or {}).get("asserts") or []
except Exception:
    expected = []

docker_rc = int(docker_rc)
unmatched = []
if not expected:
    congruent = True                                 # no asserts to earn -> exit tracks docker rc
elif not harvested:
    congruent, unmatched = False, list(expected)     # supposed to emit lines, emitted none
else:
    congruent, unmatched = asserts_congruent(expected, harvested)

# DRAIN-13: the AUTHORITATIVE (persisted) exit_code. The harness re-reads THIS from the
# file (exit_from_artifact) and exits with it -- never the raw docker rc.
authoritative = 0 if (docker_rc == 0 and congruent) else 1

asserts_failed = []
if docker_rc != 0:
    asserts_failed.append(f"container {container} exited with docker rc {docker_rc}")
if unmatched:
    asserts_failed.append(
        "DRAIN-22/F-K4b: expected assert(s) not earned by any harvested ASSERT-PASS line: "
        + " | ".join(unmatched)
    )

data = {
    "ts": ts,
    "row_id": rid,
    # AUTHORITATIVE persisted exit_code -- the harness re-reads THIS and exits with it, so a
    # docker rc that disagrees can never mask it (DRAIN-13 rc-masks-artifact gap closed).
    "exit_code": authoritative,
    "docker_rc": docker_rc,   # diagnostic only: the raw docker/timeout rc (may disagree)
    "container": container,
    "command": command,
    "stdout": stdout,
    "stderr": stderr,
    # HARVESTED from the container's ASSERT-PASS lines, NEVER copied from row.expected.asserts.
    "asserts_passed": list(harvested),
    "asserts_failed": asserts_failed,
    # Diagnostic context only -- NOT a congruence source.
    "diagnostic": f"container {container} ran command; docker rc={docker_rc}, authoritative exit_code={authoritative}",
    "harvested_assert_pass_count": len(harvested),
}
open(artifact, "w").write(json.dumps(data, indent=2) + "\n")
PY

# DRAIN-13 (SC4): exit STRICTLY from the persisted artifact exit_code -- re-read the value
# just written and exit with it. The harness rc now provably equals the artifact exit_code,
# closing the W1a-reproduced rc=0-masks-artifact-exit_code=1 disagreement.
exit_from_artifact
