# v0.11.0 mkdocs site audit (live)

**Audit target:** https://reubenjohn.github.io/reposix/ (deployed `main`, mkdocs-material 9.7.6, mermaid 11.14.0 via the bundled `mkdocs-material` mermaid integration).
**Audit date:** 2026-04-25.
**Auditor:** automated playwright-MCP traversal (chromium, JS-enabled, 4–5 s wait per page for mermaid render).
**Tools:** `browser_navigate`, `browser_evaluate` (DOM snapshot + mermaid AST inspection), `browser_console_messages`, raw HTML via `curl`.

---

## TL;DR

- **One real bug, one cosmetic-but-global symptom, two clarity debts.**
- The owner's report is **largely correct**, with a useful clarification: there is **exactly one mermaid block** that throws a hard parse error (`how-it-works/git-layer.md`), but the resulting orphan error SVG bleeds onto **every page** because `navigation.instant` does not clean up body-level orphans across SPA-style page swaps. So the user is genuinely seeing "Syntax error in text / mermaid version 11.14.0" at the bottom of every page they visit *after* visiting `git-layer/`.
- A second mermaid block (`how-it-works/trust-model.md`) **silently fails to render** (empty `<div class="mermaid"></div>`, no console error, no SVG). Likely cause: the doubly-encoded `&amp;lt;` HTML entities in the source produce a flowchart that mermaid drops without an exception.
- A third mermaid block (`how-it-works/filesystem-layer.md`) renders an empty div on initial load — likely the same render-race / DOM-mutation issue, since the source itself parses cleanly when fed to `mermaid.parse()` directly.
- Jargon density is real on the `how-it-works/*` cluster and the agent guide, but `.planning/research/v0.11.0/jargon-inventory.md` (also dated 2026-04-25) already enumerates the gap precisely. This report cross-validates that inventory against the live site rather than re-deriving it.

---

## Chapters

- **[Findings](./findings.md)** — full per-page findings, site-wide issues, root cause analysis, recommended fixes (priority-ordered), and honest scope of this audit.

---

## Cross-references to existing planning artefacts

- `.planning/research/v0.11.0/jargon-inventory.md` — the existing terminology audit. F9 / F10 should pick that file up as the worklist; this report cross-validates rather than replaces it.
- `mkdocs.yml` — the source of truth for site config; F4, F6, F7 all edit this file.
- `docs/notes/` (mentioned in commit `85a6fef docs(notes): backlog mkdocs --strict in pre-push`) — F11 / F12 belong in the same backlog.

---

## Upstream tracking

- **squidfunk/mkdocs-material#8584** — filed 2026-04-26 to track Bug 1 (superfences `<pre class="mermaid">` content stripped when `minify_html: true`). <https://github.com/squidfunk/mkdocs-material/issues/8584>. Links the three workaround commits (`66836f7`, `e119006`, `100ae00`) and proposes three candidate upstream fixes. Resolution there lets us drop the local `mermaid-render.js` workaround and re-enable HTML minification.
