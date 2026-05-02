# v0.11.0 GSD hygiene report

**Auditor:** read-only sweep, 2026-04-25.
**Scope:** STATE.md / PROJECT.md / REQUIREMENTS.md / ROADMAP.md + notes/, research/, phases/, milestones/.
**Goal:** prep `.planning/` for v0.11.0 execution by flagging stale, contradictory, or broken-cross-ref entries that survived the v0.10.0 lifecycle close.

## Summary

- **STATE.md is the worst offender.** Frontmatter says `v0.11.0 / planning_started`, but the body still has v0.8.0-era cursor (line 75 "v0.8.0 complete"), a v0.1.0-era T+3h Blockers section (lines 203–212), a Session-Continuity block pointing at Phase 29 (lines 216–222), and a "Previous session (Phase 14)" block (lines 224–245). At least **6 distinct stale eras** coexist in one file.
- **PROJECT.md "Active" section is fully stale** — it lists every v0.1.0 MVD item as `[ ]` Active even though all of those shipped 2026-04-13 (lines 56–78).
- **REQUIREMENTS.md traceability** still says ARCH-01..18 status `planning` (lines 185–202) although every ARCH ID is checkmarked in PROJECT.md as Validated. ARCH-19 is the only one marked `shipped`.
- **ROADMAP.md milestone bullet** for v0.11.0 (line 15) underspecifies the new scope (doctor / gc / time-travel / Record rename / launch blog) that landed overnight 2026-04-25 per `MORNING-WALKTHROUGH-2026-04-25.md`.
- **One stray phase dir:** `.planning/phases/30-docs-ia-and-narrative-overhaul-...` should be in `.planning/milestones/v0.9.0-phases/` (deferred-then-superseded; CATALOG.md already flagged this).
- **Three "open" carry-forwards already closed:** helper-hardcodes-SimBackend (closed `cd1b0b6`), Record rename (closed `4ad8e2a`), reposix doctor + gc + time-travel (already shipped) — none of these reflect their closed state in PROJECT.md or REQUIREMENTS.md.
- **Cross-references are mostly intact** (audit, latency, testing-targets, tag scripts all exist). One cross-ref to `architecture-pivot-summary.md` is fine but the planner anchor in ROADMAP also points at `mcp-mermaid` + `playwright` for Phase 41/45 success criteria that the audit's "passed" verdict already accepts as deferred — minor wording.

**Counts.** ~35 stale entries; 1 phase dir to archive; 6 explicit contradictions (carry-forward vs audit-verdict-flipped); 0 fully broken file refs (every cited file path resolves).

---

## Chapters

- **[STATE.md issues](./chapter-state-md.md)** — S1–S8: eight issues across six stale eras coexisting in one file.
- **[PROJECT.md and REQUIREMENTS.md issues](./chapter-project-md-requirements-md.md)** — P1–P5 and R1–R4: stale MVD list, closed carry-forwards, and traceability rows stuck at `planning`.
- **[ROADMAP.md, notes/, research/, and phases/ issues](./chapter-roadmap-inventory-phases.md)** — RM1–RM4, notes/research inventories, and PV1–PV3 phase-dir archival.
- **[Cross-reference brittleness, Patch plan, and Recommended execution order](./chapter-cross-ref-patch-plan.md)** — verified path table, prioritized P0–P3 patch list (~22 edits), Wave A/B/C execution order.
