#!/usr/bin/env bash
# quality/gates/docs-repro/container-rehearse.sh -- generic container-rehearsal driver.
#
# DOCS-REPRO-02. Reads quality/catalogs/docs-reproducible.json by row id; runs
# row.command in row.verifier.container; writes artifact to row.artifact.
# <=150 lines. Stdlib bash + python3 (catalog parsing).
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

if [[ -z "$COMMAND" ]]; then
    write_skip_artifact "row has no command (manual kind?); use manual-spec-check.sh instead"
    exit 0
fi

# 3. Run in container. Mount workspace read-only; mount target/ read-write so
# pre-built debug binaries on host PATH are visible inside.
SETUP="apt-get update -qq && apt-get install -y -qq curl ca-certificates python3 git build-essential pkg-config libssl-dev sqlite3 >/dev/null 2>&1"
STDOUT_TMP=$(mktemp)
STDERR_TMP=$(mktemp)
trap 'rm -f "$STDOUT_TMP" "$STDERR_TMP"' EXIT

docker run --rm \
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

# Status is exit_code-driven for v0.12.0; per-assert grading is v0.12.1
python3 - "$ARTIFACT_ABS" "$ROW_ID" "$CONTAINER" "$COMMAND" "$EXIT_CODE" "$STDOUT_TAIL" "$STDERR_TAIL" "$(now_iso)" <<'PY'
import json, sys
artifact, rid, container, command, exit_code, stdout, stderr, ts = sys.argv[1:9]
data = {
    "ts": ts,
    "row_id": rid,
    "exit_code": int(exit_code),
    "container": container,
    "command": command,
    "stdout": stdout,
    "stderr": stderr,
    "asserts_passed": [],
    "asserts_failed": [],
}
if int(exit_code) == 0:
    data["asserts_passed"].append(f"container {container} ran command and exited 0")
else:
    data["asserts_failed"].append(f"container {container} exited with code {exit_code}")
open(artifact, "w").write(json.dumps(data, indent=2) + "\n")
PY

exit "$EXIT_CODE"
