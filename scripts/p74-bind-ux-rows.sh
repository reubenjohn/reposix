#!/usr/bin/env bash
# P74 Task 10: Bind 5 UX rows to their hash-shape verifier scripts.
#
# Each row's existing `claim` and `source` are recovered via jq from the
# catalog so the bind verb's match-or-extend check passes; --grade is
# always GREEN (the only value bind accepts).
#
# Re-running this script is idempotent: bind overwrites the row's tests
# vector and recomputes source_hash. Future docs-alignment phases that
# need to re-bind the same UX cluster can copy this file as a starting
# point.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." &> /dev/null && pwd)"
cd "$REPO_ROOT"

BIN="${REPO_ROOT}/target/release/reposix-quality"
CAT="${REPO_ROOT}/quality/catalogs/doc-alignment.json"

# row-id | verifier | decision-id
BINDINGS=(
  "docs/index/5-line-install|quality/gates/docs-alignment/install-snippet-shape.sh|D-03"
  "docs/index/audit-trail-git-log|quality/gates/docs-alignment/audit-trail-git-log.sh|D-04"
  "docs/index/tested-three-backends|quality/gates/docs-alignment/three-backends-tested.sh|D-05"
  "planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing|quality/gates/docs-alignment/connector-matrix-on-landing.sh|D-06"
  "planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01|quality/gates/docs-alignment/cli-spaces-smoke.sh|D-07"
)

for ENTRY in "${BINDINGS[@]}"; do
  IFS='|' read -r ROW VERIFIER DECISION <<< "$ENTRY"
  CLAIM=$(jq -r --arg id "$ROW" '.rows[] | select(.id == $id) | .claim' "$CAT")
  SRC=$(jq -r --arg id "$ROW" '.rows[] | select(.id == $id) | "\(.source.file):\(.source.line_start)-\(.source.line_end)"' "$CAT")
  echo "=== bind: $ROW -> $VERIFIER ==="
  "$BIN" doc-alignment bind \
    --row-id "$ROW" \
    --claim  "$CLAIM" \
    --source "$SRC" \
    --test   "$VERIFIER" \
    --grade  GREEN \
    --rationale "Hash-shape bind per P74 ${DECISION}; verifier under quality/gates/docs-alignment/."
done

echo
echo "=== verdicts ==="
jq -r '.rows[] | select(.id == "docs/index/5-line-install" or .id == "docs/index/audit-trail-git-log" or .id == "docs/index/tested-three-backends" or .id == "planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing" or .id == "planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01") | "\(.id) -> \(.last_verdict)"' "$CAT"
