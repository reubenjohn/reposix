# Docs-Alignment Design — entry point

> **Source of truth for two new v0.12.0 phases (P64, P65) plus the v0.12.1 gap-closure cluster phases (P71+). Read order matters.**

This bundle was authored 2026-04-28 in a long design session. v0.12.0 is currently `status: ready-to-tag` per `.planning/STATE.md` — **but the tag is held until P64 + P65 ship.** The v0.12.0 narrative changes from "we built quality gates" to "we built quality gates AND they surfaced what we'd silently lost."

## Why this exists

The git-native pivot (v0.9.0) silently dropped behaviors that prior milestones promised — page-tree symlinks for Confluence is the discovered case; there are likely more. They slipped because tests are derived from the implementation (the implementer tests what they just wrote), not from the user-facing surface (docs, REQUIREMENTS.md). The v0.12.0 catalog grades **new commitments**; legacy promises were never lifted into rows. Doc-alignment closes that loop: docs become the spec, an extractor produces a checklist of behavioral claims, tests close the checklist, CI fails when a claim has no test.

## Read order (progressive disclosure)

1. **`01-rationale.md`** — the regression problem in plain words. Read first if context-cold.
2. **`02-architecture.md`** — row state machine, hash semantics, the two project-wide principles ("subagents propose, tools validate" and "fail loud, structured, agent-resolvable"), the catalog schema, the binary surface, the skill layout. **The single most important doc to internalize before touching code.**
3. **`03-execution-modes.md`** — depth-2 constraint, why P65 is NOT a `/gsd-executor` phase, the new ROADMAP marker `Execution mode: top-level | executor`. Read before P65.
4. **`04-overnight-protocol.md`** — deadline 08:00, suspicion-of-haste rule, expected wall-clock per phase, verifier cadence, what to do when stuck.
5. **`05-p64-infra-brief.md`** — implementation spec for P64 (the framework). Consumed by `gsd-planner` to produce `PLAN.md`.
6. **`06-p65-backfill-brief.md`** — implementation spec for P65 (the backfill audit). Top-level execution mode; consumed by the orchestrator directly, not delegated to executor.

## TL;DR for the next agent

You are running `/gsd-autonomous` overnight, deadline 08:00. v0.12.0 has 2 unshipped phases: P64 (build the docs-alignment dimension) and P65 (run the backfill, surface the punch list). After P65, dispatch the milestone-close verifier, run the tag-gate script if GREEN, then **stop** — the human pushes the tag.

v0.12.1 carry-forward updates: existing P64–P68 placeholders renumber to P66–P70 (a mechanical bump). New gap-closure phases (P71+) are templated but their concrete cluster scopes come from P65's punch list — ROADMAP gets stub entries the human will refine post-tag.

**Do not invent design.** Every architectural decision in this bundle was deliberated for hours in the originating session. If you find yourself reaching for a different naming, a different state machine, a different dispatch shape — re-read the relevant doc first. The design is load-bearing for v0.12.1.

## Cross-references

- `.planning/ROADMAP.md` — top-level; P64 and P65 entries land in the v0.12.0 section.
- `.planning/REQUIREMENTS.md` — DOC-ALIGN-* requirements added under v0.12.0 active block.
- `.planning/milestones/v0.12.1-phases/ROADMAP.md` — phases renumbered P66–P70; gap-closure stubs added P71+.
- `quality/PROTOCOL.md` — gains the "fail loud, structured, agent-resolvable" rule with cross-tool examples (P64 deliverable).
- `CLAUDE.md` — gains the "orchestration-shaped vs implementation-shaped phases" note and a `docs-alignment` dimension row in the matrix (P64 deliverable).
