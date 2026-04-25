#!/usr/bin/env bash
#
# scripts/check-docs-site.sh — POLISH-17 / OP-4 self-improving infrastructure.
#
# Validates the mkdocs documentation site:
#   1. `mkdocs build --strict` must succeed (catches broken cross-refs,
#      missing nav entries, broken admonition syntax).
#   2. The build log must NOT contain the substring "Syntax error in text"
#      — that's mermaid's failure mode (HTML entities inside mermaid code
#      blocks render as a JS-injected error SVG instead of a diagram).
#   3. The rendered HTML must NOT contain "Syntax error in text" either —
#      catches the case where mermaid swallowed the error rather than
#      surfacing it during build (regression for POLISH-03).
#
# Why this script exists:
#
# Two VM-crash incidents back, a single `&lt;id&gt;` HTML entity inside a
# `sequenceDiagram` block leaked an orphan error SVG to every page on the
# site via `navigation.instant`. `mkdocs build` alone did not catch it —
# the failure was in the rendered HTML. This script grep's the rendered
# output as a guardrail. See CLAUDE.md "Docs-site validation" rule.
#
# Run: bash scripts/check-docs-site.sh
# Exit codes:
#   0 — site clean.
#   1 — mkdocs build log emitted "Syntax error in text".
#   2 — rendered HTML contained "Syntax error in text".
#   non-zero from mkdocs — build failed (--strict).

set -euo pipefail
cd "$(dirname "$0")/.."

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

echo "scripts/check-docs-site.sh: running mkdocs build --strict..." >&2
mkdocs build --strict --site-dir "$TMP/site" 2>&1 | tee "$TMP/build.log"

# Catch mermaid syntax errors that mkdocs reports as warnings during build.
if grep -qF "Syntax error in text" "$TMP/build.log"; then
  echo "ERROR: mkdocs build log contains 'Syntax error in text' — likely mermaid HTML-entity bug" >&2
  exit 1
fi

# Catch the case where the error leaked into rendered HTML even though
# the build did not fail (the POLISH-03 regression mode).
if find "$TMP/site" -name '*.html' -exec grep -lF "Syntax error in text" {} + 2>/dev/null | grep -q .; then
  echo "ERROR: rendered HTML contains 'Syntax error in text' — mermaid block needs HTML-entity fix" >&2
  find "$TMP/site" -name '*.html' -exec grep -lF "Syntax error in text" {} + >&2
  exit 2
fi

echo "OK: docs site clean" >&2
