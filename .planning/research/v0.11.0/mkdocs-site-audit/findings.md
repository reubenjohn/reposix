# v0.11.0 mkdocs site audit — findings

← [back to index](./index.md)

---

## Site-wide issues

### S1. Orphan mermaid error SVG persists across pages (the "every page" symptom)

**Severity:** P0 (most-visible cosmetic bug; first impression for every visitor who lands on a `how-it-works/` page first).

**What you see:** at the very bottom of the page, after the `Made with Material for MkDocs` footer, the literal text

```
Syntax error in text
mermaid version 11.14.0
```

**Where it actually lives:** mermaid renders parse failures as an SVG with `id="__mermaid_0"` and `aria-roledescription="error"`, wrapped in a sibling `<div id="d__mermaid_0">` that is appended **directly to `<body>`** (not into the original `<pre class="mermaid">` slot). When mkdocs-material's `navigation.instant` does an in-place article swap (clicking any nav link), the swap touches `article.md-content__inner` only. The body-level `d__mermaid_0` div is never removed, so the error SVG stays at the bottom of every page the user visits *after* the original failure.

**Reproduction (verified):** load `/how-it-works/git-layer/` (mermaid throws on line 13 of the diagram — see P1 below), then click the Home nav link. The home page now shows `Syntax error in text` at the bottom even though `index.md` has no mermaid block of its own. Hard-reloading the home page (`location.reload()`) clears it. Navigating in via direct URL also clears it. So the symptom is **path-dependent**: the user's overnight nav path through `git-layer/` is what stuck the orphan.

**Console evidence on `git-layer/`:**

```
Uncaught (in promise) Error: Parse error on line 13:
...ET /issues/&lt;id&gt; (current version)
-----------------------^
Expecting '()', 'SOLID_OPEN_ARROW', ... 'DOTTED_POINT', got 'NEWLINE'
    at parseError (https://unpkg.com/mermaid@11/dist/mermaid.min.js:1824:19907)
```

### S2. Mermaid 11.14.0 floats from unpkg.com

mkdocs-material's bundled loader pulls `mermaid@11` (currently 11.14.0) from unpkg.com on first use. Two implications: (a) version drift — a future 11.x could change parse strictness; (b) the docs site itself relies on a third-party CDN at view time. Not a security regression (docs site, not runtime), but worth pinning for v0.11.0.

---

## Per-page findings

Pages are visited in nav order. For each: mermaid status, jargon density (compared against `.planning/research/v0.11.0/jargon-inventory.md`), and other clarity issues. Pages with nothing to report are listed at the end.

### `https://reubenjohn.github.io/reposix/` (Home, `index.md`)

- Mermaid: none on page; inherits S1 after visiting `git-layer/`.
- Jargon: low. `partial clone` is operationalised by surrounding prose. Acceptable for a landing page.
- Other: `:material-graph:`, `:material-compare:`, `:material-chart-line:` shortcodes render as **literal text** in the "Where to go next" section — the `pymdownx.emoji` extension is not enabled. Separate cosmetic bug not in the owner's report. Also: "92.3% input-context-token reduction vs MCP" is unlinked to `benchmarks/v0.9.0-latency`.

### `/concepts/mental-model-in-60-seconds/`, `/concepts/reposix-vs-mcp-and-sdks/`, `/tutorials/first-run/`

Clean. No mermaid, no errors, jargon glossed inline. The first-run tutorial in particular reads well to a fresh agent (5134 chars, seven copy-pasteable steps).

### `/how-it-works/filesystem-layer/` **(diagram silently fails)**

- Mermaid: `<div class="mermaid"></div>` is empty after 5 s wait. No console error, no SVG. The flowchart source parses cleanly when fed to `mermaid.parse()` directly. Best hypothesis: SPA-nav timing race — mkdocs-material replaced `<pre class="mermaid"><code>` with an empty wrapper before mermaid scanned for fresh blocks. Worth confirming on a hard reload; symptom may not reproduce that way.
- Missing diagram: "How a `cat` becomes a REST call (or doesn't)" flowchart — central to the page's narrative.
- Jargon: medium. `OID` and `delta sync` are not glossed at first use; the rest have inline definitions (e.g. `partial clone` is glossed via "built into git ≥ 2.27 ... --filter=blob:none asks the remote for the tree without blobs").
- Broken link: "Wire-level details ... live in [Reference](../../reference/)" → resolves to `reference/cli/` which is in `not_in_nav` (404).

### `/how-it-works/git-layer/` **(hard parse error — root cause of S1)**

- Mermaid: **the global-error culprit.** Parse error on diagram line 13:
  - Source: `Helper->>API: GET /issues/&lt;id&gt; (current version)`.
  - Author wrote `&lt;id&gt;` thinking of the rendered HTML. mkdocs renders the fenced block to `<pre class=mermaid><code>...&amp;lt;id&amp;gt;...</code></pre>`. mermaid reads `code.textContent` → decodes once → parser sees literal `<id>` → `sequenceDiagram` parser bails on the `<` token.
  - The parse error logs to console AND mermaid attaches an error SVG to `<body>` — the SVG that says `Syntax error in text / mermaid version 11.14.0`.
- Missing diagram: the "push round-trip (happy path and conflict)" sequence diagram (the one the owner specifically flagged).
- Jargon: **highest density on the site.** `stateless-connect`, `export`, `option`, `fast-import`, `protocol-v2`, `refspec namespace`, `fast-export`, `pkt-line` (implicit), `want <oid>`, `command=fetch`. Many are inline-glossed but mixed with unglossed terms in the same paragraph. Jargon-inventory flags this page as 9 terms (the most in the project).
- Broken cross-ref: `decisions/008-helper-backend-dispatch.md` — file does not exist.
- The footnote on `refs/heads/*:refs/reposix/*` ("collapsing it makes fast-export emit an empty delta") presumes git-internals fluency.

### `/how-it-works/time-travel/`

Clean. `partial-clone` glossed in surrounding prose. One nit: references `.planning/research/v0.11.0/vision-and-innovations.md §3b` from user-facing docs — the planning tree is not on the published site.

### `/how-it-works/trust-model/` **(diagram silently fails)**

- Mermaid: empty `<div class="mermaid"></div>` after 5 s wait. No console error. Source uses doubly-HTML-encoded entities (`&lt;` in markdown → `&amp;lt;` in HTML) for `Tainted<Vec<u8>>` and `Untainted<T>`. Direct `mermaid.parse()` accepts the source. So the failure is not a parse error — likely a layout/render issue or the same SPA-nav race as filesystem-layer.
- Missing diagram: "Concentric rings — taint in, audited bytes out" `flowchart TB`.
- Jargon: medium-high but mostly project-specific (`Tainted<T>`, `sanitize`, `BackendConnector`, `audit_events_cache`, `helper_push_*`, `REPOSIX_ALLOWED_ORIGINS`, `REPOSIX_BLOB_LIMIT`); each first-mention has an inline gloss. External terms (`lethal trifecta`, `SQLite WAL`) are introduced by example or external link.
- The `What's NOT mitigated` section is excellent — keep as-is.

### `/guides/write-your-own-connector/`

Clean for its intended audience (Rust contributors). `BackendConnector`, `RecordId`, `tainted`, `egress allowlist` are all glossed.

### `/guides/integrate-with-your-agent/` **(owner-flagged)**

- Mermaid: none.
- Jargon in main text: `sparse-checkout` (2), `frontmatter` (1), `allowlist` (3), `egress` (2), `helper` (3), plus contextual `MCP`, `dark-factory regression test`, `dispatch verbs`, `system prompt`, `subprocess.run`.
- The owner's framing is **partially** correct: this page uses fewer obscure git-internals terms than `git-layer.md` does, but presumes you've internalised the trust-model. Specific gaps:
  - `dark-factory regression test` / `dark-factory teaching loop` — no gloss; links only to `.claude/skills/reposix-agent-flow/SKILL.md` (an internal artefact).
  - `dispatch verbs` — used as if the reader knows reposix-v0.1's verb system.
  - `MCP tool registration` — appears in negation ("Not an MCP server") but readers unfamiliar with MCP get no help.
  - `git sparse-checkout` — recovery move named without a git-docs link or a sentence on *why* it reduces blob-fetch count.
- "v0.12.0 (planned)" recipe list (Aider, Continue, Devin, SWE-agent) is phrased as if certain to land; could use a "subject to roadmap" hedge.
- Pattern 3 uses `:=` pseudocode that's unusual against the rest of the docs (Rust + shell).

### `/guides/troubleshooting/`

Clean. Symptom → cause → fix structure makes its jargon land with context.

### Pages clean of every checked-for issue

All `/reference/*`, all `/decisions/*`, `/benchmarks/v0.9.0-latency/`. Research pages not deep-inspected (out of scope per brief).

---

## Root cause analysis

### Bug 1 — Hard parse error in `git-layer.md` mermaid block

**File:** `docs/how-it-works/git-layer.md`, line 24:

```
        Helper->>API: GET /issues/&lt;id&gt; (current version)
```

**Why it breaks:** the markdown author wrote `&lt;id&gt;` because they were thinking of the rendered HTML. Inside a `mermaid` fenced block, mkdocs-material treats the body as raw mermaid source and only HTML-escapes for the `<code>` wrapper. When mermaid reads `code.textContent`, the text is decoded once → the parser sees the literal characters `<id>`. In `sequenceDiagram`, `<` is not a valid character outside specific edge syntax, so the parser bails.

**Fix (smallest):** double-quote the message — `Helper->>API: "GET /issues/<id> (current version)"`. mermaid's quoted-message syntax permits reserved chars and preserves the `<id>` semantics. Alternative: use `{id}` or `[id]` in place of `<id>`.

### Bug 2 — Silent render failure in `trust-model.md` and `filesystem-layer.md`

**Symptom:** `<pre class="mermaid"><code>` is replaced by `<div class="mermaid"></div>` (empty) — mermaid neither rendered an SVG nor logged an error.

**For `trust-model.md`:** lines 28 and 30 have `Tainted&lt;Vec&lt;u8&gt;&gt;` and `Untainted&lt;T&gt;`. After mkdocs HTML rendering, these become `Tainted&amp;lt;Vec&amp;lt;u8&amp;gt;&amp;gt;` inside `<code>`. `code.textContent` decodes once → mermaid sees `Tainted&lt;Vec&lt;u8&gt;&gt;` (literal entities). Inside a `["..."]` quoted node label, this *is* legal mermaid — and indeed direct `mermaid.parse()` accepts it. So this is not a parse-error path; it's something else, possibly a downstream render error in flowchart layout when the label width exceeds a threshold. The empty div is the visible artefact.

**For `filesystem-layer.md`:** the source has no doubled-encoding issue. The flowchart parses cleanly. The empty div likely indicates a navigation.instant timing race: when the user lands on the page via SPA-style nav, the article DOM is swapped *after* mermaid has already inspected the prior page — so the new `<pre class="mermaid">` is replaced by mkdocs-material's wrapper before mermaid scans for fresh blocks. We confirmed via `await mermaid.run()` after page load that nothing happens (no `<pre.mermaid>` is left to process).

**Fix candidates (see F2/F5/F6 in Recommended Fixes):** drop entity encoding from the trust-model labels (write `Tainted<Vec<u8>>` raw inside `["..."]`); add a `document$.subscribe(() => mermaid.run())` re-init hook OR disable `navigation.instant`; verify with a local `mkdocs build --strict` and a hard reload before committing.

### Bug 3 — `:material-graph:` icon shortcodes rendering as raw text on the home page

**File:** `docs/index.md` (and possibly `mkdocs.yml`).

**Why:** the Material for MkDocs icon shortcode (`:material-graph:`) requires the `pymdownx.emoji` extension with the Material twemoji generator. The current `markdown_extensions` block in `mkdocs.yml` does not include `pymdownx.emoji`. Without it, the shortcodes ship as literal text. Visible on the home page in the "Where to go next" / footer area.

**Fix:** add to `mkdocs.yml`:

```yaml
markdown_extensions:
  - pymdownx.emoji:
      emoji_index: !!python/name:material.extensions.emoji.twemoji
      emoji_generator: !!python/name:material.extensions.emoji.to_svg
```

### Bug 4 — Broken cross-references after `not_in_nav`-driven page removal

**File:** `docs/how-it-works/filesystem-layer.md` line ending in "Wire-level details ... live in [Reference](../../reference/)" — the link target `reference/cli.md` is in `not_in_nav` and so the rendered link is dead.

**File:** `docs/how-it-works/git-layer.md` references `decisions/008-helper-backend-dispatch.md` — the file does not exist in `docs/decisions/`.

**Fix:** either point at a live reference page or add the missing ADR.

### Why the symptom appears "global"

`navigation.instant` (enabled in `mkdocs.yml`) swaps only `<article class="md-content__inner">` on a nav click; it does not clean up SVGs mermaid attached directly to `<body>`. The orphan `<div id="d__mermaid_0">` containing the error SVG persists across every subsequent page swap until a full reload. The content bug (Bug 1) is the root cause; `navigation.instant` is the amplifier.

---

## Recommended fixes (priority-ordered)

**P0 — ship today (eliminates global symptom):**

- **F1.** Edit `docs/how-it-works/git-layer.md` line 24: replace `&lt;id&gt;` with double-quoted form `"GET /issues/<id> (current version)"`. 30 s. Eliminates the parse error and the global "Syntax error in text" footer symptom on every downstream page.
- **F2.** Edit `docs/how-it-works/trust-model.md` lines 28, 30: replace `Tainted&lt;Vec&lt;u8&gt;&gt;` / `Untainted&lt;T&gt;` with literal `Tainted<Vec<u8>>` / `Untainted<T>` (no entity encoding) inside the `["..."]` labels. If still empty after rebuild, fall back to a plain-text label like `"Tainted bytes (Vec u8)<br/>(cache hands these out)"`. Renders the concentric-rings diagram.
- **F3.** Add a small `extra_javascript` cleanup hook: subscribe to mkdocs-material's `document$` observable and remove any `body > div[id^="d__mermaid_"]` orphan divs on each nav event. Defends against future mermaid regressions producing the same global symptom.

**P1 — within the v0.11.0 docs sweep:**

- **F4.** Enable `pymdownx.emoji` in `mkdocs.yml` (snippet in §Bug 3) so `:material-graph:` etc. on the home page render as icons.
- **F5.** Diagnose `filesystem-layer.md` silent render — verify whether the flowchart appears on a hard reload. If yes, the cause is the SPA-nav race and the fix is F6.
- **F6.** Either disable `navigation.instant` (one-line config) OR add a `document$.subscribe(...)` re-init hook that runs `mermaid.run()` after each in-page swap. Prefer the hook to keep the SPA UX. Eliminates the broader "diagram missing on inner pages after SPA nav" class.
- **F7.** Pin mermaid to a specific version via `extra_javascript` (e.g. `https://unpkg.com/mermaid@11.14.0/dist/mermaid.min.js`) to stop the auto-update on `@11`.
- **F8.** Fix broken cross-refs: `filesystem-layer.md`'s `[Reference](../../reference/)` (target is in `not_in_nav`); `git-layer.md`'s `decisions/008-helper-backend-dispatch.md` (file doesn't exist). Repoint or add the ADR.

**P2 — clarity debt (jargon):**

- **F9.** Inline-gloss + external-link audit on `git-layer.md` and `integrate-with-your-agent.md`, using `.planning/research/v0.11.0/jargon-inventory.md` as the worklist. Add git-scm.com links on first use for `stateless-connect`, `protocol-v2`, `extensions.partialClone`, `fast-import`, `fast-export`, `refspec namespace`, `pkt-line`, `partial clone`. Gloss `OID`, `dark-factory teaching mechanism`, `dispatch verbs` in the agent guide. 1–2 h focused.
- **F10.** Promote a one-page `docs/reference/glossary.md` (already recommended in the jargon inventory's "Highest-Impact Wins"). Cross-link from first occurrence on every page.

**P3 — regression prevention:**

- **F11.** Add a CI smoke test (playwright or headless puppeteer) that fails the deploy gate if any nav-discoverable page contains `svg[aria-roledescription="error"]` after full load. Catches Bug 1 / Bug 2 regressions automatically. Follows the "self-improving infrastructure" principle in `~/.claude/CLAUDE.md` §4.
- **F12.** Add a pre-commit regex on `docs/**/*.md` that flags `&lt;` or `&gt;` inside ` ```mermaid ` fences. Catches Bug 1 at author time, not deploy time. Belongs in the existing `mkdocs build --strict` pre-push backlog (commit `85a6fef`).

---

## Honest scope of this audit

- **Audited:** every page in `mkdocs.yml`'s `nav:`, plus ground-truth checks of mermaid render state, console errors, jargon density via word-list match against the project's existing inventory, and broken cross-references where they were spotted in passing.
- **Not audited:** `research/initial-report/` and `research/agentic-engineering-reference/` (long-form research; out of scope per the brief). ADR pages 001–007 (skimmed for mermaid only — content not reviewed). Docs marked `not_in_nav` (`architecture.md`, `security.md`, `why.md`, `demo.md`, etc.) — these are explicitly excluded from the published nav and therefore from a reader's path.
- **Test conditions:** chromium via playwright-MCP, JS-enabled, ~3–5 s wait per page, default viewport. Mobile width and dark-mode rendering not exercised. Search functionality not exercised. RSS / sitemap not exercised.
- **Confidence:** P0 / P1 findings (S1, Bug 1, Bug 4 cross-ref) are reproducible and verified against console output and raw HTML. Bug 2 (silent render fail on trust-model and filesystem-layer) is reproducible but the *cause* is hypothesised, not proven — F2 / F5 ship with diagnose-then-fix steps for that reason.
