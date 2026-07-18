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

write_skip_artifact() {
    local error="$1"
    cat > "$ARTIFACT_ABS" <<EOF
{
  "ts": "$(now_iso)",
  "row_id": "$ROW_ID",
  "exit_code": 0,
  "asserts_passed": [],
  "asserts_failed": [],
  "error": "$error"
}
EOF
    echo "container-rehearse: $error -- emitted NOT-VERIFIED artifact at $ARTIFACT_PATH" >&2
}

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
SIM_BIN="$REPO_ROOT/target/debug/reposix"
SIM_PID=""
STDOUT_TMP=$(mktemp)
STDERR_TMP=$(mktemp)
HARVEST_TMP=$(mktemp)
trap 'rm -f "$STDOUT_TMP" "$STDERR_TMP" "$HARVEST_TMP"; [[ -n "$SIM_PID" ]] && kill "$SIM_PID" 2>/dev/null; wait "$SIM_PID" 2>/dev/null' EXIT

if [[ ! -x "$SIM_BIN" ]]; then
    write_skip_artifact "target/debug/reposix not built; cannot start sim for rehearsal (NOT-VERIFIED) -- run 'cargo build -p reposix-cli' first"
    exit 0
fi
"$SIM_BIN" sim --bind 127.0.0.1:7878 --ephemeral > "$REPO_ROOT/quality/reports/verifications/docs-repro/.sim-${ROW_ID//\//-}.log" 2>&1 &
SIM_PID=$!
SIM_READY=0
for _ in $(seq 1 30); do
    if curl -fsS "http://127.0.0.1:7878/projects/demo/issues" >/dev/null 2>&1; then
        SIM_READY=1; break
    fi
    if ! kill -0 "$SIM_PID" 2>/dev/null; then break; fi
    sleep 0.5
done
if [[ "$SIM_READY" -ne 1 ]]; then
    write_skip_artifact "ephemeral sim failed to become ready on 127.0.0.1:7878 (NOT-VERIFIED)"
    exit 0
fi

# 3. Run in container. Mount workspace read-only; mount target/ read-write so
# pre-built debug binaries on host PATH are visible inside. `--network host`
# makes the container's 127.0.0.1:7878 reach the host sim started above.
# Compiler toolchain (build-essential pkg-config libssl-dev) intentionally EXCLUDED
# (fix-it-twice, ruling b773c04): examples run the pre-built host-mounted target/debug/reposix
# on PATH -- there is NO in-container cargo build, so those compile-time deps were never
# exercised yet consumed the whole timeout budget via apt. Do NOT re-add build-essential.
SETUP="apt-get update -qq && apt-get install -y -qq curl ca-certificates python3 git sqlite3 >/dev/null 2>&1"

docker run --rm \
    --network host \
    -v "$REPO_ROOT:/workspace:ro" \
    -v "$REPO_ROOT/target:/workspace/target:rw" \
    -w /workspace \
    "$CONTAINER" \
    sh -c "$SETUP && export PATH=/workspace/target/debug:\$PATH && $COMMAND" \
    > "$STDOUT_TMP" 2> "$STDERR_TMP"
EXIT_CODE=$?

# 4. Tempfile-then-grep stdout/stderr (P56 SIGPIPE lesson: do NOT pipe-into-head).
STDOUT_TAIL=$(tail -c 4096 "$STDOUT_TMP")
STDERR_TAIL=$(tail -c 4096 "$STDERR_TMP")

# 4b. EARN congruence (DRAIN-22): harvest the container's own `ASSERT-PASS: <text>`
# lines from STDOUT ONLY. Each example emits one such line AFTER the load-bearing
# step under `set -euo pipefail`, so a harvested line is proof the step ran. STDERR
# is excluded so the [RPX-xxxx] teaching strings can never masquerade as an earned
# assert. Tempfile-then-grep (P56 SIGPIPE lesson), never copied from expected.asserts.
grep '^ASSERT-PASS: ' "$STDOUT_TMP" > "$HARVEST_TMP" 2>/dev/null || true

# Status is exit_code-driven; asserts_passed is the HARVESTED set (never the row's
# expected.asserts verbatim). The generic run line is emitted as DIAGNOSTIC context
# only -- deliberately NOT a congruence source (closes the F-K4b tautology).
python3 - "$ARTIFACT_ABS" "$ROW_ID" "$CONTAINER" "$COMMAND" "$EXIT_CODE" "$STDOUT_TAIL" "$STDERR_TAIL" "$(now_iso)" "$HARVEST_TMP" <<'PY'
import json, sys
artifact, rid, container, command, exit_code, stdout, stderr, ts, harvest_path = sys.argv[1:10]
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
data = {
    "ts": ts,
    "row_id": rid,
    "exit_code": int(exit_code),
    "container": container,
    "command": command,
    "stdout": stdout,
    "stderr": stderr,
    # DRAIN-22: asserts_passed is HARVESTED from the container's ASSERT-PASS lines,
    # NEVER copied from row.expected.asserts. asserts_congruent() (grade time) now
    # falsifies a row whose example stopped emitting a covering line for some
    # expected.assert -- a no-op `exit 0` that emits nothing earns no congruence.
    "asserts_passed": list(harvested),
    "asserts_failed": [],
    # Diagnostic context only -- NOT a congruence source (see comment above).
    "diagnostic": f"container {container} ran command and exited {exit_code}",
    "harvested_assert_pass_count": len(harvested),
}
if int(exit_code) != 0:
    data["asserts_failed"].append(f"container {container} exited with code {exit_code}")
open(artifact, "w").write(json.dumps(data, indent=2) + "\n")
PY

exit "$EXIT_CODE"
