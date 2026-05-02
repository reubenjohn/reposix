#!/usr/bin/env bash
# quality/gates/code/lint-invariants/clippy-pedantic-targeted-allows.sh
#
# Asserts the contributing.md invariant: "Clippy is pedantic with targeted
# allows only; blanket #[allow(clippy::pedantic)] is forbidden."
#
# Two checks:
#   1. Every crate root (crates/*/src/lib.rs OR src/main.rs) declares
#      `#![warn(clippy::pedantic)]` (or includes pedantic in a multi-lint
#      warn attribute).
#   2. No source file under crates/ contains a blanket
#      `#![allow(clippy::pedantic)]` (the forbidden pattern).
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

failed=0

# Check 1: every crate has pedantic enabled at root
for cargo in crates/*/Cargo.toml; do
  crate_dir=$(dirname -- "$cargo")
  has_pedantic=0
  for root in "$crate_dir/src/lib.rs" "$crate_dir/src/main.rs"; do
    [[ -f "$root" ]] || continue
    if grep -qE '#!\[warn\(.*clippy::pedantic.*\)\]' "$root"; then
      has_pedantic=1
      break
    fi
  done
  if [[ "$has_pedantic" -eq 0 ]]; then
    echo "FAIL: $crate_dir has no '#![warn(...clippy::pedantic...)]' at any crate root" >&2
    failed=1
  fi
done

# Check 2: no blanket #![allow(clippy::pedantic)] anywhere under crates/
if grep -rEn '#!\[allow\(clippy::pedantic\)\]' crates/ 2>/dev/null; then
  echo "FAIL: found blanket #![allow(clippy::pedantic)] (forbidden — use targeted allows)" >&2
  failed=1
fi

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi

echo "PASS: every crate root warns clippy::pedantic; no blanket allow(clippy::pedantic) under crates/"
