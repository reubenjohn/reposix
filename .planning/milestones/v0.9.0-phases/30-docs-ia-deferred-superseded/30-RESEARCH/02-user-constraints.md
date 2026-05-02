← [back to index](./index.md) · phase 30 research

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
