---
phase: 30
plan: 05
type: execute
wave: 2
depends_on: [30-01, 30-02, 30-04]
files_modified:
  - docs/how-it-works/filesystem.md
  - docs/how-it-works/git.md
  - docs/how-it-works/trust-model.md
  - docs/how-it-works/index.md
  - docs/security.md
  - docs/architecture.md
autonomous: true
requirements: [DOCS-02]
must_haves:
  truths:
    - "Each how-it-works sub-page carries exactly ONE mermaid fence (the one placed in plan 30-02's skeleton; no second diagram added)."
    - "docs/how-it-works/filesystem.md prose is carved from docs/architecture.md §Read path + §Write path + §The async bridge — approximately 90 lines total."
    - "docs/how-it-works/git.md prose is carved from docs/architecture.md §git push + §Optimistic concurrency as git merge — approximately 80 lines, concurrency section includes the V2 supporting-vignette merge-conflict snippet."
    - "docs/how-it-works/trust-model.md prose is carved from docs/security.md (all 99 lines absorbed) + docs/architecture.md §Security perimeter; preserves SG-01..08 table rows verbatim with file:line evidence."
    - "docs/how-it-works/index.md bridge paragraph is tightened (no 'Stub' marker remains); reader transitions from Layer 2 to Layer 3 with the 'Under the hood' framing lifted from source-of-truth lines 63-66."
    - "docs/architecture.md and docs/security.md are still present at this point (Wave 3 plan 30-08 deletes them after link audit) but have had a sentinel block added declaring 'content carved into docs/how-it-works/ — this file is scheduled for deletion' so a reader discovering the file during Wave 2 → Wave 3 window knows why."
  artifacts:
    - path: "docs/how-it-works/filesystem.md"
      provides: "Filled filesystem-layer explanation with one mermaid sequence diagram"
      min_lines: 60
    - path: "docs/how-it-works/git.md"
      provides: "Filled git-layer explanation with one mermaid flowchart + merge-conflict example"
      min_lines: 60
    - path: "docs/how-it-works/trust-model.md"
      provides: "Filled trust-model explanation with SG-01..08 table + lethal-trifecta narrative + one mermaid security-perimeter diagram"
      min_lines: 80
    - path: "docs/how-it-works/index.md"
      provides: "Tightened section-landing (stub marker removed)"
      min_lines: 18
  key_links:
    - from: "docs/how-it-works/trust-model.md"
      to: "crates/reposix-core/src/http.rs"
      via: "SG-01 evidence link"
      pattern: "reposix-core/src/http.rs"
    - from: "docs/how-it-works/trust-model.md"
      to: "crates/reposix-remote/src/diff.rs"
      via: "SG-02 evidence link"
      pattern: "reposix-remote/src/diff.rs"
---

# Phase 30 — Plan 05: Carve how-it-works pages

<objective>
Carve docs/architecture.md (259 lines) + docs/security.md (99 lines) into three focused how-it-works pages, each ~60-90 lines with a single mermaid diagram (the placeholder from plan 30-02 — content is now filled around it). After this plan lands:

- `docs/how-it-works/filesystem.md` is a complete Layer-3 Explanation page. The one mermaid fence (placed in plan 30-02) is the canonical sequence diagram. Prose covers the read path, write path, filename-validation boundary, and the async bridge story.
- `docs/how-it-works/git.md` is complete — git push round-trip prose + the one mermaid fence + the V2 supporting-vignette showing the merge-conflict experience (from `.planning/notes/phase-30-narrative-vignettes.md` lines 197-210).
- `docs/how-it-works/trust-model.md` is complete — lethal-trifecta narrative opener + the one mermaid subgraph diagram + SG-01..08 table with evidence paths + deferred-items reference.
- `docs/how-it-works/index.md` has the "Stub" admonition removed; the bridge paragraph is the final version.
- `docs/architecture.md` and `docs/security.md` remain on disk (deletion in Wave 3 plan 30-08 after grep-audit) but each now carries a single sentinel block at the top declaring that content has been carved.

Purpose: DOCS-02 — "How it works" section is the anchor for the Layer-3 reveal. Wave 2's other plan (30-07) fills guides + reference/simulator; this plan is solely about the how-it-works carve.

Output: 3 filled pages, 1 tightened section-landing, 2 source files annotated with sentinel blocks.

**Locked decisions honored:**
- DOCS-02 requires ONE diagram per page. Do NOT add a second diagram. The placeholder from plan 30-02 IS the final diagram.
- Palette convention preserved (trust-model uses #d32f2f / #ef6c00 / #00897b per PATTERNS.md §"Shared Patterns → Mermaid diagrams").
- No new claims in trust-model beyond what's already in docs/security.md (PATTERNS.md §"docs/how-it-works/trust-model.md" — security domain constraint).
- filesystem.md actor naming in diagram: "POSIX" + "reposix" (not "Kernel VFS" + "reposix-fuse") per RESEARCH.md §Example 6 — readable to Layer-2 readers clicking through.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md
@.planning/notes/phase-30-narrative-vignettes.md

@docs/architecture.md
@docs/security.md
@docs/how-it-works/index.md
@docs/how-it-works/filesystem.md
@docs/how-it-works/git.md
@docs/how-it-works/trust-model.md

<interfaces>
Source-material line-number map (from PATTERNS.md §Pattern Assignments — use for precise carving):

- `docs/architecture.md` §"Read path" — lines 82-110
- `docs/architecture.md` §"Write path" — lines 112-140
- `docs/architecture.md` §"git push: the central value prop" — lines 142-167
- `docs/architecture.md` §"Optimistic concurrency as git merge" — lines 169-192
- `docs/architecture.md` §"The async bridge" — lines 194-223
- `docs/architecture.md` §"Security perimeter" — lines 225-254
- `docs/security.md` — all 99 lines (lethal-trifecta opener + SG-01..08 table + deferred items)

The `<action>` sections below reference these ranges; the executor reads the source files and lifts prose with minimal rephrasing. Keep citations intact — file:line pointers in SG table rows are evidence, not decoration.
</interfaces>
</context>

## Chapters

- **[Task 1: Carve filesystem.md + git.md](./task-1.md)** — Fill `docs/how-it-works/filesystem.md` and `docs/how-it-works/git.md` from `docs/architecture.md` sections. Remove "Stub" marker from `docs/how-it-works/index.md`.
- **[Task 2: Carve trust-model.md + add sentinel blocks](./task-2.md)** — Fill `docs/how-it-works/trust-model.md` from `docs/security.md` + `docs/architecture.md §Security perimeter`. Prepend sentinel admonition blocks to `docs/architecture.md` and `docs/security.md`.
- **[Verification, success criteria, and output](./verification.md)** — Threat model, STRIDE register, verification steps, success criteria, and output artifact spec.
