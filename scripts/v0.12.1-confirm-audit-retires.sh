#!/usr/bin/env bash
# v0.12.1 — confirm 6 legitimate retirements identified by P67 audit.
#
# These are the rows the audit subagent classified as CONFIRM_RETIRE
# (not over-retired): 2 redirect-only stub docs, 1 status note, 1
# macFUSE transport detail, 2 superseded-by-current-implementation.
#
# Source files stay untouched. The catalog row stays as
# RETIRE_CONFIRMED (audit history) but stops counting against
# alignment_ratio.
#
# Run from a real TTY (the binary's `confirm-retire` verb refuses if
# stdin is non-tty AND/OR if $CLAUDE_AGENT_CONTEXT is set).
#
# Usage:
#   bash scripts/v0.12.1-confirm-audit-retires.sh
#
# Verify after:
#   target/release/reposix-quality doc-alignment status

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Sanity: env-guard + tty check
if [[ -n "${CLAUDE_AGENT_CONTEXT:-}" ]]; then
  echo "ERROR: \$CLAUDE_AGENT_CONTEXT is set ($CLAUDE_AGENT_CONTEXT). Run from a fresh shell with the var unset."
  exit 1
fi
if [[ ! -t 0 ]]; then
  echo "ERROR: stdin is not a TTY. Run interactively (no piping, no <  redirect)."
  exit 1
fi

BIN="target/release/reposix-quality"
[[ -x "$BIN" ]] || { echo "ERROR: $BIN missing. Run: cargo build -p reposix-quality --release"; exit 1; }

# 6 rows from P67 audit (quality/reports/doc-alignment/backfill-20260428T085523Z/RETIRE-AUDIT.md):
ROWS=(
  "docs/architecture/redirect"
  "docs/demo/redirect"
  "planning-milestones-v0-10-0-phases-REQUIREMENTS-md/helper-sim-backend-tech-debt-closed"
  "planning-milestones-v0-8-0-phases-REQUIREMENTS-md/hard-04"
  "planning-milestones-v0-8-0-phases-REQUIREMENTS-md/swarm-01"
  "planning-milestones-v0-8-0-phases-REQUIREMENTS-md/swarm-02"
)

echo "Confirming retirement for ${#ROWS[@]} audit-recommended rows."
echo "Each was reviewed by an unbiased Opus subagent and classified CONFIRM_RETIRE."
echo "See quality/reports/doc-alignment/backfill-20260428T085523Z/RETIRE-AUDIT.md"
echo "for per-row reasoning."
echo

read -r -p "Proceed? [y/N] " confirm
[[ "$confirm" == "y" || "$confirm" == "Y" ]] || { echo "Aborted."; exit 1; }

for row_id in "${ROWS[@]}"; do
  echo -n "  $row_id ... "
  if "$BIN" doc-alignment confirm-retire --row-id "$row_id"; then
    echo "OK"
  else
    echo "FAILED — stopping. Re-run after fixing."
    exit 1
  fi
done

echo
echo "All ${#ROWS[@]} audit-recommended rows confirmed retired."
echo "Verify:"
echo "  $BIN doc-alignment status | grep -E 'claims_retired|alignment_ratio'"
echo
echo "Commit the catalog change with:"
echo "  git add quality/catalogs/doc-alignment.json"
echo "  git commit -m 'docs(p67): bulk-confirm 6 audit-recommended retirements'"
