---
phase: 40
name: Hero + concepts — landing page lands the value prop in 10 seconds
milestone: v0.10.0
status: in-progress
date: 2026-04-24
skip_discuss: true
requirements: [DOCS-01, DOCS-03, DOCS-08]
---

# Phase 40 CONTEXT — landing hero + mental-model + vs-MCP + README hero rewrite

## Phase boundary (from ROADMAP.md)

> **Goal:** Rewrite `docs/index.md` and the README hero so a cold reader states reposix's value prop within 10 seconds. Hero opens with a V1 before/after code block (Jira-close vignette from `.planning/notes/phase-30-narrative-vignettes.md`) and a three-up value-prop grid that cites *measured* numbers from `docs/benchmarks/v0.9.0-latency.md` (`8 ms` get-issue, `24 ms` `reposix init` cold, `9 ms` list-issues, `5 ms` capabilities probe). Add the two home-adjacent concept pages: `docs/concepts/mental-model-in-60-seconds.md` and `docs/concepts/reposix-vs-mcp-and-sdks.md`.

Phase 40 ships:

- `docs/index.md` — landing hero rewrite (V1 vignette + three-up value props + tested-against backends)
- `docs/concepts/mental-model-in-60-seconds.md` — three conceptual keys (clone = snapshot · frontmatter = schema · `git push` = sync verb)
- `docs/concepts/reposix-vs-mcp-and-sdks.md` — positioning table grounding P1
- `README.md` — hero rewrite (numbers, not adjectives) + cross-link to v0.9.0 latency benchmark

## Inputs (already-committed artifacts)

- `.planning/notes/phase-30-narrative-vignettes.md` — V1 hero vignette + framing principles P1/P2 (banned-word list revised for git-native: P2 banned terms above Layer 3 are now `partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2`; the FUSE family is irrelevant since FUSE is deleted)
- `docs/benchmarks/v0.9.0-latency.md` — measured numbers source (init=24ms, list=9ms, get=8ms, patch=8ms, caps=5ms; soft threshold 500ms sim cold init)
- `docs/reference/testing-targets.md` — sanctioned real-backend targets (TokenWorld / `reubenjohn/reposix` / JIRA `TEST`)
- `CLAUDE.md` Architecture (git-native partial clone) section — elevator-pitch wording the README hero mirrors

## Claude's discretion (skip-discuss decisions)

- **Vignette source code.** Use the V1 "close a Jira ticket" before/after from `.planning/notes/phase-30-narrative-vignettes.md` verbatim where possible. Tighten only enough to fit above the fold (≤ 250 words).
- **Three-up value props.** Pick three of: `8 ms get-issue`, `24 ms reposix init cold`, `9 ms list issues`, `5 ms capabilities probe`, `0 MCP tokens before first useful op`, `1 command to bootstrap`. Selection: number-first claims that are most legible to a cold reader. Default trio: `8 ms`, `0 tokens`, `1 command` (mix of latency, token-economy, friction).
- **Linked but-not-yet-existent pages.** `docs/how-it-works/filesystem-layer.md` is a Phase 41 deliverable. The hero links to it as a relative anchor; `mkdocs build --strict` will warn but Phase 40 is not running `--strict` (Phase 45 finalizes that). Leave a TODO comment near the broken link.
- **mkdocs.yml not touched.** Phase 43 owns the nav restructure. Phase 40 only adds new files in existing or new directories — `docs/concepts/` is new; mkdocs default `nav: null` will pick them up under "alphabetical Pages" rendering for now.
- **No diagrams in Phase 40.** Mermaid diagrams + screenshots are Phase 41 (how-it-works trio). Phase 40 is text-only narrative content.
- **README rewrite scope.** Replace adjectives in top 30 lines + Status section + Why section. Keep the v0.9.0 quickstart block as-is (Phase 35 wrote it). Polish wording. The "Demo / Tier 1..5" section below the quickstart is FUSE-era and dies in Phase 45 — leave it for now with a brief disclaimer (the v0.9.0 banner above already alerts readers).
- **CHANGELOG.** Phase 40 does not touch CHANGELOG. Phase 45 is the release-cycle phase.

## Wave sizing hint (for the executor — single-pass, no planner hop)

This phase is small enough that the orchestrator runs it inline rather than through `/gsd-planner` + `/gsd-executor`. Two atomic-commit waves:

- **Wave A — docs/index.md hero + concepts pages.** Three new/rewritten files, one commit per file (`docs(40-01): ...`, `docs(40-02): ...`, `docs(40-03): ...`).
- **Wave B — README.md hero rewrite.** One commit (`docs(40-04): ...`).

Verification (`40-VERIFICATION.md`) runs after both waves, with the goal-backward checks the runner spec calls out:

- `wc -w` above-fold line range of `docs/index.md` ≤ 250.
- `grep -c -iE 'replace|fusermount|fuse|kernel|syscall|daemon' docs/index.md docs/concepts/*.md` returns 0.
- `wc -w docs/concepts/mental-model-in-60-seconds.md` ≤ 350.
- `wc -w docs/concepts/reposix-vs-mcp-and-sdks.md` ≤ 700.
- README hero (top 30 lines) — every adjective dereferences a measured number from `docs/benchmarks/v0.9.0-latency.md` or v0.7.0 token-economy benchmark.

## Success criteria (from ROADMAP.md, condensed)

1. `docs/index.md` hero ≤ 250 words above fold.
2. Word "replace" absent from hero + value props.
3. P2 banned terms (FUSE / fusermount / kernel / syscall / daemon) absent from `docs/index.md` and `docs/concepts/*.md`.
4. `docs/concepts/mental-model-in-60-seconds.md` ≤ 350 words.
5. `docs/concepts/reposix-vs-mcp-and-sdks.md` ≤ 700 words; numbers-table cites `docs/benchmarks/v0.9.0-latency.md` by relative link.
6. README hero rewritten — every adjective sourced to a number.
7. Mention of three real backends (TokenWorld / `reubenjohn/reposix` / JIRA `TEST`) under "Tested against" on `docs/index.md`.

Phase 41 owns success criterion 1 from ROADMAP "mkdocs build --strict green" — Phase 40 does not run `--strict` to avoid blocking on the not-yet-existent `docs/how-it-works/*.md` pages.

## Out of scope (deferred)

- mkdocs.yml nav restructure — Phase 43.
- mermaid diagrams + playwright screenshots — Phase 41 (how-it-works trio) and Phase 45 (final landing-page screenshots).
- Banned-words linter wiring — Phase 43.
- doc-clarity-review release gate — Phase 44.
- README "Demo / Tier 1..5" FUSE-era cleanup — Phase 45.
