---
phase: 30
plan: 02
type: execute
wave: 0
depends_on: []
files_modified:
  - docs/mental-model.md
  - docs/vs-mcp-sdks.md
  - docs/tutorial.md
  - docs/how-it-works/index.md
  - docs/how-it-works/filesystem.md
  - docs/how-it-works/git.md
  - docs/how-it-works/trust-model.md
  - docs/guides/write-your-own-connector.md
  - docs/guides/integrate-with-your-agent.md
  - docs/guides/troubleshooting.md
  - docs/guides/connect-github.md
  - docs/guides/connect-jira.md
  - docs/guides/connect-confluence.md
  - docs/reference/simulator.md
  - docs/screenshots/phase-30/.gitkeep
  - .planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/screenshots/.gitkeep
autonomous: true
requirements: [DOCS-02, DOCS-03, DOCS-04, DOCS-05, DOCS-06]
must_haves:
  truths:
    - "Every new page required by the Phase 30 IA sketch exists as a stub file with a valid H1 and a locked 'stub — will be authored in Wave N' marker."
    - "Each how-it-works page stub has a placeholder mermaid fence (empty) so the mermaid-count structural check flags missing content rather than missing file."
    - "`docs/how-it-works/index.md` exists so nav entries created in Wave 1's mkdocs.yml update don't dangle."
    - "The three connector stubs (GitHub/Jira/Confluence) exist with 1-paragraph links to existing reference pages — they are not rewritten; they are new thin redirect-like pages honoring the Guides IA."
    - "Phase screenshot directory + public screenshots directory exist with `.gitkeep` so Wave 4 playwright output commits cleanly."
  artifacts:
    - path: "docs/mental-model.md"
      provides: "Skeleton with H1 + 3 locked H2s (mount = git working tree / frontmatter = schema / `git push` = sync verb) + 'Now what' pointer stub"
      min_lines: 20
    - path: "docs/vs-mcp-sdks.md"
      provides: "Skeleton with H1 + comparison table stub + complement-line paragraph stub"
      min_lines: 15
    - path: "docs/tutorial.md"
      provides: "Skeleton with H1 + 4 step H2s + prereq list stub"
      min_lines: 15
    - path: "docs/how-it-works/index.md"
      provides: "Section landing — 1 paragraph transition + grid-cards TOC stub"
      min_lines: 20
    - path: "docs/how-it-works/filesystem.md"
      provides: "Skeleton with H1 + placeholder mermaid fence + 'carved from architecture.md' marker"
      min_lines: 15
      contains: "```mermaid"
    - path: "docs/how-it-works/git.md"
      provides: "Skeleton with H1 + placeholder mermaid fence"
      min_lines: 15
      contains: "```mermaid"
    - path: "docs/how-it-works/trust-model.md"
      provides: "Skeleton with H1 + placeholder mermaid fence"
      min_lines: 15
      contains: "```mermaid"
    - path: "docs/guides/write-your-own-connector.md"
      provides: "Empty file (content will be moved from docs/connectors/guide.md in Wave 2)"
      min_lines: 3
    - path: "docs/guides/integrate-with-your-agent.md"
      provides: "Skeleton with H1 + 4 section stubs (Claude Code / Cursor / Custom SDK / Gotchas)"
      min_lines: 20
    - path: "docs/guides/troubleshooting.md"
      provides: "Skeleton with H1 + 3 symptom/cause/fix stubs"
      min_lines: 15
    - path: "docs/guides/connect-github.md"
      provides: "1-paragraph stub linking to docs/reference/ and demo scripts"
      min_lines: 10
    - path: "docs/guides/connect-jira.md"
      provides: "1-paragraph stub linking to docs/reference/jira.md"
      min_lines: 10
    - path: "docs/guides/connect-confluence.md"
      provides: "1-paragraph stub linking to docs/reference/confluence.md"
      min_lines: 10
    - path: "docs/reference/simulator.md"
      provides: "Skeleton with H1 + dev-tooling framing stub + sections for CLI flags / endpoints / seeding"
      min_lines: 20
  key_links:
    - from: "docs/how-it-works/index.md"
      to: "docs/how-it-works/filesystem.md"
      via: "grid-cards link"
      pattern: "how-it-works/filesystem"
    - from: "docs/how-it-works/index.md"
      to: "docs/how-it-works/git.md"
      via: "grid-cards link"
      pattern: "how-it-works/git"
    - from: "docs/how-it-works/index.md"
      to: "docs/how-it-works/trust-model.md"
      via: "grid-cards link"
      pattern: "how-it-works/trust-model"
---

<objective>
Ship every empty-but-nav-ready page skeleton so that Wave 1's mkdocs.yml nav edit does not dangle. After this plan lands:

- All 14 new pages required by RESEARCH.md §"Recommended docs/ tree structure" exist as committed skeletons.
- Each skeleton is small (~15-30 lines) but contains a valid H1, any LOCKED content (e.g. `docs/mental-model.md`'s three H2s), and a visible "stub — filled in Wave N" breadcrumb.
- `docs/how-it-works/index.md` is a proper section-landing page with a grid-cards TOC pointing to its three sibling pages — no dangling links.
- `docs/screenshots/phase-30/` and `.planning/phases/30-.../screenshots/` directories exist with `.gitkeep`.

Purpose: Nyquist gate — page existence is a prerequisite for `mkdocs build --strict` to work after Wave 1's nav edit. Without skeletons, mkdocs rejects the new nav entries with "file not found." Also: Wave 1 subagents can fill in content in-place without creating files, which narrows their blast radius.

Output: 14 new markdown files (11 content stubs + 1 how-it-works landing + 3 connector stubs), 2 `.gitkeep` files.

**Locked decisions honored:**
- DOCS-03 three conceptual keys appear as H2s verbatim: `mount = git working tree`, `frontmatter = schema`, `` `git push` = sync verb `` (PATTERNS.md §docs/mental-model.md, source-of-truth lines 110-113).
- DOCS-05 simulator page exists under `reference/` (not `how-it-works/`).
- DOCS-02 three how-it-works pages each have one `## ...` section and one placeholder `\`\`\`mermaid` fence so the structural linter's "1 mermaid fence" check can differentiate missing file (fails at page-existence check) vs missing diagram (fails at fence count).
- PATTERNS.md analog mapping preserved: filesystem.md carves from architecture.md, git.md carves from architecture.md, trust-model.md carves from security.md (Wave 2).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/REQUIREMENTS.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/CONTEXT.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md
@.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md
@.planning/notes/phase-30-narrative-vignettes.md

@docs/index.md
@docs/why.md

<interfaces>
Existing mkdocs-material markdown helpers the skeletons rely on (syntax only — no behavior change needed):

- `!!! note` admonition with 4-space-indented body → used to mark stubs.
- `<div class="grid cards" markdown>` block → used in how-it-works/index.md landing.
- ` ```mermaid ` fenced block → placeholder empty diagram (Wave 1 fills).
- `[^N]` footnote syntax → preserved for vs-mcp-sdks.md lethal-trifecta citation.

These are already configured in `mkdocs.yml` markdown_extensions (current lines 45-70). No config change in this plan.
</interfaces>
</context>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Stub files -> mkdocs build | Stubs must not contain malformed frontmatter or unclosed fences; that would break `--strict` in Wave 1. |
| Stub files -> Vale linter | Stubs for Layer-1/2 pages must not contain banned P1/P2 tokens in prose (the linter was installed in plan 01 and runs in CI). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-30-02-01 | Tampering | Stub content leaks banned Layer-3 terms above the fold | mitigate | Wave 0 stubs use only phenomenology language ("file", "folder", "edit", "commit"); `vale --config=.vale.ini docs/` passes on all skeletons except docs/mental-model.md (per-file exception in .vale.ini covers it). |
| T-30-02-02 | Information Disclosure | Stubs accidentally include `process.env.*`, API tokens, real endpoints | accept | Stubs contain no sensitive examples; only placeholder prose. Reviewer scans diff for accidental inclusions. |
| T-30-02-03 | Denial of Service | Malformed frontmatter breaks `mkdocs build --strict` | mitigate | Stubs omit YAML frontmatter entirely (per PATTERNS.md §"Frontmatter on markdown pages" — the repo convention is no YAML frontmatter on docs/ pages). |
</threat_model>

## Chapters

- **[T01 — mental-model, vs-mcp-sdks, tutorial, reference/simulator skeletons](./t01.md)** — Four top-level concept/reference page skeletons with LOCKED content verbatim.
- **[T02 — how-it-works section (index + 3 sub-pages)](./t02.md)** — Landing page with grid-cards TOC and three sub-pages each with a placeholder mermaid fence.
- **[T03 — guides/ stubs + screenshots dirs](./t03.md)** — Six guide stubs and two `.gitkeep` screenshot directories.

<verification>
1. `python3 scripts/check_phase_30_structure.py` exits 1 but only with "deleted paths still present" + "missing mermaid/content" errors (expected until Waves 1/2/3).
2. `mkdocs build --strict` still fails because nav entries for new pages don't exist yet — plan 30-04 (Wave 1 IA) fixes this.
3. `~/.local/bin/vale --config=.vale.ini docs/guides/ docs/how-it-works/ docs/mental-model.md docs/vs-mcp-sdks.md docs/tutorial.md docs/reference/simulator.md` exits 0.
4. `git status --short docs/` shows 14 new files + 2 .gitkeep.
</verification>

<success_criteria>
- 14 new markdown files committed as skeletons with LOCKED content preserved verbatim.
- 2 `.gitkeep` files committed so Wave 4 playwright commits land cleanly.
- Vale passes on all Layer-1/2 pages; exemptions (how-it-works/, mental-model.md per-file) work correctly.
- No dangling links within the new pages (grid-cards in how-it-works/index.md point to the three real sibling stubs).
- Structural linter `check_phase_30_structure.py` reports missing content rather than missing files — sets up Wave 1/2 to fill in place.
</success_criteria>

<output>
After completion, create `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-02-SUMMARY.md` documenting:
- The 14 files and 2 .gitkeep committed
- A "stub markers" grep cheat sheet showing which Wave owns each file
- Confirmation Vale passes and structural linter's remaining-gaps list is exactly the expected set (no surprises)
</output>
