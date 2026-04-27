#!/usr/bin/env bash
# scripts/green-gauntlet.sh -- thin shim post-P60 SIMPLIFY-09.
#
# The composite "run everything" wrapper is supplanted by the cadence-tagged
# Quality Gates runner. Mode mapping:
#   --quick / default -> python3 quality/runners/run.py --cadence pre-pr
#   --full            -> --cadence pre-pr + --cadence weekly (full sweep)
#
# This shim survives for one merge cycle per OP-5 reversibility (any hidden
# caller surfaces). Caller audit at P60 Wave D commit time: only the script
# itself references its own name; no external invokers in .github/, docs/,
# CLAUDE.md, or examples/. P63 SIMPLIFY-12 audits whether to delete or keep.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

mode="${1:-default}"

case "$mode" in
  --quick)
    exec python3 quality/runners/run.py --cadence pre-pr
    ;;
  --full)
    python3 quality/runners/run.py --cadence pre-pr || exit $?
    exec python3 quality/runners/run.py --cadence weekly
    ;;
  default)
    exec python3 quality/runners/run.py --cadence pre-pr
    ;;
  *)
    printf 'usage: %s [--quick|--full]\n' "$0" >&2
    exit 2
    ;;
esac
