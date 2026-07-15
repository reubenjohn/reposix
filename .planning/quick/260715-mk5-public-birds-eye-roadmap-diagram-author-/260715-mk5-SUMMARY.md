---
quick_id: 260715-mk5
title: Public birds-eye roadmap diagram
status: complete
date: 2026-07-15
---

# Quick Task 260715-mk5 — Public birds-eye roadmap diagram

Owner-approved lane (manager w1:p7, 2026-07-15). Delivered the five points of
`.planning/todos/pending/2026-07-15-public-birds-eye-roadmap-diagram.md` in order.

## What was built

1. **`docs/roadmap.md`** — a public bird's-eye roadmap page whose centerpiece is ONE
   color-coded mermaid diagram, registered in the mkdocs nav (`- Roadmap: roadmap.md`,
   right after Home). Arc structure (`graph TB`, top-to-bottom, fan-out-then-converge):
   - **Shipped** (green band) → **Now — Floor** (blue band) → four **Ahead** arcs (grey
     band: stronger quality gates · rebuilt docs and navigation · verified benchmark
     numbers · end-to-end journey walkthroughs) fanning out and **converging** on
     **Public launch** (gold band: live terminal demo · honest headline numbers ·
     one-command install · Show-HN launch kit = the OD-4 launch-readiness pillars).
   - Labelled by **arcs/capabilities only** — no phase numbers, no dates (stays truthful
     without weekly upkeep). Color bands via mermaid `classDef` with explicit dark text
     for light+dark theme contrast.
   - Prose kept to plain vocabulary; passes the reposix-banned-words self-check (no
     `replace`, no plumbing words).

2. **Bi-directional SYNC cross-links.**
   - `docs/roadmap.md` → planning ledger via the **absolute GitHub URL**
     `https://github.com/reubenjohn/reposix/blob/main/.planning/PROJECT.md` with an
     adjacent `<!-- SYNC: ... -->` comment (relative out-of-tree links break
     mkdocs-strict, so GitHub URL is correct here).
   - `.planning/PROJECT.md` → public roadmap via the **repo-relative** `../docs/roadmap.md`
     with a matching `<!-- SYNC: ... -->` comment, placed at the end of the
     `## Current Milestone: v0.15.0 Floor` section.
   - `.planning/PRACTICES.md` OP-9 milestone-close ritual gained ONE line reminding the
     distiller to keep the roadmap↔PROJECT SYNC pair in step and re-color the arc bands
     at milestone close.

3. **link-resolution gate extended to check BOTH directions (catalog-first).**
   - `quality/catalogs/docs-build.json` `docs-build/link-resolution` row `asserts[0]`
     updated FIRST to name the new glob contract (`docs/*.md` + `.planning/PROJECT.md`).
   - `quality/gates/docs-build/link-resolution.py` `DEFAULT_GLOBS` extended (appended
     `docs/*.md` and `.planning/PROJECT.md`; existing entries untouched). This is a
     script-internal default; the ONLY catalog contract that changed is the assert string
     above — no other catalog row, no runner wiring changed.
   - The `.planning/PROJECT.md` → `../docs/roadmap.md` direction is now mechanically
     link-checked; the docs→PROJECT direction is a GitHub URL (skipped by the resolver by
     design, validated as a well-formed external link by mkdocs).

4. **Optional SYNC-marker structure gate → FILED (not built).** A real gate is a
   multi-file structure-dimension add (verifier fn in `freshness-invariants.py` DISPATCH +
   catalog row + `.selftest.sh`), beyond the quick-lane budget. Filed as **GTH-V15-24** in
   `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` with a concrete fix-sketch.

## Verify-against-reality evidence

- **mermaid render-review:** iterated the diagram **3 times** with local `mmdc`
  (playwright's chromium) — v1 (subgraph bands) was a too-wide single horizontal strip
  with invisible inter-band arrows; v2 switched to `graph TB` fan-out-converge; v3 made
  every box one-item-per-line for consistency. Eyeballed v3: no spaghetti edges, no
  overlapping labels, readable node text, color bands clearly grouped, convergence on the
  gold node obvious. (mcp-mermaid MCP server was failed-to-connect; used `mmdc` + the real
  mkdocs page instead.)
- **Playwright render proof:** `mkdocs serve` on 127.0.0.1:8791 → playwright MCP navigated
  to `/reposix/roadmap/`. DOM eval: 1 `.mermaid` container, exactly 1 `<svg>` child
  (viewBox `0 0 1158.234375 814`, width `100%`), all 8 node labels present in the rendered
  SVG, **zero error-level console messages**. Full-page screenshot confirmed the on-page
  render in the real mkdocs-material theme (nav entry present, SYNC comments non-rendering,
  legend renders). Artifact: `.planning/verifications/playwright/roadmap.json`.
- **Gate battery (all exit 0):** `mkdocs-strict` (OK: docs site clean), `mermaid-renders`
  (7 pages all valid artifacts), `link-resolution` (0 broken across 30 files),
  `banned-words-lint` (passed default mode).

## Files changed

- `docs/roadmap.md` (new)
- `mkdocs.yml` (nav: `- Roadmap: roadmap.md`)
- `.planning/verifications/playwright/roadmap.json` (new render proof)
- `.planning/PROJECT.md` (reverse SYNC cross-link)
- `.planning/PRACTICES.md` (OP-9 one-line SYNC-pair reminder)
- `quality/catalogs/docs-build.json` (catalog-first assert update)
- `quality/gates/docs-build/link-resolution.py` (DEFAULT_GLOBS extension)
- `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` (GTH-V15-24 filed)

## Noticing (OD-3)

- **`docs/development/roadmap.md` (the internal snapshot) is stale.** Its "Active milestone"
  section still says v0.11.0 Polish & Reproducibility (PLANNING, Phases 50–55) and its
  shipped table stops at v0.10.0 — it has not been refreshed for v0.11–v0.14. It is
  nav-excluded (`not_in_nav`), so it is not public, but it is a lying doc for anyone who
  opens it. Not in this lane's scope; worth a refresh or a redirect to the new public
  `docs/roadmap.md`. (Left unfiled to avoid scope creep, surfaced here per OD-3.)
- **`GOOD-TO-HAVES.md` is itself over the 20000-char file-size ceiling** (29159 B before
  this lane, now larger) — kept passing only by the `structure/file-size-limits` waiver
  (`--warn-only` until 2026-08-08). Same class as the already-filed GTH-V15-21 (archived
  handovers). The active intake file will need its own progressive-disclosure split soon;
  the back-pointer note at its tail already flags the part-file variant of this.
- **`.planning/REQUIREMENTS.md` cross-milestone index is stale** (per PROJECT.md's own
  Noticing: still lists v0.14.0 as "Active" and a stale v0.13.0 date) — unrelated to this
  lane, already tracked in PROJECT.md.
- **link-resolution `docs/*.md` overlaps `docs/index.md`** — index.md is now read twice
  (cosmetic double-count in the "across N files" total, never a false BROKEN). Documented
  in the code comment; left as-is per the plan (minimal change).

## Discipline

- **NO push performed** — local commits only. L0 orchestrator owns the push + CI verify.
- **Targeted staging by explicit path only** — never `git add -A`/`.` (shared working tree).
- Two stray playwright screenshot PNGs that landed at repo root were deleted (never staged).
