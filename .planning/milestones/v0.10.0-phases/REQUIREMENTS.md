# Requirements — v0.10.0 Docs & Narrative Shine (HISTORICAL)

> **Status:** SHIPPED 2026-04-25. Phases 40–45 closed.
>
> Extracted from the top-level `.planning/REQUIREMENTS.md` on 2026-04-27 during v0.12.0 milestone scaffolding. Convention reference: `CLAUDE.md` §0.5 / Workspace layout.

**Milestone goal:** Make the reposix value proposition land in 10 seconds for a cold reader, with progressive disclosure of architecture and a tested 5-minute first-run tutorial. Sales-ready docs with hard numbers, agent-SDK guidance, and a banned-word linter that enforces P1/P2 framing rules.

**Source of truth:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md`. Framing principles inherited from `.planning/notes/phase-30-narrative-vignettes.md`.

### Validated

- ✓ **DOCS-01**: Reader can understand reposix's value proposition within 10 seconds of landing on `docs/index.md` — Phase 40
- ✓ **DOCS-02**: Three-page "How it works" section (`filesystem-layer`, `git-layer`, `trust-model`) — Phase 41
- ✓ **DOCS-03**: Two home-adjacent concept pages (`mental-model-in-60-seconds`, `reposix-vs-mcp-and-sdks`) — Phase 40
- ✓ **DOCS-04**: Three guides (`write-your-own-connector`, `integrate-with-your-agent`, `troubleshooting`) — Phase 42
- ✓ **DOCS-05**: Simulator relocated from "How it works" to Reference (`docs/reference/simulator.md`) — Phase 42
- ✓ **DOCS-06**: 5-minute first-run tutorial verified by `scripts/tutorial-runner.sh` — Phase 42
- ✓ **DOCS-07**: MkDocs nav restructured per Diátaxis — Phase 43
- ✓ **DOCS-08**: mkdocs-material theme tuning + README hero rewrite — Phase 43 (linter wiring) + Phase 45 (README)
- ✓ **DOCS-09**: Banned-word linter + skill — Phase 43
- ✓ **DOCS-10**: 16-page cold-reader clarity audit; zero critical friction points — Phase 44
- ✓ **DOCS-11**: README points at mkdocs site; CHANGELOG `[v0.10.0]` — Phase 45 (playwright screenshots deferred to v0.11.0 — cairo system libs unavailable on dev host; tracked under POLISH-11 and shipped)

### Carry-forward (closed in v0.11.x)

- Playwright screenshots — closed in v0.11.0 Phase 53 (POLISH-11).
- Helper-hardcodes-SimBackend tech debt — closed in v0.11.0 (commit `cd1b0b6`, ADR-008).
