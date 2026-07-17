# Phase 117: Doc-truth launch-blocker purge — Research

**Researched:** 2026-07-16
**Domain:** Documentation truth-repair + Rust-CLI error-message UX + static-asset (animation) embedding
**Confidence:** HIGH (every claim below is grounded in a read file:line; no cargo builds run — grep/read only, per one-cargo-machine-wide rule)

## Summary

Phase 117 is a **doc-truth purge**: six concrete false-or-stale claims (SC1–SC5) across the published docs and CLI error strings, plus two owner mandates — a furnished-product cold-reader/IA polish pass (GTH-V15-36) and productionizing the 7-scene launch animation into `docs/index.md` (GTH-V15-37). This is **not** a library-selection phase; the "stack" is the existing mkdocs-material site, the `reposix-cli` crate's error-message convention, and the doc-alignment catalog. The work is surgical edits verified against reality, each of which drifts one or more **doc-alignment catalog rows** that a top-level `/reposix-quality-refresh <doc>` must re-bind (executor-level agents cannot run that command — a hard planning constraint).

Two findings **correct the phase brief's assumptions** and must reach the planner: (1) **SC2's false "`cat` triggers a network call" claim is concentrated in `filesystem-layer.md` alone** — it is NOT propagated into `git-layer.md`, `time-travel.md`, or `trust-model.md` (grep empty); `docs/index.md` states the mechanism correctly ("file content fetched lazily", index.md:160). The false claim IS, however, encoded in a doc-alignment row (`filesystem-layer/blob-lazy-first-cat`) which must be rewritten, not just re-bound. (2) **SC5's fabricated provenance lives in `benchmarks/README.md:34`, NOT `docs/benchmarks/token-economy.md`** — the token-economy doc is already accurate; `benchmarks/README.md` claims `reposix_session.txt` is "the literal output of `scripts/demo.sh`", a script that **does not exist** and never produced it (the file is a live capture).

**Primary recommendation:** Treat P117 as five independent doc/CLI fix lanes + one furnished-product lane + one animation lane. For SC4, take **Option B (rewrite the attach.rs error to reference only commands that exist today)** and FILE the real `reposix detach` subcommand (Option A) as a GOOD-TO-HAVE — B fixes the blocking defect (a false command reference) surgically without committing new semver-locked CLI surface. Budget a top-level doc-alignment refresh pass for every touched doc.

## User Constraints

No `CONTEXT.md` exists for this phase (`.planning/phases/117-*/` did not exist before this research; no `/gsd-discuss-phase` was run). The constraints below are derived from the dispatch brief and `CLAUDE.md`, not from a locked decisions file. **The planner should treat the animation interactive-embed scope (SC/GTH-V15-37 items 1–3) as the main open decision** — see Open Questions.

### Locked Decisions
- None (no CONTEXT.md). Dispatch brief scopes SC1–SC5 + GTH-V15-36 + GTH-V15-37 into P117.

### Claude's Discretion
- SC4 design fork (implement `detach` vs rewrite error) — this research recommends B.
- Animation embed depth (mp4-only fallback vs full interactive JSX productionization).

### Deferred Ideas (OUT OF SCOPE)
- None declared. Candidate deferrals surfaced by this research (SC4 Option A `reposix detach`; self-hosting `cdn.simpleicons.org`) are recommended as GOOD-TO-HAVES, not P117 scope.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SC1 | `docs/index.md` mislabels Confluence as an issue tracker; shows `git clone` as the bootstrap verb | Confirmed at **index.md:13** (single line carries both defects). README.md:5 already softened. §SC1 |
| SC2 | False claim that `cat` secretly triggers a network call | Confirmed concentrated in **filesystem-layer.md:7–13, 19, 30–40, 42, 63**; NOT propagated to siblings (grep empty). Doc-alignment row `filesystem-layer/blob-lazy-first-cat` (L42) encodes it. §SC2 |
| SC3 | `list`/`refresh` connection-refused errors don't teach-the-fix like the exemplar | Confirmed deficient at **list.rs:77–80** and **refresh.rs:206–209**; exemplar shape at **init.rs:365–373** + **init.rs:130–153**. §SC3 |
| SC4 | `attach.rs` error references nonexistent `reposix detach` | Confirmed at **attach.rs:142–145**; no `Detach` clap variant (main.rs enum) and no impl anywhere. §SC4 Decision |
| SC5 | Fabricated `reposix_session.txt` provenance; stale FUSE architecture in social copy | Provenance fabrication at **benchmarks/README.md:34** (`scripts/demo.sh` absent). FUSE at **twitter.md:16**; retired 89.1% at **twitter.md:18** + **linkedin.md:21**. §SC5 |
| GTH-V15-36 | Furnished-product cold-reader/IA polish across index.md, README.md, landing surfaces | Cold-reader rubric `subjective/cold-reader-hero-clarity` (WAIVED to 2026-09-15). IA weaknesses catalogued. §Furnished-product polish |
| GTH-V15-37 | Productionize + embed the 7-scene launch animation into index.md (5-item checklist) | mp4 confirmed (7.1 MB); DC-runtime architecture reverse-engineered; blockers named. §Animation embed feasibility |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Doc prose truth (SC1, SC2, SC5b) | Docs site (`docs/**`, mkdocs-material) | doc-alignment catalog | Claims are rendered to the published site; catalog binds each claim to code/test evidence |
| Repo-internal provenance truth (SC5a) | Repo docs (`benchmarks/README.md`) | perf bench generator | `benchmarks/` is not in the mkdocs nav; it is contributor/provenance surface, not site content |
| CLI error UX (SC3, SC4) | `reposix-cli` crate (Rust binary boundary) | docs/reference/cli.md | Errors are emitted by the binary at runtime; the three-part bar is a code convention (`crates/CLAUDE.md`) |
| Cold-reader clarity (GTH-V15-36) | Docs site + README (landing) | subjective rubric verifier | Hero clarity graded on README.md:1–50 + index.md:1–50 by a dispatched subagent |
| Animation embed (GTH-V15-37) | Static assets (`docs/assets/animation/`) served by mkdocs | GitHub release assets (mp4) | mkdocs copies `docs/assets/**` verbatim; the 7.1 MB mp4 is a release asset, never committed |

## SC1 — Confluence mislabel + `git clone` bootstrap verb

**What exists now (index.md:13, single line):**
> "reposix exposes REST-based issue trackers (Jira, GitHub Issues, **Confluence**) as a real git working tree. An autonomous LLM agent can **`git clone`**, `cat`, `grep`, edit, and `git push` tickets ..."

Two defects on one line: (a) Confluence is a **wiki**, not an issue tracker — grouping it under "issue trackers" is false; (b) `git clone` is shown as the agent's bootstrap verb, but the actual bootstrap is `reposix init` (index.md's own later examples use `reposix init sim::demo` at :79, :93). `git clone` at **index.md:75** is legitimate (build-from-source of the repo, not a record-tree bootstrap) — do NOT touch it.

**Cross-surface check:** `README.md:5` already reads "REST-based issue trackers (**and similar SaaS systems**)" and lists verbs `cat, grep, sed, and git` (no `git clone` bootstrap) — README is already clean; **index.md is the sole SC1 offender**. `docs/why.md`, `docs/social/*`, and `_build_demo_gif.py` say "issue tracker" generically but are either out-of-nav or accurate framing (a generic category label, not a per-backend miscategorization).

**What's needed:** Rewrite index.md:13 to (a) describe backends honestly — e.g. "REST-backed systems of record — issue trackers (Jira, GitHub Issues) and wikis (Confluence)" (matches CLAUDE.md's own elevator pitch phrasing), and (b) name `reposix init` (or "one `reposix init`, then pure git") as the bootstrap verb instead of `git clone`.

**Approach:** Single-line prose edit. Keep the "80/20 alongside REST" framing intact.
**Risks:** Drifts two doc-alignment rows bound to L13 (see drift surface). Low blast radius.
**Effort:** ~15 min prose + refresh.

## SC2 — false "`cat` triggers a network call"

**What exists now — the false mechanism claim (filesystem-layer.md):**
- **:7–13** Plain-English summary: "when you `cat` an issue file ... the *first* read might **secretly trigger a network call** to the issue tracker behind the scenes."
- **:19** Section heading: "## How a `cat` becomes a REST call (or doesn't)".
- **:30–40** Mermaid: node `A["agent: cat issues/PROJ-42.md"]` → edge `O -.->|"miss → lazy fetch"| H` → helper → REST. The diagram attributes the blob fetch to the `cat`.
- **:42** "**The first `cat` of a given issue triggers one REST call.**"
- **:63** Failure mode: "Network down on first read ... the `cat` fails."

**Why it is false (architecture):** In a git partial clone (`--filter=blob:none`), the working tree's blob contents are materialized by **`git checkout`** (the index.md flow is `reposix init` then `git checkout -B main refs/reposix/origin/main`, index.md:80/94). Checkout is the moment git asks the promisor helper (`git-remote-reposix`) to lazy-fetch the missing blobs. After checkout, the file is a real, materialized file on disk; a subsequent **POSIX `cat` invokes no git machinery and triggers no network** — it reads bytes already on disk. `cat` on a not-yet-materialized path returns nothing/ENOENT; it does not itself drive a fetch. So the network trigger is **`git checkout` / `git fetch`**, not `cat`. filesystem-layer.md:42's own next sentence is correct ("The tree ... is fetched once at `init` ... only blob *contents* are lazy") — the error is attributing the content fetch to the `cat` verb rather than to checkout.

**Propagation is NARROW (brief correction):** The dispatch brief anticipated propagation into `docs/index.md`, `git-layer.md`, `time-travel.md`, `trust-model.md`. Verified grep (`cat` within 60 chars of network/REST/fetch/materialize across those files) returns **empty** — the false claim does not appear in them. `git-layer.md:8` says "your edits become REST writes ... **behind the scenes**" — that is the **write/push** path and is accurate. `index.md:160` says "file content fetched lazily" — accurate. **The fix surface for SC2 is `filesystem-layer.md` alone** (five locations above) **plus one doc-alignment row.**

**Doc-alignment row encodes the falsehood:** row `filesystem-layer/blob-lazy-first-cat` (bound to filesystem-layer.md:42) claims verbatim *"The first cat of an issue file triggers one REST call; subsequent cats are local."* Fixing the doc requires **rewriting this row's claim text**, not merely re-binding a hash. Correct claim: e.g. "`git checkout`/`git fetch` materializes missing blobs via the promisor helper; a subsequent `cat` is a pure local read."

**What's needed:** Reframe the section/summary/diagram/failure-mode around **checkout (or explicit `git fetch`) as the network moment**, `cat` as always-local. Update the mermaid entry node from `cat issues/…` to `git checkout` (or `git fetch --filter=blob:none`). Rewrite the catalog row claim.
**Risks:** The mermaid edit must survive `mermaid-renders.sh` (no HTML entities inside the block — see §Common Pitfalls). Getting the corrected mechanism subtly wrong re-introduces a doc-truth bug. HIGH-value correctness lane.
**Effort:** ~45–60 min (prose + diagram + row rewrite + refresh).

## SC3 — `list`/`refresh` connection-refused errors don't teach

**Exemplar (the bar to match), init.rs:365–373** — the `reposix init` fetch-failure `bail!`:
> "reposix init: could not sync `{path}` ... Fix: confirm the backend is running and reachable — **for the simulator, start it in another terminal with `reposix sim`** — then re-run `reposix init`, or sync in place with `git -C {path} fetch --filter=blob:none origin`."

And the canonical three-part-bar pattern `refuse_existing_repo_root` (init.rs:130–153): refuses fail-closed, names the cause, suggests the alternative (`reposix attach`), prints a copy-paste recovery line. `crates/CLAUDE.md` § Error-message convention names this the pattern to copy.

**What exists now — deficient (Sim arm):**
- **list.rs:77–80:** `SimBackend::new(origin)...; b.list_records(&project).await.with_context(|| format!("sim list_records project={project}"))?` — when the sim is down, the surfaced error is a raw reqwest connection-refused wrapped only with `"sim list_records project=demo"`. No fix taught, no `reposix sim` suggestion, no copy-paste recovery.
- **refresh.rs:206–209:** identical pattern — `.with_context(|| format!("sim list_records project={}", cfg.project))`.

The GitHub/Confluence/JIRA arms in both files add an allowlist hint (`REPOSIX_ALLOWED_ORIGINS must include …`) which is better but still not the full three-part bar, and does not name the "start the sim" recovery for the default backend.

**What's needed:** Wrap the Sim-arm (and ideally all-arm) connection-refused failure in a teaching error mirroring init.rs:365–373: (1) teach — "the backend at `{origin}` refused the connection"; (2) alternative — "the default backend is the simulator; start it with `reposix sim`"; (3) copy-paste — `reposix sim &` then re-run. Detect connection-refused specifically (vs. other errors) so the message is accurate. Consider a shared helper so `list`, `refresh`, and future subcommands share one teaching wrapper (Don't-Hand-Roll: don't duplicate the string four ways).
**Risks:** Over-broad matching (labeling an auth error as "start the sim") would mislead — key on the error kind, not all failures. Sim arm is the highest-value target (default backend).
**Effort:** ~1–1.5 h (both files + a shared helper + unit tests asserting the message names `reposix sim`).

## SC4 Decision — attach.rs references nonexistent `reposix detach`

**Exact defect (attach.rs:142–145), the multi-SoT-conflict `bail!`:**
> `"working tree already attached to {existing_sot}; multi-SoT not supported in v0.13.0 (Q1.2). Run \`reposix detach\` first or pick the existing SoT."`

**Verification that `detach` does not exist:** the clap `Command` enum (main.rs) enumerates `Init, Attach, List, Refresh, Spaces, Sync, Doctor, History` (+ log/at/gc/tokens/cost/version per docs/reference/cli.md) — **no `Detach` variant**. Repo-wide grep for `detach`/`Detach` finds only this error string and an unrelated "detached HEAD" comment (init.rs:316). The decision-009 stability catalog row lists the locked surface as `init|attach|sim|list|refresh|spaces|sync|log|history|tokens|cost|gc|doctor|version` — `detach` is absent there too. So the error tells a stuck agent to run a command that will fail with "unrecognized subcommand" — a teaching-free dead end, violating the north-star error bar.

### What `detach` would DO (if implemented)
Attach (attach.rs) binds a working tree to a SoT by setting: `remote.<name>.url` (the reposix/bus URL), `extensions.partialClone = <new remote>`, and `remote.pushDefault = <new remote>`, leaving `origin` (the vanilla mirror) untouched. The inverse `detach` would: unset `extensions.partialClone`, remove the reposix remote (`git remote remove <name>`), and clear `remote.pushDefault` if it points at that remote — restoring a plain-git mirror checkout. Working tree + `origin` stay intact. It is a genuine, well-defined inverse operation.

### Option A — implement `reposix detach` (real subcommand)
- **Scope:** new clap variant in main.rs; new `detach.rs` module (unbind logic, idempotent, teaching errors); docs/reference/cli.md entry; help text; unit + integration tests; a new doc-alignment row for the subcommand.
- **Blast radius:** drifts the **locked** `docs/decisions/009-stability-commitment/cli-subcommand-surface` row (main.rs:39–319) AND `docs/reference/cli.md/subcommands_exist` (both enumerate the subcommand list, which changes). Additive-under-semver is *permitted* by decision 009 but is a **permanent** surface commitment.
- **Effort:** ~3–5 h. **Upside:** makes the attach error literally true; symmetric attach/detach UX reads as a furnished product ("impressive to a skeptical dev").

### Option B — rewrite the attach.rs error (RECOMMENDED)
- **Scope:** replace attach.rs:142–145 with a three-part-bar error that references only commands that exist **today**. Example shape:
  > "working tree already attached to `{existing_sot}`; multi-SoT is not supported (Q1.2). To rebind to a different SoT, first remove the existing reposix remote: `git -C {path} remote remove {remote_name} && git -C {path} config --unset extensions.partialClone`, then re-run `reposix attach {new_sot}`. Or keep the current SoT and drop the new spec."
- **Blast radius:** attach.rs only (~10 lines) + one unit test asserting the message names real recovery commands. **Does not** drift the locked CLI-surface row; no cli.md change.
- **Effort:** ~30–45 min.

### Recommendation
**Option B for P117; FILE Option A as a GOOD-TO-HAVE.** SC4 is a *doc-truth launch-blocker* — the blocking defect is the reference to a nonexistent command, which B fixes correctly and surgically while satisfying the error bar with commands that work now. Option A is a real feature carrying a permanent semver-locked surface commitment; shipping it inside a "purge launch-blockers" phase over-commits scope and drifts a locked catalog row. If the owner wants symmetric attach/detach as a furnished-product signal, A is the more impressive answer — so both are presented honestly; the fork is genuinely the owner's to gate. **One-liner:** *Rewrite the attach error to teach a real `git remote remove` + re-attach recovery (Option B); file the real `reposix detach` subcommand as a future GOOD-TO-HAVE.*

## SC5 — fabricated provenance + stale FUSE social copy

### SC5a — fabricated `reposix_session.txt` provenance (brief correction: it's in `benchmarks/README.md`, not `docs/benchmarks/token-economy.md`)
**The fabrication (benchmarks/README.md:34):**
> "**The reposix session is the literal output of `scripts/demo.sh`** — the bytes the agent's shell would actually place in context. ANSI escapes are stripped."

**Verified false:** (1) `scripts/demo.sh` **does not exist** (`ls scripts/*demo*` → no matches). (2) `reposix_session.txt`'s own header reads "Captured live from a headless `claude -p` reposix-arm session (ZERO MCP tools loaded)" against `github::reubenjohn/reposix` — a live capture, not script output. (3) `benchmarks/fixtures/README.md:16` **directly contradicts** benchmarks/README.md: "Real git-native reposix session ... captured P115 T4. **No `/mnt/` paths, no `scripts/demo.sh`.**" (4) A test even guards it: `quality/gates/perf/test_bench_token_economy.py:238` asserts `"scripts/demo.sh" not in content`. **`docs/benchmarks/token-economy.md` is already accurate** (line 78–79 correctly calls it "an ANSI-stripped transcript of the reposix arm's git-native shell session against the live GitHub backend").

**Secondary stale reference:** `quality/gates/perf/bench_token_economy_io.py:353` embeds "workflow through `scripts/demo.sh`" in generator text — repair alongside (it feeds generated docs).

**What's needed:** Rewrite benchmarks/README.md:34 to state the true provenance (live P115-T4 headless capture against `reubenjohn/reposix`), matching benchmarks/fixtures/README.md. Fix the io.py:353 reference. **No doc-alignment row is bound to benchmarks/README.md** (see drift surface) — which is *why this drifted undetected*; consider filing a new row pinning `reposix_session.txt` provenance.

### SC5b — stale FUSE architecture + retired 89.1% in social copy
- **twitter.md:16:** "Result: **reposix** — a **FUSE filesystem** + git-remote-helper for issue trackers." FUSE was **deleted** in the v0.9.0 pivot (`crates/reposix-fuse/` removed; filesystem-layer.md:46 documents this). Stale.
- **twitter.md:18:** "**89.1% fewer tokens** for the same task" — the **retired** synthetic figure; token-economy.md now publishes the live **~94.3%** output-token / **~74.9%** cost medians and explicitly retires 89.1%/85.5%.
- **linkedin.md:21:** "git-native partial clone + git-remote-helper" (architecture **correct**) but "📉 **89.1% fewer tokens**" (retired number). Fix the number only.

**What's needed:** twitter.md — replace "FUSE filesystem" with "git-native partial clone + git-remote-helper" and update 89.1% → the live 94.3% framing (or a hedged "~94% fewer output tokens, measured live"). linkedin.md — update the number to the live figure.
**Note (blast-radius):** `social/*` is `not_in_nav` (mkdocs.yml:47–49) — these pages are **not published on the docs site**; they are repo-resident launch copy. Truth still matters (owner will post them), but this is lower site-facing blast radius than SC1/SC2. Two doc-alignment rows are bound (twitter/linkedin), see drift surface.
**Effort:** ~20 min total.

## Furnished-product polish (GTH-V15-36)

**Cold-reader gate that must pass before phase close:** rubric `subjective/cold-reader-hero-clarity` (`quality/catalogs/subjective-rubrics.json`). It dispatches the **`/doc-clarity-review`** subagent (skill at `$HOME/.claude/skills/doc-clarity-review`) against **README.md:1–50** and **docs/index.md:1–50**, requiring a numeric score **≥ 7/10 (CLEAR)**, artifact at `quality/reports/verifications/subjective/cold-reader-hero-clarity.json`. **Status: WAIVED until 2026-09-15** — but the waiver is a *runner-dispatch* limitation (the runner subprocess lacks Task-tool access and would overwrite the ratified artifact with a stub), **not** a statement that the copy is exempt from review. A **top-level** `/doc-clarity-review` dispatch still works and should gate phase close; the ratified artifact (currently score 8 CLEAR) must not be clobbered by an executor-level re-sweep.

**mkdocs-material affordances already enabled** (mkdocs.yml:73–101) — use these, don't hand-roll HTML: `admonition` + `pymdownx.details` (collapsible), `pymdownx.tabbed` (the install tabs at index.md:46–65 already use `===`), `attr_list` + `md_in_html` (grid cards `<div class="grid cards" markdown>`), `pymdownx.superfences` mermaid, `content.code.copy`. Progressive-disclosure primitives are all present.

**IA / cold-reader weaknesses observed (index.md):**
1. **Hero line conflates two claims and carries the SC1 falsehood** (index.md:13) — the first substantive sentence both mislabels Confluence and shows the wrong bootstrap verb. The single most-read line on the site is currently false. (Fixing SC1 *is* the top furnished-product win.)
2. **Two competing "what it is" blocks** — the grid cards (index.md:15–21) and "What it looks like underneath" (index.md:160–164) both explain the three-piece architecture at different depths; the 30-second path is not cleanly first.
3. **Build-from-source is `<details>`-collapsed (good, index.md:69–85) but the "After — one commit" block (index.md:87–102) repeats the same `reposix sim`/`init`/`checkout` bootstrap** shown in the collapsed block — duplicated bootstrap noise above the fold.
4. **Honest-scope footer (index.md:166–168) is a dense single paragraph** mixing pivot history, alpha caveat, and the token headline — a candidate for an admonition (`!!! note "Honest scope"`).
5. **Numbers are repeated inconsistently** — "~94%" (index.md:17,168), "6 ms/278 ms" (multiple), "~1.2k / ~21k output tokens" (index.md:31,35) — a cold reader re-derives the same figure several times. Consolidate to one hero card + one methodology link.

**What's needed:** After the SC fixes, run a cold-reader IA pass on index.md (30-second path first: one-line honest pitch → one install tab-block → one "edit + push" example → links out; collapse advanced/build-from-source/architecture deep-dive), then dispatch `/doc-clarity-review` top-level and confirm ≥7 on both hero surfaces. README hero (README.md:1–8) is already close — light touch.
**Risks:** IA churn drifts many index.md doc-alignment rows (29 bound) — budget the refresh. Keep headline numbers byte-consistent with the catalog claims.

## Animation embed feasibility (GTH-V15-37)

**Source dir confirmed** — `/home/reuben/workspace/reposix-animation-pitch/` (verified `ls`):

| File | Size | Role |
|------|------|------|
| `Reposix Launch Animation.mp4` | **7,145,308 B (~6.8 MiB)** | Rendered video — the item-4 fallback. **Never commit** (repo hygiene + size). |
| `Reposix Launch Animation.dc.html` | 1,407 B | Entry HTML — a bespoke **"DC" artifact runtime** (`<x-dc>` / `<x-import>` custom elements) loaded by support.js |
| `support.js` | 64,222 B | **The DC runtime**, "GENERATED from `dc-runtime/src/*.ts` — do not edit. Rebuild with `cd dc-runtime && bun run build`." |
| `reposix-scenes.jsx` | 42,875 B | Scene definitions (JSX; uses `React`, `window.Easing`, `interpolate`, `clamp` globals) |
| `animations-v2.jsx` | 60,925 B | SceneStage engine (autoplay, localStorage, OM_SCENES/OM_PLAYBACK) |
| `tweaks-panel.jsx` | 24,809 B | The in-page **motion editor** panel |
| `.thumbnail` | 3,158 B | Poster candidate |
| `uploads/*.png` | — | Scene image assets |

**Source format:** NOT a single self-contained html. It is a DC-runtime bundle: `support.js` (the runtime) + three `.jsx` sources loaded at runtime via `<x-import component-from-global-scope="ReposixVideo" from="./animations-v2.jsx ./tweaks-panel.jsx ./reposix-scenes.jsx">` (dc.html:29). **support.js lazy-loads `https://unpkg.com/react`, `https://unpkg.com/react-dom`, and Babel-standalone from unpkg, then transpiles the JSX in-browser** (`compile` + `new Function` in support.js) — this is the ~2.8 s runtime-compile the checklist targets.

**5-item checklist feasibility:**

| # | Item | Feasible? | Finding / blocker |
|---|------|-----------|-------------------|
| 1 | Pre-compile JSX → plain JS (drop unpkg + runtime Babel) | Yes, with a build step | Removes **three** CDN deps (react, react-dom, babel-standalone) + the in-browser transpile. **Blocker:** the DC compile toolchain (`dc-runtime/src/*.ts`, built with `bun`) is **NOT in the pitch dir** — only the generated `support.js` is. Two paths: (a) obtain the dc-runtime source, or (b) **replace the DC runtime with a standard esbuild+React bundle** (esbuild JSX loader, vendor React/ReactDOM locally). Path (b) is the pragmatic offline route and the main effort/risk of this lane. |
| 2 | Self-host Google Fonts (Space Grotesk, JetBrains Mono) | Yes | dc.html:12–14 loads them from `fonts.googleapis.com`. Download WOFF2 + serve from `docs/assets/animation/fonts/` with `@font-face`. Straightforward. |
| 3 | Embed-mode config (no editor, no localStorage, click-to-play) | Yes | All toggles exist: `window.TWEAK_DEFAULTS.motionEditor` (dc.html:22–26; tweaks-panel.jsx is the editor), `window.OM_PLAYBACK` (animations-v2.jsx:34,529 — currently `{"mode":"times","count":1}` = autoplay once → change to click-to-play), and `localStorage` usage at animations-v2.jsx:545,569 (gate behind an embed flag). No code archaeology needed — flip the config. |
| 4 | Video fallback (7.1 MB mp4 as GitHub release asset, never commit) | Yes — **recommended baseline** | mp4 confirmed present. Host via `gh release upload` as a release asset; embed a click-to-play `<video controls preload="none" poster="…thumbnail">` (or a poster image linking to the release asset). Lowest-risk, gate-friendly path. |
| 5 | Docs gates (assets under `docs/assets/animation/`, pass mkdocs-strict + mermaid + playwright) | Yes, with care | `docs/assets/` **does not exist yet** — create it (mkdocs copies `docs/assets/**` verbatim). `mkdocs build --strict` (mkdocs-strict.sh) fails on broken nav/links, not on raw asset files; embed via `<iframe src="../assets/animation/index.html">` to isolate the React runtime from Material's JS. `minify_html: false` already (mkdocs.yml:67) so embedded HTML won't be mangled. Playwright coverage: ensure **click-to-play** (item 3) so capture screenshots don't race a mid-animation frame. |

**NOTICED extra external dep (not in the checklist):** `reposix-scenes.jsx:533,545` fetch brand icons from **`https://cdn.simpleicons.org/`** at runtime. A hermetic offline embed must self-host these too, or the animation makes live CDN calls on every docs page-view (also a minor tainted-egress/consistency smell for a project whose thesis is hermetic reproducibility). Add to the item-1/item-2 self-hosting work or file as a GOOD-TO-HAVE.

**Recommendation:** Ship **item 4 (mp4, click-to-play, release-asset) as the P117 baseline** — it is robust, gate-passing, and satisfies "the animation is embedded." Treat the **interactive JSX productionization (items 1–3 + simpleicons)** as a **stretch/optional lane** gated on the item-1 blocker (dc-runtime source absent → needs an esbuild rebuild). The planner should decide embed depth explicitly (see Open Questions).

## Doc-alignment drift surface

Every doc line bound by a `quality/catalogs/doc-alignment.json` row is **hash-drift-detected**: editing that line drifts the row, and the pre-push `gates/docs-alignment/walk.sh` flags it (BLOCKs if `alignment_ratio` < 0.5 or `coverage_ratio` < 0.1; current 0.837 / 0.173 — headroom exists but drifted rows still need re-binding). **Re-binding is done by the top-level `/reposix-quality-refresh <doc>` (or `/reposix-quality-backfill`) — these slash commands are top-level ONLY and are unreachable inside `gsd-executor`** (`quality/CLAUDE.md`). **Plan a dedicated top-level doc-alignment refresh pass as the final P117 wave.**

**Rows that WILL go stale from P117 edits (must refresh/rewrite):**

| Row id | Bound to | Why it drifts | Action |
|--------|----------|---------------|--------|
| `filesystem-layer/blob-lazy-first-cat` | filesystem-layer.md:42 | SC2 — **claim text itself is the falsehood** | **REWRITE claim** + re-bind |
| `docs/index/rest-supported-backends` | index.md:13 | SC1 — "Jira, GitHub Issues, Confluence as REST-based backends" (Confluence=wiki) | Rewrite claim + re-bind |
| `docs/index/real-git-working-tree` | index.md:13 | SC1 — `git clone` verb removed from L13 | Re-bind (hash drift) |
| `docs/social/twitter/token-reduction-92pct` | twitter.md:18 | SC5b — 89.1% + FUSE line rewritten; claim asserts "number now matches measured" (itself false) | Rewrite claim + re-bind |
| `docs/social/linkedin/token-reduction-92pct` | linkedin.md:21 | SC5b — 89.1% → live figure | Rewrite claim + re-bind |
| `docs/decisions/009-stability-commitment/cli-subcommand-surface` | main.rs:39–319 | **SC4 Option A ONLY** — adding `Detach` changes the locked subcommand list | Rewrite claim + re-bind (avoided by Option B) |
| `docs/reference/cli.md/subcommands_exist` | cli.md:5–29 | **SC4 Option A ONLY** — new subcommand documented | Rewrite + re-bind (avoided by Option B) |
| index.md rows near IA churn (up to ~29 bound) | index.md various | GTH-V15-36 IA rewrite shifts line anchors | Bulk re-bind (`/reposix-quality-refresh docs/index.md`) |

**No coverage (drift undetected — file new rows):**
- **`benchmarks/README.md` has ZERO bound rows** → the SC5a provenance fabrication drifted undetected. `benchmarks/fixtures/README.md` also unbound. Recommend filing a doc-alignment row pinning `reposix_session.txt` provenance to reality (guards recurrence; the meta-rule "fix it twice").

**Pre-existing stale rows in the blast zone (NOT caused by P117, but the planner/refresh should sweep them while here):**
- `docs/index/token-reduction-89-percent` (index.md:17) + `docs/why/token-economy-89-1-percent` (index.md:17) — claim "89.1%" but index.md:17 now says "~94%". Already stale; a co-located `docs/index/hero-token-economy-94-75` row holds the *current* value. The 89.1 rows are orphaned.
- `docs/benchmarks/token-economy.md` rows `reduction-89-percent`, `mcp-mediated-baseline-4883-tokens`, `reposix-baseline-531-tokens`, `confluence-reduction-76-percent`, `github-reduction-85-percent` — all reference the **retired** synthetic methodology; superseded by the live `output-reduction-94-percent` (L37) + `cost-reduction-75-percent` (L40) rows. token-economy.md itself is accurate; the **catalog rows are stale**. (P117 need not touch token-economy.md for SC5, so these are a filed sweep item unless the planner folds them in.)

## Standard Stack (existing — do not introduce new libraries)

| Component | Version/Location | Purpose | Why standard here |
|-----------|-----------------|---------|-------------------|
| mkdocs-material | mkdocs.yml (Material theme) | Docs site | Already the site generator; admonitions/tabs/details cover all polish needs |
| pymdownx (superfences/tabbed/details/emoji) | mkdocs.yml:73–101 | Progressive-disclosure markdown | All GTH-V15-36 primitives already enabled |
| `anyhow` `bail!`/`.context()` | reposix-cli (binary boundary) | CLI errors | `crates/CLAUDE.md` mandates the three-part-bar wrapper; exemplar init.rs:130–153 |
| doc-alignment catalog + `reposix-quality doc-alignment` | quality/catalogs + binary | Claim↔evidence binding | Every touched doc must re-bind via top-level refresh |
| esbuild (NEW, animation lane only) | — | Precompile JSX offline | Only if interactive embed (item 1) is scoped; replaces the absent dc-runtime `bun` toolchain |

## Don't Hand-Roll

| Problem | Don't build | Use instead | Why |
|---------|-------------|-------------|-----|
| Collapsible/tabbed sections in docs | Raw `<details>`/JS | `pymdownx.details` + `pymdownx.tabbed` (already on) | Material styles + a11y + dark-mode for free |
| Connection-refused teaching error, 4× | Copy the string into each backend arm | One shared helper wrapping the failure (SC3) | DRY; one place to keep the `reposix sim` recovery accurate |
| Re-binding drifted doc-alignment rows | Hand-edit hashes in the JSON | Top-level `/reposix-quality-refresh <doc>` | The walker mutates coverage counters as a side effect; the wrapper grades against a /tmp copy |
| JSX runtime transpile in the browser | Ship Babel-standalone from unpkg | Precompile with esbuild, vendor React | Removes 3 CDN deps + ~2.8 s compile; hermetic |
| Video hosting | Commit the 7.1 MB mp4 | GitHub release asset via `gh release upload` | Repo hygiene; CLAUDE.md/task hard rule "never commit the mp4" |

## Common Pitfalls

### Pitfall 1: Editing a mermaid block introduces an HTML entity → site-wide error SVG
**What goes wrong:** SC2's diagram edit — a `<`/`>`/`&` inside a `mermaid` fence renders as a JS-injected "Syntax error in text" SVG that, via `navigation.instant`, leaks to **every** page (two prior VM-crash incidents). **How to avoid:** run `bash quality/gates/docs-build/mermaid-renders.sh` + `mkdocs-strict.sh` (greps rendered HTML for "Syntax error in text", exit 1/2). Keep node labels entity-free.

### Pitfall 2: SC3 error over-matches non-network failures
**What goes wrong:** Labeling an auth/allowlist error "start the sim" misleads. **How to avoid:** key the teaching message on the connection-refused error kind specifically; leave the allowlist hint for the real-backend arms.

### Pitfall 3: Forgetting the top-level refresh → pre-push doc-alignment BLOCK
**What goes wrong:** Executor edits a bound doc line, cannot run `/reposix-quality-refresh` (top-level only), pre-push walk flags drift, phase stalls. **How to avoid:** schedule the refresh as an explicit top-level final wave; the planner must own it.

### Pitfall 4: SC4 Option A silently drifts a *locked* catalog row
**What goes wrong:** Adding `detach` changes the decision-009 locked subcommand list; the stability row + cli.md row go stale and a semver surface is committed under a docs phase. **How to avoid:** prefer Option B; if A, treat the locked-row rewrite + cli.md as first-class tasks.

### Pitfall 5: Committing the mp4 or leaking a `cdn.simpleicons.org`/unpkg call into the shipped site
**What goes wrong:** 7.1 MB in git history; or the "hermetic reproducibility" thesis undercut by live CDN calls on page-view. **How to avoid:** mp4 → release asset; self-host fonts + simpleicons if the interactive embed ships.

## Runtime State Inventory

This is a docs + CLI-string + static-asset phase — **no rename/migration of stored runtime state.** Explicit per-category check:
- **Stored data:** None — no datastore keys/collections/user_ids change. (SC5 `reposix_session.txt` is a committed fixture, not live state; its *description* changes, not the bytes.)
- **Live service config:** None — no n8n/Datadog/Cloudflare config touched.
- **OS-registered state:** None — no Task Scheduler/pm2/systemd registrations.
- **Secrets/env vars:** None renamed. SC3 references `REPOSIX_ALLOWED_ORIGINS`/backend creds by name only (no key change).
- **Build artifacts:** SC4 Option A adds a clap subcommand (recompile needed to observe it); the mp4/animation assets are new static files, not stale artifacts. Otherwise none.

## Validation Architecture

`workflow.nyquist_validation` is `true` (config.json) → section included.

### Test Framework
| Property | Value |
|----------|-------|
| Rust framework | `cargo nextest` (workspace) + `#[cfg(test)] mod tests` in-crate; integration in `crates/reposix-cli/tests/` |
| Python (bench/docs gates) | `pytest` — e.g. `quality/gates/perf/test_bench_token_economy.py` |
| Docs gates | `bash quality/gates/docs-build/mkdocs-strict.sh`, `.../mermaid-renders.sh`, `.../link-resolution.py` |
| Doc-alignment | `bash quality/gates/docs-alignment/walk.sh` (grades against /tmp copy) |
| Quick run (CLI) | `cargo nextest run -p reposix-cli` (per-crate; one-cargo-machine-wide) |
| Full suite | pre-push cadence: `python3 quality/runners/run.py --cadence pre-push` |

### Phase Requirements → Test Map
| Req | Behavior | Test type | Command | Exists? |
|-----|----------|-----------|---------|---------|
| SC1 | index.md:13 no longer calls Confluence an issue tracker / shows `git clone` bootstrap | docs-alignment | `bash quality/gates/docs-alignment/walk.sh` (after refresh) | ✅ (rows exist; claims rewritten) |
| SC2 | filesystem-layer no longer attributes fetch to `cat`; mermaid renders | docs-build + alignment | `mermaid-renders.sh` + `mkdocs-strict.sh`; `walk.sh` | ✅ / ❌ Wave 0: rewrite `blob-lazy-first-cat` row claim |
| SC3 | `list`/`refresh` connection-refused names `reposix sim` recovery | unit | `cargo nextest run -p reposix-cli` asserting message substring | ❌ Wave 0: add unit tests for the Sim-arm teaching error |
| SC4 (B) | attach multi-SoT error references only real commands | unit | `cargo nextest run -p reposix-cli` asserting no `reposix detach` substring + names `git remote remove` | ❌ Wave 0: add attach error test |
| SC5a | benchmarks/README provenance matches reality | pytest/grep | extend `test_bench_token_economy.py` to assert README says live-capture, not demo.sh | ⚠️ partial (line 238 guards token-economy.md, not benchmarks/README.md) |
| SC5b | no "FUSE" in twitter.md; social numbers = live figure | grep/structure | freshness/grep gate | ❌ Wave 0: optional grep guard for banned "FUSE filesystem" in social copy |
| GTH-V15-36 | hero clarity ≥7 CLEAR on README + index heroes | subagent-graded | top-level `/doc-clarity-review`; artifact `cold-reader-hero-clarity.json` | ✅ (rubric exists; dispatch top-level) |
| GTH-V15-37 | animation embedded; site builds; playwright captures | docs-build + manual | `mkdocs-strict.sh` + playwright walk | ⚠️ Wave 0: create `docs/assets/animation/`; verify mp4/iframe under strict |

### Sampling Rate
- **Per task commit:** `cargo nextest run -p reposix-cli` (CLI lanes) OR `mkdocs-strict.sh` (docs lanes) — never both cargo lanes concurrently.
- **Per wave merge:** `python3 quality/runners/run.py --cadence pre-push`.
- **Phase gate:** full pre-push green + top-level doc-alignment refresh clean + `/doc-clarity-review` ≥7 before `/gsd-verify-work`.

### Wave 0 Gaps
- [ ] Unit tests for SC3 Sim-arm teaching error (`list`, `refresh`) — asserts message names `reposix sim`.
- [ ] Unit test for SC4-B attach error — asserts it does NOT contain `reposix detach` and names a real recovery command.
- [ ] Extend `test_bench_token_economy.py` (or a new grep guard) to assert `benchmarks/README.md` provenance = live-capture, not `scripts/demo.sh`.
- [ ] Rewrite doc-alignment row `filesystem-layer/blob-lazy-first-cat` claim (catalog-first, before the doc edit).
- [ ] Create `docs/assets/animation/` + a smoke check that `mkdocs build --strict` copies it and the page renders.
- [ ] (Optional) freshness grep guard banning "FUSE filesystem" in `docs/social/*`.

## Security Domain

`security_enforcement` is not set in config (absent = enabled), but P117 introduces no new untrusted-input path.

| ASVS category | Applies | Control |
|---------------|---------|---------|
| V5 Input Validation | Marginal | SC3/SC4 emit error *strings*; ensure backend `origin`/path values interpolated into messages are not attacker-controlled beyond what already flows (they are user-supplied CLI args, echoed, not executed). |
| V6 Cryptography | No | None. |
| Egress/taint (project-specific) | **Yes — animation lane** | The interactive embed must not add live CDN calls (`unpkg`, `cdn.simpleicons.org`, Google Fonts) to the shipped site — self-host per items 1–2. Consistent with the hermetic-reproducibility thesis and the tainted-by-default posture. mp4 → release asset (no remote byte routed into an outbound side-effect). |

| Threat pattern | STRIDE | Mitigation |
|----------------|--------|------------|
| Misleading error string sends agent down a wrong recovery (e.g. run nonexistent `reposix detach`) | (UX-integrity) | SC4-B: reference only real commands; three-part bar |
| Docs claim diverges from code (doc-truth rot) | Repudiation/Info | doc-alignment binding + top-level refresh; file a row for the uncovered `benchmarks/README.md` |

## Environment Availability

| Dependency | Required by | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `mkdocs` + material | docs-build gates | Assumed (site is live at gh-pages) | — | — |
| `cargo`/`nextest` | SC3/SC4 tests | Yes (repo toolchain) | Rust 1.82+ | — |
| `gh` CLI | mp4 release-asset upload (item 4) | To confirm | — | Manual upload via GitHub UI |
| `esbuild`/`bun` | Interactive JSX precompile (item 1) | **Absent** (dc-runtime source not in pitch dir) | — | **mp4-only embed (item 4)** — recommended baseline |
| playwright + mcp-mermaid | animation/mermaid render verification | Available (global MCP per user CLAUDE.md) | — | — |

**Missing with no fallback:** none blocks the baseline (mp4) path.
**Missing with fallback:** the dc-runtime/esbuild toolchain for the interactive embed — fallback is the mp4 baseline; the interactive lane is a stretch decision.

## Assumptions Log

| # | Claim | Section | Risk if wrong |
|---|-------|---------|---------------|
| A1 | `git checkout` (not `cat`) is the blob-materialization moment in the reposix partial-clone flow | SC2 | If reposix uses a sparse/on-read materialization that DOES fetch on git-mediated read, the reframing is subtly off. Mitigation: verified against index.md's own `init`→`checkout` flow + partial-clone semantics; recommend the executor confirm by observing an audit `materialize` row fires at checkout, not at `cat`, in a `/tmp` sim clone. |
| A2 | No doc-alignment row is bound to `benchmarks/README.md` | Drift surface | If a row exists under a path I didn't match, the "undetected" framing is wrong. Mitigation: exact-file scan of all 400 rows returned none. |
| A3 | Interactive embed item 1 requires re-tooling because `dc-runtime/src` is absent from the pitch dir | Animation | The dc-runtime source may live elsewhere on disk; if found, item 1 is cheaper. Recommend the executor search for it before choosing the esbuild path. |
| A4 | `social/*` pages are not published (mkdocs `not_in_nav`) so SC5b is lower site-blast-radius | SC5b | Confirmed at mkdocs.yml:47–49; still repo-truth-relevant. |
| A5 | The cold-reader rubric waiver is a runner limitation, not a copy exemption — a top-level `/doc-clarity-review` still gates | Furnished-product | Confirmed via the waiver `reason` text; if the runner has since been fixed, dispatch is even easier. |

## NOTICED

Owner-mandate deliverable — items found near the research surface, with severity + fix sketch (research FILES, does not fix):

1. **[HIGH] `benchmarks/README.md:34` fabricates provenance AND has zero doc-alignment coverage.** The "literal output of `scripts/demo.sh`" claim is false (script absent; file is a live capture) and directly contradicts `benchmarks/fixtures/README.md:16`. *Fix:* rewrite to true provenance (SC5a) + **file a doc-alignment row** pinning `reposix_session.txt` provenance so this cannot recur (the "fix it twice" meta-rule — the gap is *why* it drifted).
2. **[HIGH] The single most-read line on the docs site (index.md:13) is currently false** (SC1). A skeptical first-time dev's first sentence mislabels a backend and shows the wrong bootstrap verb. This is the top furnished-product regression, not a footnote.
3. **[MEDIUM] The false SC2 claim is baked into a doc-alignment *row* (`filesystem-layer/blob-lazy-first-cat`), so the "claim has a test" machinery is currently *certifying a falsehood*.** *Fix:* rewrite the row claim (catalog-first) as part of SC2 — otherwise the refresh re-binds the lie.
4. **[MEDIUM] Pre-existing stale token-economy rows** (`docs/index/token-reduction-89-percent`, `docs/why/token-economy-89-1-percent`, and five `token-economy/*` synthetic-era rows) still assert the retired 89.1%/85.5%/4883/531 figures superseded by the live 94.3%/74.9% medians. *Fix:* sweep during the P117 refresh or file a GOOD-TO-HAVE.
5. **[MEDIUM] Animation makes live `cdn.simpleicons.org` calls** (reposix-scenes.jsx:533,545) — an external runtime dependency the 5-item checklist omits, and a hermetic-reproducibility smell for this project. *Fix:* self-host brand icons alongside fonts, or file as GOOD-TO-HAVE.
6. **[LOW] The leaf-isolation PreToolUse hook false-positives on grep patterns** containing the literal setup-verb strings (a `grep -n "reposix at""tach"` was BLOCKED as if it were a real setup command). Documented coverage boundary, but a research/read-only grep is a safe operation. *Fix:* file a GOOD-TO-HAVE to let the guard distinguish `grep`/`rg` argv from an actual setup invocation.
7. **[LOW] Secondary stale `scripts/demo.sh` reference** in generator text at `quality/gates/perf/bench_token_economy_io.py:353` ("workflow through `scripts/demo.sh`") — repair with SC5a so regenerated docs don't reintroduce it.
8. **[LOW] `git-layer.md:8` and multiple docs still say "issue tracker"** as the generic category label. Not false per se, but for a tool that also serves Confluence wikis, "systems of record" (CLAUDE.md's own phrasing) reads more accurately. Optional furnished-product tightening.

## Open Questions

1. **Animation embed depth (owner decision).**
   - Known: mp4 fallback (item 4) is low-risk and gate-friendly; interactive embed (items 1–3) is feasible but blocked on the absent dc-runtime toolchain (needs an esbuild rebuild + self-hosting fonts/icons).
   - Recommendation: ship mp4 baseline in P117; scope the interactive embed as an explicit stretch lane or a follow-up GOOD-TO-HAVE.
2. **SC4 fork (owner gate).** Option B recommended; Option A (`reposix detach`) is a real feature the owner may want for symmetric UX. Recommendation: B now, file A.
3. **Should the pre-existing stale token-economy catalog rows be swept in P117** (planner folds into the refresh wave) or filed separately? They're in the blast zone but not strictly SC1–SC5.

## Sources

### Primary (HIGH — read file:line this session)
- `docs/index.md` (SC1: :13, :75; furnished-product IA) · `docs/how-it-works/filesystem-layer.md` (SC2: :7–13,19,30–40,42,63) · `docs/how-it-works/git-layer.md` :8 · `docs/benchmarks/token-economy.md` (accurate) · `docs/social/twitter.md` :16,18 · `docs/social/linkedin.md` :21
- `crates/reposix-cli/src/init.rs` :130–153, :365–373 (exemplar) · `list.rs` :77–80 · `refresh.rs` :206–209 · `attach.rs` :142–145 · `main.rs` clap enum (no `Detach`)
- `benchmarks/README.md` :34 · `benchmarks/fixtures/README.md` :16 · `benchmarks/fixtures/reposix_session.txt` (header) · `quality/gates/perf/test_bench_token_economy.py` :238 · `bench_token_economy_io.py` :353
- `quality/catalogs/doc-alignment.json` (400 rows; exact-file scan) · `quality/catalogs/subjective-rubrics.json` (cold-reader rubric) · `mkdocs.yml` · `quality/gates/docs-build/mkdocs-strict.sh`
- `/home/reuben/workspace/reposix-animation-pitch/` — `Reposix Launch Animation.dc.html`, `support.js` (unpkg React/ReactDOM/Babel; JSX compile), `reposix-scenes.jsx` (:533,545 simpleicons), `animations-v2.jsx` (localStorage/autoplay/OM_PLAYBACK), mp4 (7,145,308 B)

### Secondary (MEDIUM)
- `crates/CLAUDE.md` § Error-message convention · `quality/CLAUDE.md` § Docs-alignment dimension · project `CLAUDE.md` (elevator pitch phrasing, threat model)

### Tertiary (LOW / to verify)
- Partial-clone checkout-vs-cat materialization semantics (A1) — verify with an audit-row observation in a /tmp sim clone.

## Metadata

**Confidence breakdown:**
- SC1/SC3/SC4/SC5: **HIGH** — exact offending file:line quoted and cross-checked.
- SC2: **HIGH** on location + narrow-propagation correction; **MEDIUM** on the precise corrected mechanism wording (A1 — recommend a checkout-vs-cat audit-row confirmation).
- Doc-alignment drift surface: **HIGH** — exact-file scan of all 400 rows.
- Animation feasibility: **HIGH** on file inventory + CDN/JSX/localStorage/autoplay facts; **MEDIUM** on item-1 effort (dc-runtime toolchain absent → esbuild rebuild path).
- Furnished-product: **MEDIUM** — IA weaknesses are reasoned from the rendered structure; the cold-reader score is the falsifying evidence at phase close.

**Research date:** 2026-07-16
**Valid until:** ~2026-08-15 (stable — internal docs/CLI; re-verify line numbers if other phases edit these files first)
