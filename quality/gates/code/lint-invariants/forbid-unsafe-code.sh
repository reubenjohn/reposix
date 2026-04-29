#!/usr/bin/env bash
# quality/gates/code/lint-invariants/forbid-unsafe-code.sh
# Binds catalog rows:
#   - README-md/forbid-unsafe-code
#   - docs-development-contributing-md/forbid-unsafe-per-crate
# (D-01: single source of truth — both rows share this verifier.)
#
# TODO(P72 task 3): implement walk over crates/*/src/{lib,main}.rs and assert
# every entry contains `#![forbid(unsafe_code)]`. On FAIL, name offending
# files in stderr (Principle B — agent-resolvable).

set -euo pipefail

echo "STUB: $0 not yet implemented" >&2
exit 1
