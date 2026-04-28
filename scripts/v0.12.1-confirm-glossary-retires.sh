#!/usr/bin/env bash
# v0.12.1 — bulk-confirm glossary retirements identified by P65 backfill.
#
# Why: docs/reference/glossary.md has 24 definitional terms (audit-log,
# backendconnector, bare-repo, etc.). The P65 extractor treated each term
# as a "claim" and proposed retirement for all 24. Definitional glossary
# entries are NOT behavioral claims — they don't bind to tests; they
# document terminology. Retirement is correct (these rows shouldn't exist
# as alignment claims).
#
# This script confirms all 24 retirements in one sitting.
#
# Run from a real TTY (the binary's `confirm-retire` verb refuses if
# stdin is non-tty AND/OR if $CLAUDE_AGENT_CONTEXT is set).
#
# Usage:
#   bash scripts/v0.12.1-confirm-glossary-retires.sh
#
# Verify after:
#   target/release/reposix-quality doc-alignment status | grep claims_retired

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Sanity: tty check + env clear
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

# 24 glossary RETIRE_PROPOSED rows from quality/catalogs/doc-alignment.json
# (extracted via:
#  jq -r '.rows[] | select(.last_verdict == "RETIRE_PROPOSED")
#                 | select((.source.file // .source[0].file) | contains("glossary")) | .id'
#  quality/catalogs/doc-alignment.json)
ROWS=(
$(jq -r '.rows[] | select(.last_verdict == "RETIRE_PROPOSED") | select((.source.file // .source[0].file) | contains("glossary")) | .id' quality/catalogs/doc-alignment.json)
)

echo "Confirming retirement for ${#ROWS[@]} glossary rows."
echo "Each row is a definitional glossary entry, not a behavioral claim."
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
echo "All ${#ROWS[@]} glossary rows confirmed retired."
echo "Verify:"
echo "  $BIN doc-alignment status | grep -E 'claims_retired|alignment_ratio'"
echo
echo "Commit the catalog change with:"
echo "  git add quality/catalogs/doc-alignment.json"
echo "  git commit -m 'docs(p67): bulk-confirm glossary retirements (24 rows)'"
