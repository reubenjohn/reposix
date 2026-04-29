# Persona audit: MCP user evaluating reposix

**Persona:** Senior dev at a 50-person SaaS shop. 4 months on Anthropic's MCP for GitHub Issues + a homegrown MCP for internal Confluence. Claude Code + Cursor team license daily. Runbooks, training, sunk cost. Arrived from a HN link with a snarky #eng-tools comment about "the MCP tax." Stance: cautiously interested; prior is "another shiny thing that 80%-solves what we already do."

**Site:** https://reubenjohn.github.io/reposix/ (live; header chip `v0.8.0`).

---

## TL;DR (the headline I'd give my skeptical CTO)

> Smart positioning ("agents already know `cat` and `git`"), one good comparison table, a punchy `curl`-vs-`sed` `before/after`. Three things stop me forwarding this: (1) the headline claim — *"92.3% token reduction vs MCP"* — links to nothing (404 on `/benchmarks/RESULTS/`); (2) the header chip shows `0 stars / 0 forks` and `v0.8.0` while the page text talks about "v0.9.0 architecture pivot"; (3) the latency table that backs the "8 ms" hero number has the GitHub / Confluence / JIRA columns **blank**. The numbers I'd need to argue against MCP — *measured* latency on real backends — are not on the site yet. Thesis is good. Evidence is half-shipped. Bookmarked, not installed; back in 4 weeks.

---

## Walk transcript (what I clicked, what I saw, what I thought)

**0–10s, home page.** Clean Material theme. Tagline: *"Agents already know `cat` and `git`. They don't know your JSON schema."* That's a great hook — names the exact pain I feel (every new MCP server is another schema my agent has to load). The next paragraph is explicitly anti-MCP positioning. I stay. But I also notice: header chip reads `v0.8.0 · 0 stars · 0 forks`. No logo. No screencast. No diagram above the fold. HN-reflex: *who's actually using this?*

**10–60s.** I scroll. The `5 curl lines → 3 sed/git lines` `before/after` works — vivid. Then "Three measured numbers": 8 ms cache read, 0 schema tokens, 1 bootstrap command. Decent, but **8 ms vs what?** My MCP tool dispatch is ~250 ms, so 8 ms is impressive — but I have to do that math myself. Two CTAs: "Mental model in 60 seconds" and "How it complements MCP and SDKs." I click the second.

**Click 1 — `concepts/reposix-vs-mcp-and-sdks/`.** This page is the strongest for my persona. The comparison table (tokens before first op, latency, conflict semantics, pre-training overlap, egress surface, audit trail) is the slide I'd send my CTO. The footnote that reposix's numbers are *measured* but MCP/SDK numbers are *characterized from public-API behaviour* is fair. I'd push back on the "~100k MCP schema discovery" cell — one server is ~10-20k, but I grant the multi-server compounding. The "Use reposix for / Use REST or MCP for" split is the right pitch — they're not claiming a replacement for JQL or bulk imports. That softens my "land grab" reflex.

**Click 2 — first-run tutorial.** Eight copy-pastable steps. Install has 5 tabs (curl, PowerShell, Homebrew, cargo binstall, source) — credible distribution. But the home page and mental-model page both lead with `cargo build --release --workspace --bins`; the tutorial path is the right one; home should agree. Step 4 makes me type `git checkout -B main refs/reposix/origin/main` — non-default refspec, docs call it "one line of friction at clone time." Step 8 reads an audit row from SQLite. **That audit table is a strong MCP differentiator** — every MCP server reinvents logging; here it's a `sqlite3` query I could pipe to Splunk. I'd demo this to security.

**Click 3 — trust model.** Lethal-trifecta framing, explicit Simon Willison cite, mitigations table, *and a "What's NOT mitigated" section*. Bigger trust signal than 1000 stars — someone is thinking adversarially and naming what they didn't fix. `REPOSIX_ALLOWED_ORIGINS` loopback-default is exactly the property my MCP servers don't have. Net positive.

**Click 4 — `/benchmarks/v0.9.0-latency/`.** Where the home page's hero number should be defended. The latency table:

| Step | sim | github | confluence | jira |
| --- | --- | --- | --- | --- |
| `reposix init` cold | 24 ms | (blank) | (blank) | (blank) |
| List records | 9 ms | (blank) | (blank) | (blank) |
| Get one record | 8 ms | (blank) | (blank) | (blank) |

**The columns I care about are empty.** Footnote: "Real-backend cells are populated by the bench-latency-v09 CI job." So this is sim-only — a localhost in-process HTTP simulator, which is a transport-overhead lower bound, not "what an agent will see against my Atlassian tenant." Interest cools.

**Click 5 — chase the 92.3% number.** Home page footnote: *"The v0.7 token-economy benchmark measured a 92.3% input-context-token reduction vs MCP for the same task."* Only head-to-head MCP number on the site. Latency page mentions `benchmarks/RESULTS.md` — plain text, not a link. I navigate to `/benchmarks/RESULTS/`. **404.** The boldest claim sources to a missing page. Either the doc didn't ship, the number is from a deprecated v0.7 architecture (home says v0.9.0 pivoted from FUSE), or it was never re-derived. Can't forward this to my CTO.

**Click 6 — left-nav tour.** "Decisions" section exposes ADR-001 through ADR-008. ADR-002/003 reference deprecated FUSE architecture. I'm reading internal scaffolding. The sitemap also has orphans not in nav: `/why/`, `/security/`, `/architecture/`, `/demo/`, `/development/contributing/`, `/development/roadmap/`, `/research/initial-report/`, `/social/{linkedin,twitter}/`. `/why/` and `/security/` are stub redirects saying "this page moved during the v0.10.0 narrative overhaul." Site is mid-restructure; seams visible.

**Click 7 — GitHub repo (header link).** Single-author repo. No shields.io anywhere on the docs. No "who's using this."

**5-minute verdict.** Close the tab. Bookmark vs-MCP page and trust-model page. Don't install. Come back when: one real-backend latency row populated, 92.3% claim has a methodology page, header chip stops contradicting itself.

---

## Friction inventory (ranked, P0 first)

### P0 — kills the deal

1. **Hero claim links to a 404.** *"92.3% input-context-token reduction vs MCP"* on the home page → `/benchmarks/RESULTS/` returns 404. The only head-to-head MCP number on the site is unverifiable. Either ship the page, replace the number, or remove the claim.
2. **Latency table real-backend columns are empty.** Home page promises "tested against three real backends." Benchmark page shows sim-only with a "CI job will fill these" footnote. The 8 ms hero number is in-process loopback, not Atlassian Cloud — the comparison I actually need against my MCP path is missing.
3. **`0 stars / 0 forks · v0.8.0` in header chip.** Stars are gameable, but literal zero is a HN-reflex trigger. Worse: chip says `v0.8.0`, page text references "v0.9.0 architecture pivot," URL is `/benchmarks/v0.9.0-latency/`. Either ship v0.9.0 release or update the chip.

### P1 — meaningful friction

4. **No author / backing / "honest scope" callout above the fold.** No logo, no bio, no design partners, no shields, no license, no SECURITY.md link in footer ("Made with Material for MkDocs" is the entire footer). The honest-scope line *"Treat as alpha — every demo is reproducible on stock Ubuntu in <5 min"* IS on the home page, but it's the LAST paragraph. That line earns trust; move it up as an admonition.
5. **Side-nav exposes internal ADRs.** Top-level "Decisions" peer of "Tutorials" — ADR-002/003 reference deprecated FUSE architecture. Gate ADRs behind a Contributing section or remove from public site.
6. **Compile-vs-curl inconsistency.** Home page quickstart and mental-model page both lead with `cargo build --release` — implies I have to compile a Rust workspace. First-run tutorial offers a `curl … | sh` installer. Make the home page lead with the installer.
7. **Zero diagrams above the fold.** Architecture is described in prose. Tool whose thesis is "we replace a complicated thing with a simpler thing" needs a 3-box picture (cache + helper + init) on the home page.
8. **No demo video / asciinema / GIF.** The sitemap has `/demos/asciinema-script/` but it's not linked. A 30-sec terminal-cast above the fold (MCP path vs reposix path side-by-side) would beat the prose `before/after`.

### P2 — annoyances

9. **`/why/` and `/security/` are stale redirect stubs** still in sitemap and search results. Either 301 server-side or remove from sitemap.
10. **Search surfaces orphan pages.** Querying "install" returns "Contributing → Pre-commit hooks" — but Contributing isn't in side nav. Search shows pages I can't reach from navigation.
11. **`refs/reposix/origin/main` is a checkout-time friction the docs apologize for.** Why not have `reposix init` set up a local `main` ref tracking it, so the user never types the namespaced ref? Docs treat it as inevitable; I treat it as a missing convenience.
12. **No copy-button affordance** visible on the hero `before/after` code blocks (Material has them — make them obvious).
13. **No license / privacy / contact in footer.** Need at least: license shield, SECURITY.md link, mailto for vuln disclosure.
14. **Material instant-nav can race the prefetcher** on slow connections — clicking link X can occasionally settle on link Y. Bad first impression on flaky wifi.
15. **Internal-phase numbering leaks onto public site.** Phrases like "Phase 36" / "OP-1" / "v0.3 shipped" appear on benchmark and ADR pages. That's `.planning/`-speak; on the public site say "next release" or "v0.10.0."

---

## What WOULD make me install today

1. **One real-backend latency cell, populated.** Not all 9. *One* row × *one* backend that shows reposix beats my MCP path against an actual REST API. "List records, GitHub Issues: 47 ms" beats every other claim on the site.
2. **The 92.3% page actually publishes.** Methodology, raw numbers, what task, what model, what MCP server. If it's stale (v0.7 vs v0.9.0 pivot), say so and re-run.
3. **30-second asciinema on the home page**: agent doing a tracker task via MCP (5 round trips) vs same task via reposix (one `git push`). Side-by-side. No prose needed.
4. **An "alpha — here's the v1.0 ETA" callout near the top** plus license shield + SECURITY.md link. Honesty is a feature when the tech is genuinely interesting.
5. **A "Migrate from MCP" page.** What stays on MCP (complex JQL, bulk, admin)? What moves? Gotchas? This page is worth more to me than another concept page.

---

## What MUST be on the home page (above the fold)

1. The current tagline (it works).
2. A 3-box architecture diagram (cache + helper + init).
3. **One** measured head-to-head number vs MCP, with a clickable methodology link. The 8 ms / 0 / 1 box without an MCP-baseline cell isn't enough.
4. A one-line install command (`curl … | sh`) — not `cargo build --release`.
5. Honest-scope callout up top, NOT in the last paragraph.
6. Social proof or its honest absence: "Built solo, looking for design partners — email X" beats `0 stars` showing in a chip.
7. Footer with license, SECURITY.md, contact.

---

## What I'd cut from the home page

1. **The `cargo build --release` six-line quickstart.** Replace with the curl installer; move the source-build path to Contributing.
2. **The "Tested against" section as currently worded.** Until the columns have data, it overpromises. Replace with one line: "Working against GitHub, Confluence, JIRA — measured numbers in [Latency benchmarks]."
3. **Internal phase references.** "Phase 36 / OP-1 / Phase 11 shipped" don't belong on a public marketing page.
4. **The "Decisions" section in side nav** as a top-level peer of Tutorials. Move ADRs to `/contributing/decisions/` or hide from public site.
5. **Orphan stub pages** (`/why/`, `/security/`, `/architecture/`, `/demo/`) — delete or 301-redirect; don't ship in sitemap.

---

## Final verdict (unsoftened)

The thesis is genuinely good. *"Agents already know `cat` and `git`"* is the kind of one-liner I'd quote in a Slack thread. The trust-model page is more rigorous than what most MCP servers ship. The vs-MCP comparison earns the click. But the site reads as a research project mid-pivot (FUSE → git-native), and three load-bearing claims (token reduction, latency, real-backend coverage) point to artifacts that aren't there yet. As an evaluator with an MCP investment, I need EVIDENCE, not a roadmap. Populate one benchmark row, ship the 92.3% methodology page, fix the version chip — I'll pilot in 4 weeks. Right now: bookmarked, not installed.
