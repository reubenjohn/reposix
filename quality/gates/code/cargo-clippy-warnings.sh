#!/usr/bin/env bash
# quality/gates/code/cargo-clippy-warnings.sh -- code/cargo-clippy-warnings verifier (P60 Wave D).
#
# Wraps `cargo clippy --workspace --all-targets -- -D warnings` and writes a
# runner-readable artifact. Exit codes: 0 = PASS, 1 = FAIL.
#
# Honors --row-id <id> (defaults to code/cargo-clippy-warnings).
# Anti-bloat: thin subprocess wrapper. NO --locked here -- the pre-push hook
# already enforces lockfile presence; redundant --locked errors out clean
# workspaces. Match the existing scripts/hooks/pre-push:223 invocation verbatim.
#
# Pivot rule: if cargo clippy in pre-push exceeds 60s on warm cache, P60 Wave E
# may move it OUTSIDE the runner; the wrapper still ships, the runner contract
# stays intact, only the hook body composition changes. Document in SURPRISES.md.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
readonly ARTIFACT_DIR="${REPO_ROOT}/quality/reports/verifications/code"
readonly ARTIFACT="${ARTIFACT_DIR}/cargo-clippy-warnings.json"

row_id="code/cargo-clippy-warnings"
if [[ "${1:-}" == "--row-id" && -n "${2:-}" ]]; then
  row_id="$2"
fi

cd "$REPO_ROOT"
mkdir -p "$ARTIFACT_DIR"

stdout=$(cargo clippy --workspace --all-targets -- -D warnings 2>&1) && exit_code=0 || exit_code=$?
ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)

if [ "$exit_code" -eq 0 ]; then
  asserts_passed='["cargo clippy --workspace --all-targets -- -D warnings exits 0"]'
  asserts_failed='[]'
else
  asserts_passed='[]'
  asserts_failed='["cargo clippy --workspace --all-targets -- -D warnings exit '"${exit_code}"'"]'
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
