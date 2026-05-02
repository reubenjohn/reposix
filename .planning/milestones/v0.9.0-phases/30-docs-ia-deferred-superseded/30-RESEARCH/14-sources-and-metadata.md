← [back to index](./index.md) · phase 30 research

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
