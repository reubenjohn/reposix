# v0.10.0 Docs & Narrative Shine — ROADMAP archive

**Status:** SHIPPED 2026-04-25 · audit `.planning/v0.10.0-MILESTONE-AUDIT.md` (`status: tech_debt`)
**Phases:** 40, 41, 42, 43, 44, 45
**Phase artifacts:** archived under `.planning/milestones/v0.10.0-phases/`

## Milestone goal

Make the v0.9.0 architecture pivot legible. A cold visitor understands
reposix's value proposition within 10 seconds of landing on the docs site,
runs the 5-minute first-run tutorial against the simulator, and ends with
a real edit committed and pushed via `reposix init` + standard git. The
architecture pivot becomes a *story* (cache layer / git layer / trust
model — three pages, each with one mermaid diagram), not a code change.

## Phases

### Phase 40 — Hero + concepts (DOCS-01, DOCS-03, DOCS-08-half) — SHIPPED

`docs/index.md` rewritten with V1 vignette + 3 measured numbers
(`8 ms` get-issue, `24 ms` `reposix init` cold, `92.3%` token reduction
vs MCP). Above-fold ≤ 250 words. Two home-adjacent concept pages
(`docs/concepts/{mental-model-in-60-seconds,reposix-vs-mcp-and-sdks}.md`)
shipped. README hero rewritten in lockstep (`757416f`).

Verification: `40-VERIFICATION.md`. Status `passed`.

### Phase 41 — How-it-works trio (DOCS-02) — SHIPPED

`docs/how-it-works/{filesystem-layer,git-layer,trust-model}.md` carved
from `docs/architecture.md` + `docs/security.md` + the v0.9.0
architecture-pivot summary. Each page has exactly one mermaid diagram.
P2 layer rules honored: `FUSE`/`fusermount`/`kernel` allowed at Layer 3
(historical references), banned above.

Verification: `41-VERIFICATION.md`. Status `passed`.

### Phase 42 — Tutorial + guides + simulator relocate (DOCS-04, DOCS-05, DOCS-06) — SHIPPED

5-min first-run tutorial (`docs/tutorials/first-run.md`) runnable
end-to-end against the simulator. Three guides:
`write-your-own-connector`, `integrate-with-your-agent` (pointer page;
v0.12.0 ships full recipes), `troubleshooting`. Simulator relocated to
`docs/reference/simulator.md`.

Verification: `42-VERIFICATION.md`. Status `passed`.

### Phase 43 — Nav restructure + theme + banned-words linter (DOCS-07, DOCS-08-half, DOCS-09) — SHIPPED

`mkdocs.yml` Diátaxis-restructured (Home / Concepts / Tutorials /
How it works / Guides / Reference / Decisions / Research). Theme
tuning (indigo + teal palette, navigation features,
`content.code.copy`). `scripts/banned-words-lint.sh` +
`docs/.banned-words.toml` + pre-commit + CI integration +
`reposix-banned-words` skill all committed (`d910ead`, `aa61828`,
`a77925a`).

Verification: no formal `43-VERIFICATION.md`; commits + Phase 44
banned-words-linter-green report serve as evidence. Status `passed`.

### Phase 44 — doc-clarity-review release gate (DOCS-10) — SHIPPED

16 user-facing pages cold-reader audited (`44-AUDIT.md`). 3 critical
findings: 2 fixed in `docs/reference/jira.md` and
`docs/reference/confluence.md` (replaced `reposix mount` blocks with
`reposix init`), 1 escalated to Phase 45 (README rewrite — closed
there). 9 major + 17 minor findings deferred to v0.11.0 backlog
(`.planning/notes/v0.11.0-doc-polish-backlog.md`). Promoted ad-hoc
doc-link checker to `scripts/check_doc_links.py`.

Verification: `44-VERIFICATION.md`. Status `passed`.

### Phase 45 — README + CHANGELOG + screenshots + lifecycle (DOCS-08-half, DOCS-11) — SHIPPED

README rewritten 332 → 102 lines; Tier 1–5 demo blocks (advertising
removed `reposix mount`/`reposix demo`) replaced with v0.9.0-aligned
hero + 5-min quick start + connectors table + project status; every
adjective replaced with a measured number. CHANGELOG `[v0.10.0]` block
finalized. `mkdocs build --strict` green (4 anchor INFOs fixed in this
phase). Playwright screenshots deferred via `scripts/take-screenshots.sh`
stub (cairo system libs unavailable on dev host). Milestone audit
(`.planning/v0.10.0-MILESTONE-AUDIT.md`) written.

Verification: `45-VERIFICATION.md`. Status `passed` (with 2 deferrals).

## Tag gate

`bash scripts/tag-v0.10.0.sh` — NOT YET AUTHORED. Owner gate. v0.9.0
precedent at `scripts/tag-v0.9.0.sh`.

## Carry-forward to v0.11.0

- Playwright screenshots (DOCS-11 success criterion 4) — blocked on
  cairo system libs.
- Helper hardcodes `SimBackend` in `stateless-connect` handler —
  inherited from v0.9.0; resolution before v0.11.0 benchmark commits.
- 9 major + 17 minor doc-clarity findings — backlog in
  `.planning/notes/v0.11.0-doc-polish-backlog.md`.
- `tag-v0.10.0.sh` script — author when owner is ready to tag.
