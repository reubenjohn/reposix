# Phase 117: Doc-truth launch-blocker purge — Pattern Map

**Mapped:** 2026-07-16
**Artifacts analyzed:** 6 doc-truth doc edits + 2 CLI error paths (SC3/SC4) + 1 animation-embed lane (GTH-V15-37) + catalog-row wiring
**Analogs found:** 10 / 11 (one genuinely-new convention: `docs/assets/` does not yet exist)

> Read-only pattern map. No source edited. Every analog cites a real `file:line`.
> The planner should have each plan copy conventions from the cited analog rather
> than invent them. Phase has NO CONTEXT.md/RESEARCH.md on disk yet — this map is
> keyed off ROADMAP P117 SC1–SC5 (`.planning/ROADMAP.md:135-152`) + GTH-V15-36/37
> (`GOOD-TO-HAVES.md:270-288`).

---

## Artifact → Analog map

| New/changed artifact | Closest analog (file:line) | Convention to follow | Notes |
|---|---|---|---|
| **SC1** `docs/index.md` — Confluence-is-a-wiki + `reposix init` (not `git clone`) truth fix | itself, `docs/index.md:44-95` (install-tabs + "After — one commit" already use `reposix init`) | Keep the git-native bootstrap verb `reposix init sim::demo <path>` then `git checkout -B main refs/reposix/origin/main`, matching the already-correct §"After" block; fix only the stale `git clone` framing in the hero prose (`index.md:13`) and any "issue tracker" label on Confluence | The hero para at `index.md:13` says "REST-based issue trackers (Jira, GitHub Issues, Confluence)" — Confluence is a **wiki**, mirror the connector-matrix framing at `index.md:130-137` |
| **SC2** `docs/how-it-works/filesystem-layer.md` — drop "cat secretly triggers a network call" | `docs/how-it-works/filesystem-layer.md:42` (already-correct framing: "The first `cat` … triggers one REST call") | Rewrite the Plain-English summary (`:7-13`) to match the accurate body at `:42` and the failure-mode bullet at `:63`; keep the "Plain-English summary" + `---` + mermaid-flowchart + "What lives where" + "Failure modes" section skeleton | The lie is ONLY in the summary block (`:8-9` "*first* read might **secretly** trigger a network call"); body is honest. This page is otherwise a furnished-product exemplar (see below) |
| **SC2** cross-link propagation: `docs/index.md`, `git-layer.md`, `time-travel.md`, `trust-model.md` | `docs/index.md:154` (the "One diagram each" how-it-works link block) | Every inbound link keeps the glossary-anchored, relative-path style (`../reference/glossary.md#partial-clone`) seen at `filesystem-layer.md:48,56,57` | Grep the four pages for the "secret/hidden network call" phrasing before editing; link-resolution gate (`docs-build/link-resolution`) will catch a broken relative path |
| **SC3** `crates/reposix-cli/src/list.rs` connection-refused error | GOLD exemplar `crates/reposix-cli/src/init.rs:363-373` (`bail!` teaches fix + names sim alternative + copy-paste recovery) | Replace the terse `.with_context(\|\| "sim list_records project={project}")` at `list.rs:79-80` with a teaching `bail!` that (1) says the backend is unreachable, (2) suggests `reposix sim` in another terminal, (3) gives the copy-paste retry — the 3-part bar in `crates/CLAUDE.md` § Error-message convention | Current error surfaces raw `Connection refused` under a terse context string — does NOT meet the bar. Detect the connect failure (reqwest error) and wrap; keep the non-connect paths as-is |
| **SC3** `crates/reposix-cli/src/refresh.rs` connection-refused error | same exemplar `init.rs:363-373`; local sibling `refresh.rs:206` (`SimBackend::new(...).context("build SimBackend")`) | Same teach-the-fix `bail!` wrap on the sim `list_records`/round-trip await; reuse the exact recovery-command wording chosen for `list.rs` so the two errors read identically | `refresh.rs` already has teaching `bail!`s for env-var gaps (`:227,:250`) — mirror THAT tone for the connection case |
| **SC4 (route A)** new `reposix detach` subcommand — clap registration | `crates/reposix-cli/src/main.rs:90-105` (`Attach(attach::AttachArgs)` enum arm) + `attach.rs` handler module | Add a `Detach { ... }` arm to `enum Cmd` (`main.rs:40`) with a `///` doc-comment block matching the `Init`/`Attach` density (`main.rs:63-105`); put the handler in a new `crates/reposix-cli/src/detach.rs` mirroring `attach.rs` structure; wire dispatch in the `match cmd` block | Detach must UNDO what attach set: remove the reposix remote + `extensions.partialClone`, leave `origin` untouched (inverse of `attach.rs:150+`). Read `attach.rs` fully before implementing |
| **SC4 (route B)** rewrite `attach.rs` multi-SoT error to not name a nonexistent subcommand | `crates/reposix-cli/src/attach.rs:142-145` (the `bail!` referencing `reposix detach`) + teach-the-fix exemplar `init.rs:363-373` | If detach is NOT built, replace "Run `reposix detach` first" (`attach.rs:144`) with a real recovery: e.g. `git remote remove <name> && git config --unset extensions.partialClone` then re-attach — a copy-paste command that actually works today | Route A vs B is a planner decision per CONSULT-DECISIONS SC4 depth ruling (`CONSULT-DECISIONS.md:146` — no B-class build mandated this milestone). Route B is the lower-risk default; A is the furnished-product move |
| **SC5** `docs/benchmarks/token-economy.md` — fix `reposix_session.txt` provenance | `docs/benchmarks/token-economy.md:67-79` ("Capture provenance" section) | Correct the claim at `:78-79` ("an ANSI-stripped transcript of the reposix arm's git-native shell session") to the TRUE provenance; keep the bulleted "Capture provenance" list format + the `benchmarks/captures/*.json` citation style | Verify the real provenance of `benchmarks/fixtures/reposix_session.txt` against reality (is it a live-session dump or a hand-authored demo?) BEFORE rewording — do not swap one unverified claim for another |
| **SC5** `docs/social/twitter.md` — drop deleted-FUSE description | `docs/social/twitter.md:16` ("reposix — a FUSE filesystem + git-remote-helper") | Rewrite to the git-native partial-clone framing used in `docs/index.md:13` + `filesystem-layer.md:46` ("v0.9.0 design superseded that virtual filesystem"); the 89.1% figure at `:18` is a separate BENCH concern (P118 DOCS-07), leave unless in scope | `docs/social/*` is `not_in_nav` (`mkdocs.yml:56`) so it escapes mkdocs-strict + doc-alignment gates — no automated backstop; get the prose right by hand |
| **GTH-V15-37** animation assets under `docs/assets/animation/` + mkdocs wiring | `mkdocs.yml:104-113` (`extra_javascript`/`extra_css` block) + `docs/javascripts/mermaid-render.js` (self-hosted JS include pattern) + `docs/stylesheets/extra.css` | Declare the precompiled bundle + self-hosted fonts via `extra_javascript`/`extra_css` (mkdocs.yml); embed on `docs/index.md` with `md_in_html` + `attr_list` (already enabled, `mkdocs.yml:76,101`) — same extension set the `<div class="grid cards" markdown>` block at `index.md:15` relies on | **No `docs/assets/` dir exists yet** (see NOTICED). Precedent for a self-hosted, custom-loaded asset is `mermaid-render.js`; precedent for CDN-pin discipline is `mkdocs.yml:107-113` (mermaid pinned to `@11`). Poster+click-to-play, NOT autoplay, per checklist item 3 (`GOOD-TO-HAVES.md:280`) |
| **Catalog row** — playwright coverage of the animation (docs-build) | `quality/catalogs/docs-build.json:42-76` (`docs-build/mermaid-renders` row) + verifier `quality/gates/docs-build/mermaid-renders.sh` | If the animation gets browser-coverage, mint a docs-build row shaped like `mermaid-renders`: `kind: mechanical`, artifact under `.planning/verifications/playwright/<section>/<slug>.json`, `cadences: [pre-push, pre-pr]`, and add `minted_at` in the SAME first commit (catalog-first rule, `quality/CLAUDE.md`) | The playwright pattern is source→artifact assertion, NOT a live browser walk at commit time (see `mermaid-renders.sh:11-33` rationale). Reuse the `mkdocs-strict` row (`docs-build.json:6-40`) if only build-validity coverage is wanted |

---

## Furnished-product style exemplars (the gold-standard pages to match)

The owner mandate (`GOOD-TO-HAVES.md:270`, ROADMAP `:147-150`) is "furnished product,
not merely doc-truth-correct." These pages already hit that bar — copy their moves:

1. **`docs/index.md` — the single richest exemplar.** It is the ONLY published page
   using the full mkdocs-material feature set: grid cards (`:15-21`, `:150-158`),
   a mermaid sequence diagram (`:25-40`), content tabs (`=== "curl…"`, `:46-65`),
   a collapsible `<details markdown>` "advanced" fold (`:69-85`), md-button
   attr_list CTAs (`:106-107`), a blockquote hero callout (`:11`), and
   backtick-emphasized inline metrics (`:17-19`). Any new furnished section (the
   animation, a polished nav landing) should draw from here.

2. **`docs/how-it-works/filesystem-layer.md` — the how-it-works template.** Despite the
   SC2 truth defect in its summary, its STRUCTURE is exemplary: a bold "Plain-English
   summary" lede (`:7-13`), a `---` rule, one mermaid flowchart (`:21-40`), "What lives
   where" / "Failure modes" / "Next" sections, and dense glossary-anchored cross-links
   (`:48,56,57`). The other how-it-works pages (git-layer, time-travel, trust-model)
   should be leveled up to this shape.

3. **`docs/concepts/reposix-vs-mcp-and-sdks.md` — admonition + measured-claim style.**
   Uses `!!! note "About the MCP comparison (live, 2026-07-16)"` (`:23`) to scope a
   live-number caveat inline — the cleanest admonition-for-honesty pattern on the site.

4. **`docs/tutorials/first-run.md` + `docs/guides/integrate-with-your-agent.md` —
   admonition variety done right.** `!!! note "Comments are connector-specific"`
   (`first-run.md:132`) and `!!! info "What this guide assumes"`
   (`integrate-with-your-agent.md:7`) show the two sanctioned admonition types and the
   "title in quotes + blank line + 4-space-indented body" convention.

---

## Shared patterns (cross-cutting — apply to all relevant plans)

### Error-message 3-part bar (SC3, SC4-route-B)
**Source:** `crates/reposix-cli/src/init.rs:130-154` (`refuse_existing_repo_root`) +
`init.rs:363-373` (the fetch-failure `bail!`). **Apply to:** every CLI error this phase
touches. Bar (from `crates/CLAUDE.md` § Error-message convention): (1) teach the fix,
(2) suggest the alternative, (3) give a copy-paste recovery command. A bare
`.context("…")` that surfaces `Connection refused` raw does NOT qualify.

### Mermaid embed + render workaround
**Source:** fenced ` ```mermaid ` blocks (`index.md:25`, `filesystem-layer.md:21`) →
superfences `fence_div_format` (`mkdocs.yml:80-84`) → rendered by the custom
`docs/javascripts/mermaid-render.js` (Material strips `div.mermaid`; the script re-fetches
+ `mermaid.render()`s, and subscribes to `document$` for instant-nav). **Apply to:** any
new diagram — do NOT hand-roll `<div class="mermaid">`; use the fence, and expect a
required playwright artifact.

### Relative-link + glossary-anchor cross-links
**Source:** `filesystem-layer.md:48,56,57` (`../reference/glossary.md#partial-clone`).
**Apply to:** all SC1/SC2/SC5 doc edits. Enforced by `docs-build/link-resolution`
(`docs-build.json:78-108`) — a broken relative path fails pre-push.

### Catalog-first row minting
**Source:** `quality/catalogs/docs-build.json:42-76` (mermaid-renders row) +
`quality/CLAUDE.md` § Catalog-first rule. **Apply to:** any gate this phase adds. The
FIRST commit writes the row (with `minted_at` in the same commit); the verifier grades
rows that predate the implementation.

---

## No analog found

| Artifact | Reason | Planner guidance |
|---|---|---|
| `docs/assets/animation/` directory + embedded-media (`<video>`/iframe/JS-bundle) on a **published nav page** | **No `docs/assets/` dir exists** (assets today live in `docs/social/assets/`, `docs/screenshots/`, `docs/javascripts/`, `docs/stylesheets/`), and **no published nav page currently embeds an inline image or media element** — index.md's only visuals are grid cards + mermaid. The two `docs/*/assets/` + `docs/screenshots/` trees are BOTH `not_in_nav` / social-only. | Establish the convention deliberately per GTH-V15-37 checklist item 5 (`GOOD-TO-HAVES.md:282`): assets under `docs/assets/animation/`, strip Windows `Zone.Identifier` files, mkdocs-strict + playwright-walk coverage. Closest wiring analog is the `extra_javascript`/self-hosted-`mermaid-render.js` pattern; closest embed-mechanics analog is index.md's `md_in_html`+`attr_list` grid-card block. The 7.1MB mp4 (`GOOD-TO-HAVES.md:283-286`) is a GitHub-release attachment, NOT committed (file-size gate `structure/file-size-limits`). |

---

## NOTICED (convention inconsistencies across the docs site — deliverable per charter)

Read-only; filed as observations for the planner. Severities are my estimate.

1. **[MEDIUM] Furnished-product features are quarantined to `index.md`.** Grid cards,
   content tabs, `<details>` folds, and md-button CTAs appear on EXACTLY ONE page
   (`docs/index.md`). Every how-it-works / concepts / guides page is plain prose + at
   most one mermaid diagram. The owner's "we can do so much better" (`GOOD-TO-HAVES.md:270`)
   maps directly onto this gap: the IA/polish pass should propagate index.md's furnishing
   vocabulary (cards for "where to go next", `<details>` for advanced material,
   admonitions for caveats) across the how-it-works quartet. Fits P117's cold-reader
   mandate — file as GTH if it can't land in-phase.

2. **[LOW] Admonition usage is near-absent — only 3 of ~30 nav pages use `!!!`.**
   `first-run.md`, `integrate-with-your-agent.md`, `reposix-vs-mcp-and-sdks.md` use them
   well (`note`/`info`); the rest bury caveats in prose. `pymdownx.details` + `admonition`
   are both enabled (`mkdocs.yml:99-100`) but under-used. Honesty caveats (e.g.
   token-economy's "What this does NOT measure", `token-economy.md:52-65`) read as prose
   where an `!!! warning` would furnish them.

3. **[LOW] `docs/social/*` escapes ALL docs gates.** It is `not_in_nav` (`mkdocs.yml:56`),
   so mkdocs-strict, link-resolution, doc-alignment, and banned-words never scan it. That
   is exactly why the SC5 stale-FUSE line (`twitter.md:16`) survived to launch. Consider a
   lightweight freshness-invariant that greps `docs/social/**` for known-dead terms
   (`FUSE`, `/mnt/`, `mount`) — file as GTH (natural home: P126 doc-alignment polish lane,
   sibling to GTH-V15-41's banned-words scope gap).

4. **[LOW] Two truth defects sit adjacent to already-correct copy on the SAME page.**
   `filesystem-layer.md` lies in its summary (`:8-9`) but tells the truth in its body
   (`:42`); `index.md` frames Confluence as an "issue tracker" in the hero (`:13`) but
   correctly as a wiki-capable connector in the matrix (`:130-137`). The SC edits are
   small internal-consistency fixes, not rewrites — low risk, but a reviewer should read
   the whole page so the fixed summary matches the body's existing numbers/claims.

5. **[LOW] `token-economy.md` mixes en-dash `--` for bullet dashes** (`:15-22`, `:47-50`)
   where the rest of the site uses `—` or `-`. Cosmetic; the file is machine-regenerated
   (`bench_token_economy.py --offline`, `:24-26`), so a hand-edit to the provenance line
   (SC5) risks being overwritten — the planner must fix the PROVENANCE STRING in the
   generator's template, not just the rendered `.md`, or the next regen reverts it.
   (Verify: does the SC5 line come from the generator or is it static in the committed md?)

---

## Metadata

**Analog search scope:** `docs/` (index, how-it-works, concepts, tutorials, guides,
benchmarks, social, stylesheets, javascripts), `crates/reposix-cli/src/`
(init/attach/list/refresh/main), `quality/catalogs/docs-build.json`,
`quality/gates/docs-build/`, `mkdocs.yml`, ROADMAP + GOOD-TO-HAVES + CONSULT-DECISIONS.
**Files scanned:** ~24 (docs + CLI + catalog + config).
**Pattern extraction date:** 2026-07-16.
**Cargo builds run:** none (one-cargo-machine-wide rule honored).
