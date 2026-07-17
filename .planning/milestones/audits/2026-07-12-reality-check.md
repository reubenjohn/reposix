# reposix Reality Check — read-only audit (2026-07-12)

Prepared for the project owner and the incoming manager session. Seven read-only audit
lanes: docs truth-rot, IA/nav, CLI skeptical-dev, benchmarks honesty, bloat map,
quality-gate blind-spot diagnosis, live-site visual walk. Most lanes ran twice (an
accidental cancel/resume produced paired passes); pairs are unioned below, with the two
substantive disagreements stated precisely rather than averaged. No code, docs, or planning
artifacts were modified.

> **UPDATE 2026-07-17 (P118 · DOCS-09) — STALE-TAG PREMISE SUPERSEDED.** This 2026-07-12
> audit repeats a "cut the two stalled/uncut owner-gated tags (v0.13.0, v0.14.0)" premise in
> five places (§2 bloat map ~L338, §3 Exhibit E ~L410, §4 punch-list ~L467, §4 Option ii
> ~L483, §5 open-question #2 ~L550). **That premise is RESOLVED.** `git tag -l` shows
> **v0.13.0, v0.13.1, v0.14.0 all cut**, and all three are **published GitHub releases**
> (`gh release list` → **v0.14.0 = "Latest"**, v0.13.1, v0.13.0). The archival cascade has
> since run: P78-94 → `.planning/milestones/v0.13.0-phases/`, P102-113 →
> `.../v0.14.0-phases/`; neither range remains in live `.planning/phases/`. So the "stalled
> tags block the archival cascade" framing recurring below no longer holds — the tags exist
> and the cascade completed. **This is a prose correction, NOT a literal tag cut** (the tags
> already exist); the original 2026-07-12 sentences are preserved verbatim below as the
> historical audit record — do NOT act on them. Companion: §5 open-question #1 (~L547, "is the
> v1.0 headline the honest-but-synthetic 89.1%, or fund a live-MCP-server re-measurement?") is
> likewise **RESOLVED** — P115 funded the live re-measurement (`docs/benchmarks/token-economy.md`).

## §1 North star recap

v1.0 = git as the native interface for agent access to systems of record. A skeptical dev
installs in one command, attaches real JIRA/Confluence/GitHub, and agents ls/cat/grep/edit/
git-push records with zero JSON-schema or REST awareness — inside audited lethal-trifecta
security cuts, with token claims backed by honest CI-verified benchmarks. The familiar
"~2k tokens per op vs ~150k MCP-mediated" figure carries an evidence flag: it is FUSE-era
and unevidenced in-repo (§2 Lane 4, §5 Q1). The north star is FIXED and open-ended; every
milestone toward it is TENTATIVE by design — planned milestones alternate with reserved
stub milestones (own version numbers) for surprises, maintenance, or pivots (§4). v0.14.0 just closed GREEN awaiting tag; the
previously planned next milestone was OD-4 "launch-readiness", whose premise this audit
revises.

## §2 Reality check — gap inventory

### Lane 1 — Docs truth-rot sweep (owner's three examples confirmed, then 14 more)

**Owner examples — all three CONFIRMED, two worse than reported:**
- LAUNCH-BLOCKER — docs/index.md:13 category+scope error confirmed ("REST-based issue
  trackers (Jira, GitHub Issues, Confluence)"). Fix shape: "systems of record — issue
  trackers (Jira, GitHub Issues) and wikis (Confluence)". Rider: the same sentence says
  agents "can git clone" — the actual bootstrap verb is `reposix init`.
- LAUNCH-BLOCKER — filesystem-layer.md is WORSE than reported: L7-10 "secretly trigger a
  network call", L19 heading "How a cat becomes a REST call", L42 "first cat triggers one
  REST call", L63 "cat fails" on network-down; the whole mermaid (L21-40) is built on the
  false premise AND the page self-contradicts (L57 says reposix never touches the tree except
  git fetch/push).
- LAUNCH-BLOCKER — the page TITLE "Filesystem layer" is itself architecture rot (the page
  admits "there is no virtual filesystem" at L17), propagated by cross-links in index.md:144,
  git-layer.md:120, time-travel.md:90/92, trust-model.md:103.
- MEDIUM — redundancy confirmed and worse: FOUR links to token-economy.md on the landing page
  (index.md L17, L42, L146, L158).

**New findings:**
- HIGH — integrate-with-your-agent.md:48+88 claims "`reposix list`… removed in v0.9.0" and
  "init… is the entire user-facing surface" — flatly false; contradicts cli.md:102-117 (~15
  subcommands; the removed-list at cli.md:325-330 has only mount+demo).
- HIGH — cli.md:59 "blobs fetched on demand on first read" — the read=fetch rot inside the
  CLI reference itself.
- HIGH — git-remote.md:11,36,52-54 describe stale v0.1 "import capability" for pulls
  (reality: stateless-connect); contradicts git-layer.md:53-58, glossary.md:48, and its OWN
  capabilities block at L17-21.
- HIGH — docs/social/twitter.md:16 calls the current product "a FUSE filesystem +
  git-remote-helper for issue trackers"; linkedin.md:21 was updated, twitter was not — the
  two social surfaces now disagree (linkedin.md:36 still says "mount layout": MEDIUM).
- MEDIUM — 24ms-vs-27ms cold-init: mental-model:21/69, vs-mcp:15, first-run:73 all say 24ms
  while LINKING latency.md:36 which says 27ms.
- MEDIUM — Padded `issues/0001.md` examples on 7+ pages (mental-model:17/18/28, glossary:176,
  dvcs-topology:108/125/142, git-remote:64/72/84, troubleshooting:316/317, index:32 mermaid)
  vs the real UNPADDED `1.md` (first-run:95 sim output; confluence.md:131 canonical) —
  copy-pasting `cat issues/0001.md` against the sim FAILS with no-such-file.
- MEDIUM — http-api.md:147 "still 4-digit padded" + references "the FUSE mount's 11-digit
  form" as current.
- MEDIUM — ADRs 001 (:16,:27), 002 (:19), 005 (:26,:91) present FUSE rendering in PRESENT
  tense, Status Accepted/Active, no superseded banner (ADR-003 has the correct banner — the
  model to copy).
- POLISH — "seven-step" references to the eight-step tutorial (integrate:93,
  troubleshooting:413 vs first-run:7); "8ms cached read" label actually names the
  get-one-record helper op (latency.md:38) while a cat of a materialized file is sub-ms —
  the label undercuts the local-reads selling point; generic "issue tracker" recurs on 7 more
  pages (git-layer:8, integrate:32/87, latency:29, time-travel:86, dvcs-topology:101,
  simulator:14).

**Second-pass additions (50 pages swept; ~23 with ≥1 defect, ~27 clean):**
- HIGH — docs/development/roadmap.md:5-24 stops at "shipped v0.10.0, planning v0.11.0" —
  four shipped milestones (incl. the whole DVCS story) unlisted; a skeptical dev reads the
  project as a year staler than it is.
- HIGH — git-remote.md also claims "v0.1 does not authenticate outbound requests" (:115-124,
  false — GITHUB_TOKEN/ATLASSIAN_*/JIRA_* auth today) and "recomputes the world on every
  pull … v0.2 deliverable" (:126-131).
- MEDIUM — cli.md:91 "sim … as a child process" (it's in-process); cli.md:349-351 exit-code
  table contradicts exit-codes.md; cli.md:5 "as of v0.9.0" then lists v0.13 commands.
- MEDIUM — crates.md:60 says helper advertises `import` (it rejects it); crates.md:3 "none
  publish to crates.io yet" contradicts the advertised `cargo binstall`.
- MEDIUM — confluence.md:6 names the pre-v0.8 `IssueBackend` trait; :10-11 "write path lands
  later in v0.4" while :170-178 documents it shipped.
- MEDIUM — glossary.md:20 "contents arrive lazily on first `cat` or `grep`" (the cat-myth in
  the glossary); http-api.md:3/:74 v0.1/v0.2 framing; index.md:93 + testing-targets.md
  (x4) "Phase 36" internal jargon; integrate-with-your-agent.md:32/:98 "ships in v0.12.0
  (planned)"; linkedin.md:9-11 "as POSIX file systems" + lists Google Keep (never a
  backend; twitter.md:7 too); agentic-engineering-reference.md:30/139/144 FUSE framing in a
  doc the pointer map cites as current, unbannered.
- POLISH — jira.md:71 "mounted issue file"; simulator.md:36 "five issues" (seed is six);
  glossary gix pin =0.82 vs workspace 0.83; contributing.md:151 stale SECURITY.md
  forward-ref; ADRs 001/002 pre-rename type names (passes disagreed on ADR-banner severity:
  one graded missing-superseded-banners MEDIUM, the other found 002/003 adequately bannered
  — the real exposure is the nav, see Lane 7).

**Clean (verified, don't flag):** zero dead links/anchors, zero orphans; docs/research/*
correctly bannered historical; git-layer, trust-model, time-travel, dvcs-topology,
troubleshooting, dvcs-mirror-setup, write-your-own-connector, exit-codes, latency.md and
token-economy.md page-level prose are architecturally correct; why/security/connectors-guide
redirect stubs correct. Highest-leverage single fixes: rewrite filesystem-layer.md (and
rename it), retire/rewrite git-remote.md, fix index.md:13, refresh roadmap.md, delete the
two false "reposix list was removed" claims.

### Lane 2 — Information architecture / nav (mkdocs.yml nav block: lines 118–167)

**Numbers:** 40 nav leaf items; 9 top-level entries; max depth 3; **8 ADRs in nav**
(mkdocs.yml:148–155) + **10 research pages** (mkdocs.yml:156–167) = **18 internal items =
45% of the nav**. Orphan pages: 0 (strict-mode clean — `not_in_nav` covers the rest).
Cutting ADRs + Research alone: 40 → 22 leaves (−45%).

**Newcomer journey verdicts:**

| Goal | Where it actually lives | Verdict |
|---|---|---|
| What is this? | Home + Concepts › "Mental model in 60 seconds" | OK |
| Install | Buried inside Tutorials › "First run (5 min)" §1 — no nav label says Install | HIGH friction |
| First run | Tutorials › First run (only tutorial; simulator-only, no bridge to real backend) | MEDIUM |
| Connect Confluence | Real walkthrough hides behind Guides › **"DVCS mirror setup"** (mkdocs.yml:134); Reference › "Confluence backend" is 6th of 7 reference items | **LAUNCH-BLOCKER** (confirms owner finding 4) |
| Connect JIRA | Reference › "JIRA backend" — setup filed as reference | HIGH |
| Connect GitHub | **No page exists at all** (no `docs/reference/github.md`; passing mention in first-run.md only) | **LAUNCH-BLOCKER** |

**Itemized:**
- LAUNCH-BLOCKER — No "Connect GitHub" page anywhere; north star names GitHub Issues first-class.
- LAUNCH-BLOCKER — Confluence setup undiscoverable: content exists but is labeled "DVCS mirror setup".
- HIGH — ADRs (8) + Research (10) in user nav; single largest clutter source (mkdocs.yml:147–167).
- HIGH — No top-level Install / Getting-started nav entry.
- HIGH — No unified "Connect your backend" section; backends scattered across Reference and Guides.
- MEDIUM — Reference section mixed: jira/confluence are setup-masquerading-as-reference; simulator/testing-targets are contributor-facing (mkdocs.yml:140–143).
- MEDIUM — Tutorials section has exactly one item; quickstart dead-ends at the simulator.
- POLISH — "Write your own connector" (contributor guide) sits top-of-Guides (mkdocs.yml:132).

Net: nav is Diátaxis-shaped but launch-hostile — 45% internal artifacts, no install label,
and the three backends a buyer wants to connect are unlabeled, scattered, or missing.

**Extended IA findings (second pass on the same lane):**
- LAUNCH-BLOCKER (sharpened) — GitHub backend setup has no nav destination and no
  `docs/reference/github.md`; setup facts scattered across five files: testing-targets.md:135,
  cli.md:341, exit-codes.md:59 and :139, git-layer.md:76, troubleshooting.md:152. Mitigating
  factor: the GitHub backend is read-only and needs one env var — a single short page fixes it.
- HIGH — Clutter is structural, not just count: `navigation.sections` is on with
  `navigation.tabs` off, so all 18 newcomer-irrelevant items (8 ADRs + 10 Research) render
  **always expanded** in the sidebar — the single biggest visual-clutter driver.
- MEDIUM — Concepts vs How-it-works are two competing explanation buckets (trust-model vs
  mental-model placement unpredictable); "connector" vs "backend" terminology split.
- MEDIUM — Three substantive pages reachable only via in-body links (reference/http-api.md,
  git-remote.md, crates.md); Decisions (8 children) and Research chapters (8) both exceed a
  ~7-children scanability budget.
- POLISH — Moved stubs (why.md, security.md, connectors/guide.md) lack redirect entries;
  Reference mixes user and contributor audiences.
- **Reconciliation (resolved by Lane 7):** install discoverability — the hero/Home covers
  install functionally (Lane 7 grades the hero PASS), so "no Install/Getting-Started nav
  entry" settles at MEDIUM, not HIGH. A first-class nav entry is still the fix.

### Lane 7 — Live-site visual audit (two passes: curl HTML sweep, then full playwright browser)

Site: https://reubenjohn.github.io/reposix/ — header shows **v0.13.1** while v0.14.0 closed
(deploy lag worth checking at tag time).

- PASS — **Hero: strong.** 10-second test passes at 1280x800 AND 390x844; WHAT/WHO/NEXT all
  on the first screen ("Agents already know `cat` and `git`. They don't know your JSON
  schema." + 89.1% / 8ms-27ms / 5-line-install cards + inline 30-second install). Minor nit:
  bare "reposix" H1 with 3 CI badges stacked above the value prop.
- **"Connect my Confluence" — the two passes agree on facts, disagree on grade.** Facts:
  Guides (where a newcomer looks first) has NO backend-setup entry → first click dead-ends;
  Home's "Where to go next" never links backend setup; the quickstart connects to the SIM;
  the actual content — excellent, complete with env vars + credential steps — lives under
  Reference → "Confluence backend", which reads as API docs. Curl pass graded LAUNCH-BLOCKER
  (dead-end); browser pass graded SUCCESS-WITH-FRICTION/HIGH (content findable on the second
  guess). Resolution: HIGH as the site stands, LAUNCH-BLOCKER against a Show-HN newcomer
  funnel where the first dead-end loses the reader. Confirms owner finding 4 either way.
- HIGH — Nav clutter as rendered and auto-expanded: ~40 leaves / 8 sections + Home;
  Decisions (8 ADRs) + Research (~11) ≈ 48% internal. **A nav label literally reading
  "Research → FUSE architecture" contradicts the git-native model at a glance** (page body
  correctly bannered historical — the nav placement is the exposure).
- MEDIUM — No Install / Getting-Started / Connect-a-backend nav label (install content
  itself: 0-1 clicks via hero — SUCCESS; settles the Lane 2 reconciliation at MEDIUM).
- CLEAN (browser-verified) — no 404s across both passes' link sweeps (~50 links incl.
  anchors + externals); all mermaid diagrams render at both widths; 390px reflow clean
  (cards stack, code blocks scroll internally, no page overflow); contrast OK; live
  FUSE/mount mentions outside Research/ADR nav entries are strictly historical.
- POLISH — token-economy table renders literal `** 531**` (unparsed bold inside table cell);
  connectors/guide.md published but nav-orphaned.
- Screenshots (8): audit-01/02 (landing desktop viewport/fullpage) landed at the REPO ROOT,
  audit-03..08 under /home/reuben/workspace/reposix/.playwright-mcp/ — playwright MCP
  refused the scratchpad path and resolved against cwd. Untracked audit droppings in the
  repo tree; disclosed in §5 Q8; safe to delete.

### Lane 3 — CLI skeptical-dev audit (crates/reposix-cli; two passes merged)

- LAUNCH-BLOCKER — **The natural first exploratory command strands a newcomer.**
  `reposix list` / `reposix refresh` with the sim not running yield an OPAQUE reqwest
  connection-refused with zero teaching (list.rs:80, refresh.rs:207) — and sim is the DEFAULT
  backend. init.rs:370 proves the 3-part fix text exists in the codebase; these paths simply
  don't reuse it.

Beyond that, the golden documented path is solid: `init` with the sim down bails at
init.rs:365-366 with an exemplar-grade 3-part error; docs quickstart commands match the clap
definitions 1:1; no user-reachable panics. Strongest surfaces: doctor.rs (DoctorFinding
structurally carries a `fix:` line for every WARN/ERROR — the model to refactor toward) and
list.rs:204/254 credential errors. 15 subcommands total (sim, init, attach, list, refresh,
spaces, sync, doctor, history, log, at, gc, tokens, cost, version).

- HIGH — **Phantom recovery command**: attach.rs:135-138 says "Run `reposix detach` first" —
  `detach` does not exist in the Cmd enum; copy-paste yields a clap unrecognized-subcommand
  error. Propagated to docs/guides/troubleshooting.md:322.
- HIGH — **Benign-retry contradiction**: init.rs:365 says "re-run reposix init", but the retry
  hits the corruption-worded refuse_existing_repo_root guard (init.rs:130-154, which cites the
  2026-07-12 shared-tree incident); the actually-correct in-place `git -C <path> fetch` is
  buried as option #2. A newcomer concludes the tool corrupts repos on benign retry.
- HIGH — **First-run strand**: attach.rs:112-114 "not a git working tree: … (.git/ missing)" —
  no fix, no alternative, no recovery, even though attach's own help example says "attach CWD".
- MEDIUM — **CLI-side truth-rot**: cache_db.rs:101 tells the user to "unmount" — a FUSE-era
  concept deleted in v0.9.0, reachable via refresh under SQLITE_BUSY. (Pairs with docs rot, §3.)
- MEDIUM — Planning jargon leaks to users: `reposix sync` happy-path stdout cites
  "architecture-sketch.md § Performance subtlety … (P85 forthcoming)" (sync.rs:55-62); help
  texts cite DVCS-ATTACH-01..04, "L1 escape hatch", and unopenable .planning/*.md paths
  (main.rs:217,261,313,330).
- MEDIUM — Credential-error inconsistency: refresh.rs:227/250 ship terse bare-list errors while
  list.rs:204/254 has full teach-the-fix versions of the SAME failure; Atlassian env naming
  makes users set 4 vars for 2 secrets (ATLASSIAN_API_KEY vs JIRA_API_TOKEN, KEY vs TOKEN).
- MEDIUM — Bad-spec first-run errors (init.rs:48, :51, :104) teach the shape but give no
  runnable example — none reach the 3-bar exemplar standard sitting 260 lines away in the
  same file.
- MEDIUM — doctor prints literal `<cache-dir>` placeholder in "copy-paste" fixes
  (doctor.rs:561,576,581,632,905) despite knowing the real path; some fixes say `git fetch
  origin` where init requires `--filter=blob:none`.
- MEDIUM — Stale surface advertisement: main.rs:1-13 module doc + Cargo.toml:10 description
  list only 6 of 15 subcommands (crates.io renders the stale description); attach.rs:196-200
  duplicate-id abort has no recovery; bare `reposix sync`/`reposix log` error instead of
  acting; `refresh --offline` is a live flag whose only behavior is to error (refresh.rs:69).
- MEDIUM — refresh git-failure paths use `.status()` not `.output()`, so git's stderr is
  swallowed entirely (refresh.rs:300/313/344); rustdoc names REPOSIX_SIM_URL but the code
  reads REPOSIX_SIM_ORIGIN (backend_dispatch.rs:202); `log`/`history` near-duplicate
  subcommands.
- POLISH — ~10 bare git-passthrough failure errors (init.rs:541/559, attach.rs:386,
  gc.rs:103/110, cost.rs:113, tokens.rs:81); no after_help/getting-started epilogue anywhere
  (grep-confirmed) and nothing in top-level help points a stuck newcomer at `doctor`; doctor
  never checks backend credentials (greens on a credential-less confluence tree); `version`
  subcommand duplicates `--version`; project/space/key/owner-repo/record/issue vocabulary split.
- Positive exemplars worth replicating: attach proactively warns of a missing helper binary
  WITH the install command (attach.rs:331-340); doctor.rs's DoctorFinding fix-line structure;
  list.rs:204/254 credential errors.

### Lane 4 — Benchmarks honesty (three passes merged)

**The len÷4 fear is dead.** The 89.1% headline IS real Anthropic `count_tokens`
(claude-3-haiku-20240307, SHA-256-cached sidecars, `--offline` repro) — fixed in v0.10.0;
the untracked `.planning/phases/22-op-8-*` stub describes work already DONE (commit c804625);
1 − 531/4883 = 89.13% checks out. The dishonesty migrated one level up:

- LAUNCH-BLOCKER — **False provenance of the 531-token reposix fixture.**
  token-economy.md:51-52 + benchmarks/README claim `reposix_session.txt` is "the literal
  output of `scripts/demo.sh`". That script does not exist, and the fixture depicts the
  **deprecated FUSE architecture** (`/mnt/reposix/...` paths, internally inconsistent IDs).
  The reposix side of the headline number rests on a stale, hand-authored, misattributed
  artifact. (One earlier pass graded the baseline issue HIGH; the provenance discovery is what
  upgrades this to LAUNCH-BLOCKER against a v1.0 "honest" bar.)
- LAUNCH-BLOCKER — **Zero CI enforcement of the headline.** ci.yml has NO token-economy job
  (grep-confirmed; only bench-latency-v09). Catalog row `perf/token-economy-bench` is WAIVED
  until 2026-09-15; `perf/headline-numbers-cross-check` is WAIVED **and its verifier script is
  absent** (dangling row — perf-targets.json:114,121); quality-weekly.yml:40-48 admits in
  writing this keeps the weekly verdict "permanently yellow". The doc number can silently drift.
- HIGH — **Synthetic baseline**: the 4,883-token MCP side is a hand-modeled 35-tool catalog
  (not a live MCP transcript); full catalog counted against a ~3-tool task maximizes the gap;
  MCP output tokens excluded as "small and comparable".
- HIGH — **"Measured" framing on terse surfaces**: README.md:21 "## Three measured numbers";
  docs/index.md:17 hero card with no inline caveat — the honest "synthesized baseline" wording
  sits 141 lines below (index.md:158). README/concepts/social disclose inline; the hero is the
  outlier.
- HIGH — **The north-star figure itself is FUSE-era and unevidenced**: "~150,000 → ~2,000
  tokens (98.7%)" at docs/research/initial-report/performance.md:9 + initial-report.md:78
  cites uncited "benchmark studies", describes the abandoned FUSE architecture, contradicts
  the project's own measurement (89.1%, ≈9×, not ≈75×) by an order of magnitude — and is
  still quoted in .planning/PROJECT.md's Context ("~2k tokens vs ~150k"). Needs explicit
  retraction or relabeling as aspiration. (Feeds §5 — the v1.0 north-star sentence quotes it.)
- HIGH — Latency table cells hand-pasted (2026-04-27 snapshot, ~2.5 months old); the "cells
  match bench ±2ms" assert is deferred [v0.12.1] and its row WAIVED; CI runs the bench but
  nothing fails when the committed table diverges.
- MEDIUM — Tokenizer proxy (haiku-3) unnamed at point of claim; single scenario ("read 3
  issues, edit 1, push") generalized; concepts page says the 531 fixture was "captured against
  the in-process simulator" — inconsistent with its FUSE-era content; per-backend reductions
  85.5%/76.4% reuse the SAME 531 sim fixture presented as backend-specific; concepts' "~100k
  MCP" figure (hardcoded tokens.rs:214) sits 20x off the measured 4,883 one click from the
  hero; `reposix tokens` CLI still prints chars/4 (labeled, but a divergent second surface).
- POLISH — 27ms vs 24ms cold-init inconsistency (index.md:18 vs concepts:15); stale catalog
  source paths (perf-targets.json:52-53) and reproduce commands pointing at the pre-migration
  scripts/ shim; 89.1% baked into docs/social/assets/benchmark.svg.

**Blast radius of any headline change:** README, index hero + mermaid, token-economy.md,
concepts page, social assets + benchmark.svg, both perf catalogs, CHANGELOG, and
.planning/PROJECT.md.

### Lane 5 — Bloat map (quantified)

**.planning/ = 9.3M, 825 files.** phases/ 3.3M (28 dirs), milestones/ 2.8M, research/ 2.4M.
Top-level *.md ~208K/16 files; no runaway append file (STATE.md is 12K over 161 commits;
history-rotation already practiced).

- HIGH — **17 of 28 phase dirs (P78–P94, ~3.0M) belong to v0.13.0, CLOSED GREEN, and were
  never archived** — `.planning/milestones/v0.13.0-phases/` exists but got only the loose
  files, not the phase dirs. Per the gsd-cleanup convention these are overdue. This is the
  single largest crack-source: a fresh agent sees 28 dirs and cannot tell 17 dead ones from
  the live ones. Fix = run the existing `/gsd-cleanup`.
- HIGH — Untracked `.planning/phases/21-*, 22-*` (OP-7/OP-8 scratch): numbers collide with
  ancient P21/P22, invisible to git. Renumber+commit or delete — owner call.
- MEDIUM — v0.14.0 half-archived: P111/112/113 already moved, P102/105/106/110 not — the
  half-done state is the smell.
- MEDIUM — 999.2–999.6 debt-stub dirs (5, one file each): no convention covers them; fold
  into GOOD-TO-HAVES or prune.
- MEDIUM — research/ 2.4M includes v0.1-fuse-era (272K) + v0.9-fuse-to-git-native (364K)
  describing the deleted architecture, still cited as pivot source-of-truth — needs a
  research-retention convention (none exists).
- POLISH — quality/reports/ 5.3M/851 files is ~95% untracked regenerable fluff, already
  gitignore-rotated (tracked growth is controlled); periodic working-tree sweep would cut
  851→~330 files. Disk hygiene: target/ 119G, .claude/ worktrees 5.9G (gitignored;
  cargo clean / worktree prune candidates).
- Signal-to-noise: root CLAUDE.md is 20,149 chars (healthy vs the 40k budget); ~8-9
  mandatory-ish reads before first tool call + ~20 conditional via pointer map (load is real
  but mostly conditional); scripts/ is mostly referenced — orphans are the untracked
  scripts/demos/08-*.sh and scripts/dev/list-confluence-spaces.sh + 2 historical migrations.

**Second-pass deltas (merged):**
- HIGH — quality/reports/transcripts/ (523 files, 2.5M) appends one file per run with NO
  retention rule anywhere in quality/CLAUDE.md or PROTOCOL.md — unbounded by construction.
  (Reconciled with pass 1: git-TRACKED growth is gitignore-controlled; the working tree an
  agent walks is not.)
- HIGH — repo-root research/v0.13.0-dvcs/poc/ = 548M untracked and NOT gitignored.
- MEDIUM — .planning/ROADMAP.md is stale: still shows v0.13.0/v0.13.2 in progress, omits
  v0.14.0 entirely — the biggest slips-through-cracks doc a fresh agent hits. (A successor
  workhorse session is reportedly mid-flight on STATE/ROADMAP progressive-disclosure splits —
  reconcile before acting.)
- MEDIUM — Two stale catalog JSONs parked at .planning top level (v0.11.1-catalog.json 80K,
  docs_reproducible_catalog.json 26K); transient MANAGER/SESSION-HANDOVER files loose.
- **Causal link worth naming:** the archival cascade is convention-blocked on the un-pushed
  owner-gated tags (v0.13.0, v0.14.0) — /gsd-complete-milestone archives at tag time, so part
  of the bloat is a *symptom of stalled tag cuts*, not neglect. Cutting the two tags unblocks
  the 21 loose phase dirs.
  > **[SUPERSEDED 2026-07-17 — P118/DOCS-09: tags cut (v0.13.0/v0.13.1/v0.14.0) + released; cascade ran. See the top-of-doc banner.]**
- Perspective numbers: ~99% of .planning/ is historical reference; the live agent entry-path
  load is ~71.6K ≈ 1% of the tree — the signal exists, it's just surrounded. The untracked
  21-*/22-* dirs collide with historical v0.7.0 phase numbering specifically. The 548M
  untracked research dir is poc/scratch/target compiled artifacts. SURPRISES-INTAKE.md is at
  181 of its 200-line cap. Migration scripts are archivable.
- **Cross-lane escalation (with Lane 2):** docs/research FUSE-era chapters
  (fuse-architecture.md, rest-to-posix.md) are PUBLISHED in the live mkdocs nav
  (mkdocs.yml:156-166) — the retired architecture is presented on the public site with no
  historical banner. This upgrades "Research in nav" from clutter to active truth-rot
  exposure. Meanwhile the Confluence setup guide that WOULD help newcomers
  (connectors/guide.md) sits in not_in_nav as a stub.

## §3 Systemic diagnosis (Lane 6 — quality-gate blind spots, + bloat causality)

**Root cause in one line:** every docs gate at a BLOCKING cadence checks *mechanical
correspondence* (claim↔test hash match, jargon-layer token lists, page presence, build
passes), while every gate that could judge *meaning* (framing truth, architecture-epoch
honesty, category correctness, cross-page journeys, nav ergonomics) is nonexistent, scoped to
the first 50 lines of one page, WAIVED, or weekly-only. All four owner defects are meaning
defects; they fell into that seam. This is structural, not negligence.

**Exhibit A — the quality system KNEW (benchmarks).** The catalogs self-document the gaps:
`perf/token-economy-bench` WAIVED until 2026-09-15; `perf/headline-numbers-cross-check`
WAIVED with its verifier script absent (dangling row, perf-targets.json:114,121);
quality-weekly.yml:40-48 admits the weekly verdict is "permanently yellow". Waived/manual/
dangling states are load-bearing and nothing escalates them — a waiver on a P0-adjacent claim
behaves identically to a pass at every decision point. Permanent yellow = a metric generated
but not watched.

**Exhibit B — the catalog actively DEFENDS the stale sentence.** Sharpest single finding:
doc-alignment row `filesystem-layer/blob-lazy-first-cat` (doc-alignment.json:8772-8789) binds
the claim "The first cat of an issue file triggers one REST call" (filesystem-layer.md:42) to
a test asserting exactly the pre-pivot behavior — BOUND, green. The gate checks
claim-vs-test, never claim-vs-current-architecture; both sides are self-consistently stale,
and drift detection keys only on source_hash — it fires only when someone FIXES the prose.
Same pattern for owner finding 1: row `docs/index/rest-supported-backends` REPEATS the
"issue trackers (Confluence)" category error, which also appears in .planning/PROJECT.md:5.
The framing error is systemic, enshrined in the very artifacts meant to police it.

*Precision note (two lanes disagreed; stating the truth, not the average):* in a partial
clone, lazy-blob materialization happens at GIT operation boundaries — checkout/diff pulling
missing blobs through the helper, audited, blob-limit-capped. Once a file is checked out,
`cat` is pure local disk. So filesystem-layer.md:42's "first cat triggers one REST call"
conflates checkout-time fetch with read-time fetch, and the L7-10 "secretly triggers a
network call" framing is wrong either way — the selling point is exactly that reads are
boring local disk.

**Exhibit C — flagged by humans-in-the-loop once, never institutionalized.** v0.11.1 persona
research already caught owner findings 3 and 4, with dates: persona-harness-author.md:21
("FUSE-era debris… The doc set telling two different stories about the architecture is a
tell"); persona-mcp-user.md:39/:58/:102 (explicit fix: "Move ADRs to /contributing/decisions/
or hide from public site"). One-time artifacts; never converted to a recurring gate; no
GOOD-TO-HAVE filed. Noticing happened — the pipeline from noticing to gate did not exist.

**Exhibit D — coverage arithmetic.** docs-alignment coverage_ratio is 0.179 against a floor
of 0.10 — 82% of prose is invisible to the strongest gate, and the floor is set so low the
gate can't fail. All 3 landing-page subjective rubrics (cold-reader-hero-clarity P1,
install-positioning P0, headline-numbers-sanity P2) are WAIVED until 2026-09-15 — waiver
reason is a runner limitation (subprocess lacks Task-tool dispatch, the "P95 gap"), not a
judgment — and the sharper formulation: they are **frozen gates carrying pre-finding scores**
(cold-reader 8 CLEAR, install 9 CLEAR, ratified before the defects existed or were noticed);
two of the three artifacts contain empty asserts (never actually graded); dvcs-cold-reader's
"score 8" artifact is hand-written over a failed dispatch (exit_code 2, "rubric not found").
Layer-3 (how-it-works/) banned-words list is EMPTY by design ("where the technical reveal
happens") — so the one mechanical lint that IS wired pre-push (structure/banned-words,
.githooks/pre-push:62) exempts exactly the page that was wrong, and it's a token matcher:
"secretly trigger a network call", "filesystem", "mount" are on no list. Nav (mkdocs.yml) is
in NO gate's scope.

**Exhibit E — bloat is partly stalled-tag fallout, and it feeds the misses.** The archival
ritual runs at tag time; two owner-gated tags (v0.13.0, v0.14.0) are uncut, so 21 dead phase
dirs sit in live .planning/phases/, ROADMAP.md is a milestone behind, and transcripts grow
unboundedly (523 files, no retention rule written anywhere). A fresh agent's context fills
with historical reference before it reaches the live surface — the owner's "things slip
through the cracks" diagnosis, mechanized.

> **[SUPERSEDED 2026-07-17 — P118/DOCS-09: tags cut (v0.13.0/v0.13.1/v0.14.0) + released; cascade ran. See the top-of-doc banner.]**

*Remediation candidate (owner-floated idea, 2026-07-12 — a decision, not a directive; see
§5 Q7): ONE LARGE SWEEP instead of gradual archival-by-convention.* Archived planning detail
would be DELETED or aggressively compressed — "it should just sit in the git history."
In-repo precedent: MANAGER-HANDOVER.md ("keep this file lean; git history is the archive")
and OP-9 itself (post-distillation detail is deletable). Deletion is reversible by
construction — cut a tag/ref before the sweep and recovery is trivial, per the project's
reversibility principle. If chosen, non-negotiable caveats:
(1) REFERENCE-INTEGRITY pre-check — some "historical" files are load-bearing
(freshness-invariant gates reference archived REQUIREMENTS.md paths; the docs-alignment
backfill audits archived REQUIREMENTS; CLI help text cites .planning/*.md paths; quality
catalogs cite planning paths) — a blind delete flips gates RED, so the sweep's verifier is
an inbound-reference grep + full gate run. (2) Scope from the bloat map: ~99% of .planning/
is historical (phases/, milestones/, research/, quick/); quality/reports/transcripts (523
files) needs a standing RETENTION RULE, not a one-time purge; the 548M POC target/ is a
plain delete+gitignore. (3) Distill-before-delete only where OP-9 hasn't already run
(RETROSPECTIVE covers closed milestones).

**Five gate shapes that would have caught findings 1-4** (each = one catalog row + one
verifier, per the framework's own extension rule; efforts from Lane 6):

| Gate | Catches | Effort |
|---|---|---|
| (A) structure/pivot-vocabulary-lint — architecture-truth lint over how-it-works/ (read-path network claims, mount/FUSE outside a historical allowlist), pre-push | F3, CLI rot | ~2h |
| (B) subjective/cold-journey-newcomer — persona WHOLE-JOURNEY rubric at a blocking/post-push cadence, unwaived (requires fixing the runner Task-dispatch gap) | F2, F4 | ~3h + runner fix |
| (C) structure/nav-budget — leaf cap, no ADRs in public nav, every shipped backend reachable from a Guides/setup entry (extends the existing freshness-invariants.py nav parser) | F4 | ~2h |
| (D) structure/hero-redundancy — duplicate normalized link targets in index.md hero | F2 | ~1h |
| (E) framing-claim docs-alignment rows (bind taxonomy sentences, e.g. to BackendCapabilities) + raise coverage_floor | F1 | ~2-3h |

## §4 Alternative milestone scopings — ALL TENTATIVE BY DESIGN

**Framing (owner's calibration — the ALTERNATING-milestone convention):** the north star is
fixed; every milestone below is a hypothesis, re-scoped at its boundary. Each arc follows
the owner's alternating scheme: **planned milestones alternate with reserved STUB milestones
for surprises, maintenance, or pivots — each with its own version number** — v0.15 planned
→ v0.16 stub → v0.17 planned → v0.18 stub → … Six planned + six stubs = the owner's "2×6" ≈ twelve
version numbers to get close. Stubs are first-class roadmap citizens scoped only at open
time from what the preceding milestone surfaced: drain SURPRISES-INTAKE + GOOD-TO-HAVES,
absorb pivots and audit fallout, ratchet health (tags, deploy lag, gate regressions). A stub
may close small if little surprised us — but it is never skipped silently (OP-8 promoted to
milestone granularity). That's what keeps the plan organized: deviation has a scheduled
version number waiting for it. Only the first 2-3 planned milestones of each arc are
concrete; later ones are direction markers. In the arcs below, version numbers name the
PLANNED milestones (v0.15, v0.17, v0.19, v0.21, v0.23, v0.25); the even numbers between
them are the stubs and are not re-listed. Documentation depth alone — error behavior, known
caveats, known bugs — is multiple planned milestones in every arc, before implementation
work is counted.

**Shared floor (in every alternative, first milestone, ~days):** kill the four cheap
LAUNCH-BLOCKER classes — fix index.md:13 category error; rewrite/rename filesystem-layer.md;
un-strand `reposix list/refresh` (reuse init.rs:370's error text); delete or implement
`reposix detach`; fix token-fixture provenance lie (relabel honestly even before
re-measuring); fix twitter.md FUSE line. Also: cut the two stalled tags.
> **[SUPERSEDED 2026-07-17 — P118/DOCS-09: the two tags (plus v0.13.1) are cut + released; nothing to cut. See the top-of-doc banner.]**

**Bloat remediation — two candidate shapes, owner decision pending (§5 Q7).** Either way it
slots early (M1.5/M2 in arcs A-C; folded into D's M2 meta-milestone):
- *Option i — ONE LARGE SWEEP (owner-floated idea):* a single delete/compress milestone or
  phase removing archived planning detail outright — "it should just sit in the git
  history." Precedent: MANAGER-HANDOVER.md's own header; OP-9's distill-then-archive already
  implies post-distillation detail is deletable. Mechanics if chosen: (1) pre-sweep tag/ref
  → trivially reversible; (2) REFERENCE-INTEGRITY pre-check as the sweep's verifier —
  inbound-reference grep + full gate run, because archived REQUIREMENTS.md, .planning/*.md
  paths in CLI help, and catalog-cited planning paths are load-bearing (a blind delete flips
  gates RED); (3) scope from Lane 5's map: ~99% of .planning/ (phases/, milestones/,
  research/, quick/) + stale top-level JSONs; transcripts need a standing RETENTION RULE
  either way; the 548M POC target/ is a plain delete+gitignore under both options;
  (4) distill-before-delete only where OP-9 hasn't already run.
- *Option ii — convention-driven gradual archival:* cut the two stalled tags → /gsd-cleanup
  moves the 21 dead phase dirs per the existing convention, plus Exhibit C's
  archival-at-close gate so it never re-accumulates. Less blast radius, keeps grep-able
  local history; leaves ~9M of historical tree in place.
  > **[SUPERSEDED 2026-07-17 — P118/DOCS-09: tags cut (v0.13.0/v0.13.1/v0.14.0) + released; the archival cascade already ran. See the top-of-doc banner.]**

### A — Docs-truth deep-clean first ("truth before traffic")
v0.15 floor + truth-rot purge (Lane 1's findings) + bloat remediation; v0.17 IA rebuild
(nav cut 40→~22, Connect Confluence/JIRA/GitHub pages, Install entry); v0.19 benchmark
honesty (re-fixture against the REAL architecture, live-MCP baseline or honest relabel, CI
token job); v0.21 error-behavior + caveats reference (every CLI error documented,
known-bugs page); v0.23 CLI UX debt (Lane 3 HIGHs/MEDIUMs) + launch kit start; v0.25 launch
kit (hero demo, Show-HN, distribution) → launch; stubs v0.16-v0.26 interleave throughout.
**Tradeoff:** highest truth confidence at launch; launch sits at the end of the arc; gates
aren't fixed first, so rot can regrow while you clean — you may re-clean v0.15's work by
v0.23 (the stubs will absorb some of that re-cleaning, which is exactly the waste D avoids).

### B — Skeptical-dev journey slice ("restructure launch around the newcomer path")
v0.15 floor + bloat remediation; v0.17 journey slice 1: land→install→first-run (hero,
Install nav, quickstart, sim-default CLI errors) shippable end-to-end; v0.19 slice 2:
connect-real-backend (Confluence page + attach UX + credential errors as one vertical);
v0.21 slice 3: agent-integration + token story (honest benchmarks INSIDE the journey);
v0.23 slice 4: recovery journeys (doctor, conflict, blob-limit, troubleshooting truth);
v0.25 launch kit → launch; depth docs (error catalog, caveats) continue post-launch; stubs
interleave. **Tradeoff:** every planned milestone ships a walkable journey (demo-able
progress, natural asciinema fodder); but cross-cutting rot (ADR banners, research pages)
has no natural slice home and lands in the stubs; depth-docs land late.

### C — Audit-first punch-list ("scope from evidence")
v0.15 floor + convert THIS report into a graded punch-list with per-item effort; scope
v0.17+ from evidence at v0.15-close. **Tradeoff:** this report already IS most of that
audit — C's remaining value is CLI-runtime verification (execute every error path for
real; our audit was read-only). Cheapest de-risking, but spends a version number on latency
before visible progress, and the org has already shown audits that don't become gates decay
(Exhibit C).

### D — Ratchet-first (RECOMMENDED): fix the system that lets rot regrow, then drain
v0.15 floor (as above); v0.17 **meta-milestone**: the five gate shapes from §3
(pivot-vocabulary lint, nav-budget, hero-redundancy, framing-claim rows + coverage floor,
persona whole-journey rubric) + fix the subjective-runner Task-dispatch gap (unfreezes
three WAIVED meaning-gates) + waiver-escalation rule (a waived P0/P1 row blocks
milestone-close) + transcript retention + bloat remediation (either §5 Q7 option); v0.19
truth purge + IA rebuild (now behind ratchets — the banned-phrase lint prevents regression
while you work); v0.21 benchmark honesty (re-fixture, live baseline, CI job — the
headline-cross-check verifier now exists); v0.23 journey slices as in B (connect-backend,
agent-integration); v0.25 launch kit + Show-HN → launch; post-launch: real-backend
hardening, distribution (skill/MCP directories/llms.txt); stubs v0.16-v0.26 interleave —
and in this arc the stubs stay small because the ratchets catch regressions at push time
instead of letting them pile up. **Why D:** the audit's central lesson (Exhibits A-E) is
that this org's failure mode is not finding defects — personas found them in v0.11.1 —
it's that findings don't become standing gates, and frozen/waived gates rot silently.
Cleaning before ratcheting means cleaning twice. D spends one planned milestone on meta to
make every later milestone's output durable; each gate is ~2-3h by Lane 6's estimates.
**Tradeoff:** launch lands at v0.25 like A, but with regression insurance; requires
resisting the temptation to "just fix the docs first."

Rough shape: 6 planned + 6 stub milestones (v0.15-v0.26) ≈ multiple weeks at current
cadence in every arc — the owner's 2×6, structurally. The alternatives differ in ORDERING
PHILOSOPHY (truth-first / journey-first / evidence-first / ratchet-first), not total scope;
pick by appetite for launch latency vs regression risk.

## §5 Open questions for the owner

1. **North-star numbers.** The "~150k → ~2k tokens (98.7%)" figure is FUSE-era, cites
   uncited external studies, contradicts your own measurement (89.1%) by an order of
   magnitude, and still anchors .planning/PROJECT.md's Context. Retract, or relabel as
   aspiration? And is the v1.0 headline the honest-but-synthetic 89.1%, or do you fund a
   live-MCP-server re-measurement (new fixture against the REAL architecture) first?
   > **[SUPERSEDED 2026-07-17 — P118/DOCS-07+DOCS-09: RESOLVED — P115 funded the live re-measurement (`docs/benchmarks/token-economy.md`); the synthetic 89.1% is retired and PROJECT.md now cites the live figure. See the top-of-doc banner.]**
2. **Tag cuts.** v0.13.0/v0.14.0 tags are owner-gated and stalled; they block the archival
   cascade that produces most of the .planning/ bloat. Cut both now?
   > **[SUPERSEDED 2026-07-17 — P118/DOCS-09: ANSWERED — all three tags (v0.13.0/v0.13.1/v0.14.0) are cut + released (v0.14.0 = "Latest"); the cascade ran. See the top-of-doc banner.]**
3. **Launch definition.** Is Show-HN gated on the real-backend journey (Connect
   Confluence/JIRA/GitHub pages + attach UX polished), or is a sim-first launch acceptable
   with real-backend guides marked beta?
4. **Subjective-runner fix (the P95 gap).** Without Task-dispatch in the runner, every
   meaning-gate stays frozen at pre-finding scores. Fund in M2 (per D), or accept manual
   rubric runs at milestone-close?
5. **Public site scope.** ADRs + FUSE-era research chapters: remove from nav, or keep with
   "historical" banners? (Lane 1 found research/* correctly bannered; the nav placement is
   the exposure.)
6. **`reposix detach`.** Implement it, or reword the two references? Smallest decision
   unlocking two HIGHs.
7. **Bloat remediation shape: one large sweep, or gradual archival-by-convention?** You
   floated the sweep as an idea (delete/compress; git history is the archive) — §4 carries
   both options with mechanics and caveats. Sub-decisions either way: transcript retention
   rule (523 files, unbounded), research/ archive home (incl. 548M untracked poc/ — plain
   delete+gitignore under both), stale catalog JSONs at .planning top level, handover-file
   home, 999.x debt stubs, untracked 21-*/22-* phase dirs (numbering collides with v0.7.0
   history).
8. **Audit droppings to approve deleting.** The browser lane wrote 8 screenshots into the
   repo tree (audit-01/02-*.png at repo root; audit-03..08 under .playwright-mcp/) because
   playwright MCP refused the scratchpad path. Untracked, safe to `rm`; this audit otherwise
   left the tree untouched. (Visual verification itself is now COMPLETE — no 404s, clean
   390px reflow, mermaid renders; one cosmetic bug: token-economy table's literal `** 531**`.)
9. **Scale check on your 2×6 estimate.** The docs surface is finite (~40 pages; ~60 defects
   inventoried here) — the truth-clean itself looks like 2-3 milestones, not 6. What pushes
   the arc to ~12 is error-behavior/caveats depth docs + CLI UX + benchmark re-measurement +
   gates + launch kit. Does that decomposition match your intuition, or do you see docs
   depth we haven't inventoried (e.g. per-backend caveat matrices, failure-mode catalogs)?

---

## ADDENDUM — Owner decisions (2026-07-14, live session, manager rotation #6)

Recorded by the manager; supersedes the "pending" status of the §5 questions below.

- **Q3 — Launch definition: DECIDED.** Show-HN launch is gated on a walkable
  REAL-BACKEND journey (GitHub minimum), not sim-first.
- **Q4 — Subjective-runner fix: FUNDED** (Task-dispatch in the meta-milestone).
- **Q5 — Public site scope: DECIDED, stronger than either §5 option.** DELETE
  legacy/historical files outright — no keep-with-banners. Owner verbatim: "I hate
  how much legacy and bloat is there across md files in docs, .planning/, etc — lets
  not make it worse — very confusing! Please please ensure we are explicitly planning
  simplification of the project docs."
- **Q7 — Bloat remediation: DECIDED, same ruling as Q5.** Aggressive sweep;
  delete/compress, git history is the archive. Docs/planning SIMPLIFICATION is a
  first-class, explicitly-planned roadmap goal.
- **Q8 — Audit droppings: CONFIRMED delete** (incl. root audit-01/02;
  .playwright-mcp/audit-03..08 sweep already queued as post-tag item 3).
- **Q9 — Scale: DECIDED, keep** the v0.15→v0.25 ~6-milestone scale ("gives a lot of
  room to squeeze in enhancements, cleanups and pivots").
- **NEW CALIBRATION MANDATE (owner):** assume ONE deep survey pass surfaces only
  ~10% of the latent work; expect ~10 deep survey passes over the roadmap to reach
  convergence. Recurring deep surveys are a standing practice baked into milestone
  planning — not a one-time audit.
- **Arc (§4): DECIDED — Arc D (ratchet-first), ratified by the manager under
  explicit owner delegation ("Your call", 2026-07-14 live session).** Recording: Arc D
  with the simplification mandate as a strengthened bloat-purge component, per-milestone
  deep surveys (per the ~10-pass convergence calibration above), and the Q3 real-backend
  launch gate at v0.25 — each survey's findings become standing gates so pass N+1 never
  re-finds pass N's issues. This closes the last pending item of this audit; the
  /gsd-new-milestone re-anchor folds this addendum into PROJECT.md.
