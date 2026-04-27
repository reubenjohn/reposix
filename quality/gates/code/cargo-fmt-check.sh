#!/usr/bin/env bash
# quality/gates/code/cargo-fmt-check.sh -- code/cargo-fmt-check verifier (P60 Wave D).
#
# Wraps `cargo fmt --all -- --check` and writes a runner-readable artifact.
# Exit codes: 0 = PASS (no formatting drift), 1 = FAIL (drift detected).
#
# Honors --row-id <id> (defaults to code/cargo-fmt-check) for catalog discrimination.
# Anti-bloat: thin subprocess wrapper. If cargo fmt regressed, run `cargo fmt --all`
# locally and commit. Wave E delegates pre-push to the runner via this wrapper.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
readonly ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/code"
readonly ARTIFACT="${ARTIFACT_DIR}/cargo-fmt-check.json"

row_id="code/cargo-fmt-check"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  row_id="$2"
fi

cd "$REPO_ROOT"
mkdir -p "$ARTIFACT_DIR"

stdout=$(cargo fmt --all -- --check 2>&1) && exit_code=0 || exit_code=$?
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)

if [ "$exit_code" -eq 0 ]; then
  asserts_passed='["cargo fmt --all -- --check exits 0 (no formatting drift)"]'
  asserts_failed='[]'
else
  asserts_passed='[]'
  asserts_failed='["cargo fmt --all -- --check exit '"${exit_code}"' (drift detected; run cargo fmt --all)"]'
fi

stdout_json=$(printf '%s' "$stdout" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))")

cat > "$ARTIFACT" <<EOF
{
  "ts": "$ts",
  "row_id": "$row_id",
  "exit_code": $exit_code,
  "stdout": $stdout_json,
  "stderr": "",
  "asserts_passed": $asserts_passed,
  "asserts_failed": $asserts_failed
}
EOF

if [ "$exit_code" -ne 0 ]; then
  echo "$stdout" >&2
fi
exit "$exit_code"
