#!/usr/bin/env bash
# P74 UX-BIND-05 (D-07): spaces-01 row claims `reposix spaces` subcommand
# exists and is reachable. Verifier asserts (a) the release binary is
# built, (b) `reposix spaces --help` exits 0, and (c) the help text
# mentions "List all readable Confluence spaces" (header line in
# crates/reposix-cli/src/spaces.rs).
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
BIN="${REPO_ROOT}/target/release/reposix"
if [[ ! -x "$BIN" ]]; then
  echo "FAIL: ${BIN} not built — run 'cargo build -p reposix-cli --release' first" >&2
  exit 1
fi
OUT="$("$BIN" spaces --help 2>&1)" || {
  echo "FAIL: 'reposix spaces --help' exited non-zero. Output: $OUT" >&2
  exit 1
}
if ! printf '%s' "$OUT" | grep -qF 'List all readable Confluence spaces'; then
  echo "FAIL: 'reposix spaces --help' stdout missing expected header. Got: $OUT" >&2
  exit 1
fi
echo "PASS: reposix spaces --help exits 0 with expected header"
exit 0
