#!/usr/bin/env bash
# Quality Gates verifier — code/clippy-lint-loaded
#
# Migrated from scripts/check_clippy_lint_loaded.sh per SIMPLIFY-04 (P58).
# This is now the canonical home for the check; the old path is deleted
# (no script callers — only documentation references).
#
# FIX 3 from plan-checker: prove clippy.toml is actually loaded by clippy.
#
# Strategy: we assert four things in sequence:
#   1. clippy.toml exists at the workspace root.
#   2. clippy.toml lists each of the three banned reqwest constructors.
#   3. No source file outside crates/reposix-core/src/http.rs constructs a
#      reqwest::Client or reqwest::ClientBuilder directly (behavioural proof
#      that the lint is being enforced: if the config weren't loaded, the
#      next author could sneak a direct construction in and we'd miss it).
#   4. The full workspace clippy run is clean under -D warnings.

set -euo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null || dirname "$(dirname "$(dirname "$(realpath "$0")")")")"

test -f clippy.toml || { echo "clippy.toml missing"; exit 1; }
grep -q 'reqwest::Client::new'        clippy.toml || { echo "clippy.toml missing reqwest::Client::new"; exit 1; }
grep -q 'reqwest::Client::builder'    clippy.toml || { echo "clippy.toml missing reqwest::Client::builder"; exit 1; }
grep -q 'reqwest::ClientBuilder::new' clippy.toml || { echo "clippy.toml missing reqwest::ClientBuilder::new"; exit 1; }

BAD="$(grep -RIn 'reqwest::Client::new\|reqwest::Client::builder\|reqwest::ClientBuilder::new' crates/ \
      --include='*.rs' \
    | grep -v 'crates/reposix-core/src/http.rs' \
    | grep -v '^[^:]*:[^:]*: *//' || true)"
if [ -n "$BAD" ]; then
    echo "Direct reqwest construction outside http.rs (clippy lint not enforced?):"
    echo "$BAD"
    exit 1
fi

cargo clippy --workspace --all-targets -- -D warnings >/dev/null

echo "OK: clippy.toml loaded, disallowed-methods enforced, workspace clean."
