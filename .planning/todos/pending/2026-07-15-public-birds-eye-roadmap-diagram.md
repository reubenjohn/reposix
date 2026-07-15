---
created: 2026-07-15T18:22:22Z
title: Public birds-eye roadmap diagram
area: docs
files:
  - docs/roadmap.md
  - .planning/PROJECT.md
  - quality/gates/docs-build/link-resolution.py
  - .planning/PRACTICES.md
---

## Problem

Owner-approved via manager w1:p7, 2026-07-15. Schedule AFTER P114 — gsd-quick scale;
enter through `/gsd-quick`.

1. New `docs/roadmap.md` added to mkdocs nav — ONE mermaid, color-coded birds-eye view: **shipped arcs / active milestone (v0.15.0 Floor) / future arcs** to the golden end state (OD-4 launch-readiness: asciinema demo, honest headline numbers, install excellence, Show-HN kit). Show **ARCS/CAPABILITIES, NOT phase numbers or dates** (truthful without weekly upkeep).
2. OWNER REQUIREMENT — bi-directional cross-links between `docs/roadmap.md` and `.planning/PROJECT.md`, BOTH sides carrying a non-rendering HTML comment `<!-- SYNC: ... -->` next to the link with concise keep-in-check instructions: edit either side → update the other; re-color at milestone close. ADD that one line to the OP-9 distill checklist in `.planning/PRACTICES.md`.
3. GATES: mkdocs-strict + mermaid-renders must pass; mcp-mermaid render-review before commit (render + eyeball for spaghetti edges / overlapping labels / unreadable text); reposix-banned-words progressive-disclosure layer check applies to the new `docs/` file; mind docs-build/link-resolution when linking `docs/`→`.planning/` (use a GitHub URL if relative links break the gate).
4. OPTIONAL noticing-grade extra: propose a cheap structure-dimension gate row asserting the SYNC marker pair exists on BOTH sides.
5. REQUIRED (manager addendum) — mechanically enforce link-resolution on BOTH directions: today `DEFAULT_GLOBS` in `quality/gates/docs-build/link-resolution.py` excludes top-level `docs/*.md` (only `index.md` by name) AND all `.planning/**`, so both cross-link directions are UNCHECKED. Lane MUST extend `DEFAULT_GLOBS` to cover `docs/*.md` and `.planning/PROJECT.md`. Catalog-first rule applies if a row contract changes.

## Solution

Execute as a `/gsd-quick` lane after P114 closes, delivering the five points above in
order. Concretely:

- Author `docs/roadmap.md` (single color-coded mermaid, arcs/capabilities not phase
  numbers/dates), register it in mkdocs nav, and render-review via mcp-mermaid before
  commit.
- Add the bi-directional `docs/roadmap.md` ↔ `.planning/PROJECT.md` cross-links, each
  carrying the `<!-- SYNC: ... -->` keep-in-check comment, and append the one-line
  SYNC-marker reminder to the OP-9 distill checklist in `.planning/PRACTICES.md`.
- Extend `DEFAULT_GLOBS` in `quality/gates/docs-build/link-resolution.py` to cover
  `docs/*.md` and `.planning/PROJECT.md` so BOTH cross-link directions are mechanically
  link-checked (currently the globs only name `docs/index.md` plus subdirectories and
  exclude all `.planning/**`, so neither direction is checked today — verified against
  the live `DEFAULT_GLOBS` list). Apply the catalog-first rule if a row contract changes.
- Pass mkdocs-strict + mermaid-renders, and the reposix-banned-words
  progressive-disclosure layer check on the new `docs/` file.
- OPTIONAL: propose a cheap structure-dimension gate row asserting the SYNC marker pair
  exists on BOTH sides.
