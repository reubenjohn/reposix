← [back to index](./index.md)

# Task 4: Compose 30-SUMMARY.md + CHANGELOG.md v0.9.0 entry

<task type="auto">
  <name>Task 4: Compose 30-SUMMARY.md + CHANGELOG.md v0.9.0 entry</name>
  <files>.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-SUMMARY.md, CHANGELOG.md</files>
  <read_first>
    - Each per-plan SUMMARY (01-08) — 8 files to be consolidated
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-09-doc-clarity-feedback.md` (verdict evidence)
    - `/tmp/30-09-gate-summary.txt` (automated-gate timing from Task 1)
    - Existing `CHANGELOG.md` to see the entry format
    - $HOME/.claude/get-shit-done/templates/summary.md
  </read_first>
  <action>
    Step 1 — Compose `.planning/phases/30-.../30-SUMMARY.md`. Template:

```markdown
# Phase 30 — Docs IA and narrative overhaul (v0.9.0) — SUMMARY

**Shipped:** 2026-04-17 (or date of Wave 4 completion)
**Plans:** 9 plans across 5 waves
**Requirements closed:** DOCS-01..09 (all 9)

## What shipped

### Landing page (DOCS-01)
- `docs/index.md` — Layer-1 hero with V1 before/after code pair (Jira ticket-close ceremony vs `sed` + `git push`), mandatory complement-line blockquote, three-up value-props grid, where-to-go-next grid.
- doc-clarity-review verdict: **LANDED** — single-sentence value prop: "<copy from 30-09-doc-clarity-feedback.md>".

### How it works (DOCS-02)
- `docs/how-it-works/filesystem.md` — read path, write path, async bridge, one mermaid sequence diagram.
- `docs/how-it-works/git.md` — git push round-trip, optimistic concurrency, V2 merge-conflict example, one mermaid flowchart.
- `docs/how-it-works/trust-model.md` — lethal trifecta narrative, SG-01..08 table verbatim from security.md, one mermaid security-perimeter diagram (palette: #d32f2f / #ef6c00 / #00897b).

### Concept pages (DOCS-03)
- `docs/mental-model.md` — ~350 words, three locked H2s (`mount = git working tree`, `frontmatter = schema`, `` `git push` = sync verb ``).
- `docs/vs-mcp-sdks.md` — comparison table + P1 paragraph + footnote to `why.md#token-economy-benchmark`.

### Guides (DOCS-04)
- `docs/guides/write-your-own-connector.md` — moved from `docs/connectors/guide.md` via `git mv` (465 lines preserved).
- `docs/guides/integrate-with-your-agent.md` — 4 sections (Claude Code / Cursor / Custom SDK / Gotchas).
- `docs/guides/troubleshooting.md` — 3 symptom/cause/fix triads.
- `docs/guides/connect-{github,jira,confluence}.md` — thin stubs linking to reference + demo scripts.

### Simulator reference (DOCS-05)
- `docs/reference/simulator.md` — CLI flags table + endpoint summary + new "Seeding + fixtures" section.

### Tutorial (DOCS-06)
- `docs/tutorial.md` — 4-step 5-minute run against `reposix-sim`. "Aha" lands in step 4 (server-side version bump via `curl | jq`).
- `scripts/test_phase_30_tutorial.sh` — end-to-end runner; proves the tutorial is accurate on every CI run.

### IA + theme (DOCS-07, DOCS-08)
- `mkdocs.yml` — 11 top-level nav entries per source-of-truth IA sketch; `navigation.tabs` + `navigation.footer` + `content.action.edit` added; `social` plugin registered; `site_description` reframed to Val Town positioning-line.
- Deleted: `docs/architecture.md`, `docs/security.md`, `docs/demo.md`, `docs/demo.transcript.txt`, `docs/demo.typescript`, `docs/demos/`, `docs/connectors/`.
- Updated: `README.md` (Quick Start link).

### Linter (DOCS-09)
- Vale 3.10.0 installed locally + in CI.
- `.vale.ini` + `.vale-styles/Reposix/ProgressiveDisclosure.yml` + `.vale-styles/Reposix/NoReplace.yml` — P1 + P2 enforcement with per-glob scope (how-it-works, reference, decisions, research, development exempt; mental-model.md per-file exception).
- `scripts/hooks/pre-commit-docs` — auto-installed via `scripts/install-hooks.sh`.
- `scripts/hooks/test-pre-commit-docs.sh` — 5 test cases covering clean / replace / FUSE-above / FUSE-below / code-fence-exempt.
- CI `.github/workflows/docs.yml` — Vale step added before `mkdocs build --strict`.

## Automated gates (from Wave 4 verification)

| Gate | Command | Result | Runtime |
|------|---------|--------|---------|
| mkdocs strict | `mkdocs build --strict` | PASS | ~7s |
| Vale | `vale --config=.vale.ini docs/` | PASS | ~2s |
| Structure | `python3 scripts/check_phase_30_structure.py` | PASS | <1s |
| Tutorial e2e | `bash scripts/test_phase_30_tutorial.sh` | PASS | ~18s |

Combined feedback latency: **~28 seconds** (budget was 90s per 30-VALIDATION.md).

## Manual verifications

- Playwright screenshots (14 total): `.planning/phases/30-.../screenshots/`. Reviewed per user CLAUDE.md OP #1 checklist. No critical issues.
- doc-clarity-review cold-reader verdict: LANDED. Evidence: `30-09-doc-clarity-feedback.md`.

## Deferred (follow-ups for future milestones)

- Observability / audit-log deep-dive page.
- "What reposix is not" sidebar.
- Use-case gallery / case studies.
- Expanded comparison covering GraphQL wrappers.

## How to reproduce the verification locally

```bash
cargo build --release --workspace --bins
bash scripts/install-vale.sh
mkdocs build --strict
~/.local/bin/vale --config=.vale.ini docs/
python3 scripts/check_phase_30_structure.py
bash scripts/test_phase_30_tutorial.sh
```

## Screenshots

![Home (desktop)](screenshots/home-desktop.png)
![Home (mobile)](screenshots/home-mobile.png)
... (remaining 12 embedded)
```

Step 2 — Add a `CHANGELOG.md` entry. Append under the existing `[Unreleased]` heading (or promote to `[v0.9.0]`):

```markdown
## [v0.9.0] — 2026-04-17

### Docs & Narrative

- **Landing page rewritten** as a Layer-1 narrative hero: V1 before/after code pair (curl/jq ceremony vs `sed` + `git push`), mandatory complement-line blockquote, three-up value props. Cold-reader verdict: LANDED.
- **"How it works" section** — three new pages (filesystem layer, git layer, trust model), each with one mermaid diagram. Content carved from `docs/architecture.md` + `docs/security.md`.
- **Mental model page** — three conceptual keys (`mount = git working tree`, `frontmatter = schema`, `` `git push` = sync verb ``).
- **reposix vs MCP / SDKs comparison page** — grounds P1 ("complement, not replace").
- **Three new guides** — connector-authoring (moved from `docs/connectors/guide.md`), agent-integration (greenfield), troubleshooting (stub).
- **Simulator page** moved from how-it-works to reference/.
- **5-minute tutorial** against the simulator — self-tested via `scripts/test_phase_30_tutorial.sh`.
- **Nav restructured** per Diátaxis mapping (Home / Why / Mental model / vs-MCP / Tutorial / How it works / Guides / Reference / Decisions / Research / Development).
- **Theme tuned** — `navigation.tabs` + `navigation.footer` + edit/view GitHub pencils + `social` plugin enabled.
- **Banned-word linter** — Vale 3.10.0 with Reposix.ProgressiveDisclosure (P2) + Reposix.NoReplace (P1) rules, CI-gated.

### Removed

- `docs/architecture.md` (carved into how-it-works/)
- `docs/security.md` (carved into how-it-works/trust-model.md)
- `docs/demo.md` (superseded by tutorial.md)
- `docs/demos/`, `docs/connectors/` directories.

### Infrastructure

- `scripts/install-vale.sh`, `scripts/render-mermaid.sh`, `scripts/screenshot-docs.sh`, `scripts/check_phase_30_structure.py`, `scripts/test_phase_30_tutorial.sh` — committed repeatable scripts per CLAUDE.md OP #4.
- `scripts/hooks/pre-commit-docs` — pre-commit Vale gate.
```

Step 3 — Commit everything. Structure the commit sequence:

```bash
# 1. Screenshots
git add .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/*.png

# 2. Phase SUMMARY + doc-clarity evidence
git add .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-SUMMARY.md
git add .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-09-doc-clarity-feedback.md

# 3. CHANGELOG
git add CHANGELOG.md

# Commit with phase-grade message
git commit -m "docs(30): Phase 30 Docs & Narrative ships — v0.9.0 ready to tag

- DOCS-01..09 closed. 9 plans across 5 waves.
- Landing page rewritten (cold-reader verdict: LANDED).
- How-it-works section with 3 pages + mermaid diagrams.
- Mental model, vs-MCP, tutorial, guides filled.
- Vale linter + pre-commit hook + CI integration.
- 14 Playwright screenshots committed.
- CHANGELOG.md updated for v0.9.0 tag.

Phase SUMMARY: .planning/phases/30-.../30-SUMMARY.md"
```

Do NOT run `scripts/tag-v0.9.0.sh` — tag push is a separate user-gated action.
  </action>
  <verify>
    <automated>test -f .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-SUMMARY.md && wc -l .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-SUMMARY.md | awk '{exit !($1 >= 60)}' && grep -c '\[v0.9.0\]' CHANGELOG.md | awk '{exit !($1 >= 1)}' && grep -c 'Docs & Narrative' CHANGELOG.md | awk '{exit !($1 >= 1)}' && grep -c 'DOCS-01..09\|DOCS-0[1-9]' .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-SUMMARY.md | awk '{exit !($1 >= 1)}'</automated>
  </verify>
  <acceptance_criteria>
    - `.planning/phases/30-.../30-SUMMARY.md` exists with `>= 60` lines.
    - 30-SUMMARY.md contains sections for each DOCS-01..09 requirement.
    - 30-SUMMARY.md contains the automated-gate results table with timings.
    - 30-SUMMARY.md references the 14 screenshots by filename.
    - 30-SUMMARY.md records the doc-clarity-review verdict (LANDED or otherwise).
    - `CHANGELOG.md` has a new `[v0.9.0]` entry.
    - CHANGELOG entry mentions "Docs & Narrative" as the milestone name.
    - CHANGELOG entry enumerates deletions (architecture.md, security.md, demo.md, demos/, connectors/).
    - CHANGELOG entry mentions Vale + pre-commit hook infrastructure additions.
    - Git commit message references phase 30 + SUMMARY path.
  </acceptance_criteria>
  <done>
    Phase 30 shipped. CHANGELOG + SUMMARY committed. Tag push remains user-gated per project precedent (scripts/tag-v0.9.0.sh to be authored by roadmap keeper or orchestrator in a follow-up, NOT by this plan).
  </done>
</task>
