---
phase: 30
title: "Docs IA and narrative overhaul — landing page aha moment and progressive-disclosure architecture reveal"
depends_on: [phase-29]
status: awaiting-plan
created: 2026-04-17
---

# Phase 30 — Docs IA and narrative overhaul

## Source of truth

The narrative intent, framing principles, hero vignette, supporting vignettes,
IA sketch, and scope fence for this phase are captured in a dedicated
exploration note. **The planner must read this note in full before producing
`30-NN-PLAN.md` files.**

> **Primary input:** `.planning/notes/phase-30-narrative-vignettes.md` (committed
> at `1ba0479`, renamed from `phase-27-*` at Phase 30 addition).

## One-line scope

Rewrite the landing page and restructure the MkDocs nav so reposix's value
proposition lands hard within 10 seconds of a cold reader arriving, with the
technical architecture (FUSE, remote helper, simulator) progressively revealed
in a "How it works" section rather than leaked above the fold.

## Non-negotiable framing principles

1. **Complement, not replace.** reposix does not replace REST APIs; it absorbs
   the ceremony around the 80% of common operations. The word "replace" is
   banned from hero and value-prop copy.
2. **Progressive disclosure — phenomenology before implementation.** Layer 1
   (hero) describes what the user *experiences*; Layer 3 (how-it-works) is
   where FUSE/daemon/helper first appear. Banned terms above layer 3: FUSE,
   inode, daemon, helper, kernel, mount, syscall.

Full layer model, tonal rules, and rationale: see source-of-truth note
§"Framing principles".

## In scope (from source-of-truth note)

- Hero rewrite (landing page, one before/after code block from Vignette 1, three-up value props).
- New "How it works" section split from `docs/architecture.md` into three
  pages, each with one mcp-mermaid diagram (playwright-screenshot verified).
- MkDocs nav restructure per the IA sketch (new Home + new "How it works" +
  promoted "Guides").
- Real 5-minute tutorial against the simulator (first-run narrative).
- mkdocs-material theme tuning (palette, hero features, social cards).
- Banned-word linter enforcing progressive-disclosure layer rules.

## Out of scope

- New features, new CLI surface, new backend connectors.
- Changes to `REQUIREMENTS.md` beyond the phase itself.
- Rewrites of `docs/reference/` or `docs/decisions/` trees — Phase 26 already
  made those correct.

## Verification (per user OP#1: close the feedback loop)

- `mkdocs build --strict` green.
- `playwright` screenshots of landing + how-it-works + tutorial pages at
  desktop and mobile widths — attached to the phase SUMMARY.
- `gh run view` green on CI after push.
- Banned-word linter passes on every doc commit.

## Suggested subagent fan-out (planner to finalize)

1. **Explore** — competitor narrative scan (Linear, Turso, Fly.io, Tailscale,
   Warp, Val Town, Raycast, Stripe): extract one pattern per site that fits
   the hero vignette style.
2. **Copy** — hero + three value props, constrained by the P1 banned-word list.
3. **IA** — two competing nav structures against the sketch, scored against
   Diátaxis + the three personas in the note.
4. **Diagrams** (mcp-mermaid) — three architecture diagrams (filesystem layer,
   git layer, simulator), rendered and playwright-screenshotted.
5. **Tutorial** — authors the 5-minute getting-started path, runs it end-to-end
   against the simulator, screenshots each step.

## Milestone status

Phase 30 was added to Backlog because v0.8.0 was archived and no new milestone
is active. Before planning, the roadmap keeper should either (a) open a new
milestone (e.g. v0.9.0 "Docs & narrative") and promote Phase 30 into it, or
(b) plan Phase 30 as a standalone docs release while deferring the milestone
decision. Planner does not decide this — orchestrator does.
