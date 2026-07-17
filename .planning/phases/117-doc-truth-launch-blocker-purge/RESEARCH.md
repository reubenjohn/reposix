# Phase 117: Doc-truth launch-blocker purge - Research

**Researched:** 2026-07-16
**Domain:** Documentation truth-correction (mkdocs site + CLI error UX) + owner-approved animation embed
**Confidence:** HIGH (every claim below is grounded in a live `grep`/`cat -n`/`cargo run` against this exact checkout, not training knowledge)

## Summary

All 6 doc-truth defects named in the phase brief are confirmed still-present, but **most
cited line numbers in the phase brief, ROADMAP.md, and REQUIREMENTS.md are stale by
several lines to several dozen lines** relative to the current checkout -- this research
re-locates every defect by direct `grep`/`cat -n` and reports the CURRENT line, not the
ticket's line. One defect (SC5 / DOCS-05) is **partially already fixed**: the fixture
content and `docs/benchmarks/token-economy.md`'s provenance claim are both already
accurate (fixed during P115); the actual remaining lie lives in a file
(`benchmarks/README.md`) that no ticket text names. Following REQUIREMENTS.md's literal
DOCS-05 instruction ("relabel as hand-authored, not captured") would **introduce a new
lie** on top of an already-fixed claim -- see SC5 below, this is the single most
important correction in this research.

SC2's propagation set is also broader than the phase brief's "at minimum" list: the four
named files (`index.md`, `git-layer.md`, `time-travel.md`, `trust-model.md`) do **not**
actually repeat the false claim (verified negative -- no edit needed there for SC2), but
three files the brief does NOT name (`docs/reference/glossary.md`,
`docs/reference/cli.md`, `docs/reference/git-remote.md`) **do** repeat it, plus
`docs/reference/confluence.md` has a softer, more defensible version of the same
ambiguity.

**Primary recommendation:** ground every fix against the current-checkout line numbers in
this document (not the ticket's), fix SC5's real location (`benchmarks/README.md`, not
`token-economy.md`), and treat SC4 as a build-vs-reword decision the planner must make
explicitly (recommendation: reword now, defer the real subcommand -- see decision table).

<phase_requirements>
## Phase Requirements

| ID | Description (from REQUIREMENTS.md) | Research Support |
|----|-------------|------------------|
| DOCS-01 | Fix `docs/index.md:13` category+scope error (Confluence-as-issue-tracker + `git clone` bootstrap verb) | § SC1 below -- confirmed at current line 13, not the ticket's ~135-152 |
| DOCS-02 | Rewrite `filesystem-layer.md` off its false "cat triggers network call" framing + propagated cross-links | § SC2 below -- real propagation set differs from the brief's guess; confirmed via code read of `stateless_connect.rs` |
| DOCS-03 | Un-strand `reposix list`/`reposix refresh` connection-refused errors (teach-the-fix bar) | § SC3 below -- exact error text captured via live `cargo run` against a closed port |
| DOCS-04 | Delete-or-implement the phantom `reposix detach` subcommand | § SC4 below -- decision table + recommendation |
| DOCS-05 | Relabel the token-fixture provenance lie (`reposix_session.txt` / `scripts/demo.sh`) | § SC5a below -- ticket's own citation is stale; real defect is in `benchmarks/README.md`, already-fixed elsewhere |
| DOCS-06 | Fix `docs/social/twitter.md:16`'s FUSE framing | § SC5b below |
| GTH-V15-36 | Furnished-product cold-reader/IA polish mandate (P117-shaping input, not a standalone requirement ID) | § Furnished-Product Gaps below |
| GTH-V15-37 | Embed owner's 80s launch animation on `docs/index.md`, 5-item productionization checklist | § Animation Embed Lane below |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Enter through GSD** -- this research feeds `gsd-planner`; no direct edits happened during this research pass (research-only charter honored).
- **One cargo invocation at a time** -- honored; only `cargo run -p reposix-cli -- list/refresh` against a closed port was run (single invocations, sequential), no `--workspace` build.
- **Leaf isolation** -- the one live-error-capture test ran fully inside `/tmp/reposix-sc3-test` (created and `cd`'d into within the same Bash invocation), cleaned up after.
- **Uncommitted = didn't happen / external mutations need owner-named-target approval** -- directly relevant to the Animation Embed Lane's GitHub Release upload (checklist item 4): that is a real mutation against `github.com/reubenjohn/reposix` and MUST be raised to the owner/coordinator before any plan executes it, not treated as a routine implementation step. Flagged in Open Questions.
- **Push cadence / doc-alignment gates** -- every doc this phase edits must be checked against `quality/catalogs/doc-alignment.json` for BOUND rows whose line ranges the edit will touch; see § Doc-Alignment Drift Map. A touched BOUND row requires a top-level `/reposix-quality-refresh` pass before/at phase-close push.
- **Ownership charter (noticing is a deliverable)** -- see § NOTICED below; several defects and one requirement-text error were found outside the 6 named blockers.

## SC1 -- `docs/index.md` Confluence-as-issue-tracker + `git clone` bootstrap verb

**Current-state evidence.** The phase brief's cited range (~line 135-152) is **stale** --
that range is now the "Connector capability matrix" / "Where to go next" sections, which
are already correct. The actual defect is at **`docs/index.md:13`**:

```
13: reposix exposes REST-based issue trackers (Jira, GitHub Issues, Confluence) as a
    **real git working tree**. An autonomous LLM agent can `git clone`, `cat`, `grep`,
    edit, and `git push` tickets without learning a single Model Context Protocol (MCP)
    tool schema or REST SDK surface.
```

Two defects in one sentence: (1) Confluence is grouped with "issue trackers" (Jira,
GitHub Issues) -- it is a wiki, and the SAME page's own connector-capability matrix
(`index.md:130-137`) and footnote (`index.md:137`) correctly treat Confluence
differently (no Comments support, different capability shape); (2) `git clone` is named
as the agent-facing bootstrap verb, but the correct verb -- used consistently everywhere
else on the same page (`index.md:79`, `:93`, `:162`) -- is `reposix init`.

**Also grep-confirmed:** the ONLY other `git clone` occurrence on the page is line 75,
inside the "Build from source (advanced)" `<details>` fold, where it correctly means
"clone the reposix TOOL's own source repo to build the binary" -- that one is NOT a
defect, do not touch it.

**Required end-state:** Confluence described as a wiki/SaaS system distinct from issue
trackers (mirror README.md's already-correct phrasing, see below); bootstrap verb is
`reposix init`, not `git clone`.

**Correct replacement text sketch** (mirrors the site's own more-accurate copy at
`README.md:5-8`: *"reposix exposes REST-based issue trackers (and similar SaaS
systems)..."*):

```
reposix exposes REST-based issue trackers (Jira, GitHub Issues) and wikis
(Confluence) as a **real git working tree**. An autonomous LLM agent can
`reposix init`, `cat`, `grep`, edit, and `git push` tickets without learning a
single Model Context Protocol (MCP) tool schema or REST SDK surface.
```

**Risk/gotchas:** `docs/index.md/real-git-working-tree` and
`docs/index.md/rest-supported-backends` are BOTH doc-alignment BOUND rows anchored
exactly at line 13 (see § Doc-Alignment Drift Map) -- this edit WILL trip both.

## SC2 -- "cat secretly triggers a network call" + propagation

**Ground truth (verified via code, not doc prose).** Read `crates/reposix-remote/src/stateless_connect.rs`
(handles `command=fetch`) and `crates/reposix-cache/src/builder.rs`/`cache.rs`
(`read_blob` = materialize-on-demand). Blob materialization is triggered by **`git
fetch`/`git checkout`** (whichever git operation first needs the missing blob object --
typically the `git checkout -B main refs/reposix/origin/main` step every quickstart in
this repo runs immediately after `reposix init`), never by the bare shell `cat`. `cat` is
a plain POSIX read of an already-materialized file; it has no git awareness and cannot
trigger a fetch. Confirmed no FUSE/VFS layer remains that could make `cat` network-aware
(deleted in v0.9.0 per `filesystem-layer.md:46`).

**Current-state evidence -- the actual lie (`docs/how-it-works/filesystem-layer.md`):**

```
 7: **Plain-English summary.** When you `cat` an issue file in a reposix
 8: working tree, you're reading a real file on disk — but the *first*
 9: read might secretly trigger a network call to the issue tracker
10: behind the scenes. After that, the file is local and reads are free.
```
Also line 63: `- **Network down on first read.** ... the `cat` fails.` -- same framing
error (implies `cat` does the fetching; the fetch already happened at checkout, so a
`cat` after a successful checkout can never fail for network reasons).

**Important: the page's OWN body already gets it right** at line 42: *"The first `cat` of
a given issue triggers one REST call."* -- also imprecise per the ground truth above
(should say "the checkout" not "the `cat`"), but this is the LOWER-risk of the two
inaccuracies since it's at least consistent internally. The summary (`:7-13`) is the
worse, more misleading version and is a doc-alignment BOUND row
(`filesystem-layer/blob-lazy-first-cat`, anchored at line 42 -- editing line 42 WILL trip
it; editing only 7-13 will not, but should still be fixed for internal consistency, per
PATTERNS.md NOTICED #4).

**Real propagation set (verified negative + positive):**

| File | Verdict | Evidence |
|---|---|---|
| `docs/index.md` | **NOT affected** -- verified negative | Only has `cat issues/0001.md` in the sequence diagram / quickstart, no claim about triggering network |
| `docs/how-it-works/git-layer.md` | **NOT affected** -- verified negative | `stateless-connect ... handles all read traffic (git clone, git fetch, lazy blob fetches)` -- correctly ties fetching to `git fetch`, not `cat` |
| `docs/how-it-works/time-travel.md` | **NOT affected** -- verified negative | No lazy-fetch claim of any kind |
| `docs/how-it-works/trust-model.md` | **NOT affected** -- verified negative | `materialize` op described as "Cache lazy-fetched a blob from the backend" -- no `cat` framing |
| `docs/reference/glossary.md:20` | **AFFECTED (not in brief's list)** | "file *contents* arrive lazily on first `cat` or `grep`" -- same defect, explicit `cat` |
| `docs/reference/cli.md:59` | **AFFECTED (not in brief's list)** | "blobs are fetched on demand on first read" -- ambiguous "read", inside `BOUND` row `docs/reference/cli.md/init_documented` (lines 42-65) |
| `docs/reference/git-remote.md:23` | **AFFECTED (not in brief's list)** | "blobs are fetched lazily on first read" -- same ambiguity, inside `BOUND` row `git-remote/stateless-connect-read-path` (line 23 exactly) |
| `docs/reference/confluence.md:135-137` | **Borderline -- lower priority** | "individual page bodies download on first read via ... `stateless-connect`" then immediately clarifies "before the first `git checkout`" -- ties it to checkout, more defensible than the others, but "on first read" phrasing is still loose |

**Required end-state:** every instance reframed as "materializes at `git
fetch`/`checkout` time" (or "at first `git sparse-checkout`/`checkout` of that path"),
never "at `cat` time."

**Correct replacement sketch for filesystem-layer.md summary:**
```
**Plain-English summary.** A reposix working tree is a real git checkout: `git
checkout` (right after `reposix init`) is what pulls blob *contents* down from the
issue tracker, lazily and on demand for exactly the files being checked out. Once
checked out, `cat` is a plain local file read — no network, ever. This page
explains how that lazy-checkout trick works...
```

## SC3 -- `reposix list` / `reposix refresh` connection-refused errors

**Exemplar (read in full):** `crates/reposix-cli/src/init.rs:363-373` --
`refuse_existing_repo_root`-adjacent `bail!` in the fetch-failure path. Three parts
present: (1) teaches the fix ("confirm the backend is running and reachable"), (2)
suggests the sim alternative ("for the simulator, start it in another terminal with
`reposix sim`"), (3) copy-paste recovery (`git -C {path_str} fetch --filter=blob:none
origin`).

**Current state -- captured live** (ran `cargo run -p reposix-cli -- list --project demo
--origin http://127.0.0.1:19999` against a guaranteed-closed port, inside a `/tmp`
isolate):

```
Error: sim list_records project=demo

Caused by:
    0: error sending request for url (http://127.0.0.1:19999/projects/demo/issues)
    1: client error (Connect)
    2: tcp connect error
    3: Connection refused (os error 111)
```

`reposix refresh --project demo --origin http://127.0.0.1:19999 <path>` produces the
IDENTICAL error (both go through `SimBackend::list_records` -> raw `reqwest` error ->
bare `.with_context(...)` wrap, no `bail!`, no teaching).

**Source locations:**
- `crates/reposix-cli/src/list.rs:76-81` -- `ListBackend::Sim` arm: `SimBackend::new(origin)... .list_records(&project).await.with_context(|| format!("sim list_records project={project}"))?`
- `crates/reposix-cli/src/refresh.rs:205-210` (`fetch_issues`, `ListBackend::Sim` arm) -- identical pattern, same missing teaching.

Neither meets the 3-part bar: no "the sim isn't running" diagnosis, no `reposix sim`
suggestion, no copy-paste retry command. (The github/confluence/jira arms in both files
DO inline a "REPOSIX_ALLOWED_ORIGINS must include ..." hint in their `.with_context`, so
they're closer to the bar than the sim arm, but still lack an explicit copy-paste
recovery line -- lower priority than the sim arm, which is the DEFAULT backend and the
one every quickstart uses.)

**Required end-state:** wrap the sim-backend connection failure in a `bail!` (or a
`Context`-decorated error) that: names the likely cause (sim not running / wrong
`--origin`), suggests `reposix sim &` (matching the exact phrasing used everywhere else
in this repo, e.g. `README.md:61-63`), and gives the literal retry command
(`reposix list --project {project} --origin {origin}`).

## SC4 -- phantom `reposix detach` subcommand (DECISION FORK -- do not pick, present both)

**Current-state evidence.** `crates/reposix-cli/src/attach.rs:144` (current line; the
brief/REQUIREMENTS.md cite 135-138/144, both stale by a handful of lines):

```
142:        if existing_sot != new_sot {
143:            bail!(
144:                "working tree already attached to {existing_sot}; multi-SoT not supported in v0.13.0 (Q1.2). \
145:                 Run `reposix detach` first or pick the existing SoT."
146:            );
```

`grep -rn detach crates/ docs/` confirms: `reposix detach` is referenced in exactly TWO
places -- `attach.rs:144` (this error) and `docs/guides/troubleshooting.md:329` (same
claim, PLUS a manual fallback already spelled out: *"To switch SoT, run `reposix detach`
first (or remove the `extensions.partialClone` config + cache directory by hand)."*) --
and is **wired nowhere** in `crates/reposix-cli/src/main.rs`'s `enum Cmd` (12 real
subcommands: Sim, Init, Attach, List, Refresh, Spaces, Sync, Doctor, History, plus
Time-travel/gc/cost/tokens per `lib.rs`'s module list -- no Detach).

**What Option A (implement) would need to do, concretely:** troubleshooting.md's own
manual fallback text IS the spec: unset `extensions.partialClone` config + remove the
reposix remote (`git remote remove <name>`) + optionally delete the cache directory.
`crates/reposix-cli/src/worktree_helpers.rs::cache_path_from_worktree` already exists and
is reused by `gc.rs`/`cost.rs` for exactly this cache-path derivation --
`detach.rs` would be a thin new module reusing that helper plus the
`git_config_get`/`git_config_set` pattern already used by `doctor.rs:342-348`'s
`--fix` path. Wiring cost: one new `Cmd::Detach { path: Option<PathBuf> }` enum arm
(`main.rs:40`+), one dispatch match arm (`main.rs:376`-area), one new ~60-100 line
`detach.rs` module with a couple of unit tests (matching `list.rs`'s test-density
precedent).

### Decision table

| | Option A: implement `reposix detach` | Option B: reword `attach.rs`'s error |
|---|---|---|
| **What it does** | New subcommand: unsets `extensions.partialClone`, removes the reposix remote, optionally deletes the cache dir | Error text stops promising a command that doesn't exist; points at the already-documented manual recovery instead |
| **Effort** | Small-medium: ~1 new file (~80-120 lines incl. tests), 1 enum arm, 1 dispatch arm, reuses `cache_path_from_worktree` + `doctor.rs`'s config-mutation pattern -- no new external deps | Trivial: edit one `bail!` string (attach.rs:144-145), ~10 min |
| **Blast radius** | New, isolated code path (multi-SoT re-attach flow only); does not touch existing attach/init/sync dataflow; needs its own unit tests + a manual/integration smoke test | Zero -- pure string edit, no behavior change |
| **Risk** | Low-medium: a mutating command (unsets config, removes a remote) needs care around idempotency + a git-repo-shaped-argument guard (mirror `attach.rs:118-119`'s `.git/` existence check) | Near-zero: cannot break anything functional, only prose |
| **Reversibility** | High (new command, easy to revert/remove) | Trivial (one string) |
| **Leaves doc-truth clean?** | Yes -- `reposix detach` becomes real, matching troubleshooting.md's existing promise | Yes -- error + troubleshooting.md both point at the SAME already-correct manual recipe; no promise of a nonexistent command remains |
| **Consistent with phase scope** | Adds real CLI surface to a phase titled "doc-truth ... purge" -- arguably scope-creep beyond doc/error-text fixes (though DOCS-04 explicitly allows either) | Stays inside the phase's doc/error-text-only footprint; matches SC1/SC2/SC3's "fix the words" pattern |

**IMPORTANT correction to prior pattern-mapping:** `.planning/phases/117-doc-truth-launch-blocker-purge/PATTERNS.md`
(pre-existing pattern map found in the phase dir) cites `CONSULT-DECISIONS.md:146` as an
"SC4 depth ruling" governing this decision. **That citation is a different SC4** --
`CONSULT-DECISIONS.md:129-146` is the 2026-07-16 `[MANAGER]` ruling on **P116 FIX-03
(slug->id durable-create, Option A/B/D)**, an unrelated decision in a different phase.
There is no existing ruling that constrains this phase's detach decision one way or the
other -- it is open for this phase's planner/coordinator to decide.

**Recommendation:** **Option B now** (reword only), because (1) this phase's other 5
success criteria are pure doc/error-text fixes and Option A would be the only
behavior-adding task in an otherwise text-only phase; (2) the manual recovery is already
fully documented in `troubleshooting.md:329` and is genuinely simple (two git commands);
(3) Option A is a clean, low-risk, well-scoped addition that fits naturally as its own
small phase or a P120 (CLI error hardening) task if the owner wants the real subcommand
later -- file as a GOOD-TO-HAVE with the Option-A scope sketch above rather than
folding a new subcommand into a doc-truth phase. Flag this as an explicit RAISE/decision
point for the coordinator rather than silently picking B.

## SC5a -- `docs/benchmarks/token-economy.md` / `benchmarks/README.md` provenance lie

**This is the most important correction in this research: REQUIREMENTS.md's own DOCS-05
citation (`docs/benchmarks/token-economy.md:51-52`) is stale and, if followed literally,
would introduce a NEW lie on top of an already-fixed one.**

**What DOCS-05 currently says (verbatim):** *"`docs/benchmarks/token-economy.md:51-52` +
`benchmarks/README` claim `reposix_session.txt` (531 tokens) is 'the literal output of
`scripts/demo.sh`' -- that script does not exist, and the fixture depicts the deprecated
FUSE architecture (`/mnt/reposix/...` paths, internally inconsistent IDs). Relabel
honestly (hand-authored fixture, not a captured demo run) even before BENCH-01
re-measures."*

**Ground truth, verified three independent ways:**

1. **`docs/benchmarks/token-economy.md` lines 48-56 (the cited range) contain NO such
   claim today** -- that range is the "What this does NOT measure" honest-caveats
   section. The ACTUAL `reposix_session.txt` provenance line is at line 78: *"an
   ANSI-stripped transcript of the reposix arm's git-native shell session against the
   live GitHub backend"* -- this is machine-generated (confirmed by reading
   `quality/gates/perf/bench_token_economy_captures.py:244`, the exact source template
   for that bullet, called from `bench_token_economy.py:main()` which is the ONLY
   function that writes to `docs/benchmarks/token-economy.md`) and it is **TRUE**.
2. **`benchmarks/fixtures/reposix_session.txt` (read in full, 200 lines) contains NO
   `/mnt/` paths and NO internally-inconsistent IDs.** It is a genuine git-native
   transcript: `reposix init github::reubenjohn/reposix /tmp/reposix-t4-r1` ->
   `git checkout -B main refs/reposix/origin/main` -> `cat issues/56.md` / `57.md` /
   `60.md` -> edit -> `git push` (correctly rejected, read-only GitHub adapter) ->
   `git status`/`git log`. `grep -c '/mnt/' reposix_session.txt` = 0.
3. **`benchmarks/fixtures/README.md`** (a SEPARATE, newer file, git history:
   `2103d0c fix(115-05): restore literal content_hash term in fixtures README`, `fd098c7
   docs(115-05): regen token-economy.md from live GitHub captures + honest provenance`)
   already states explicitly: *"`reposix_session.txt` ... **Real** git-native reposix
   session against the live GitHub backend ... captured P115 T4. No `/mnt/` paths, no
   `scripts/demo.sh`."* -- this file was written specifically to correct the OLD,
   pre-P115 fixture that DID have the defects DOCS-05 describes.

**Conclusion: the fixture and `docs/benchmarks/token-economy.md` were both already fixed
during P115 (2026-07-16, commits `4db6b64`/`fd098c7`/`2103d0c`). DOCS-05's premise about
those two locations is now stale.**

**The actual remaining lie is `benchmarks/README.md:34`** (confirmed: `scripts/demo.sh`
does not exist anywhere in the repo, `find . -iname demo.sh` = no hits):

```
34: - **The reposix session is the literal output of `scripts/demo.sh`** — the bytes the agent's shell would actually place in context. ANSI escapes are stripped.
```

This file is a **larger fossil than just this one line** -- it describes the ENTIRE
retired pre-P115 methodology: the old 98.7%/"150,000 -> ~2,000" figure (line 5, matches
the also-retired claim in `docs/research/initial-report.md:78`, out of this phase's
scope -- that's a research/pitch doc, not a claim-of-fact page), the synthetic
`mcp_jira_catalog.json` "representative" MCP fixture (line 13, superseded by the real
`mcp_github_catalog.json` capture), `ANTHROPIC_API_KEY`/`count_tokens`-based measurement
(lines 21-25, 32, superseded by the live-session-usage methodology), and a reference to
`RESULTS.md` (line 39) that **does not exist** -- `find . -iname RESULTS.md` = no hits.
`benchmarks/README.md` is NOT in `mkdocs.yml`'s nav (confirmed) so it escapes
`mkdocs-strict`/doc-alignment gates entirely -- 0 doc-alignment rows for this file.

**Required end-state:** `benchmarks/README.md` rewritten to describe the CURRENT P115
methodology (live session-usage capture, `benchmarks/captures/*.json`,
`quality/gates/perf/bench_token_economy.py --offline`) matching
`benchmarks/fixtures/README.md`'s already-correct provenance framing, OR (smaller scope,
if the planner wants to stay minimal) at minimum strike the `scripts/demo.sh` line and
the dead `RESULTS.md` pointer.

**Noticed (dead code, filed here since it's directly adjacent):** `render_results_markdown`
in `quality/gates/perf/bench_token_economy_io.py:291` is exported but has **zero
callers** anywhere in the codebase (`main()` in `bench_token_economy.py:137-154` only
calls `render_token_economy_markdown` from the sibling `bench_token_economy_captures.py`
module). This is the vestigial function that would have written the nonexistent
`RESULTS.md` -- dead code from the pre-P115 methodology, safe to delete or worth a
GOOD-TO-HAVES row if out of this phase's scope.

## SC5b -- `docs/social/twitter.md` deleted-FUSE architecture description

**Current-state evidence** (full file read, 35 lines):

```
16: Result: **reposix** — a FUSE filesystem + git-remote-helper for issue trackers.
18: In a simulated benchmark (representative 35-tool Jira-shaped MCP fixture vs reposix shell session): **89.1% fewer tokens** for the same task. Real-world numbers TBD — but the direction is clear.
```

Line 16 describes the pre-v0.9.0 architecture, deleted per `filesystem-layer.md:46`
("The `crates/reposix-fuse/` crate was deleted... the `fuser` dependency, the `/dev/fuse`
permission song-and-dance, and the WSL2 kernel-module quirks all went with it"). Line 18
also carries the retired 89.1%/synthetic-fixture figure (same one being scrubbed from
`index.md`/`token-economy.md`/`reposix-vs-mcp-and-sdks.md`/`latency.md` per commit
`5a5dd29`, 2026-07-16, which did NOT touch `docs/social/`) -- DOCS-06's scope is narrowly
the FUSE line, but line 18's stale figure sits immediately adjacent and a reviewer will
likely want to fix both in one pass (both are `not_in_nav`, escape all doc gates, zero
doc-alignment rows except one: `docs/social/twitter/token-reduction-92pct`, already
flagged `STALE_TEST_DRIFT` at line 18 -- the catalog already knows this row is stale).

**Required end-state:** line 16 rewritten to the current git-native partial-clone
framing (mirror `docs/index.md:13`'s fixed version or `filesystem-layer.md:46`'s
"superseded that virtual filesystem" framing). Suggest: *"Result: **reposix** — a
git-native partial clone + git-remote-helper for issue trackers."* Line 18's figure is
techncially DOCS-07/P118 territory (disputed-figure retraction), but low-cost to align
here since it's the same file, same edit pass, same currently-unbound row.

## Furnished-Product Gaps (GTH-V15-36 -- REQUIRED acceptance-bar input)

Enumerated, actionable items -- owner mandate is explicit that clearing SC1-SC6 alone is
NOT sufficient; this is a first-class cold-reader/IA pass, not a leftover:

1. **Admonitions are used on only 5 of ~30 nav pages** (`contributing.md`,
   `reposix-vs-mcp-and-sdks.md`, `agentic-engineering-reference.md`,
   `integrate-with-your-agent.md`, `first-run.md`) despite `admonition` +
   `pymdownx.details` both being enabled in `mkdocs.yml`. **`docs/index.md` itself uses
   ZERO admonitions** -- the "Honest scope" alpha-caveat paragraph at the bottom
   (`index.md:168`, plain italic prose) is a natural `!!! note "Honest scope"` candidate,
   matching the sanctioned pattern at `reposix-vs-mcp-and-sdks.md:23` (`!!! note "About
   the MCP comparison (live, 2026-07-16)"`).
2. **Furnished-product features (grid cards, tabs, `<details>` folds, md-button CTAs) are
   quarantined to `docs/index.md` alone.** Every how-it-works/concepts/guides page is
   plain prose + at most one mermaid diagram. Propagating index.md's "Where to go next"
   grid-card pattern (`index.md:150-158`) to the bottom of `filesystem-layer.md`,
   `git-layer.md`, `trust-model.md`, `time-travel.md` (a natural "how-it-works quartet"
   nav aid) is a concrete, scoped win.
3. **`docs/social/*` escapes every docs gate** (`not_in_nav` in `mkdocs.yml`) -- exactly
   why the SC5b/SC6 stale-FUSE line survived undetected. No action needed in-phase beyond
   fixing the content, but worth flagging as a GTH for a lightweight freshness grep
   (`FUSE`, `/mnt/`, `mount`) over `docs/social/**`.
4. **Two of the six defects sit on the SAME page as already-correct copy**
   (`filesystem-layer.md` lies in its summary but tells the truth in its body at line 42;
   `index.md` mislabels Confluence in the hero at line 13 but correctly frames it in the
   connector matrix at 130-137) -- these are internal-consistency fixes, not full
   rewrites; a reviewer should read the whole page, not just patch the cited line, so the
   fixed prose matches the page's own existing accurate claims.
5. **Badge/link resolution:** `docs/index.md:7-9` and `README.md:10-17` carry 4-7 badges
   each (CI, Quality weekly, Quality score, Docs, codecov, License, Rust, crates.io) --
   not verified live in this research pass (would require network egress outside this
   research's scope); flag as a pre-phase-close checklist item (`curl -I` each badge URL,
   per global CLAUDE.md Operating Principle 1).
6. **Sanctioned tool:** `/doc-clarity-review` exists as a global slash command (confirmed
   present, referenced by root `CLAUDE.md` § "Cold-reader pass on user-facing surfaces")
   and should run on `docs/index.md` + `README.md` + any page this phase substantially
   rewrites, BEFORE phase close, per GTH-V15-36 item 4.

## Animation Embed Lane (GTH-V15-37)

**Source directory inspected** (read-only, nothing copied): `/home/reuben/workspace/reposix-animation-pitch/`

| File | Real size | Role |
|---|---|---|
| `Reposix Launch Animation.mp4` | 7,145,308 bytes (6.81 MiB / ~7.1MB decimal, matches GOOD-TO-HAVES addendum) | Video fallback / social asset |
| `.thumbnail` | 3,158 bytes, WebP image | Poster frame for the fallback `<video>` |
| `animations-v2.jsx` | 1,483 lines | `Stage`/`SceneStage` playback engine (scrub bar, play/pause, autoplay, localStorage persistence) |
| `reposix-scenes.jsx` | 740 lines | The 7 scene components (Hook/MCP/Flip/Init/Push/Proof/CTA) + top-level `ReposixVideo` mount |
| `tweaks-panel.jsx` | 542 lines | Editor overlay, gated behind a `postMessage` handshake protocol |
| `support.js` | 1,768 lines, `// GENERATED from dc-runtime/src/*.ts` | Runtime harness: loads React 18.3.1 + ReactDOM (UMD, unpkg CDN) + `@babel/standalone` (unpkg CDN), then Babel-transforms the `.jsx` files client-side |
| `Reposix Launch Animation.dc.html` | Entry point | Declares scenes (`OM_SCENES`, 7 entries totalling 80s), `TWEAK_DEFAULTS` (`motionEditor: true` today), Google Fonts `<link>` tags |

**Feasibility per checklist item:**

1. **Pre-compile JSX -> plain JS (remove unpkg/Babel CDN compile).** NEEDS WORK, not
   trivial. `support.js` is a bespoke "dc-runtime" preview harness (Claude-artifact-style
   `<x-dc>`/`<x-import>` custom elements), not something to embed as-is on a production
   docs site. Requires standing up a real build step (the repo has no JS bundler today --
   no `package.json`, no esbuild/vite in this repo; `node v22.22.2`/`npm 10.9.7` ARE
   available on the research machine, `npx esbuild` would work ad hoc) to compile
   `animations-v2.jsx` + `reposix-scenes.jsx` + `tweaks-panel.jsx` into a self-contained
   bundle with React inlined or CDN-pinned. **Open question:** does "pre-compile JSX"
   also mean self-host React/ReactDOM, or is pinned-version CDN React acceptable? Flagged
   below.
2. **Self-host Google Fonts (Space Grotesk, JetBrains Mono).** TRIVIAL, mechanical.
   Confirmed exactly 2 font families, loaded via `fonts.googleapis.com`/`fonts.gstatic.com`
   `<link>` tags in the `.dc.html`; used pervasively (45 `fontFamily:` references in
   `reposix-scenes.jsx`) but only 2 distinct family names (`DISP = 'Space Grotesk'`,
   `MONO = 'JetBrains Mono'`) with 3-4 weights each (400/500/600/700). Download the
   `.woff2` files, add `@font-face`, host under `docs/assets/animation/fonts/`.
3. **Embed-mode config (disable editor, disable localStorage, click-to-play).**
   **Editor: ALREADY effectively disabled for free.** Read `tweaks-panel.jsx:198-238`:
   `TweaksPanel`'s `open` state defaults `false` and `if (!open) return null` -- it only
   opens on a `postMessage({type:'__activate_edit_mode'})` from a PARENT frame, which a
   normal docs-page embed will never send. Setting `TWEAK_DEFAULTS.motionEditor=false`
   per the owner's literal checklist is still worth doing (belt-and-braces, and it's an
   explicit owner instruction) but has no visible runtime effect today since
   `motionEditor` only feeds a checkbox INSIDE the already-hidden panel
   (`reposix-scenes.jsx:731`), not a visibility gate. **localStorage: NEEDS a small code
   patch.** `animations-v2.jsx:544-545,569` reads/writes
   `localStorage[persistKey + ':t']` unconditionally (`persistKey` defaults to
   `'animstage'`, a generic non-namespaced key); GOOD-TO-HAVES confirms the exact bug
   symptom this causes: *"returning visitors currently get a frozen end frame."* No
   existing prop disables this -- needs a small (~5-10 line) patch adding a `persist`
   boolean prop guarding both call sites before pre-compiling. **Autoplay: TRIVIAL, prop
   already exists.** `Stage`/`SceneStage` accept an `autoplay` prop (default `true`);
   pass `autoplay={false}` at the mount site (`reposix-scenes.jsx:726`). A play/pause
   button (`IconButton onClick={onPlayPause}`, `animations-v2.jsx:902`) is ALREADY part
   of the existing playback-bar UI -- no new click-to-play affordance needs to be built,
   just flip the default so it starts paused.
4. **Video fallback hosted as a GitHub release asset, never committed.** File exists,
   size confirmed (7.1MB). **This is an external mutation against the real
   `github.com/reubenjohn/reposix` repo** (creating/editing a GitHub Release, uploading
   an asset) -- per this project's CLAUDE.md Non-negotiables ("External mutations need
   owner-named-target approval"), this needs explicit owner sign-off before any plan
   executes it, not a routine implementation step. Also worth flagging: `release-plz.toml`
   deliberately keeps `git_release_enable = false` for the VERSIONED crate-release tags
   (per root CLAUDE.md's "Release pipeline" section, to avoid 404-ing installer URLs) --
   the animation asset should almost certainly go on a SEPARATE, dedicated release tag
   (e.g. `docs-assets` or `media-v1`), not mixed into an existing `v0.15.x` crate release,
   to avoid interacting with that automation. Flagged as an Open Question.
5. **Docs gates: assets under `docs/assets/animation/`, mkdocs-strict + playwright
   coverage.** `docs/assets/` **does not currently exist** as a directory (confirmed via
   `find`) -- this is a net-new convention, no existing precedent to copy verbatim.
   Closest wiring analog: `mkdocs.yml`'s `extra_javascript`/`extra_css` block (used for
   the self-hosted `docs/javascripts/mermaid-render.js`) for declaring the precompiled
   bundle + fonts; closest embed-mechanics analog: `docs/index.md`'s existing
   `md_in_html`+`attr_list`-based `<div class="grid cards" markdown>` block (`index.md:15`)
   for embedding the mounted component inside markdown. `mkdocs-strict.sh` will catch
   broken relative asset paths/nav entries; a NEW playwright-coverage artifact
   (`.planning/verifications/playwright/...`) following the `mermaid-renders.sh`
   source-artifact pattern would need a matching NEW `docs-build` catalog row, minted in
   the SAME commit as the implementation (catalog-first rule).

**Also noticed:** `Reposix Launch Animation.mp4:Zone.Identifier` and multiple
`uploads/*.png:Zone.Identifier` files exist in the source dir (Windows download
metadata) -- GOOD-TO-HAVES checklist item 5 already anticipates this ("strip Windows
`Zone.Identifier` files from uploads/"); confirmed present, must be stripped before
anything is copied into the repo. Also a dead "export video" button
(`reposix-scenes.jsx:982`, `postMessage({type: 'omelette:request-video-export'})`) that
talks to a parent frame that won't exist in a docs embed -- harmless no-op when clicked
outside the authoring tool, but a rough edge; consider hiding it in embed mode if the
planner wants full polish, not load-bearing.

**No hard blocker found.** All 5 checklist items are buildable with tools already on the
research machine (`node`/`npm`); item 1 (bundle precompile) and item 4's localStorage
sub-piece are the only "needs care" items -- everything else is mechanical.

## Doc-Alignment Drift Map

Every doc this phase is expected to touch, checked against
`quality/catalogs/doc-alignment.json` (400 rows total). **BOUND** rows whose line range
overlaps a planned edit will trip `STALE_DOCS_DRIFT` at pre-push and require a
**top-level-only** `/reposix-quality-refresh <doc>` pass (per `quality/CLAUDE.md` §
Docs-alignment dimension) before the phase can close green.

| Doc | Rows | Touches a BOUND row at the edit line? |
|---|---|---|
| `docs/index.md` (SC1, line 13) | 29 rows total | **YES** -- `docs/index/real-git-working-tree` (line 13) AND `docs/index/rest-supported-backends` (line 13), both `BOUND`/`BIND_GREEN` |
| `docs/how-it-works/filesystem-layer.md` (SC2) | 5 rows | **YES if editing line 42** (`filesystem-layer/blob-lazy-first-cat`, `BOUND`); **NO if only editing the summary at lines 7-13** (no row overlaps) |
| `docs/reference/glossary.md:20` (SC2 propagation) | 24 rows, **ALL `RETIRE_CONFIRMED`** | NO -- glossary.md's alignment rows are already retired from active binding (no BOUND rows at all); low checkpoint risk |
| `docs/reference/cli.md:59` (SC2 propagation) | 8 rows | **YES** -- `docs/reference/cli.md/init_documented` spans lines 42-65, `BOUND` |
| `docs/reference/git-remote.md:23` (SC2 propagation) | 8 rows | **YES** -- `git-remote/stateless-connect-read-path` is anchored exactly at line 23, `BOUND` |
| `docs/reference/confluence.md:135-138` (SC2, lower priority) | 3 rows (110-128, 152-154, 6-8) | NO -- none overlap line 135-138 |
| `crates/reposix-cli/src/attach.rs` (SC4) | 0 rows | N/A -- doc-alignment tracks doc-file claims, not Rust source; editing the error string does not itself trigger `STALE_DOCS_DRIFT` |
| `docs/guides/troubleshooting.md:329` (SC4) | 7 rows (all in lines 9-75, 227) | NO -- line 329 is outside every existing row's range |
| `crates/reposix-cli/src/list.rs` / `refresh.rs` (SC3) | 0 rows each | N/A -- same reasoning as attach.rs |
| `docs/benchmarks/token-economy.md` (SC5a, if touched at all) | 9 rows, lines 8-40 only | NO -- line 78 (the provenance bullet, which needs NO edit per this research) is outside every row's range |
| `benchmarks/README.md` (SC5a, real fix location) | **0 rows** | N/A -- not in mkdocs nav, entirely untracked by doc-alignment |
| `docs/social/twitter.md:16,18` (SC5b/SC6) | 1 row: `docs/social/twitter/token-reduction-92pct` at line 18, **already `STALE_TEST_DRIFT`** | Partially -- the row is already flagged stale independent of this edit; this edit adds more reason for a refresh but doesn't newly break a GREEN row |

**Planner budget implication:** at minimum ONE `/reposix-quality-refresh` checkpoint is
needed for the wave touching `docs/index.md` (2 rows) + `filesystem-layer.md` (if line 42
is touched, 1 row) + `docs/reference/cli.md` (1 row) + `docs/reference/git-remote.md` (1
row) -- these can plausibly be refreshed together in one pass if those edits land in the
same wave. `glossary.md`, `confluence.md`, `troubleshooting.md`, `attach.rs`,
`list.rs`/`refresh.rs`, `token-economy.md`, `benchmarks/README.md` carry no such
requirement for their specific edited lines.

## NOTICED

1. **[HIGH-VALUE] REQUIREMENTS.md's DOCS-05 citation is itself stale** and, if followed
   literally, would relabel an ALREADY-ACCURATE claim as a lie -- see § SC5a. This is the
   single highest-value finding in this research; the planner must NOT blindly copy
   DOCS-05's prescribed fix ("relabel honestly as hand-authored") onto
   `token-economy.md`, only onto `benchmarks/README.md`.
2. **`.planning/phases/117-doc-truth-launch-blocker-purge/PATTERNS.md` already exists**
   (a prior pattern-mapping pass, dated 2026-07-16, found in the phase directory before
   this research began) and is a genuinely useful cross-reference -- but contains one
   factual error: its SC4 citation of `CONSULT-DECISIONS.md:146` as a governing ruling is
   actually an unrelated P116 FIX-03 decision (see § SC4 above). Use PATTERNS.md for its
   exemplar/analog citations, not for that one ruling claim.
3. **Stale line numbers are the norm, not the exception**, across the phase brief,
   ROADMAP.md, and REQUIREMENTS.md (SC1: ~135-152 vs actual 13; SC4: 135-138 vs actual
   144; SC5a: 51-52 vs actual N/A/78). Every ticket-cited line number in this phase's
   source documents should be treated as approximate; this RESEARCH.md's line numbers are
   the ones verified against the current checkout.
4. **`main.rs`'s module-doc comment (`crates/reposix-cli/src/main.rs:4-12`) is itself
   stale** -- it lists only 6 subcommands ("sim, init, list, refresh, spaces, version")
   but `enum Cmd` has 9+ variants (Attach, Sync, Doctor, History + Time-travel alias, plus
   gc/cost/tokens as separate modules per `lib.rs`). Minor, outside the 6 named
   blockers, but a first-time reader of the source (not just the docs site) hits the same
   "doc doesn't match reality" pattern this whole phase is purging. Consider a 1-line fix
   if a wave touches this file anyway for SC3/SC4.
5. **Dead code adjacent to SC5a:** `render_results_markdown`
   (`quality/gates/perf/bench_token_economy_io.py:291`) has zero callers -- see § SC5a.
6. **`docs/research/initial-report.md:78`** still carries the retired "150K -> 2K, 98.7%"
   figure -- explicitly OUT of this phase's scope (it's a historical pitch/research doc,
   not a claim-of-fact page, and P118/DOCS-07 already owns the disputed-figure-retraction
   work for `.planning/PROJECT.md`), but flagged here so the planner doesn't accidentally
   assume it's already clean when scoping DOCS-07/P118 boundaries.

## OPEN QUESTIONS / RAISE candidates

1. **SC4 build-vs-reword is a genuine decision, not a research finding** -- see decision
   table. Recommendation given (Option B / reword now), but this should be confirmed with
   the coordinator/owner before the plan locks it in, since DOCS-04's text explicitly
   allows either path.
2. **Animation Lane checklist item 4 (GitHub Release upload) is an external mutation**
   requiring owner-named-target approval per this project's CLAUDE.md Non-negotiables --
   this cannot be executed autonomously inside a GSD phase without that approval. Needs
   explicit RAISE before any plan schedules the actual `gh release create`/`gh release
   upload` step. Suggested target: a dedicated non-crate release tag (e.g. `docs-assets`
   or `media-v1`), NOT an existing `v0.15.x` crate-release tag (see `release-plz.toml`'s
   `git_release_enable = false` rationale).
3. **Animation Lane checklist item 1 scope ambiguity:** does "pre-compile JSX -> plain JS"
   also require self-hosting React/ReactDOM (currently pinned-version `unpkg.com` CDN,
   NOT the Babel-standalone step that's explicitly called out), or is CDN-hosted,
   version-pinned React acceptable to keep? Affects whether item 1 is a pure build-step
   change or also a hosting/CSP-posture change. Recommend keeping pinned CDN React (small,
   stable, cacheable, matches the `mermaid-render.js` precedent of pinning a CDN version)
   unless the owner wants zero external JS dependencies for the docs site.
4. **Should `docs/how-it-works/git-layer.md`, `time-travel.md`, `trust-model.md` still
   get a wave in this phase** even though SC2's verified-negative check found no actual
   defect in them? The phase brief named them as "at minimum" propagation targets; this
   research found them clean. Recommend the planner skip dedicated edit tasks for these
   three files for SC2 specifically, but they remain natural candidates for the
   furnished-product "propagate index.md's grid-card pattern" polish item (§
   Furnished-Product Gaps #2) if that's in scope for this phase vs. deferred to P119.

## Sources

### Primary (HIGH confidence -- verified via direct tool use against this checkout)
- `docs/index.md`, `docs/how-it-works/filesystem-layer.md`, `docs/how-it-works/{git-layer,time-travel,trust-model}.md`, `docs/reference/{glossary,cli,git-remote,confluence}.md`, `docs/benchmarks/token-economy.md`, `docs/social/twitter.md`, `benchmarks/README.md`, `benchmarks/fixtures/README.md`, `README.md` -- read in full via `cat -n`.
- `crates/reposix-cli/src/{init,attach,list,refresh,main}.rs`, `crates/reposix-cli/src/lib.rs`, `crates/reposix-remote/src/stateless_connect.rs`, `crates/reposix-cache/src/{builder,cache,lib}.rs`, `quality/gates/perf/bench_token_economy{,_io,_captures}.py` -- read/grepped directly.
- Live capture: `cargo run -p reposix-cli -- list --project demo --origin http://127.0.0.1:19999` and the equivalent `refresh` invocation, both against a guaranteed-closed port inside a `/tmp` isolate -- exact error text captured verbatim.
- `benchmarks/fixtures/reposix_session.txt` -- read in full (200 lines), zero `/mnt/` occurrences confirmed via grep.
- `quality/catalogs/doc-alignment.json` (400 rows) -- queried programmatically for every candidate edited file.
- `.planning/ROADMAP.md:135-152`, `.planning/REQUIREMENTS.md:73-105,308-313`, `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` (GTH-V15-36/37 full text), `.planning/CONSULT-DECISIONS.md:125-146` -- read for canonical, current ticket text (superseding the phase-brief's paraphrase where they disagree).
- `/home/reuben/workspace/reposix-animation-pitch/*` -- read-only inspection of all 4 `.jsx`/`.js` files, the `.dc.html` entry point, and file sizes.
- `crates/CLAUDE.md:92-95` (Error-message convention, 3-part bar) -- confirmed exact wording.

### Secondary (MEDIUM confidence)
- `.planning/phases/117-doc-truth-launch-blocker-purge/PATTERNS.md` -- pre-existing pattern map found in the phase directory, cross-checked against direct evidence; one citation error found and corrected (§ SC4 / NOTICED #2), rest corroborated.

## Metadata

**Confidence breakdown:**
- SC1-SC3, SC5a, SC5b: HIGH -- every claim grounded in a live grep/read/cargo-run against this exact checkout.
- SC4: HIGH on current-state facts; the recommendation itself is a judgment call, flagged as an Open Question for coordinator confirmation.
- Animation Lane: HIGH on source-directory facts (file sizes, code structure); MEDIUM on the exact bundling/CI approach since no JS build tooling exists in this repo yet (net-new infrastructure, no precedent to verify against).
- Furnished-product gaps: HIGH -- directly enumerated via grep across all nav pages.

**Research date:** 2026-07-16
**Valid until:** Short shelf life (~7 days) -- this phase is itself about doc drift, and this research already found the phase's OWN source tickets (ROADMAP/REQUIREMENTS) drifted from the code within the same milestone. Re-verify line numbers immediately before planning if more than a few days pass.
