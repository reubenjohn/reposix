#!/usr/bin/env bash
# Required-doc-surfaces verifier.
#
# Asserts that the v0.10 docs-restructure surface (Diataxis layout)
# remains present. This is the contract the catalog's v0.10 docs0X
# rows describe -- specific pages exist at specific paths AND appear
# in mkdocs.yml nav.
#
# Catches the failure mode where someone deletes a page AND its nav
# entry simultaneously: mkdocs-strict only fails on broken links, so
# a coordinated delete (file + nav row) silently slips past the
# strict-build gate. This verifier explicitly enumerates required
# surfaces.
#
# Bound by:
#   planning-milestones-v0-8-0-phases-REQUIREMENTS-md/docs-{01,02,03}
#   planning-milestones-v0-10-0-phases-REQUIREMENTS-md/{
#     docs02-three-page-howitworks, docs03-two-concept-pages,
#     docs04-three-guides, docs05-simulator-relocated,
#     docs08-theme-readme-rewrite, docs11-readme-mkdocs-changelog,
#     mkdocs-nav-diataxis-restructure }
#
# Exit 0 if every required surface exists; 1 with diagnostic per miss.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

required_paths=(
  # v0.8 doc relocations
  "docs/research/initial-report.md"
  "docs/research/agentic-engineering-reference.md"
  # How-it-works pages (v0.10 docs02 shipped 3+ pages; current state is 4
  # pages -- filesystem-layer, git-layer, time-travel, trust-model. The
  # original v0.10 row claim said "architecture / git / simulator" but
  # the section evolved post-shipping; current contract is "Diataxis
  # how-it-works section has multiple deep-dive pages including git-layer".
  "docs/how-it-works/filesystem-layer.md"
  "docs/how-it-works/git-layer.md"
  "docs/how-it-works/time-travel.md"
  "docs/how-it-works/trust-model.md"
  # Two concept pages (docs03)
  "docs/concepts/mental-model-in-60-seconds.md"
  "docs/concepts/reposix-vs-mcp-and-sdks.md"
  # Guides (docs04 said "three guides"; current is write-your-own-connector
  # + troubleshooting + integrate-with-your-agent. security.md was not the
  # third one shipped.)
  "docs/guides/write-your-own-connector.md"
  "docs/guides/troubleshooting.md"
  "docs/guides/integrate-with-your-agent.md"
  # Simulator docs (docs05 -- relocated under reference/, not how-it-works/)
  "docs/reference/simulator.md"
  # Tutorial
  "docs/tutorials/first-run.md"
  # CHANGELOG referenced by README + mkdocs nav (docs11)
  "CHANGELOG.md"
)

fail=0
for p in "${required_paths[@]}"; do
  if [[ ! -f "$p" ]]; then
    echo "MISSING: required doc surface $p" >&2
    fail=1
  fi
done

# Required mkdocs.yml nav structure (Diataxis: How It Works / Concepts /
# Guides / Tutorials / Reference / Development).
# Coarse check: each section header mentioned at least once in nav.
required_nav_sections=(
  "How it works"
  "Concepts"
  "Guides"
  "Tutorials"
  "Reference"
)
for section in "${required_nav_sections[@]}"; do
  if ! grep -q "$section" mkdocs.yml; then
    echo "MISSING: mkdocs.yml nav section '$section'" >&2
    fail=1
  fi
done

# README must reference CHANGELOG.md (docs11 invariant).
if ! grep -qF "CHANGELOG.md" README.md; then
  echo "MISSING: README.md does not reference CHANGELOG.md" >&2
  fail=1
fi

# mkdocs theme must be 'material' (docs08 invariant: themed).
if ! grep -qE '^\s*name:\s*material' mkdocs.yml; then
  echo "MISSING: mkdocs.yml theme is not 'material'" >&2
  fail=1
fi

if [[ $fail -ne 0 ]]; then
  exit 1
fi

echo "OK: ${#required_paths[@]} doc surfaces present + ${#required_nav_sections[@]} Diataxis nav sections + README->CHANGELOG link + material theme."
