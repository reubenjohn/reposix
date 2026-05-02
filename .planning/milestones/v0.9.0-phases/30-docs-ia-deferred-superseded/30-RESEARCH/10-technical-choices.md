← [back to index](./index.md) · phase 30 research

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
