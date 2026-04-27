#!/usr/bin/env bash
#
# check-mermaid-renders.sh — enforce that every mermaid-bearing docs page
# has a corresponding playwright artifact PROVING the diagram renders to
# at least one <svg> child with zero render-error console messages.
#
# Why a source→artifact check (and NOT a live browser walk):
#
#   The literal owner request in HANDOVER §0.1 is "loop over EVERY mkdocs
#   page and assert each <pre.mermaid> element has at least one <svg>
#   child." That requires a headless browser — Chromium + a runtime —
#   which is heavy for a Rust+mkdocs project's pre-push hook. The
#   pragmatic split:
#
#     - mkdocs --strict (via scripts/check-docs-site.sh) catches mermaid
#       SYNTAX errors at build time. Already wired into pre-push.
#     - This script asserts that the artifact JSON proving the SVG
#       renders is PRESENT and valid for every source-mermaid page.
#       Artifacts are produced via the playwright MCP tool (or by an
#       unbiased subagent) and committed to .planning/verifications/.
#     - The actual browser walk is a session-time activity, not a
#       per-commit check. The §0.8 SESSION-END-STATE framework's verdict
#       subagent re-runs `verify` on every claim before the session can
#       declare GREEN.
#
# Exit codes:
#   0  — every source-mermaid page has a current, valid artifact
#   1  — at least one source-mermaid page is missing an artifact OR has
#        an artifact whose svg_counts contains a zero OR whose
#        console_errors is non-empty
#
# Usage:
#   bash scripts/check-mermaid-renders.sh

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly DOCS_DIR="${REPO_ROOT}/docs"
readonly ARTIFACT_DIR="${REPO_ROOT}/.planning/verifications/playwright"

cd "$REPO_ROOT"

failed=0
checked=0

# Find every docs page that source-references a ```mermaid fence.
while IFS= read -r md; do
  rel="${md#docs/}"
  # Map docs/<section>/<slug>.md → .planning/verifications/playwright/<section>/<slug>.json
  section="${rel%/*}"
  base="$(basename "$rel" .md)"
  if [[ "$section" == "$rel" ]]; then
    # Top-level page (e.g. docs/index.md) — section is empty.
    artifact="${ARTIFACT_DIR}/${base}.json"
  else
    artifact="${ARTIFACT_DIR}/${section}/${base}.json"
  fi

  checked=$((checked + 1))

  if [[ ! -f "$artifact" ]]; then
    printf '✖ %s: artifact missing at %s\n' "$rel" "${artifact#$REPO_ROOT/}" >&2
    failed=$((failed + 1))
    continue
  fi

  # Validate the artifact's mermaid_count, svg_counts, console_errors.
  if ! python3 -c "
import json, sys
d = json.load(open('$artifact'))
mc = d.get('mermaid_count', 0)
sv = d.get('svg_counts', [])
ce = d.get('console_errors', [])
if mc == 0:
    sys.exit(0)
if len(sv) != mc:
    print(f'svg_counts length {len(sv)} != mermaid_count {mc}')
    sys.exit(1)
if any(c == 0 for c in sv):
    print(f'svg_counts contains zero: {sv}')
    sys.exit(1)
if ce:
    print(f'console_errors non-empty: {ce}')
    sys.exit(1)
"; then
    printf '✖ %s: artifact %s failed validation\n' "$rel" "${artifact#$REPO_ROOT/}" >&2
    failed=$((failed + 1))
  fi
done < <(grep -rl '^```mermaid' "$DOCS_DIR" --include='*.md' | sed "s|^${REPO_ROOT}/||")

if [[ "$failed" -eq 0 ]]; then
  printf '✓ check-mermaid-renders: %d source-mermaid pages all have valid artifacts.\n' "$checked"
  exit 0
fi

printf '\n✖ check-mermaid-renders: %d/%d pages failed.\n' "$failed" "$checked" >&2
printf '→ Refresh artifacts via the playwright MCP walk (see HANDOVER §0.1 fix path)\n' >&2
printf '  or via an unbiased verifier subagent dispatched per §0.8.D.\n' >&2
exit 1
