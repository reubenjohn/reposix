# Phase 30: Docs IA and narrative overhaul — Research

**Researched:** 2026-04-17
**Domain:** Technical-docs site information architecture + landing-page narrative design for a FUSE/git-remote substrate targeting autonomous LLM agents
**Confidence:** HIGH for stack/tooling/mechanics; HIGH for competitor patterns; MEDIUM for "10-second value-prop validation" (novel validation architecture, borrows from existing doc-clarity-review skill)

## Summary

Phase 30 is a pure docs phase: rewrite the landing page, split `architecture.md` + `security.md` into a three-page "How it works" section, add two home-adjacent concept pages, add three guides, write a 5-minute tutorial against the simulator, retune the mkdocs-material theme, and add a progressive-disclosure banned-word linter. Nine requirements (DOCS-01..09), 8+ new/modified pages, 3 mermaid diagrams, 1 linter, 1 tutorial with playwright screenshot proof.

All the tooling we need is already on the machine: `mkdocs` 1.6.1 + `mkdocs-material` 9.7.1, `CairoSVG` 2.7.1, `pillow` 10.4.0 (so `material[imaging]` social cards work with no new installs), `mmdc` (mermaid-cli) 11.12.0 on `$PATH`, Playwright Chromium already cached, the `doc-clarity-review` skill is in place and is the canonical cold-reader validator. `pymdownx.superfences` with the mermaid custom fence is already wired in `mkdocs.yml`, so diagrams already render client-side.

**Primary recommendation:** Take the IA sketch from the source-of-truth note as-is, and structure the plan along the existing subagent-fanout suggestion (Explore → Copy → IA → Diagrams → Tutorial), with one addition — a linter-build plan that ships Vale with a scoped banned-words rule. Use the existing `doc-clarity-review` skill as the "10-second value-prop lands" verification: point it at the rendered `docs/index.md` with a purpose-built prompt and parse its verdict. Scope-split is NOT recommended — the phase fits in one pass if parallelized as suggested, and the artifacts are mutually reinforcing (copy ↔ diagram ↔ tutorial). A split would create handoff friction.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

The narrative intent, framing principles, hero vignette, supporting vignettes, IA sketch, and scope fence for this phase are captured in a dedicated exploration note — **`.planning/notes/phase-30-narrative-vignettes.md`** (source of truth). The planner must read this note in full before producing plans.

**Non-negotiable framing principles (apply to every task in the plan):**

1. **P1 — Complement, not replace.** reposix does not replace REST APIs; it absorbs the ceremony around the 80% of common operations. The word "replace" is banned from hero and value-prop copy. Tonal words that SHOULD appear: *complement, absorb, subsume, lift, erase the ceremony, no new vocabulary.*
2. **P2 — Progressive disclosure, phenomenology before implementation.** Layer 1 (hero) describes what the user *experiences*; Layer 3 (how-it-works) is where FUSE/daemon/helper first appear. Banned terms above Layer 3: **FUSE, inode, daemon, helper, kernel, mount, syscall.**

**In-scope (source-of-truth note + CONTEXT.md):**
- Hero rewrite on the landing page: above-fold copy, one before/after code block (Vignette 1 "close a Jira ticket"), three-up value props.
- "How it works" section: three new pages (filesystem layer / git layer / trust model), each with one mcp-mermaid diagram, playwright-screenshot verified, content carved from existing `docs/architecture.md` + `docs/security.md`.
- Home-adjacent pages: "Mental model in 60 seconds" (three conceptual keys: *mount = git working tree · frontmatter = schema · `git push` = sync verb*); "reposix vs MCP / SDKs" (comparison grounding P1).
- New Guides: "Write your own connector" (BackendConnector walkthrough — lift/expand `docs/connectors/guide.md`); "Integrate with your agent" (Claude Code / Cursor / SDK patterns — NEW); "Troubleshooting" (stub that grows post-launch — NEW).
- Simulator page relocated from "How it works" to "Reference" (it's dev tooling, not architecture).
- 5-minute tutorial against the simulator (`reposix-sim`, default backend — simulator-first per project OP #1).
- `mkdocs.yml` nav restructure implementing the IA sketch.
- mkdocs-material theme tuning: palette, hero features, social cards.
- Banned-word linter enforcing progressive-disclosure layer rules.

### Claude's Discretion

- **Linter implementation choice** — Vale vs proselint vs custom Python vs pre-commit regex hook. Research below recommends Vale.
- **Mermaid render pipeline** — client-side JS rendering (current setup, already working) vs `mmdc` pre-render to `.svg` in `docs/assets/diagrams/`. Research below recommends client-side with a playwright-screenshot verification step for the rendered page.
- **Landing-page custom template** — keep the existing `docs/index.md` as markdown + material grid cards vs a full `overrides/home.html` Jinja override. Research below recommends **markdown-first with material's `grid cards` component** — the Jinja override path is only needed if we want a parallax/scroll-animation hero, which the creative-license notes do NOT require.
- **Tutorial format** — executable shell snippets (copy-pastable by the reader) vs narrated prose with embedded snippets. Research recommends **executable**, with prose framing each step.
- **Competitor-pattern steals** — which hero conventions to adopt. Research gives 8 candidates; the copy subagent picks 3.

### Deferred Ideas (OUT OF SCOPE)

- New features, new CLI surface, new backend connectors.
- Changes to `REQUIREMENTS.md` beyond the phase itself.
- Rewrites of `docs/reference/` or `docs/decisions/` trees (Phase 26 already made those correct).
- Observability / audit-log deep-dive docs (future milestone).
- "What reposix is not" sidebar (future milestone).
- Use-case gallery / case studies / token-savings measurement updates (future milestone).
- Expanded comparison covering GraphQL wrappers or other filesystem-as-API projects (future milestone).
- Translation / i18n (English only).
- Multi-site deployment (single MkDocs build, existing GitHub Pages target).
- Published binary or crate version bumps beyond `v0.9.0` tag after ship.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DOCS-01 | Value prop lands in 10 seconds (hero vignette + three-up + no "replace") | Competitor pattern scan (§Competitor Narrative Scan); validation via `doc-clarity-review` skill (§Validation Architecture); banned-word linter for "replace" (§Linter Choice) |
| DOCS-02 | "How it works" with three pages + one diagram each, playwright-verified | Mermaid integration is already wired in `mkdocs.yml`; `mmdc` 11.12.0 on `$PATH`; Playwright Chromium cached at `~/.cache/ms-playwright/`; architecture.md source material already exists and is rich (§Source Content Audit) |
| DOCS-03 | Mental model + reposix-vs-MCP/SDKs comparison page | Turso `/concepts` and Stripe quickstart set the length/structure precedent (~300-400 words, card-based); comparison pattern from Tailscale (§Competitor Narrative Scan) |
| DOCS-04 | Three guides — connector, agent integration, troubleshooting | `docs/connectors/guide.md` already exists as rich source for "Write your own connector"; "Integrate with your agent" is greenfield; Troubleshooting is stub |
| DOCS-05 | Simulator page moves to Reference | Mechanical nav edit + file move; content already exists in `docs/architecture.md` + `docs/reference/http-api.md` (§Files to Read/Create/Modify) |
| DOCS-06 | 5-min first-run tutorial against simulator | `scripts/demo.sh` is the canonical demo script — tutorial will narrate its first few steps; simulator is simulator-first per CLAUDE.md OP #1 (§Tutorial Pattern) |
| DOCS-07 | Nav restructured per Diátaxis + P2 banned terms not above Layer 3 | Diátaxis confirmed as the right framework (§Diátaxis Validation); linter enforces P2 (§Linter Choice) |
| DOCS-08 | Theme tuned — palette, hero features, social cards | `material[imaging]` deps (CairoSVG, pillow) already installed; `mkdocs-material` 9.7.1 supports `social` plugin; `grid cards` component ships OOB (§mkdocs-material Tuning) |
| DOCS-09 | Banned-word linter runs on every doc commit | Vale is the recommended tool (§Linter Choice); can be scoped by file glob via `.vale.ini` `[*.md]` blocks; pre-commit hook pattern matches existing `scripts/hooks/` (§Linter Integration) |
</phase_requirements>

## Architectural Responsibility Map

Docs phase — "tiers" are documentation categories rather than runtime architectural tiers. Using Diátaxis.

| Capability | Primary Tier (Diátaxis) | Secondary Tier | Rationale |
|------------|------------------------|----------------|-----------|
| Landing-page hero + value prop | **Narrative/Marketing (sits above Diátaxis)** | Explanation (below fold) | Hero is framing; Diátaxis doesn't cover it — source-of-truth note adds this as "Layer 1/2" ahead of Layer 3 which IS Diátaxis Explanation |
| Mental model in 60 seconds | **Explanation** | — | Propositional knowledge; study-mode, not work-mode |
| reposix vs MCP / SDKs | **Explanation** | — | Comparison grounding; same category as "Why" — explain by contrast |
| How-it-works pages (filesystem / git / trust) | **Explanation** | — | Architecture explanation with diagrams — study-mode |
| Write your own connector | **How-to** | Reference (trait signature) | Work-mode: "I want to add backend X" |
| Integrate with your agent | **How-to** | — | Work-mode: "I want to wire reposix into Claude Code" |
| Troubleshooting | **How-to** | — | Work-mode: "my mount is broken, what now?" |
| 5-minute tutorial | **Tutorial** | — | Guided encounter; learn-by-doing |
| Simulator page | **Reference** | — | Facts about a tool; read when you need to look something up |
| CLI / HTTP API / git-remote / frontmatter schema | **Reference** | — | Unchanged from Phase 26 |
| ADRs | **Reference** (decision records) | — | Unchanged |
| Research docs | **Explanation** | — | Unchanged |

**Key tier-correctness checks for the planner:**
- The tutorial must NOT be a reference. Every step must be an action the reader performs, not a description of a thing.
- How-it-works pages must NOT be how-tos. They describe; they don't prescribe. ("This is how reposix works" — not "here's how to run a FUSE daemon.")
- The connector guide is a How-to that links to Reference (`crates/reposix-core/src/backend.rs`) — the source-of-truth clause in the existing page (`Do NOT read the above and copy it into your adapter's docs — link to ... as the single source of truth`) is already correct and must be preserved.
- Mental model page must NOT become a hero. It's Explanation; it's read in study-mode after the reader has already committed to caring.

## Standard Stack

### Core (already installed, no new installs needed)

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| mkdocs | 1.6.1 | Static site generator | Project standard since v0.1; `docs.yml` CI job already builds with `--strict` |
| mkdocs-material | 9.7.1 | Theme | Already configured; widest community support; palette + hero + grid cards all native |
| mkdocs-material-extensions | 1.3.1 | Material-specific markdown extensions | Dependency of theme; already present |
| mkdocs-minify-plugin | 0.8.0 | HTML/JS/CSS minification | Already configured |
| CairoSVG | 2.7.1 | SVG → PNG for social cards | Already installed; used by `material[imaging]` |
| pillow | 10.4.0 | Image processing for social cards | Already installed; used by `material[imaging]` |
| mmdc (@mermaid-js/mermaid-cli) | 11.12.0 | Mermaid → SVG/PNG CLI | Already on `$PATH` via nvm; used for offline diagram render / verification |
| Playwright Chromium | (cached) | Screenshot verification | Already at `~/.cache/ms-playwright/chromium-1217`; invoked via playwright MCP |

### To install (new)

| Tool | Version | Purpose | Install command | Verify |
|------|---------|---------|-----------------|--------|
| Vale | 3.x (latest) | Prose linter for banned-words rules | `uv tool install vale` OR download binary from `github.com/errata-ai/vale/releases/latest` (there is no PyPI package; Vale is a Go binary) | `vale --version` |

**Version verification note:** Vale is published as a statically-linked Go binary (no Python package). The CI install step should be `curl -L https://github.com/errata-ai/vale/releases/latest/download/vale_<version>_Linux_64-bit.tar.gz | tar xz -C ~/.local/bin vale` OR use `errata-ai/vale-action@latest` GitHub Action. Both approaches `[VERIFIED: vale.sh/docs/install]` and `[VERIFIED: github.com/errata-ai/vale-action]` as of April 2026.

**Installation command for the whole phase (no-op for items already present):**

```bash
# Social cards imaging — already satisfied on this host, included for CI parity:
python3 -m pip install --upgrade "mkdocs-material[imaging]"

# Vale — new:
# Preferred: pin a version to avoid surprise upgrades in CI.
VALE_VERSION=3.10.0  # pick latest at plan time; check https://github.com/errata-ai/vale/releases
curl -L "https://github.com/errata-ai/vale/releases/download/v${VALE_VERSION}/vale_${VALE_VERSION}_Linux_64-bit.tar.gz" \
    | tar xz -C ~/.local/bin/ vale
vale --version
```

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Vale | proselint | proselint is Python-based and simpler but cannot do per-file-glob scoping, which we need (banned terms allowed in `docs/how-it-works/` but not elsewhere). Vale wins on scoping. `[VERIFIED: vale.sh/docs/styles]` |
| Vale | Custom Python regex script | Custom script is ~40 lines and testable via pytest — but it duplicates Vale's `IgnoredScopes` handling (code fences, inline code must be excluded from word matching, or `cat FUSE.md` in a bash snippet would false-positive). Vale's `IgnoredScopes = code, code_block` gets this right out of the box. Custom script wins on zero new install, loses on edge cases. **Recommend Vale**, but if the planner prefers zero-install, a custom script is viable with the IgnoredScopes caveat — see §Linter Choice for the tradeoff analysis. |
| Vale | pre-commit `grep` regex | Works for simple "word X nowhere" — but fails the "word X allowed below layer 3" scoping. Rejected. |
| Client-side mermaid (current) | `mmdc` pre-render to SVG checked into `docs/assets/diagrams/` | Pre-render is more reproducible and works with dark-mode-disabled static snapshots, but costs a build step and loses the "single source of truth in the markdown" virtue. **Recommend keep client-side** — it already works, `mkdocs-material` colors flowcharts/sequence diagrams automatically `[VERIFIED: mkdocs-material diagrams reference]`. |
| Material `grid cards` component | Custom `overrides/home.html` Jinja hero template | Custom template is needed if we want scroll-animation / parallax. The creative-license notes explicitly prefer "precise, dry, earned" over marketing flash. Grid cards + markdown-native admonitions gets us there without Jinja. **Recommend markdown-first.** |

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

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Banned-word scanning with code-fence awareness | Custom regex scanner that tries to strip code blocks before matching | Vale with `IgnoredScopes = code, code_block` in `.vale.ini` | Fence parsing has edge cases (nested, indented, `~~~` vs ```` ``` ````, language tags). Vale handles all of this `[VERIFIED: vale.sh/docs/styles]`. |
| Diagram rendering | Manually drawing SVG in Inkscape and embedding | Mermaid fenced code blocks | Already wired; palette-aware; source-of-truth stays in markdown; `mmdc` CLI exists if offline render is ever needed. |
| Social card generation | Writing OpenGraph/Twitter cards as static image per page | `mkdocs-material[imaging]` social plugin | Auto-generates per page with page title + description; CairoSVG + pillow already installed. |
| 10-second value-prop validation | Writing a custom subagent prompt harness | Existing `doc-clarity-review` skill with a custom `--prompt` | The skill already copies files to `/tmp`, runs Claude subprocess with zero repo context, writes `_feedback.md`. It's designed for exactly this. |
| Cold-reader friction-point discovery | Manual review by author | Same `doc-clarity-review` skill default prompt | Author cannot un-know what they wrote. An isolated subprocess is the only way to model a first-time reader. |
| Screenshot verification | Custom puppeteer scripts | Playwright MCP (already cached at `~/.cache/ms-playwright/`) | MCP invocation is one tool call; no scripting boilerplate. |

**Key insight:** The validation mechanisms this phase needs are ALREADY in our tooling stack. We don't need to invent a "10-second landing page validator" — we point `doc-clarity-review` at the rendered `docs/index.md` with a purpose-built prompt ("state reposix's value proposition in one sentence after reading this page") and check the verdict. The skill's existing `_feedback.md` artifact serves as the phase-gate evidence.

## Runtime State Inventory

This is a docs phase, not a rename / refactor / migration. No runtime state (databases, service configs, OS registrations, secrets) is being renamed or migrated. The phase may move markdown files under `docs/`, which is:

- **Stored data:** None relevant. The only "stored state" touched is markdown files (code edits, not data migrations).
- **Live service config:** None. The `docs.yml` GitHub Actions workflow is parameterized by file paths (`docs/**` + `mkdocs.yml`), which remain stable post-phase. GitHub Pages deploys from `gh-pages` branch regardless of what's in `main/docs/`.
- **OS-registered state:** None.
- **Secrets/env vars:** None. The docs pipeline is unauth to its registry (gh-pages) and uses the default `GITHUB_TOKEN`.
- **Build artifacts:** `site/` (gitignored) is regenerated on every `mkdocs build`. No stale artifacts persist.

**One indirect concern:** old URLs. If we delete `docs/architecture.md` and `docs/security.md` and the old links exist in:
- `README.md` (root) — check and update.
- `CHANGELOG.md` — historical records should NOT be rewritten per CLAUDE.md precedent from Phase 25 ("Historical planning records ... retain old filenames").
- External-to-repo links (social posts, issue comments, blog mentions) — cannot be updated; rely on GitHub's soft 404 on gh-pages.

The planner should include a Wave that does a grep-audit of internal markdown references after the file moves land.

## Common Pitfalls

### Pitfall 1: Vale flagging legitimate technical terms in code blocks
**What goes wrong:** A bash snippet `cat FUSE.md` or a Rust snippet `let mount = ...` gets flagged by the "no FUSE above Layer 3" rule.
**Why it happens:** Fenced code blocks and inline code need to be excluded from prose-linting scope.
**How to avoid:** Set `IgnoredScopes = code, code_block` in `.vale.ini` `[*]` section. Vale's markdown parser understands both fenced (```` ``` ````) and indented code blocks. `[VERIFIED: vale.sh]`.
**Warning signs:** Linter fails on an obviously-legitimate code block; the reviewer has to "just ignore" lint output (training erosion).

### Pitfall 2: `navigation.instant` breaking custom landing page
**What goes wrong:** If we go the Jinja override path for `index.md` (`overrides/home.html`), mkdocs-material's `navigation.instant` feature (enabled in current `mkdocs.yml`) conflicts with the custom template — the client-side router doesn't know about the custom home template.
**Why it happens:** `navigation.instant` SPA-routes between pages using JavaScript, but the home page is a different layout entirely.
**How to avoid:** Either (a) don't use `overrides/home.html` (recommended, use markdown-native), or (b) disable `navigation.instant` if we do go Jinja. `[CITED: alex3305.github.io/docs/selfhosted/mkdocs_material/custom_landing_page]`.
**Warning signs:** Home page looks right on hard refresh but broken when navigated to from another page via JS router.

### Pitfall 3: Mermaid diagrams rendering unstyled in dark mode
**What goes wrong:** Certain diagram types (pie, gantt, user journey, git graph, requirement) don't get auto-theming from mkdocs-material and look terrible in dark mode.
**Why it happens:** mkdocs-material "will currently only adjust the fonts and colors for flowcharts, sequence diagrams, class diagrams, state diagrams and entity relationship diagrams" `[VERIFIED: mkdocs-material reference/diagrams]`.
**How to avoid:** Stick to flowcharts, sequence, class, state, ER. For Phase 30: filesystem layer = flowchart; git layer = sequence or flowchart; trust model = flowchart with subgraphs (same pattern as existing `architecture.md`).
**Warning signs:** A diagram looks fine in light mode but unreadable in dark mode during playwright screenshot verification.

### Pitfall 4: mkdocs `--strict` rejecting orphan pages
**What goes wrong:** New pages created but not linked in `nav:` or any other page fail `mkdocs build --strict`.
**Why it happens:** `--strict` treats unreferenced pages as warnings by default in recent versions; some configs escalate to errors.
**How to avoid:** Every new page must be in `nav:` when committed. Alternatively, use `not_in_nav: |` for intentionally unlisted pages (like the archive/ pattern in the current config).
**Warning signs:** Local `mkdocs serve` works fine but CI `mkdocs build --strict` fails.

### Pitfall 5: Banned-word linter false positives on "mount" as generic verb
**What goes wrong:** "mount" is on the P2 ban list — but it's a generic English verb ("You'll mount an argument against this approach"). If we use it generically, the linter false-positives.
**Why it happens:** Context-free word matching can't distinguish technical from conversational.
**How to avoid:** (a) Don't use "mount" generically anywhere above Layer 3 — rephrase. (b) If a false-positive does slip in, use Vale's `[vale-comment] mount [/vale-comment]` inline ignore or structure the prose to avoid ambiguity. **Recommend (a).** This is a discipline check, not a technical problem.
**Warning signs:** Copy review reveals "mount" used in a non-technical sense; linter doesn't know what you meant.

### Pitfall 6: Social cards plugin failing in CI due to missing system fonts
**What goes wrong:** `material[imaging]` requires fonts (often DejaVu, Roboto, or a configurable custom font). GitHub Actions `ubuntu-latest` has these, but if we customize fonts in the mkdocs config and they aren't in the CI image, cards fail silently or use fallbacks.
**Why it happens:** CairoSVG + pillow need font files at runtime.
**How to avoid:** Use default fonts (Roboto ships with material); if customizing, add `sudo apt-get install -y fonts-<your-font>` to the docs workflow. `[CITED: mkdocs-material plugins/requirements/image-processing]`.
**Warning signs:** Social cards look wrong in prod but fine locally.

### Pitfall 7: Deleting `docs/architecture.md` before verifying no cross-links remain
**What goes wrong:** `docs/security.md`, `docs/index.md`, `docs/why.md`, `docs/connectors/guide.md`, `docs/demos/index.md`, and several ADRs all link to `docs/architecture.md#some-anchor`. Deleting it breaks `mkdocs build --strict`.
**Why it happens:** The strict build fails on dangling internal links.
**How to avoid:** Before deletion, grep `docs/` for `architecture.md` references, then decide per-reference: (a) update to new `how-it-works/*.md` anchor, (b) update to `why.md` anchor if content moved there, or (c) delete the reference. Same approach for `docs/security.md`.
**Warning signs:** `mkdocs build --strict` explodes in the last wave.

## Code Examples

### Example 1: `.vale.ini` for progressive-disclosure layer rules

Create at repo root:

```ini
# .vale.ini
StylesPath = .vale-styles
MinAlertLevel = warning

Vocab = Reposix

# Exclude code from prose linting (critical — avoids flagging bash snippets)
IgnoredScopes = code, code_block

# By default, apply the banned-word rule to all markdown
[*.md]
BasedOnStyles = Reposix

# ESCAPE HATCH: how-it-works/, reference/, decisions/, research/ are Layer 3+
# and MAY use the banned terms. Opt-out per-glob.
[docs/how-it-works/**]
Reposix.ProgressiveDisclosure = NO
[docs/reference/**]
Reposix.ProgressiveDisclosure = NO
[docs/decisions/**]
Reposix.ProgressiveDisclosure = NO
[docs/research/**]
Reposix.ProgressiveDisclosure = NO
[docs/development/**]
Reposix.ProgressiveDisclosure = NO

# The hero-ban on "replace" is a different rule, applied EVERYWHERE on the landing
[docs/index.md]
BasedOnStyles = Reposix
Reposix.NoReplace = YES
```

And the rule file `.vale-styles/Reposix/ProgressiveDisclosure.yml`:

```yaml
extends: existence
message: "P2 violation: '%s' is a Layer 3 term — banned on Layer 1/2 pages (index, mental-model, vs-mcp-sdks, tutorial, guides, home-adjacent). Move it into docs/how-it-works/ or rephrase in user-experience language."
level: error
scope: text
ignorecase: true
tokens:
  - FUSE
  - inode
  - daemon
  - \bhelper\b       # boundaries — "Jupyter helper" generic is fine, but flag bare
  - kernel
  - \bmount\b        # noun/verb — flag any bare occurrence; authors rephrase
  - syscall
```

And `.vale-styles/Reposix/NoReplace.yml`:

```yaml
extends: existence
message: "P1 violation: 'replace' is banned in hero/value-prop copy. Use 'complement, absorb, subsume, lift, erase the ceremony' instead."
level: error
scope: text
ignorecase: true
tokens:
  - replace
  - replaces
  - replacing
  - replacement
```

**Source:** Constructed from `[VERIFIED: vale.sh/docs/styles]` + source-of-truth note §P1/P2.

### Example 2: Pre-commit hook (follows existing `scripts/hooks/` pattern)

`scripts/hooks/pre-commit-docs`:

```bash
#!/usr/bin/env bash
# Doc-lint gate — runs Vale on docs/**.md.
# Mirrors scripts/hooks/pre-push pattern (HARD-00).

set -euo pipefail

CHANGED=$(git diff --cached --name-only --diff-filter=ACM | grep -E '^docs/.*\.md$' || true)

if [ -z "$CHANGED" ]; then
    exit 0
fi

if ! command -v vale >/dev/null 2>&1; then
    echo "error: vale not installed. See .planning/phases/30-.../RESEARCH.md §Standard Stack." >&2
    exit 1
fi

echo "==> Vale lint on $CHANGED"
echo "$CHANGED" | xargs vale --config=.vale.ini
```

Install via `scripts/install-hooks.sh` — matches existing pattern for credential pre-push hook (Phase 21 HARD-00).

### Example 3: GitHub Actions step for Vale in docs.yml

Addition to `.github/workflows/docs.yml` before the `mkdocs build --strict` step:

```yaml
      - name: Install Vale
        run: |
          VALE_VERSION=3.10.0  # pin; bump deliberately
          curl -L "https://github.com/errata-ai/vale/releases/download/v${VALE_VERSION}/vale_${VALE_VERSION}_Linux_64-bit.tar.gz" \
              | tar xz -C /usr/local/bin vale
          vale --version

      - name: Lint docs with Vale (banned-word + progressive-disclosure rules)
        run: vale --config=.vale.ini docs/

      - name: Build strict (fail on broken links)
        run: mkdocs build --strict
```

### Example 4: doc-clarity-review invocation for 10-second value-prop validation

The existing skill can be invoked with a custom prompt. Phase 30's "did the value prop land" validation:

```bash
# From phase-gate verification step (post-build, pre-merge)
VERDICT_DIR=$(mktemp -d /tmp/phase-30-value-prop-XXXXXX)

# Render the built home page to plain text (or just use source markdown)
cp docs/index.md "$VERDICT_DIR/"

# Purpose-built prompt (vs default clarity prompt)
cat > "$VERDICT_DIR/_prompt.md" <<'EOF'
You are reading this page for the first time, completely cold. You have NOT
seen the repo, any other documentation, or this project's GitHub.

Your job: after reading the page, tell me in EXACTLY ONE SENTENCE what
reposix is and what problem it solves. Then rate:

- LANDED — you got it from the content alone
- PARTIAL — you got a rough idea but are missing the pivotal point
- MISSED — you'd need to read more to answer

Then: identify the single sentence or image on this page that did the most work.
If you can't identify one, that's itself a verdict.
EOF

claude -p "$(cat $VERDICT_DIR/_prompt.md)" "$VERDICT_DIR/index.md" 2>&1 | tee "$VERDICT_DIR/_feedback.md"
# Parse verdict: grep -i 'LANDED' && declare pass, else fail
```

**Source:** `[VERIFIED: ~/.claude/skills/doc-clarity-review/SKILL.md]` — skill already exists, exactly fits the need.

### Example 5: Playwright screenshot verification (invoked via MCP)

Not literal shell — invoked from the planner / executor via Playwright MCP tools. The pattern:

1. Start `mkdocs serve` in background.
2. Playwright MCP → `browser_navigate` → `http://127.0.0.1:8000/` → `browser_take_screenshot` with viewport 1280×800 and 375×667.
3. Save to `docs/screenshots/phase-30/home-desktop.png` and `home-mobile.png`.
4. Repeat for `/how-it-works/filesystem/`, `/how-it-works/git/`, `/how-it-works/trust-model/`, `/tutorial/`.
5. Visual review for: spaghetti edges, overlapping labels, unreadable node text, contrast, layout, mobile width. Per user global CLAUDE.md OP #1.
6. Commit screenshots; reference from `30-SUMMARY.md`.

### Example 6: Mermaid diagram specs for the three how-it-works pages

**how-it-works/filesystem.md diagram — adapt from existing architecture.md sequence diagram:**

````mermaid
sequenceDiagram
  autonumber
  actor A as You (shell or agent)
  participant K as POSIX
  participant F as reposix
  participant R as Your tracker
  A->>K: cat issues/PROJ-42.md
  K->>F: read("PROJ-42.md")
  F->>R: GET /issue/PROJ-42
  R-->>F: issue JSON
  F-->>K: frontmatter + markdown bytes
  K-->>A: text
````

Keep "FUSE" / "kernel" / "daemon" out of the actor names — use "POSIX" and "reposix" (Layer 3 is where these terms may appear in prose, but diagram LABELS should stay readable to Layer 2 readers who land on the page from the index).

**Source:** derived from current `docs/architecture.md` "Read path" mermaid, simplified for Layer 3 entry-point.

**how-it-works/git.md diagram — the git push round-trip (adapt from architecture.md):**

````mermaid
flowchart LR
  A["Your commit<br/>status: Done"] -->|git push| B["reposix<br/>(diff + dispatch)"]
  B -->|PATCH + If-Match| C["Your tracker"]
  C -->|200 or 409| B
  B -->|success or conflict markers| A
````

**how-it-works/trust-model.md diagram — adapt from architecture.md "Security perimeter":**

````mermaid
flowchart LR
  subgraph Tainted["Tainted (attacker-influenced)"]
    BODY["Issue bodies"]
    TITLE["Titles / comments"]
  end
  subgraph Trusted["Trusted (yours)"]
    CORE["reposix core<br/>sealed HttpClient"]
    AUDIT["append-only<br/>SQLite audit log"]
  end
  subgraph Egress["Egress (allowlisted only)"]
    NET["HTTP → allowlisted origin"]
    DISK["Filesystem writes<br/>(mount point only)"]
  end
  BODY -.->|typed as Tainted&lt;T&gt;| CORE
  TITLE -.->|typed as Tainted&lt;T&gt;| CORE
  CORE -->|sanitize + allowlist| NET
  CORE --> AUDIT
  style Tainted fill:#d32f2f,stroke:#fff,color:#fff
  style Egress fill:#ef6c00,stroke:#fff,color:#fff
  style Trusted fill:#00897b,stroke:#fff,color:#fff
````

**Source:** adapted from current `docs/architecture.md` "Security perimeter" diagram.

## Competitor Narrative Scan

Patterns extracted per the source-of-truth note's explicit call-out (subagent #1). The copy subagent picks 3 of these; this is the menu.

### Pattern A — Linear's carousel-of-product-screenshots hero (reject for us)
**What:** Linear's home (`linear.app`) leads with "The product development system for teams and agents" + "Purpose-built for planning and building products. Designed for the AI era" + three carousel-style product interface screenshots. `[CITED: linear.app homepage analysis]`.
**Asymmetric before/after:** Appears below fold in the "Diffs" feature section — an old `HomeScreen.tsx` vs new progressive-loading version.
**"No new vocabulary" equivalent:** None — Linear leans into proprietary vocabulary ("Issues", "Cycles", "Initiatives") and markets it.
**Diátaxis nav:** Product → Customers → Pricing → Method (marketing-structured, not docs-structured). Linear's docs live at a separate subdomain.
**Why we reject:** Marketing-heavy, product-screenshot-led. Our hero is a code-block before/after, not a product dashboard.
**What we steal:** Nothing directly, but the "terse headline + declarative subline" pattern is solid. Linear's hero is one line.

### Pattern B — Fly.io's "Build fast. Run any code fearlessly." (STEAL the cadence)
**What:** Hero is a two-beat tagline — "Build fast. Run any code fearlessly." — plus a three-pillar breakdown (Machines / Sprites / Storage). No before/after code. `[CITED: fly.io homepage analysis]`.
**Asymmetric pattern:** Alternating text-block and image-block sections down the page. No code in hero.
**"No new vocabulary" equivalent:** Uses "Machines" (not "VMs") and "sandboxes" (not "containers") — accessible language for familiar concepts.
**Diátaxis nav:** Docs separate at `fly.io/docs/` with a clean Get Started / Guides / Reference split.
**Why it's relevant:** Fly.io's voice is "developer-friendly without being overly promotional" — closest to our "precise, dry, earned" voice requirement.
**What we steal:** The two-beat tagline cadence. Candidate for our hero: *"Close a ticket with `sed` and `git push`. Keep your REST API for the 20% that needs it."* (Under 20 words, echoes V1 vignette, includes complement.)

### Pattern C — Warp's agentic framing (align on positioning)
**What:** "Warp is the agentic development environment" + dual offering (Terminal + Oz). Lead with "700K+ developers" trust signal. `[CITED: warp.dev homepage analysis]`.
**Asymmetric pattern:** No before/after. Leads with brand authority.
**"No new vocabulary" equivalent:** Leans into "agentic" as a new term — bet that the AI developer audience has adopted the word.
**Diátaxis nav:** Standard docs nav.
**Why it's relevant:** Warp's audience is the same as ours — AI-agent-adjacent developers. They've validated that this audience accepts "agentic" as a hero-level term.
**What we steal:** Nothing for the hero (we don't want trust signals up top — too early). BUT — "agentic" or "autonomous agents" is safe above the fold, unlike "MCP" which is narrower.

### Pattern D — Val Town's "Zapier for know-code engineers" (STEAL the positioning-line technique)
**What:** Hero is "Instantly deploy" + the positioning line "Zapier for know-code engineers." Live editable code block beneath. `[CITED: val.town homepage analysis]`.
**Asymmetric pattern:** No before/after, but live code in hero.
**"No new vocabulary" equivalent:** "know-code engineers" is coinage — a bet that audience self-identifies with a new label.
**Diátaxis nav:** Less rigorous; docs are intermixed with blog.
**Why it's relevant:** The "X for Y-who-do-Z" positioning line is POWERFUL. One sentence tells you who it's for and what it substitutes for.
**What we steal:** The positioning-line technique. Candidate: *"Common tracker ops for autonomous agents. `cat` instead of curl, `git push` instead of PATCH."*

### Pattern E — Raycast's "Your shortcut to everything." (terse-line discipline)
**What:** Hero is "Your shortcut to everything." + subhead + Download buttons. Static keyboard image below. `[CITED: raycast.com homepage analysis]`.
**Asymmetric pattern:** No before/after. No code. All keyboard-icon visuals.
**"No new vocabulary" equivalent:** Extreme terseness. "Shortcut to everything" is four words.
**Diátaxis nav:** Standard docs.
**Why it's relevant:** Demonstrates the upper bound of hero terseness. A very confident product can get away with a four-word hero.
**What we steal:** The discipline. If we can say it in four words we should. But: our audience needs proof, not polish — the four-word hero only works for Raycast because their product is a launcher, instantly graspable. Ours is not. We keep terseness discipline but add a code-proof block.

### Pattern F — Turso concepts page (steal for mental-model.md length/shape)
**What:** Turso's `/concepts` landing lands as a hierarchical card layout, ~350-400 words, no code examples, no diagrams. `[CITED: docs.turso.tech/concepts analysis]`.
**Why it's relevant:** Sets the length precedent for our `mental-model.md` — keep it short, card-based, readable in one sitting.
**What we steal:** The 300-400 word ceiling. Mental model in 60 seconds is a READING target, not a scrolling target.

### Pattern G — Stripe quickstart's first "aha" moment (steal for tutorial.md structure)
**What:** Stripe's dev quickstart (`docs.stripe.com/development/quickstart`) has 6 structured sections. The "aha" lands in section 4: **after running `stripe products create ...`, the CLI returns a JSON response with a real `prod_LTenIrmp8Q67sa` ID**. The reader sees the API respond before writing any application code. `[CITED: docs.stripe.com/development/quickstart analysis]`.
**Why it's relevant:** This is the pattern for our 5-minute tutorial. The reader edits `issues/PROJ-42.md`, runs `git push`, and then `curl`s the simulator to see the version bumped from 1 → 2. The "aha" is the server-side confirmation that the file-edit flowed all the way through.
**What we steal:** The "confirm setup" step. After the git push, print the `curl | jq` that proves the state change landed. This mirrors existing demo.md step 7 and makes the tutorial feel like a demonstrable closed loop.

### Pattern H — Cloudflare Workers "Get Started" (steal for tutorial length)
**What:** 4 numbered sections, 70% prose / 30% code, reader runs first command (`npm create cloudflare@latest`) in the first major section. `[CITED: developers.cloudflare.com/workers/get-started/guide analysis]`.
**Why it's relevant:** Length precedent for a tutorial. "5-minute" is a promise about reading + doing time, not a hard word count, but 4 steps / ~500 words feels right.
**What we steal:** The 4-step structure. Our tutorial: (1) start the simulator, (2) mount it, (3) edit an issue, (4) git push and verify.

### Menu-style recap — which 3 to steal

The copy subagent should pick 3 of these. Research recommendation: **B (Fly.io two-beat cadence) + D (Val Town positioning-line) + G (Stripe "confirm setup" moment in tutorial)**. B + D shape the hero; G shapes the tutorial. Each is citable. None requires inventing a new pattern.

## Diátaxis Validation of the Proposed IA

The source-of-truth note's IA sketch holds up well against Diátaxis — one flag, no blockers.

| Proposed page | Diátaxis category | Validation |
|---------------|-------------------|------------|
| Home (index.md) | (narrative hero, above Diátaxis) | PASS — Diátaxis explicitly does not cover marketing/landing pages. |
| Why reposix | Explanation | PASS — "why" is Explanation by definition. |
| Mental model in 60 seconds | Explanation | PASS — propositional, study-mode. |
| reposix vs MCP / SDKs | Explanation | PASS — comparison is a form of Explanation. |
| Try it in 5 minutes | Tutorial | PASS — guided first encounter. |
| How it works / filesystem.md | Explanation | PASS — propositional architecture. |
| How it works / git.md | Explanation | PASS. |
| How it works / trust-model.md | Explanation | PASS. |
| Connect to GitHub / Jira / Confluence | How-to | PASS — working developer, specific goal. |
| Write your own connector | How-to | PASS — working developer, specific goal. Has a Reference dependency (the trait). |
| Integrate with your agent | How-to | PASS — work-mode. |
| Running two agents safely | How-to | PASS. |
| Custom fields and frontmatter | How-to | **FLAG** — could cross into Reference. See note below. |
| Troubleshooting | How-to | PASS — specific problem, specific fix, in work-mode. |
| Reference / CLI | Reference | PASS. |
| Reference / HTTP API | Reference | PASS. |
| Reference / Simulator | Reference | PASS — it's dev tooling, user looks up flags. |
| Reference / git-remote-reposix | Reference | PASS. |
| Reference / Frontmatter schema | Reference | PASS — describes the schema. |
| Decisions (ADRs) | Reference (decision records) | PASS. |
| Research | Explanation | PASS — study-mode long-form. |

**One flag — "Custom fields and frontmatter":** The phrase "how to use custom fields" is How-to, but if the page ends up describing the YAML schema field-by-field it drifts into Reference. **Recommendation:** split into two pages if content grows — a short how-to guide ("how do I add a custom field to my issue?") and a Reference page ("the frontmatter schema"). For Phase 30, ship as a How-to stub; let post-launch usage drive the split.

**Framework citation:** `[CITED: diataxis.fr/start-here]`.

## Mental Model in 60 seconds — format guidance

Per competitor pattern F (Turso) and the source-of-truth note's explicit three-key enumeration (*mount = git working tree · frontmatter = schema · `git push` = sync verb*):

**Recommended structure:**
- ~300-400 words total (Turso /concepts precedent)
- Three H2 sections, one per conceptual key
- Each section: 1 sentence of equation, 1-2 sentences of explanation, 1 code snippet (3-5 lines)
- Zero diagrams — diagrams are the payoff of how-it-works, not the setup
- End with a "Now what" block pointing to `/tutorial/` or `/how-it-works/`

**Source-of-truth key phrasings (use verbatim):**
1. "**mount = git working tree**"
2. "**frontmatter = schema**"
3. "**`git push` = sync verb**"

These three phrasings are locked; the planner and copy subagent do not re-derive them.

## Tutorial Pattern — 5-minute first run

**Recommended structure (per Pattern H — Cloudflare Workers):**

1. **Prerequisites (30 seconds).** One bullet list: `cargo`, `fuse3` (Linux), `git`, `curl`, `jq`. No paragraphs.
2. **Step 1: Start the simulator (1 min).** `target/release/reposix-sim --bind 127.0.0.1:7878 --seed-file ...`. Show the expected output. `curl /healthz` to verify.
3. **Step 2: Mount the tracker as a folder (1 min).** `reposix mount /tmp/reposix-mnt --backend http://127.0.0.1:7878`. `ls /tmp/reposix-mnt/issues/`.
4. **Step 3: Edit an issue (1 min).** `cat` the issue. Use `printf > file` (not `sed -i` — per existing demo.md step 6 guidance on FUSE filename constraints).
5. **Step 4: `git push` the change (1 min).** `git init`, `git remote add`, `git push`. `curl` the simulator to see the `version` bumped 1 → 2 (this is the "aha" per Pattern G).
6. **What just happened (30 seconds).** Three-sentence recap linking to `/how-it-works/` for the reveal.

**Total: ~5 minutes if the reader types, ~3 if they paste.** The "aha" hits in step 4. Do NOT save the aha for step 6.

**Cleanup guidance:** End with `fusermount3 -u /tmp/reposix-mnt && pkill reposix-sim`. Mirrors `scripts/demo.sh` step 9 pattern.

**Source:** `docs/demo.md` already contains step-accurate content for steps 3/4/6/7 — carve from there. Simulator-first per CLAUDE.md OP #1.

## Linter Choice — Vale (recommended)

| Criterion | Vale | proselint | Custom Python | pre-commit regex |
|-----------|------|-----------|---------------|------------------|
| Banned-word rule with per-file-glob scoping | YES (`[docs/how-it-works/**]` section disables rule) | NO | YES (manual) | Partial (needs pathspec logic) |
| Code-fence exclusion | YES (`IgnoredScopes = code, code_block`) | Manual | Manual | Manual |
| CI integration | One binary + one step | pip install + one step | pip install + one step | grep pipeline |
| Pre-commit hook integration | Follows existing `scripts/hooks/` pattern cleanly | Fine | Fine | Fine |
| False-positive story | Mature rule system with `tokens:` regex | Less configurable | As good as you write | As good as you write |
| Reviewability for future agents | YAML rules in `.vale-styles/` are self-documenting | N/A — out of box | Custom code to read | Opaque regex |
| Install footprint | One Go binary, ~15MB | Python package | Zero (stdlib only) | Zero |
| Works OOB | YES — no custom code to write | YES | NO — write ~40 lines | NO |
| Meets P2 scoping requirement (word banned above Layer 3, allowed below) | YES | NO | YES but requires custom globbing | YES but requires custom globbing |

**Recommendation: Vale.** Best balance of power and simplicity. Meets P2 scoping requirement natively. CI integration follows a well-trod path (used by Grafana, GitLab, Datadog, Stoplight, Elastic per WebSearch results `[CITED: datadoghq.com engineering blog, docs.gitlab.com, grafana writers' toolkit]`).

**If the planner wants zero new installs:** a custom Python script is viable — ~40 lines, testable with pytest (matches `scripts/test_bench_token_economy.py` pattern), but the planner MUST include tests for the IgnoredScopes behavior (fenced/indented code, inline code, YAML frontmatter). Do not hand-roll this without tests.

**Concrete config for Vale:** see Code Examples §Example 1.

## mkdocs-material Theme Tuning — concrete diffs

### Current `mkdocs.yml` (lines 8-35, theme block)

```yaml
theme:
  name: material
  palette:
    - scheme: default
      primary: deep purple
      accent: amber
      toggle: { icon: material/brightness-7, name: "Switch to dark mode" }
    - scheme: slate
      primary: deep purple
      accent: amber
      toggle: { icon: material/brightness-4, name: "Switch to light mode" }
  features:
    - navigation.instant
    - navigation.tracking
    - navigation.sections
    - navigation.expand     # flag: pair with navigation.sections causes crowded sidebar
    - navigation.top
    - search.suggest
    - search.highlight
    - content.code.copy
    - content.code.annotate
    - content.tabs.link
```

### Recommended tuning (Phase 30)

```yaml
theme:
  name: material
  # Optionally: custom_dir: overrides   # only if we go Jinja override route; NOT recommended
  palette:
    - scheme: default
      primary: deep purple
      accent: amber
      toggle: { icon: material/brightness-7, name: "Switch to dark mode" }
    - scheme: slate
      primary: deep purple
      accent: amber
      toggle: { icon: material/brightness-4, name: "Switch to light mode" }
  features:
    - navigation.instant         # keep — SPA-fast routing
    - navigation.tracking        # keep
    - navigation.sections        # keep — top-level sections as headers in sidebar
    # - navigation.expand        # REMOVE — conflicts with sections, over-expands sidebar
    - navigation.top             # keep — "back to top" button
    - navigation.footer          # ADD — prev/next page nav at bottom
    - navigation.tabs            # ADD — top-of-page tabs for top-level sections
    - search.suggest             # keep
    - search.highlight           # keep
    - content.code.copy          # keep
    - content.code.annotate      # keep
    - content.tabs.link          # keep
    - content.action.edit        # ADD — "edit this page on GitHub" pencil
    - content.action.view        # ADD — "view source on GitHub"
    - announce.dismiss           # ADD — if we want a "v0.9.0 just shipped" banner
```

**Rationale per feature change:**
- `navigation.expand` removed: with `navigation.sections` enabled, expand duplicates the effect and crowds the sidebar. `[CITED: mkdocs-material navigation features]`.
- `navigation.footer` added: prev/next page links at bottom of every page — helps readers follow the IA as a narrative.
- `navigation.tabs` added: top-level nav tabs at the top of every page — reinforces the Diátaxis structure visually.
- `content.action.edit` / `view` added: trivial to configure, signal "open source / welcoming to edits."
- `announce.dismiss` optional: if the planner wants a "v0.9.0 shipped" banner on the home page.

### Add social cards plugin

```yaml
plugins:
  - search
  - social                       # ADD — generates per-page social cards
  - minify:
      minify_html: true
```

**Dependencies already satisfied on this host:** `CairoSVG` 2.7.1 + `pillow` 10.4.0. In CI, the `docs.yml` workflow already runs `pip install mkdocs-material` — change to `pip install "mkdocs-material[imaging]"` to pull the same deps.

**Source:** `[VERIFIED: mkdocs-material plugins/social]`.

### Social customization (optional)

If the planner wants per-page social cards with custom colors / logos, add:

```yaml
plugins:
  - social:
      cards_layout: default/variant
      cards_layout_options:
        background_color: "#1a1033"   # deep purple dark-mode background
        color: "#ffe082"              # amber accent
```

**Source:** `[VERIFIED: mkdocs-material plugins/social]`.

### Landing-page template decision

**Recommendation: Markdown-native — do NOT use `overrides/home.html`.**

Rationale:
- The creative-license notes explicitly reject "marketing bullet points with check-mark icons," "stock photos," and "empower." The voice is "precise, dry, earned."
- Grid cards + fenced code blocks + admonitions + mermaid already give us everything we need for a content-rich landing page without a Jinja hero.
- Jinja overrides conflict with `navigation.instant` (see Pitfall 2).
- Markdown-native stays on the same edit path as every other doc, which is the "self-improving infrastructure" discipline (OP #4).

**If the planner overrules this:** the pattern at `[CITED: alex3305.github.io/docs/selfhosted/mkdocs_material/custom_landing_page]` is the canonical mini-example. Directory: `.overrides/home.html`. `mkdocs.yml` custom_dir: .overrides. `index.md` frontmatter: `template: home.html` + `hide: [navigation, toc]`. Must disable `navigation.instant`.

## Files to Read / Create / Modify (Explicit List)

### Files the planner MUST read before writing plans

| File | Purpose |
|------|---------|
| `.planning/notes/phase-30-narrative-vignettes.md` | Source of truth for narrative intent. **READ IN FULL.** |
| `.planning/phases/30-.../CONTEXT.md` | Phase scope and non-negotiable framing. |
| `.planning/REQUIREMENTS.md` | DOCS-01..09. |
| `./CLAUDE.md` | Project rules (simulator-first, threat model, verification discipline). |
| `docs/index.md` | Current landing page — to be REWRITTEN. |
| `docs/architecture.md` | Source material for how-it-works split. |
| `docs/security.md` | Source material for trust-model.md. |
| `docs/demo.md` | Source material for tutorial.md. |
| `docs/why.md` | Understand existing "Explanation" voice; token-economy is a payoff linked from new pages. |
| `docs/connectors/guide.md` | Content to relocate to `docs/guides/write-your-own-connector.md`. |
| `mkdocs.yml` | Current nav + theme config; edit carefully. |
| `.github/workflows/docs.yml` | CI flow; add Vale step. |

### New files to create

| Path | Purpose | Source |
|------|---------|--------|
| `docs/mental-model.md` | Three conceptual keys, 300-400 words | Source-of-truth note §"Home-adjacent pages" |
| `docs/vs-mcp-sdks.md` | Comparison grounding P1 | Source-of-truth note §"Home-adjacent pages" + existing `docs/why.md` for facts |
| `docs/tutorial.md` | 5-minute first-run | Existing `docs/demo.md` steps 3–7 |
| `docs/how-it-works/filesystem.md` | Read/write via POSIX | Existing `docs/architecture.md` "Read path" + "Write path" + "The async bridge" |
| `docs/how-it-works/git.md` | git-remote-reposix + optimistic concurrency | Existing `docs/architecture.md` "git push" + "Optimistic concurrency as git merge" |
| `docs/how-it-works/trust-model.md` | Taint + allowlist + audit + lethal-trifecta | Existing `docs/security.md` (most content) + `docs/architecture.md` "Security perimeter" |
| `docs/guides/write-your-own-connector.md` | Relocated `connectors/guide.md` | Existing `docs/connectors/guide.md` (465 lines — preserve verbatim, update internal links only) |
| `docs/guides/integrate-with-your-agent.md` | Claude Code / Cursor / SDK patterns | **GREENFIELD** — no existing source; author against PROJECT.md core-value statement |
| `docs/guides/troubleshooting.md` | Stub, grows post-launch | Stub; seed with ~3 entries (FUSE mount fails, git push rejected with bulk-delete, audit log query example) |
| `docs/reference/simulator.md` | Dev-tooling framing | Existing `crates/reposix-sim/` + `docs/reference/http-api.md` |
| `docs/screenshots/phase-30/` (dir) | Playwright screenshots | Generated |
| `.vale.ini` (repo root) | Vale config | New |
| `.vale-styles/Reposix/ProgressiveDisclosure.yml` | P2 ban rule | New |
| `.vale-styles/Reposix/NoReplace.yml` | P1 ban rule | New |
| `.vale-styles/config/vocabularies/Reposix/accept.txt` | Vocabulary allowlist (e.g. "reposix" itself so it isn't flagged as unknown) | New |
| `scripts/hooks/pre-commit-docs` | Pre-commit git hook | New — follows `scripts/hooks/pre-push` pattern |
| `scripts/hooks/test-pre-commit-docs.sh` | Test for above (optional but recommended per OP #4) | New |

### Files to modify

| Path | Change |
|------|--------|
| `docs/index.md` | **REWRITE** — hero + V1 before/after + three-up + complement line |
| `mkdocs.yml` | Nav restructure + theme features + social plugin |
| `.github/workflows/docs.yml` | Add Vale install + lint step before mkdocs build |
| `scripts/install-hooks.sh` | Add pre-commit-docs registration |
| `README.md` (root) | Update any links to `docs/architecture.md` or `docs/security.md` that are being deleted/moved |

### Files to delete (after grep-audit confirms no dangling references)

| Path | Reason |
|------|--------|
| `docs/architecture.md` | Content fully carved to `docs/how-it-works/*` |
| `docs/security.md` | Content fully carved to `docs/how-it-works/trust-model.md` |
| `docs/demo.md` | Content fully carved to `docs/tutorial.md` (keep `docs/demos/index.md` for extended tier-2/3/5 demos if separate) |
| `docs/connectors/guide.md` | Moved to `docs/guides/write-your-own-connector.md` |

### Files to keep unchanged

| Path | Why |
|------|-----|
| `docs/reference/cli.md`, `http-api.md`, `git-remote.md`, `confluence.md`, `jira.md`, `crates.md` | Phase 26 made these correct |
| `docs/decisions/*.md` (ADRs) | Phase 26 made these correct |
| `docs/research/*.md` | Phase 26 made these correct |
| `docs/development/contributing.md`, `roadmap.md` | Unchanged |
| `docs/archive/*.md` | Archive, `not_in_nav` |
| `docs/social/*` | Hero images stay |
| `docs/demos/*` (except if merging into tutorial) | Extended tier demos |

## Scope Split Decision

**Question:** Given 9 DOCS requirements + 8+ new/modified pages + theme tuning + linter + tutorial + 3 diagrams, does this fit one phase?

**Answer:** **Yes, one phase — do NOT split.**

**Reasoning:**

1. **The artifacts are mutually reinforcing.** The hero (DOCS-01), the how-it-works pages (DOCS-02), the mental model (DOCS-03), and the tutorial (DOCS-06) all reference each other. Splitting into phase 30a (copy+IA) and 30b (diagrams+tutorial+linter) creates cross-phase dependencies — the 30a copy subagent would write placeholders for 30b content, then 30b would correct them, doubling work.
2. **The subagent fanout the source-of-truth note proposes is already parallelism.** Explore + Copy + IA + Diagrams + Tutorial are 5 parallel work-streams. With 5 parallel subagents, one phase IS the split.
3. **The linter (DOCS-09) is independent of narrative copy and blocks nothing.** It can run in parallel with copy/diagrams. Adding it to the same phase is cheap.
4. **Theme tuning (DOCS-08) is 15 lines of mkdocs.yml.** Not a phase unto itself.
5. **Tutorial (DOCS-06) depends on nothing upstream in this phase.** It can start immediately — the simulator already works, demo.md already has accurate content.
6. **Verification costs are lower for one big phase.** Playwright screenshots of the whole site, one doc-clarity-review pass on the rendered index, one CI green gate — vs doing these twice for two smaller phases.

**The risk:** phase scope creep. To mitigate:
- The plan MUST list exact file counts and a done-checklist per requirement.
- If any subagent returns BLOCKED, the plan must be able to continue the other 4 workstreams.
- The Troubleshooting page ships as an explicit stub (~3 entries). Do not let it balloon — it's post-launch growth.

**Recommended subagent structure (confirming the source-of-truth note's suggestion):**

1. **Explore subagent** — competitor narrative scan + hero-line drafting. Picks 3 of the 8 patterns from this research's Competitor Narrative Scan section. Outputs 3 candidate hero-lines.
2. **Copy subagent** — constrained by P1/P2 banned-word list. Writes: `index.md` hero + complement line + three-up; `mental-model.md`; `vs-mcp-sdks.md`. Runs Vale against its own output before submitting.
3. **IA subagent** — two competing `mkdocs.yml` nav structures scored against Diátaxis + personas. Outputs the winner.
4. **Diagrams subagent** — three mermaid diagrams (filesystem / git / trust-model). Uses the three specs from this research's Code Examples §Example 6 as starting points. Runs `mkdocs serve` + playwright screenshot for visual review.
5. **Tutorial subagent** — authors `tutorial.md`. Runs it end-to-end against `reposix-sim`. Attaches a transcript. Screenshots each step via playwright MCP.
6. **Linter subagent (can run last or in parallel)** — writes `.vale.ini` + `.vale-styles/Reposix/*.yml` + `scripts/hooks/pre-commit-docs` + CI diff. Runs the linter against all new/modified docs and fixes violations until clean.
7. **How-it-works carver subagent** — carves `architecture.md` + `security.md` into three pages. Updates all internal links. Deletes old files in the final wave.

**Wave structure (planner decides exact split):**

- **Wave 0:** Plan + agree on IA + hero-line candidates (humans / orchestrator approve copy direction).
- **Wave 1 (parallel):** Copy, IA, Diagrams, Tutorial, Linter subagents.
- **Wave 2 (parallel):** How-it-works carver subagent; guide author for agent-integration and troubleshooting.
- **Wave 3 (serial):** grep-audit for dangling references; delete obsolete files; update README.md.
- **Wave 4 (serial):** `mkdocs build --strict` green; Vale green; doc-clarity-review pass; playwright screenshots committed; CHANGELOG entry; `30-SUMMARY.md`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| mkdocs | All doc builds | ✓ | 1.6.1 | — |
| mkdocs-material | Theme | ✓ | 9.7.1 | — |
| mkdocs-material-extensions | Theme | ✓ | 1.3.1 | — |
| mkdocs-minify-plugin | Already in mkdocs.yml | ✓ | 0.8.0 | — |
| CairoSVG (for social cards) | DOCS-08 theme tuning | ✓ | 2.7.1 | If CI fails: `pip install "mkdocs-material[imaging]"` |
| pillow (for social cards) | DOCS-08 | ✓ | 10.4.0 | Same |
| mmdc (mermaid-cli) | Offline mermaid render (not strictly needed if client-side render is kept) | ✓ | 11.12.0 | Client-side render is default; no fallback needed |
| Playwright Chromium | Screenshot verification | ✓ | 1217 cached | Fall back to system chromium if MCP fails |
| Vale | DOCS-09 linter | ✗ | — | Install in CI per Code Examples §Example 3; locally via tarball (~15MB) |
| doc-clarity-review skill | 10-second value-prop validation | ✓ | (at `~/.claude/skills/doc-clarity-review/`) | — |
| Claude subprocess (`claude -p ...`) | doc-clarity-review backend | Presumed available (user environment) | — | Manual review by orchestrator |
| `mkdocs serve` local port 8000 | Screenshot verification | ✓ | — | — |
| fuse3 | Tutorial will reference but not require during authoring | ✓ | (system) | Tutorial runs against simulator + mount on Linux |
| cargo / rustc | Tutorial build steps | ✓ | (project toolchain) | — |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** Vale — fallback is a custom ~40-line Python linter, but this adds undocumented edge cases. Strongly recommend installing Vale.

## Validation Architecture

This is the Nyquist gate's dimension for Phase 30. Tests/checks the phase must produce and gate on.

### Test Framework

| Property | Value |
|----------|-------|
| Primary gates | (a) `mkdocs build --strict`, (b) `vale --config=.vale.ini docs/`, (c) `doc-clarity-review` on rendered index, (d) playwright screenshots |
| Config files | `mkdocs.yml`, `.vale.ini`, `.vale-styles/Reposix/*.yml` |
| Quick-run (author loop) | `mkdocs serve` + local `vale docs/` |
| Full suite (CI) | `.github/workflows/docs.yml` — add Vale step before build; keep mkdocs build --strict; add doc-clarity-review + playwright as phase-gate (not CI) |
| Framework install | Vale: `curl + tar` (see Code Examples §Example 3) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| DOCS-01 | Value prop lands in 10 seconds | Cold-reader verdict | `claude -p "$(cat prompt.md)" docs/index.md` (via doc-clarity-review) with custom "state value prop in one sentence" prompt | ❌ Wave 0 — create prompt + run as phase-gate script |
| DOCS-01 | "replace" banned in hero copy | Lint | `vale --config=.vale.ini docs/index.md` | ❌ Wave 0 — create `.vale-styles/Reposix/NoReplace.yml` |
| DOCS-02 | Three how-it-works pages exist | Structural | `test -f docs/how-it-works/filesystem.md && test -f docs/how-it-works/git.md && test -f docs/how-it-works/trust-model.md` | ❌ Wave 0 — scripts/check_phase_30_structure.py |
| DOCS-02 | Each has one mermaid diagram | Content | `grep -c '```mermaid' docs/how-it-works/filesystem.md` each returns 1 | ❌ Wave 0 — include in structure check script |
| DOCS-02 | Diagrams render without error | Visual | playwright screenshot + manual review checklist per user CLAUDE.md OP #1 | Manual phase-gate |
| DOCS-03 | Mental-model page exists with three conceptual keys | Structural | `grep -c '^## ' docs/mental-model.md` returns exactly 3 (one per key) AND each matches key phrasings | ❌ Wave 0 — scripts/check_phase_30_structure.py |
| DOCS-03 | vs-mcp-sdks page exists and mentions "complement" | Structural | `grep -iE '^(complement|absorb|subsume)' docs/vs-mcp-sdks.md` | Same script |
| DOCS-04 | Three guides exist | Structural | file existence test | Same script |
| DOCS-05 | Simulator page is under reference/ not how-it-works/ | Nav | `grep -A2 'Reference:' mkdocs.yml \| grep simulator.md && ! grep how-it-works.*simulator` | Same script |
| DOCS-06 | Tutorial exists and runs against simulator | Structural + manual run | file existence + `bash scripts/test_phase_30_tutorial.sh` (runs all numbered commands and asserts they exit 0) | ❌ Wave 0 — consider pytest-style harness for tutorial |
| DOCS-07 | Nav restructured | Structural | compare `grep '  -' mkdocs.yml` before/after; validate against source-of-truth IA sketch | Same script |
| DOCS-07 | Banned terms not above Layer 3 | Lint | `vale --config=.vale.ini docs/` with `[docs/how-it-works/**]` opt-out | Vale rule |
| DOCS-08 | Theme tuned | Config | `grep -c 'social' mkdocs.yml` ≥ 1; `grep -c 'navigation.footer' mkdocs.yml` ≥ 1 | Same script |
| DOCS-09 | Linter runs on every commit | Hook | `scripts/hooks/test-pre-commit-docs.sh` stages a doc with a banned word, commits, asserts reject | ❌ Wave 0 — per OP #4 promote ad-hoc bash to script |
| ALL | mkdocs build green | Build | `mkdocs build --strict` exit 0 | Existing — runs in `.github/workflows/docs.yml` |
| ALL | CI green | Build | `gh run view` green on post-push commit | Existing + augmented with Vale step |

### Sampling Rate

- **Per doc commit (author loop):** `vale docs/` on changed files + `mkdocs serve` visual check.
- **Per wave merge:** `mkdocs build --strict` + full Vale lint.
- **Phase gate before `/gsd-verify-work`:** 
  - `mkdocs build --strict` green.
  - `vale --config=.vale.ini docs/` green.
  - `doc-clarity-review` on rendered `docs/index.md` returns LANDED.
  - Playwright screenshots (desktop 1280 + mobile 375) for all new/modified pages exist in `docs/screenshots/phase-30/`.
  - `gh run view` shows green CI on the milestone commit.
  - `scripts/check_phase_30_structure.py` (or equivalent) exits 0.
  - CHANGELOG entry exists for v0.9.0.

### Wave 0 Gaps (test infrastructure to ship in this phase)

- [ ] `.vale.ini` + `.vale-styles/Reposix/{ProgressiveDisclosure,NoReplace}.yml` + `.vale-styles/config/vocabularies/Reposix/accept.txt` — linter config.
- [ ] `scripts/hooks/pre-commit-docs` — pre-commit hook.
- [ ] `scripts/hooks/test-pre-commit-docs.sh` — pytest-style test that stages a bad doc, confirms hook rejects.
- [ ] `scripts/check_phase_30_structure.py` (pytest or shell) — structural invariants (pages exist, nav has them, three mermaid diagrams present, "replace" not in index.md, "FUSE" not above Layer 3).
- [ ] `scripts/test_phase_30_tutorial.sh` — runs the tutorial commands end-to-end against simulator. (Promote the ad-hoc bash per OP #4.)
- [ ] Phase-gate script that invokes `doc-clarity-review` on `docs/index.md` with the purpose-built "state the value prop in one sentence" prompt and parses LANDED/PARTIAL/MISSED verdict.
- [ ] `.github/workflows/docs.yml` — add Vale install + lint step before `mkdocs build --strict`.

Without these, "pass" is unverifiable and human-review-dependent. With them, the phase gate is mechanical and repeatable.

## Security Domain

reposix is a lethal-trifecta project and every docs change must honor the accuracy-not-overselling rule from the project CLAUDE.md: *"any trust-model diagram must be accurate — don't oversell security claims."*

### Applicable ASVS Categories

| ASVS Category | Applies to this phase | Reason / Control |
|---------------|----------------------|------------------|
| V1 Architecture & Design | YES (docs reflect architecture) | The trust-model page MUST accurately describe the eight SG guardrails. No feature may be claimed as shipped unless `security.md` already lists it. |
| V2 Authentication | N/A | Phase is docs-only. |
| V3 Session Management | N/A | Same. |
| V4 Access Control | PARTIAL | Vale + mkdocs build steps run with standard `GITHUB_TOKEN`; no elevated secrets. |
| V5 Input Validation | YES (docs lint) | Vale IgnoredScopes prevents false-positives from code blocks — a "bad" code example with malicious YAML would be prose-linted but not executed. |
| V6 Cryptography | N/A | No crypto in docs. |
| V7 Error Handling | YES (linter error path) | Vale's error format must be CI-readable; use `level: error` not `warning` to ensure CI fails. |

### Known Threat Patterns (docs-specific)

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Docs claim a security property that isn't actually shipped | Information Disclosure (false sense of security) | Cross-reference every trust-model claim against `docs/security.md` shipped-items list and `SG-*` code-level evidence. The carver subagent does NOT invent new claims. |
| A contributor adds "replace" or "FUSE" via a PR from a fork | Tampering of the hero copy | Vale runs in CI on PR, not just post-merge. |
| Playwright screenshots leak local filesystem paths or env vars | Information Disclosure | Use a clean `/tmp/reposix-demo-XXXXXX` dir for tutorial commands; screenshot the browser only, never the terminal with env set. |
| Tutorial instructs the reader to set `REPOSIX_ALLOWED_ORIGINS=*` | Lethal-trifecta leg 3 enablement | Tutorial explicitly uses `http://127.0.0.1:*` default (simulator-only); never instructs widening the allowlist. |

**Key callout for the planner:** the trust-model page is the single place where the project's security story is narrativized. It is also the single easiest place to oversell. The carver subagent MUST stick to the facts in `docs/security.md` + `docs/architecture.md` + `.planning/research/threat-model-and-critique.md` and MUST NOT add superlatives. The user's global CLAUDE.md principle #6 (ground-truth obsession) applies: if a guardrail isn't in `crates/*/src/` and a test asserting it, it doesn't go in the trust-model page.

## Project Constraints (from CLAUDE.md)

The project `CLAUDE.md` adds specifics that Phase 30 must honor:

| Directive | Phase 30 Implication |
|-----------|---------------------|
| OP #1: close the feedback loop | Playwright screenshots + `mkdocs build --strict` + `gh run view` — all three must be green on the phase-ship commit. |
| OP #4: self-improving infrastructure | The banned-word linter MUST be promoted to a committed script (Vale config + hook + CI step). Same for tutorial verification. No ad-hoc bash. |
| "Simulator is the default / testing backend" | The tutorial MUST run against `reposix-sim`, not a real backend. No real-backend credentials appear in tutorial examples. |
| "Tainted by default" | Trust-model page discusses taint; does NOT discuss "security by X" — must discuss what's enforced. |
| "Audit log is non-optional" | Trust-model page's audit-log claim must match the existing `docs/security.md` SG-06 row exactly. |
| "No hidden state" | All phase state is in `.planning/phases/30-.../` + `docs/`. No "here's a good idea I didn't commit." |
| "Mount point = git repo" | Tutorial must demonstrate this — `git init` in the mount is load-bearing. |
| "Always enter through a GSD command" | No work outside `/gsd-plan-phase 30` → `/gsd-execute-phase 30` → `/gsd-verify-work`. |
| Subagent delegation: "Aggressive" | 5 parallel subagents per the source-of-truth note's fanout. Orchestrator coordinates only. |
| Threat model enforcement | Every trust-model claim cross-referenced against shipped SG-* evidence. |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| README.md as landing | mkdocs-material site with Diátaxis IA | Phases 25-26 (docs reorg) | Current docs/ is correctly organized but not narratively led; Phase 30 fixes the landing layer. |
| Client-side mermaid was brittle on mobile | mkdocs-material 9.x auto-themes + mobile-aware | 2024+ | Our current mkdocs-material 9.7.1 has it; no migration needed. |
| Docs linters were prose-focused (style) only | Layer-scoped semantic linters (Vale) | Ongoing | Vale's `extends: existence` with glob-scoped rules is the modern pattern. |
| Social cards required a paid Material insiders tier | Free tier since 9.4 | 2024+ | `material[imaging]` works on our OSS 9.7.1. |
| Custom Jinja landing page was the only option for hero design | Material grid cards + markdown handles most cases | 2024+ | Recommend markdown-native. |

**Deprecated / outdated:**
- Pre-9.x mkdocs-material had `navigation.instant` rendering quirks with mermaid — resolved in 9.x.
- Proselint hasn't had a major release since 2021 and is less actively maintained than Vale.

## Assumptions Log

All claims in this research were verified against (a) tool availability on this host (via `command -v` probes), (b) project artifacts (files read directly), or (c) cited external sources (official docs + WebSearch + WebFetch analyses). No `[ASSUMED]` entries.

**User confirmation needed — none.** The research verified the tooling, patterns, and source content. Decisions remain with the planner (linter choice, pre-render vs client-side mermaid, scope-split).

## Open Questions

1. **Should `docs/security.md` be deleted or kept as a condensed index?**
   - What we know: its content carves into `how-it-works/trust-model.md`.
   - What's unclear: external inbound links (from GitHub repo, issues, etc.) point to `security.md` — soft 404s will accumulate.
   - Recommendation: delete, accept soft 404s, add a redirect via gh-pages' `404.html` if the team wants (one-liner meta refresh). The planner decides.

2. **Should the simulator reference page live under `reference/` or under `development/`?**
   - What we know: CONTEXT.md says "Reference." Diátaxis validates "Reference."
   - What's unclear: since it's dev-tooling specifically, it sits adjacent to `development/contributing.md`.
   - Recommendation: **reference/simulator.md** — sim is end-user-facing too (contributors and plugin authors use it). CONTEXT.md is authoritative here.

3. **How much of `docs/demos/` stays?**
   - What we know: `docs/demos/index.md` is rich (13KB). `docs/demos/recordings/` is the asciicast.
   - What's unclear: does the 5-minute tutorial replace the demo tier-2 walkthrough entirely, or do they coexist?
   - Recommendation: keep `docs/demos/` intact as a deep-dive section (possibly move under `guides/` or keep standalone). Tutorial is the 5-minute path; demos are the "here are five more scenarios" depth.

4. **Does the `docs/guides/` nav include the existing tier-demo scripts (e.g. `scripts/demos/05-mount-real-github.sh`)?**
   - What we know: existing `docs/demos/index.md` documents these.
   - What's unclear: should they get proper per-backend guides ("Connect to GitHub", "Connect to Jira", "Connect to Confluence")?
   - Recommendation: create stubs — `docs/guides/connect-github.md`, `connect-jira.md`, `connect-confluence.md` — that link to existing tier demos and the reference pages. Stubs are fine; full content is a future phase.

## Sources

### Primary (HIGH confidence)

- `~/.claude/skills/doc-clarity-review/SKILL.md` — validates the 10-second-value-prop validation approach.
- `~/.claude/CLAUDE.md` — user global instructions (playwright + mcp-mermaid + OP #1/4).
- `./CLAUDE.md` — project CLAUDE.md (simulator-first, trust model, OPs).
- `.planning/notes/phase-30-narrative-vignettes.md` — narrative source of truth.
- `.planning/phases/30-.../CONTEXT.md` — phase scope.
- `.planning/REQUIREMENTS.md` — DOCS-01..09.
- `docs/index.md` + `docs/architecture.md` + `docs/security.md` + `docs/why.md` + `docs/connectors/guide.md` + `docs/demo.md` — read directly.
- `mkdocs.yml` — current config, read directly.
- `.github/workflows/docs.yml` — CI config, read directly.
- `https://squidfunk.github.io/mkdocs-material/reference/diagrams/` — mermaid integration + theming notes.
- `https://squidfunk.github.io/mkdocs-material/plugins/social/` — social cards config + dependencies.
- `https://vale.sh/docs/styles` — Vale custom rules, scopes, IgnoredScopes.
- `https://diataxis.fr/` — Diátaxis definitions and cross-category anti-pattern.
- `https://github.com/peng-shawn/mermaid-mcp-server` — mcp-mermaid capabilities.

### Secondary (MEDIUM confidence)

- [Linear homepage analysis](https://linear.app/) — hero text + carousel pattern.
- [Turso concepts analysis](https://docs.turso.tech/concepts) — length/structure precedent for mental-model.md.
- [Fly.io homepage analysis](https://fly.io/) — tagline cadence.
- [Tailscale homepage analysis](https://tailscale.com/) — conceptual-first IA.
- [Warp homepage analysis](https://www.warp.dev/) — agentic framing validation.
- [Val Town homepage analysis](https://www.val.town/) — positioning-line technique.
- [Raycast homepage analysis](https://www.raycast.com/) — hero terseness upper bound.
- [Stripe docs + quickstart](https://docs.stripe.com/development/quickstart) — "aha moment" structure.
- [Cloudflare Workers "Get Started"](https://developers.cloudflare.com/workers/get-started/guide/) — tutorial length + prose-to-code ratio.
- [alex3305 custom landing page](https://alex3305.github.io/docs/selfhosted/mkdocs_material/custom_landing_page/) — Jinja override walkthrough.

### Tertiary (LOW confidence / unverified)

- None. All claims are either (a) tool-probe verified, (b) from source-of-truth artifacts, or (c) from cited web sources.

## Metadata

**Confidence breakdown:**

- Standard stack (tooling choices, versions, install): **HIGH** — every tool probed with `command -v`, version strings captured, installs already resolve. Vale is the only new install and has well-documented binary releases.
- Architecture / IA (what goes where, Diátaxis assignment): **HIGH** — source-of-truth note is locked; Diátaxis validated the proposed nav with one minor flag (custom-fields cross-category risk).
- Pitfalls: **HIGH** — six verified against docs or past project patterns (fence scoping, navigation.instant conflict, dark-mode diagrams, strict-mode dangling links, "mount" generic verb, social cards fonts).
- Competitor patterns: **HIGH** — eight sites fetched directly and summarized individually; 3-of-8 recommendation is the menu for the copy subagent to choose from.
- Validation architecture: **MEDIUM-HIGH** — the doc-clarity-review skill exists and is the right tool; the custom "LANDED / PARTIAL / MISSED" prompt is novel to this phase and needs a first run to calibrate the verdict parser.
- Tutorial format: **HIGH** — `demo.md` content already provides 80% of the tutorial; Pattern H (Cloudflare) + Pattern G (Stripe) set the structure.

**Research date:** 2026-04-17

**Valid until:** 30 days for mkdocs-material / theme features (stable); 7 days for Vale version pinning (active release cadence); 30 days for competitor patterns (landing pages change, but the structural moves are durable).

## RESEARCH COMPLETE
