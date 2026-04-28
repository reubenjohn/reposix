#!/usr/bin/env bash
# quality/gates/code/cargo-fmt-clean.sh -- code/cargo-fmt-clean verifier (P63 POLISH-CODE final).
#
# Wraps `cargo fmt --all -- --check` and writes a runner-readable artifact.
# Exit codes: 0 = PASS (no formatting drift), 1 = FAIL (drift detected).
#
# Honors --row-id <id> (defaults to code/cargo-fmt-clean) for catalog discrimination.
# Anti-bloat: thin subprocess wrapper. ONE cargo at a time rule (CLAUDE.md
# Build memory budget): cargo fmt --check is read-only, ~5s, no compile,
# no link -- safe to run inline. Sibling of cargo-fmt-check.sh; distinct
# row id + artifact path so each row carries its own evidence.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
readonly ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/code"
readonly ARTIFACT="${ARTIFACT_DIR}/cargo-fmt-clean.json"

row_id="code/cargo-fmt-clean"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  row_id="$2"
fi

cd "$REPO_ROOT"
mkdir -p "$ARTIFACT_DIR"

stdout=$(cargo fmt --all -- --check 2>&1) && exit_code=0 || exit_code=$?
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)

if [ "$exit_code" -eq 0 ]; then
  asserts_passed='["cargo fmt --all -- --check exits 0 (no formatting drift in any workspace crate)"]'
  asserts_failed='[]'
else
  asserts_passed='[]'
  asserts_failed='["cargo fmt --all -- --check exit '"${exit_code}"' (drift detected; run cargo fmt --all locally and commit)"]'
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
