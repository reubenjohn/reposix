#!/usr/bin/env bash
# Stub: capture playwright screenshots for landing + how-it-works trio + tutorial.
#
# Status: NOT IMPLEMENTED — Phase 45 deferred screenshot capture to v0.11.0.
# Reason: `mkdocs-material[imaging]` social-card generation requires cairo
# system libraries that are not installed on the dev host (and the host has
# no passwordless sudo). Playwright via the global MCP server is the
# fallback path, but Phase 45's time budget did not cover wiring up the
# 8-screenshot capture script + the assertions on rendered output.
#
# Future contributor: implement against the global playwright MCP server
# (`mcp__playwright__browser_navigate` + `browser_take_screenshot`). Pages
# to capture (DOCS-11):
#   - https://reubenjohn.github.io/reposix/                     (landing)
#   - https://reubenjohn.github.io/reposix/how-it-works/filesystem-layer/
#   - https://reubenjohn.github.io/reposix/how-it-works/git-layer/
#   - https://reubenjohn.github.io/reposix/how-it-works/trust-model/
#   - https://reubenjohn.github.io/reposix/tutorials/first-run/
#
# Each at desktop (1280x800) and mobile (375x667). Output to
# docs/screenshots/{landing,how-it-works,tutorial}/{desktop,mobile}.png.
#
# Tracked in .planning/notes/v0.11.0-doc-polish-backlog.md.

set -euo pipefail

cat <<'EOF'
take-screenshots.sh: not implemented

Phase 45 deferred playwright screenshots to v0.11.0. See script header for
the planned implementation. Until then, this is a stub so the contract
(DOCS-11 success criterion 4) is visible in `git log` rather than implicit.
EOF

exit 0
