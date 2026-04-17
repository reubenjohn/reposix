# Phase 26 CONTEXT — Docs Clarity Overhaul

> Status: scoped in ROADMAP.md. Auto-generated from roadmap entry (no discuss-phase needed — inventory and methodology fully specified).
> Depends on: Phase 25 (OP-11 docs reorg) — SHIPPED.

## Phase Boundary

Every user-facing Markdown document can be understood in isolation by an LLM agent or human contributor arriving cold — no other files read, no links followed. Uses the `doc-clarity-review` skill to run an isolated subagent review on each doc. Also removes stale root-level orphan stubs and aligns version numbers across all pages.

**Scope:** docs-only. No Rust code changes, no API changes, no Cargo bumps.

## Implementation Decisions

### Root-level files — locked actions

| File | Action |
|------|--------|
| `README.md` | Update version references (says v0.3.0, now v0.7.0); update phase table; run clarity review |
| `MORNING-BRIEF.md` | Archive to `docs/archive/` or delete — explicitly v0.1/v0.2 era, obsolete |
| `PROJECT-STATUS.md` | Archive to `docs/archive/` or delete — v0.1/v0.2 timeline, superseded by HANDOFF.md |
| `HANDOFF.md` | Update to reflect v0.7 state (OP items partially closed); run clarity review |
| `AgenticEngineeringReference.md` | Delete (redirect stub — canonical copy at `docs/research/`) |
| `InitialReport.md` | Delete (redirect stub — canonical copy at `docs/research/`) |
| `CHANGELOG.md` | Keep; run clarity check only |

### docs/ pages — clarity review + version fixes where stale

| File | Notes |
|------|-------|
| `docs/index.md` | Says v0.4; now v0.7 — update version + clarity review |
| `docs/architecture.md` | Clarity review |
| `docs/why.md` | Clarity review |
| `docs/security.md` | Clarity review |
| `docs/demo.md` | Clarity review |
| `docs/demos/index.md` | Clarity review |
| `docs/development/contributing.md` | Clarity review |
| `docs/development/roadmap.md` | Stale (stops at v0.5) — update + clarity review |
| `docs/reference/cli.md` | Clarity review |
| `docs/reference/confluence.md` | Clarity review |
| `docs/reference/git-remote.md` | Clarity review |
| `docs/reference/http-api.md` | Clarity review |
| `docs/reference/crates.md` | Clarity review |
| `docs/connectors/guide.md` | Clarity review |
| `docs/decisions/001-github-state-mapping.md` | Clarity review |
| `docs/decisions/002-confluence-page-mapping.md` | Partially stale — clarify scope + clarity review |
| `docs/decisions/003-nested-mount-layout.md` | Clarity review |
| `docs/research/initial-report.md` | Clarity review |
| `docs/research/agentic-engineering-reference.md` | Clarity review |
| `docs/social/twitter.md` | Skip — social/archive, not a doc page |
| `docs/social/linkedin.md` | Skip — social/archive, not a doc page |

### Review methodology (locked)

Each doc is reviewed using the `doc-clarity-review` skill (isolated subagent, zero repo context):
1. Run isolated subagent review on the file in isolation.
2. Collect: friction points, unanswered questions, over-explained sections, under-explained sections, missing references.
3. Having questions about things that are linked elsewhere is usually acceptable — fix is normalizing with links, not repeating content.
4. Fix each doc, then re-run review to confirm CLEAR verdict.
5. Success criterion: zero critical friction points remaining in re-review.

### Claude's Discretion

- Grouping of docs into plans/waves (which docs to batch together)
- Whether to use `doc-clarity-review` skill or spawn inline subagents
- Whether MORNING-BRIEF.md and PROJECT-STATUS.md are archived vs deleted (prefer archive to preserve history)

## Canonical References

- `.planning/ROADMAP.md §Phase 26` — original phase scope definition
- `CLAUDE.md §Quick links` — docs/research/ paths already updated in Phase 25
- `mkdocs.yml` — nav structure; any new archive pages must be excluded from nav
- `docs/research/initial-report.md` — moved here in Phase 25
- `docs/research/agentic-engineering-reference.md` — moved here in Phase 25

## Deferred Ideas

- `docs/social/twitter.md`, `docs/social/linkedin.md` — skip clarity review (social/archive content)
- JIRA docs — not yet written (Phase 27+), not in scope
