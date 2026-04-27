#!/usr/bin/env bash
# quality/gates/structure/banned-words.sh — Quality Gates wrapper.
# Canonical impl: scripts/banned-words-lint.sh (kept for pre-push compat).
# SIMPLIFY-01 P57 closure via wrapper; full SIMPLIFY-01 hard-cut deferred to P60.
set -euo pipefail
exec "$(dirname "${BASH_SOURCE[0]}")/../../../scripts/banned-words-lint.sh" --all "$@"
