#!/usr/bin/env bash
# quality/gates/code/lint-invariants/forbid-unsafe-code.sh
# Binds catalog rows:
#   - README-md/forbid-unsafe-code
#   - docs-development-contributing-md/forbid-unsafe-per-crate
# (D-01: single source of truth — both rows share this verifier.)
#
# Walks every `crates/*/src/lib.rs` and `crates/*/src/main.rs` and asserts
# each contains `#![forbid(unsafe_code)]` at the file level. On FAIL the
# offending paths are named in stderr (Principle B — agent-resolvable).

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$REPO_ROOT"

mapfile -t entries < <(find crates -path '*/src/lib.rs' -o -path '*/src/main.rs' | sort)

if [ "${#entries[@]}" -eq 0 ]; then
  echo "FAIL: no crate entry points found under crates/*/src/{lib,main}.rs" >&2
  exit 1
fi

missing=()
for f in "${entries[@]}"; do
  if ! grep -qE '^#!\[forbid\(unsafe_code\)\]' "$f"; then
    missing+=("$f")
  fi
done

if [ "${#missing[@]}" -gt 0 ]; then
  echo "FAIL: ${#missing[@]} crate entry point(s) lack #![forbid(unsafe_code)]:" >&2
  printf '  %s\n' "${missing[@]}" >&2
  exit 1
fi

echo "PASS: all ${#entries[@]} crate entry points contain #![forbid(unsafe_code)]"
exit 0
