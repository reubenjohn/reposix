← [back to index](./index.md) · phase 30 research

## Architecture Patterns

### System Architecture Diagram — docs pipeline (mental model for the planner)

```
                 ┌───────────────────────────────────┐
                 │  Human author / copy subagent     │
                 │    writes + edits .md files       │
                 └──────────────┬────────────────────┘
                                │
                                ▼
     ┌──────────────────────────────────────────────┐
     │  docs/ tree (markdown source of truth)       │
     │   • index.md   ← hero, value prop            │
     │   • mental-model.md, vs-mcp.md               │
     │   • how-it-works/{filesystem,git,trust}.md   │
     │   • guides/{connector,agent,troubleshoot}.md │
     │   • tutorial.md                              │
     │   • reference/{cli,http-api,git-remote,      │
     │        simulator,frontmatter}.md             │
     │   • decisions/, research/ (unchanged)        │
     └──────────────┬───────────────────────────────┘
                    │
       ┌────────────┼────────────────┬─────────────────┐
       │            │                │                 │
       ▼            ▼                ▼                 ▼
 ┌──────────┐ ┌────────────┐  ┌────────────┐  ┌───────────────┐
 │  vale   │ │  mkdocs    │  │ mermaid.js │  │  doc-clarity- │
 │ lint    │ │  build     │  │ (client    │  │  review       │
 │ (git    │ │ --strict   │  │  side in   │  │  (isolated    │
 │  hook + │ │            │  │  browser)  │  │  subprocess,  │
 │  CI)    │ │            │  │            │  │  no context)  │
 └────┬─────┘ └─────┬──────┘ └─────┬──────┘  └──────┬────────┘
      │             │              │                │
      │  fail on    │  fail on     │  renders at    │ verdict:
      │  P2 ban     │  broken      │  page load     │ CLEAR /
      │             │  link / bad  │                │ NEEDS WORK /
      │             │  syntax      │                │ CONFUSING
      │             │              │                │
      ▼             ▼              ▼                ▼
 ┌─────────────────────────────────────────────────────────┐
 │  CI (.github/workflows/docs.yml) — one gate per commit  │
 │      mkdocs gh-deploy on main                           │
 └────────────┬────────────────────────────────────────────┘
              │
              ▼
 ┌──────────────────────────────────────┐
 │ github pages @ reubenjohn.github.io/ │
 │ reposix/                             │
 └──────────┬───────────────────────────┘
            │
            ▼
 ┌──────────────────────────────────────┐
 │  Playwright MCP screenshots          │
 │  (desktop 1280px + mobile 375px) —   │
 │  committed to 30-SUMMARY.md          │
 └──────────────────────────────────────┘
```

**How to read:** source markdown is the single source of truth. Four parallel consumers work on it independently — Vale (lint), mkdocs (build), mermaid.js (runtime render), and doc-clarity-review (cold-reader verdict). CI gates on Vale + mkdocs-strict; the other two are phase-gate artifacts attached to the SUMMARY. The deployment step (`mkdocs gh-deploy`) already exists in `.github/workflows/docs.yml` and needs no changes.

### Recommended docs/ tree structure (post-Phase 30)

```
docs/
├── index.md                           # REWRITTEN — hero, V1 before/after, three-up, complement line
├── mental-model.md                    # NEW — three conceptual keys, 300-400 words
├── vs-mcp-sdks.md                     # NEW — comparison grounding P1
├── tutorial.md                        # NEW — 5-minute against simulator (replaces demo.md role on hero path; demo.md can redirect)
├── how-it-works/                      # NEW SECTION
│   ├── filesystem.md                  # CARVED from architecture.md (read/write paths + inode layout + first mermaid)
│   ├── git.md                         # CARVED from architecture.md (git-remote + optimistic concurrency + second mermaid)
│   └── trust-model.md                 # CARVED from security.md (taint + allowlist + audit + third mermaid)
├── guides/                            # NEW SECTION (promote from connectors/)
│   ├── connect-github.md              # (stub OR placeholder — see Scope Split Decision)
│   ├── connect-jira.md                # (stub OR placeholder)
│   ├── connect-confluence.md          # (stub OR placeholder)
│   ├── custom-fields.md               # (stub — grow post-launch)
│   ├── running-two-agents-safely.md   # (stub — grow post-launch)
│   ├── write-your-own-connector.md    # MOVED from connectors/guide.md (preserved ~460 lines of high-value content)
│   ├── integrate-with-your-agent.md   # NEW — Claude Code / Cursor / SDK patterns
│   └── troubleshooting.md             # NEW STUB — grows post-launch
├── reference/
│   ├── cli.md                         # unchanged
│   ├── http-api.md                    # unchanged
│   ├── git-remote.md                  # unchanged
│   ├── simulator.md                   # NEW — carved from architecture.md + some of http-api.md; dev-tooling framing
│   ├── confluence.md                  # unchanged
│   ├── jira.md                        # unchanged
│   └── crates.md                      # unchanged (may relocate under guides/ or keep in reference — plan decision)
├── decisions/                         # unchanged (Phase 26 correct)
├── research/                          # unchanged (Phase 26 correct)
├── development/                       # unchanged
│   ├── contributing.md
│   └── roadmap.md
├── archive/                           # unchanged (not_in_nav)
├── social/                            # unchanged (hero/demo/benchmark images)
├── screenshots/                       # NEW — add phase-30 landing screenshots here
├── assets/                            # OPTIONAL — if we pre-render mermaid to SVG; probably skip
├── demo.md                            # REDIRECT or delete — superseded by tutorial.md + how-it-works/
├── architecture.md                    # DELETE after content is carved (all content migrates)
├── security.md                        # REWRITE / SHRINK after content is carved — likely becomes a /how-it-works/trust-model.md redirect or a condensed "security index" page
└── why.md                             # REPOSITION — still useful as a "deep dive" explanation page below the fold; link from home; rename if needed
```

**Key IA moves:**

1. `docs/index.md` is the new narrative landing page, not a reference TOC. Current index.md is ~85 lines of TOC-style "what's in the box" — it becomes a Layer 1 hero.
2. `docs/architecture.md` (259 lines of mermaid-heavy content) splits into THREE how-it-works pages. Each carves approximately one-third: read path → filesystem.md, write path + git → git.md, security perimeter → trust-model.md. The original file is deleted after carving.
3. `docs/security.md` (99 lines) becomes the source material for `how-it-works/trust-model.md`. The eight-guardrails table and deferred-items lists may stay in a condensed `docs/security.md` reference page OR migrate into the trust-model page. Recommend: merge everything into trust-model.md and delete security.md. The reference to it from `architecture.md`'s SG-* footer is carved along with the rest.
4. `docs/connectors/guide.md` (465 lines, rich high-value content) moves to `docs/guides/write-your-own-connector.md`. Update internal links in the Phase-11 and Phase-12 references.
5. `docs/demo.md` is obsoleted. The V1-hero + tutorial + how-it-works trio replaces it. Redirect or delete.
6. `docs/why.md` stays — it's the "token economics deep dive" and is Explanation-category. Link from the new vs-mcp-sdks page and from mental-model.md as a "further reading" pointer.

### Pattern 1: Material grid cards (landing-page three-up)
**What:** Material's `grid cards` is an HTML-in-markdown component that renders a responsive card grid. Already used in the current `docs/index.md`.
**When to use:** The three-up value props below the hero; the three-up "where to go next" at the bottom.
**Example (verified from current `docs/index.md`):**
```markdown
<div class="grid cards" markdown>

-   :material-file-document: **[Five-crate workspace](reference/crates.md)**

    `-core`, `-sim`, `-fuse`, `-remote`, `-cli`. 317+ tests. All crates `#![forbid(unsafe_code)]`.

-   :material-shield-lock: **[Eight security guardrails](security.md)**

    SG-01 allowlist · SG-02 bulk-delete cap · ...

</div>
```
**Source:** `docs/index.md` lines 44-62 (existing working pattern); `[VERIFIED: mkdocs-material reference]`.

### Pattern 2: Before/after code block pair for the hero
**What:** Two fenced code blocks stacked, with a prose "before" / "after" framing. Use distinct colors only via material's built-in tab or admonition system — NOT via custom CSS.
**When to use:** The landing hero (Vignette 1) — 30 lines of curl/jq vs 4 commands of file edits.
**Example (constructed from Vignette 1 in the source-of-truth note):**
```markdown
### Before — REST from an agent

```bash
# Transition PROJ-42 to Done, reassign to alice, add a comment.

# 1. Look up the transition ID (Jira uses IDs, not names)
curl -s -u "$E:$T" ...
# ... 5 round trips, 3 ID formats, 30+ lines total ...
```

### After — the same change with reposix

```bash
cd ~/work/acme-jira

sed -i -e 's/^status: .*/status: Done/' \
       -e 's/^assignee: .*/assignee: alice@acme.com/' \
       issues/PROJ-42.md

git commit -am "close PROJ-42" && git push
```

> You still have full REST access for the operations that need it — JQL queries,
> bulk imports, admin config. reposix just means you don't have to reach for it
> for the hundred small edits you'd otherwise make every day.
```

The blockquote under "after" is the **complement line** and is mandatory per the source-of-truth note. Its absence is a plan defect.
**Source:** `.planning/notes/phase-30-narrative-vignettes.md` lines 114-181.

### Pattern 3: Material admonition for the "complement" line
**What:** Consider promoting the complement sentence into a material `!!! note` admonition so it visually anchors under the "after" block. Tradeoff: admonitions are heavy visually; a plain blockquote is subtler and matches the "precise, dry, earned" voice requirement. **Recommend blockquote.**

### Pattern 4: Mermaid diagrams — client-side, already wired
**What:** `mkdocs-material` renders `mermaid` fenced blocks via client-side Mermaid.js, with palette-aware theming. Already configured in `mkdocs.yml` via `pymdownx.superfences.custom_fences`.
**When to use:** One diagram per how-it-works page. Three total.
**Example (verified pattern from existing `docs/architecture.md`):**
````markdown
```mermaid
flowchart LR
  A[LLM agent] -->|"cat / sed / grep"| B[FUSE mount]
  B --> C[git-remote-reposix]
  C -->|HTTP + allowlist| D[Jira REST]
  style A fill:#6a1b9a,stroke:#fff,color:#fff
  style B fill:#00897b,stroke:#fff,color:#fff
```
````
**Render verification workflow (recommended):**
1. Write markdown with ```` ```mermaid ```` block.
2. Run `mkdocs serve` locally (port 8000 by default).
3. Playwright MCP: navigate to `http://127.0.0.1:8000/how-it-works/filesystem/`, take screenshot at 1280px and 375px widths.
4. Visually review for spaghetti edges, overlapping labels, unreadable node text, subgraphs that don't visually group (per user global CLAUDE.md OP #1).
5. Commit screenshots to `docs/screenshots/phase-30/` and link from `30-SUMMARY.md`.

**Alternative (rejected):** Pre-rendering with `mmdc` to SVG in `docs/assets/diagrams/` — adds a build step, loses the palette-sync. Only use if client-side fails somehow (no known reason to expect failure; `mkdocs-material` explicitly supports flowcharts + sequence + class + state + ER diagrams with automatic theming `[VERIFIED: mkdocs-material reference/diagrams]`).

**Source:** `[VERIFIED: mkdocs-material reference documentation]`.

### Anti-Patterns to Avoid

- **TOC-style landing page.** Current `docs/index.md` is a "here's what we have" table of contents. This is an Explanation/Reference opener — it buries the lede. Must be replaced with a narrative hero.
- **Leaking Layer 3 technical terms above the fold.** The banned-word linter enforces this but the author must also not use the banned terms in ALT text or image captions on the landing page. The linter will catch this.
- **Stock photo heroes.** The creative-license notes explicitly ban them. Use mermaid diagrams, code blocks, or no image at all.
- **Feature-grid tables with check marks / marketing bullets.** Explicitly banned. Use grid cards with links and terse sentences — the current index.md pattern is already in the right shape.
- **"Empower" / "revolutionize" / "next-generation."** Explicitly banned. The voice is "precise, dry, earned."
- **Cross-Diátaxis pages.** A "Getting Started" page that mixes conceptual overview with installation steps is the canonical anti-pattern — "should be separated into Explanations and Tutorials" `[CITED: diataxis.fr]`. Mental model (Explanation) and tutorial (Tutorial) stay separate pages.
