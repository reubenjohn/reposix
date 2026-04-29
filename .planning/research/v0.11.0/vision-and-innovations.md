# Vision & Innovations Brainstorm — Post-v0.10.0 Trajectory

**Status:** Research draft, not a roadmap mutation. Owner reads, picks signal, ignores noise.
**Author context:** Drafted 2026-04-24 evening, immediately after v0.10.0 (Docs & Narrative Shine) shipped and v0.11.0 (Performance & Sales Assets) entered planning. The owner asked: "periodically re-think the vision and propose original innovations." This document is the artifact.
**Inputs cited (not re-derived):** `.planning/PROJECT.md`, `.planning/research/v0.10.0-post-pivot/milestone-plan.md`, `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md`, `docs/research/initial-report.md`, `docs/research/agentic-engineering-reference.md`, `CLAUDE.md`, `docs/index.md`, `docs/concepts/*.md`, `.planning/notes/v0.11.0-doc-polish-backlog.md`, `.planning/CATALOG.md`.

---

## 1. Where reposix sits today (one paragraph)

reposix as of 2026-04-25 is a Rust workspace that turns any REST issue tracker into a real git working tree. v0.9.0 deleted the FUSE layer and replaced it with a `stateless-connect` + `export` hybrid `git-remote-reposix` helper backed by a local bare-repo cache (`reposix-cache`); the working tree is now a bona fide partial-clone checkout, not a virtual filesystem. v0.10.0 made that pivot legible — three concept pages, three how-it-works pages with mermaid diagrams, a 5-minute tutorial verified by `scripts/tutorial-runner.sh`, a banned-words linter, a README hero with measured numbers (`8 ms` cache read, `24 ms` cold init, `9 ms` list, `5 ms` capabilities probe), and a CHANGELOG that finalised the architecture story. The agent UX promise is: **after `reposix init <backend>::<project> <path>`, every operation is a git command the agent already knows from pre-training; reposix is invisible.** Five-second positioning: *"reposix turns issue trackers into git repositories so coding agents stop burning 100k tokens on MCP schemas and start spending zero on `cat`."*

---

## 2. Five-year vision (3-5 bullets)

Concrete future states. Picked for credibility, not aspiration:

1. **The dark-factory pattern is a recognised industry term, with reposix cited as the canonical reference implementation.** Simon Willison's `agentic-engineering-reference.md` distillation already names it. By 2031, "ran a thousand agents against a simulator overnight" should be a job description, and `reposix-sim` should be the example people teach from. Measurable proxy: ≥3 conference talks (workshop, programming-language venue, or industry track) cite reposix when discussing simulator-first agent infrastructure.

2. **Every coding agent in the top 10 by usage offers reposix as a default issue-tracker integration alongside MCP.** Not exclusive of MCP — alongside. The model is GitHub Copilot offering reposix as a one-click "treat your tracker as a git repo" option. Measurable proxy: Claude Code skill registry, Cursor recipe library, Aider extension docs, and Continue's tool catalog all ship a `reposix-init`-flavoured guide.

3. **There are >25 community `BackendConnector` crates covering tools beyond the original three (GitHub, Confluence, JIRA).** Not 50 — that's vanity. 25 is the credibility line; it means Linear, Notion, Asana, ClickUp, Phabricator, Redmine, Trac, Bugzilla, Pivotal, GitLab Issues, Bitbucket, ServiceNow, Zendesk Support, Salesforce Cases, FreshDesk, Plane, Trello, monday.com, plus a long tail. Measurable proxy: `reposix-connector` crates.io tag has ≥25 published crates by 2031.

4. **reposix's token-economy claim is a measured CS-publishable result.** Not a marketing slide — a paper at a programming languages or systems workshop with reproducible benchmarks comparing dark-factory POSIX-over-REST against MCP and raw SDK loops on a fixed task suite, cost per task in dollars, latency CDFs, and an honest discussion of where the abstraction breaks. Measurable proxy: at least one peer-reviewed citation by 2030.

5. **`reposix gc` and `reposix doctor` are the Rust-equivalents of `git fsck` for the agent era.** When something is broken in an agent's filesystem-based tracker integration, the first instinct should be to run `reposix doctor`. This requires the diagnostics layer to be cared-for, not bolted-on. Measurable proxy: ≥80% of GitHub issues filed against `reubenjohn/reposix` include `reposix doctor` output without prompting.

Rejected because not credible inside five years: a managed cloud service business; an enterprise-RBAC differentiator; a Windows/macOS native filesystem rewrite. See §4.

---

## 3. Original innovations (the meat)

For each: name, one-paragraph description, technical sketch, and the next-step phase that would prove it. Twelve listed; the first six are the highest-conviction picks. The owner picks two for v0.11.0 / v0.12.0 ship per §5.

### 3a. `reposix doctor /path/to/repo` — the agent-era `git fsck`

**The pitch.** When the helper isn't working, an agent sees opaque protocol-v2 errors and bails. `reposix doctor` runs in 2 seconds, prints a numbered list of findings, and gives each one a copy-pastable fix command. **The dark-factory failure mode reposix has** is "agent fails to recover from setup misconfig because the error originates two layers down the stack." Doctor closes that.

**Mechanism sketch.** A new `reposix-cli` subcommand that, given a path:
- runs `git config --get extensions.partialClone` and asserts `origin`;
- inspects `remote.origin.url`, parses the `reposix::` scheme, validates `BackendConnector` is registered;
- opens `~/.cache/reposix/<project>/cache.db`, runs `PRAGMA integrity_check` + checks for the WAL append-only triggers from `crates/reposix-core/src/audit.rs`;
- reads `REPOSIX_ALLOWED_ORIGINS` and compares against `remote.origin.url`'s host;
- runs a 1-blob sample fetch through the helper with `GIT_TRACE_PACKET=1` and reports framing OK/wrong;
- reports each finding with a fix command (`git config extensions.partialClone origin`, `export REPOSIX_ALLOWED_ORIGINS=…`, `reposix init --repair`, etc.).

**Phase to prove it.** v0.11.0 candidate. ~2 days; one new module in `reposix-cli`, ~10 diagnostic checks, snapshot tests against fixture-corrupted repos. Pairs naturally with `reposix-banned-words` skill discipline — every check has a static "fix" string that's lint-checked for clarity.

**Why now.** v0.10.0 docs reduce setup friction; doctor catches the long tail when docs aren't enough. Also: every "is reposix broken" bug report turns into a `doctor` invocation, which is intelligence we can't otherwise gather.

### 3b. Time-travel via git tags — the audit log made replayable

**The pitch.** Every `Cache::sync` tags the bare repo with a UTC timestamp (`reposix/sync/2026-04-25T15:30:00Z`). An agent or human can `git checkout reposix/sync/2026-04-20T15:30Z` to see exactly what `issues/PROJ-42.md` looked like last Tuesday. The audit log says *what the helper did*; the tag history says *what the world looked like at every point*. Together they're a fully replayable history, which is forensics gold and a unique differentiator no MCP server can offer.

**Mechanism sketch.** In `crates/reposix-cache/src/builder.rs`, after every successful `sync` transaction (ARCH-07), write `git update-ref refs/tags/reposix/sync/<RFC3339-Z> <commit>`. Tags live in the bare cache, not the working tree, so they don't pollute `git tag -l` in the agent's checkout. A new `reposix log --time-travel` subcommand exposes them. A CLI option `reposix init --since=<RFC3339>` checks out a working tree pinned to the matching sync tag — useful for "show me what the bug looked like when I filed it."

**Phase to prove it.** v0.12.0. ~3 days; tag emission is one line per sync, the CLI surface is the bulk of the work. Acceptance: filing an issue, syncing, mutating the issue, syncing again — the agent can `git checkout reposix/sync/<earlier>` and see the older content, then `git diff reposix/sync/<earlier> reposix/sync/<later> -- issues/<id>.md` shows the field-level change.

**Why this is the most original idea in this document.** I cannot find prior art for "tag every external sync as a git ref." git-bug uses Lamport timestamps internally; jirafs has timestamped backups. Neither exposes them as first-class git refs that an agent can `checkout`. This pattern generalises beyond reposix — any partial-clone promisor remote could do it.

### 3c. Token-cost ledger — built-in cost telemetry

**The pitch.** Every helper invocation writes one row to a `cost` table in the cache DB: `(ts, op, bytes_in, bytes_out, est_input_tokens, est_output_tokens, model_hint)`. `reposix cost --since 7d` prints a Markdown table comparing reposix's measured token spend against an extrapolated MCP-equivalent (using the v0.7 token-economy benchmark's 92.3% reduction as a calibration). Users see real cost savings in dollars per week, committed to a queryable artifact, not to a marketing pitch.

**Mechanism sketch.** Add a `cost.rs` module to `reposix-cache` that estimates tokens from byte counts via a configurable heuristic (default: `bytes / 3.5` characters-per-token). The `model_hint` is sniffed from `CLAUDE_MODEL`/`OPENAI_MODEL`/`ANTHROPIC_MODEL` env vars — the agent harness sets these. `reposix cost` is a CLI subcommand that aggregates by op, week, project. The cells in `docs/concepts/reposix-vs-mcp-and-sdks.md` switch from "characterized" to "measured" — closes a P2 doc-clarity finding (§v0.11.0 doc backlog) without any docs work.

**Phase to prove it.** v0.11.0 candidate. ~2 days. Pairs naturally with the `reposix-bench` work in milestone-plan.md §v0.11.0 (Phase 46): bench gives the *one-shot* measured number; the cost ledger gives the *continuous* measured number. Bench informs the marketing site; the ledger informs the agent's own dashboard.

**Why now.** The v0.7 token-economy benchmark is a one-shot artifact (`benchmarks/RESULTS.md`). The cost ledger turns it into ongoing telemetry. This is also the most concrete proof point for the OP-1 "ground-truth obsession" principle: it's not believed-cost, it's measured-cost.

### 3d. Multi-project helper process — share cache + connection pool

**The pitch.** Today every `git-remote-reposix` invocation is its own short-lived process. An agent juggling 20 projects opens 20 cache DBs, 20 reqwest pools, 20 SQLite WAL files. **Multi-project helper** is one daemon (or one process per backend host) that serves N partial clones from a shared cache. Memory footprint drops 20×, connection reuse means GitHub/JIRA rate limits are honoured globally, and observability (§3e) becomes a single attachment point.

**Mechanism sketch.** Two parts:
1. **Helper-side:** `git-remote-reposix` checks for `~/.run/reposix-helper.sock` and, if it exists, proxies all stdin/stdout to the daemon. If not, spawns the daemon (or runs in-process for one-shot use). The daemon is a long-lived process that owns the cache directories, holds an LRU of `Cache` instances, and demuxes by project key.
2. **Cache-side:** `reposix-cache::Cache` becomes `Send + Sync` (it nearly is already), and the daemon multiplexes requests via tokio. Egress allowlist enforcement stays per-cache instance; auditing remains per-project (one DB per project is correct — projects are tenants).

**Phase to prove it.** v0.13.0 (Observability & Multi-Repo per `milestone-plan.md` §v0.13.0). The Phase-56 description in milestone-plan.md already names this; this section just makes the case concrete. Acceptance: 20 fresh `reposix init` calls inside a script, then `git fetch` in all 20 — total wall-clock and total memory under thresholds.

**Why now.** Real users have ≥10 projects. Today reposix doesn't fall over; it just leaks resources. This is the difference between "alpha" and "credible operational substrate."

### 3e. OpenTelemetry tracing — the dark-factory dashboard

**The pitch.** Helper emits OTLP spans for every `command=fetch`, every blob materialization, every `export` push. A user with one `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317` line on their `docker-compose` gets a Jaeger/Honeycomb/whatever dashboard with live agent activity. For team-scale dark-factory deployments (multiple agents, multiple projects, real backends), this is the operations dashboard that makes the pattern legible to humans who aren't in the tracker themselves.

**Mechanism sketch.** Already named in milestone-plan.md §v0.13.0 (Phase 54). Add `tracing-opentelemetry` to `reposix-core`; instrument the four hot paths (cache materialization, helper fetch handler, helper push handler, conflict-detection path); document a sample Jaeger compose file. The novelty isn't OTel itself — it's the *specific application*: the spans carry `taint.origin`, `audit.row_id`, `agent.harness_hint`, and `project.key` attributes that make dark-factory queries first-class. (E.g., "show me every push from a Claude-Code-flagged run that was rejected with conflict.")

**Phase to prove it.** v0.13.0 (Phase 54 already in milestone-plan.md). This document upgrades it from "nice tooling" to "the canonical dashboard story" — propose adding a `docs/reference/observability/dark-factory-queries.md` page with the 5 most useful cross-project Jaeger queries.

**Why now.** v0.11.0's bench gives one-shot numbers; OTel gives continuous numbers. They compose: bench is the calibration, OTel is the production signal.

### 3f. Native conflict resolution UI — `reposix conflict <issue-id>`

**The pitch.** When push hits the `error refs/heads/main fetch first` path, the standard recovery is `git pull --rebase`. This works but it's *textual* — the conflict marker `<<<<<<<` shows raw YAML lines, which is confusing for frontmatter where field semantics matter. `reposix conflict <issue-id>` opens a 3-way merge view (CLI or browser-rendered HTML) showing local · base · remote with **field-level diffing** (status, assignee, labels, body) instead of line-level. Better than `git mergetool` for this domain.

**Mechanism sketch.** New `reposix-cli` subcommand `reposix conflict <id>`:
- reads `.git/MERGE_HEAD` and the conflicted `issues/<id>.md`;
- parses three frontmatter blocks (HEAD / MERGE_HEAD / merge-base);
- renders a side-by-side text table for fields, plus a unified diff for the body;
- offers `--accept-local`, `--accept-remote`, `--field <key>=<value>` flags to resolve;
- writes the resolved file and stages it.

A future browser variant: `reposix conflict --web <id>` opens `localhost:7777/conflict/<id>` with a small WASM/HTML UI. Skip for v1.

**Phase to prove it.** v0.12.0 candidate. Pairs with the `reposix-conflict-replay` skill from milestone-plan.md §5. The skill regression-tests the rendering across git versions; the UI is what humans use when they're in the loop.

**Why now.** Conflict UX is currently the place where reposix's "everything is just git" promise is most stressed. An agent recovers fine from `fetch first` — but a *human* operator inspecting why the agent re-tried six times wants the field-level view.

### 3g. Plugin registry — `reposix init <plugin>::<project>` auto-installs

**The pitch.** `reposix init linear::eng-backlog ~/work/eng` should work whether or not the user has `cargo install reposix-connector-linear`d. The CLI resolves `<plugin>` against a community registry (`registry.reposix.dev` or a GitHub topic), confirms before installing, and runs `cargo install` under the hood. Lowers the barrier from "hear about a connector → write a paragraph in the docs → use it" to "hear about a connector → use it." The architectural argument from `initial-report.md` is that POSIX-over-REST is a *substrate*, not three integrations; the registry is what operationalises that.

**Mechanism sketch.** Two layers:
- **Registry mechanism:** start with the crates.io tag `reposix-connector` + a curated `reposix-registry.toml` in `reubenjohn/reposix` that lists vetted plugins. Long-term: a real registry (`registry.reposix.dev`) with signed entries.
- **CLI:** `reposix init <plugin>::<project> <path>` checks if the plugin is locally installed; if not, prompts (or `--auto-install`) and runs `cargo install reposix-connector-<plugin>`. Plugins are dynamic-loaded via `dlopen` (libloading crate) or — safer — invoked as subprocess executables (`reposix-connector-linear` binary speaking the `BackendConnector` trait over JSON-RPC on stdio).

**Phase to prove it.** v0.14.0 (Phase 60 in milestone-plan.md). Subprocess plugins are the credible v1 design; dlopen plugins are a v2 stretch.

**Why now.** v0.10.0 docs scaffolded a "Write your own connector" guide. The registry closes the loop from authorship to discovery. Without it, the connector-ecosystem vision is purely aspirational.

### 3h. Adversarial swarm replay — `reposix swarm replay <fixture>`

**The pitch.** `crates/reposix-swarm/` already runs N agent-shaped clients hammering the simulator (the StrongDM 10k-agent QA pattern at miniature scale). What it's missing is **fixture-replayable swarms**: a captured sequence of N×M operations that can be replayed deterministically against any backend (sim or real-with-write-cap) to reproduce a specific bug or stress signal. This is the dark-factory equivalent of `cargo bench --baseline`.

**Mechanism sketch.** `reposix-swarm` gains a `--record` mode that writes a JSONL transcript of every operation (timestamp, agent-id, op, target, result). A new `reposix swarm replay <transcript.jsonl>` reproduces the sequence with deterministic timing. Combined with the simulator's seed-from-fixture capability, any swarm bug becomes a single-file artifact that future agent sessions can grep, diff, and re-run.

**Phase to prove it.** v0.13.0. Pairs with the chaos-audit work already in `crates/reposix-swarm/tests/chaos_audit.rs`. Acceptance: `reposix swarm record --target sim` for 60 seconds produces a transcript; `reposix swarm replay <file>` reproduces the same audit-log shape on a clean DB.

**Why now.** OP-6 "ground-truth obsession" demands it. Today swarm bugs are session-scoped; replay makes them committed-or-fixture artifacts.

### 3i. Capability negotiation — `BackendConnector::capabilities()` exposed at clone time

**The pitch.** Today, every backend implements the full `BackendConnector` trait (read, write, delete, list_changed_since, hierarchy). But Linear has no concept of a "transition", JIRA has 50; Confluence has page hierarchies, GitHub doesn't. The current code papers over this with `Result<_, FeatureUnsupported>` errors at runtime. Capability negotiation makes it static: at `git clone` time, the helper advertises which fields are writable for this backend, and the CLI/working tree reflects that (read-only frontmatter fields render with a `# read-only` YAML comment, à la jirafs's auto-injected hints from `initial-report.md` §"Validating Workflow Transitions").

**Mechanism sketch.** Already partially modeled — `BackendFeature::Hierarchy` exists in `crates/reposix-core/src/backend.rs`. Generalise: `enum BackendFeature { Hierarchy, Comments, Attachments, Transitions(Vec<TransitionRule>), CustomFields(Schema) }`. `BackendConnector::capabilities() -> BackendCapabilities`. The cache materialiser injects `# valid_transitions: [InProgress, Done]` as a YAML comment when rendering frontmatter. The agent reads it, picks a valid value, and the push succeeds — no 400 round-trip.

**Phase to prove it.** v0.13.0 or v0.14.0. Tied to the connector-ecosystem work — a stable capabilities surface is what makes `BackendConnector` semver-able (Phase 58 in milestone-plan.md).

**Why now.** Pre-emptive validation closes the agent feedback loop one round-trip earlier, which compounds across long autonomous runs.

### 3j. `reposix archive` / `reposix gc` — bounded disk usage

**The pitch.** `~/.cache/reposix` grows monotonically. Every agent who's run reposix for ≥1 month has a stale cache for a project they touched once. Two complementary subcommands:
- `reposix gc` — explicit cache eviction. Modes: LRU (default, evict oldest blobs until size < quota), TTL (drop blobs unread for >N days), per-project quota.
- `reposix archive <project>` — when a project is dormant, archive its cache directory to a tarball+SHA256, drop the live state. Restoring is `reposix archive --restore <tarball>` (just `tar xf` plus a config rewrite).

**Mechanism sketch.** GC is the harder one — git's object store has its own GC dynamics; reposix-cache GC must work *with* git, not against it. Approach: drop blobs from the bare cache that aren't reachable from `refs/reposix/main` AND haven't been read in the last `gc.ttl` (audit table query). The blob's pack file is rewritten without it; git transparently re-fetches if needed. Archive is simpler — `tar` the cache dir, write a manifest with version/origin/last_fetched_at, drop the live dir.

**Phase to prove it.** v0.12.0 candidate. ~3 days for `gc`, ~1 day for `archive`. Pairs with §3a (`reposix doctor`) — doctor flags "cache size > 500MB" with "fix: `reposix gc --quota 200MB`".

**Why now.** The v0.11.0 doc-polish backlog explicitly flags "Cache eviction" milestone tag confusion. The longer reposix is in production use, the more this matters.

### 3k. Streaming push for bulk edits — bounded memory under load

**The pitch.** The current `export` handler buffers the full fast-import stream in memory, parses it, then issues REST writes. For 1000-issue bulk operations (a workflow renaming, a label sweep), this is a memory spike. Streaming push parses and writes incrementally, with a barrier at the `done` terminator for atomicity. Out-of-scope per `architecture-pivot-summary.md` §7 open-question 6 ("stream-parsing performance for export"); this proposal makes it concrete.

**Mechanism sketch.** Replace the in-memory parser in `crates/reposix-remote/src/fast_import.rs` with a streaming state machine. For each `commit` block: parse, validate (frontmatter allowlist), enqueue a REST write to a bounded tokio channel; the writer task issues calls with a configurable concurrency limit (`REPOSIX_PUSH_CONCURRENCY`, default 4). On `done`: drain the channel, commit-or-rollback. Conflict detection (ARCH-08) becomes a per-issue check inside the streaming loop.

**Phase to prove it.** v0.12.0 stretch or v0.13.0. Acceptance: a 5000-issue bulk push uses <50MB peak memory and respects backend rate limits.

**Why now.** Real users will hit 1000-issue scenarios. The current architecture doesn't fail; it just stresses memory.

### 3l. A research-paper draft — formalise the dark-factory pattern

**The pitch.** The token-economy claim (92.3% reduction vs MCP for the same task, per `benchmarks/RESULTS.md`) is currently a project artifact. Formalised as a peer-reviewed paper, it becomes a citation a future-agent-infra book can hang an entire chapter off. Submit to a workshop at PLDI / OOPSLA / ICSE / a programming-language venue, or to LangSec / DSN for the security framing. The paper would: (a) define dark-factory as a pattern with measurable invariants; (b) compare POSIX-over-REST against MCP and raw-SDK on a fixed task suite; (c) discuss the lethal-trifecta cuts as security trade-offs; (d) honest limitations (REST schema breakage, partial-clone GC).

**Mechanism sketch.** Not code. A 12-page paper draft. Owner decides: pure engineering or hybrid. If hybrid, allocate 4 weeks for the writing + figure work, gated on v0.11.0 Phase 46 producing reproducible numbers. Co-author candidate: Simon Willison (dark-factory pattern is his framing).

**Phase to prove it.** v0.15.0 or post-1.0. This is in the milestone-plan.md §v0.15.0 launch section but understated — propose elevating it.

**Why this is non-obvious.** Most agent-infra projects optimise for adoption; reposix is one of the few with a *measurable* claim (the 92.3% number) that survives peer review. Squander that and it stays vibes; cash it in and reposix becomes the citation.

### 3m. Other ideas (lower conviction, listed for completeness)

- **`reposix preview`** — local read-only web view of the working tree for humans. Cut: humans can use `cat`; this is solving a non-problem.
- **`reposix rotate`** — credential rotation helper. Useful but not differentiating; defer.
- **Web preview as live agent monitor** — interactive demo at `demo.reposix.dev`. The OTel dashboard (§3e) covers this better.
- **Audit replay UI** — separate from §3h; would generate a synthetic git history from audit rows. Useful for forensics but niche; build only if a real user requests it.
- **Self-hosted reposix as Kubernetes operator** — out of scope per §4. Cut.

---

## 4. Cuts and trade-offs (be honest)

Things explicitly rejected and why:

1. **"reposix as a managed cloud service in 2026."** Premature. OSS adoption first. Without ≥1000 GitHub stars and ≥3 paying customer interviews, a managed offering is a distraction. Re-evaluate at v1.0 if the OSS metrics warrant it.

2. **"Reinvent git for partial-clone improvements."** Out of scope. Git already does what we need (>2.34); fighting upstream is a multi-year sink. If git's partial-clone GC is bad, write a `reposix gc` (§3j) that works *with* git, not against it.

3. **"Compete with GitHub / Atlassian on tracker functionality."** Orthogonal. reposix is a substrate; if a backend lacks a feature, that's the backend's choice. Do not build a reposix-native sprint planner.

4. **"Polyglot connectors via subprocess."** Tempting (Python connectors are easier to write) but a serious complication for the security model. Subprocess connectors blow up the egress allowlist enforcement story unless the subprocess inherits the same `reposix_core::http::client()` factory — which means *re-implementing it in Python*. Defer to v0.14.0 plugin-registry phase, decide then. Recommend: Rust-only for stability commitment; subprocess connectors only as a stretch goal once Rust-only ecosystem proves out.

5. **Windows / macOS native filesystem rewrite.** Pre-v0.9.0 pivot this was a real concern (FUSE-on-Windows is a different VFS). Post-pivot it's moot — git works on every platform. This stays cut forever; don't reopen.

6. **"Real-time collaboration" milestone.** Cut, correctly, in milestone-plan.md §"What was cut": git already does this. The branch-per-draft pattern from `initial-report.md` §"Confluence Hierarchies and Draft Lifecycles" is the answer.

7. **A "reposix-native" agent harness.** Tempting (control the full stack), but it would invert the value proposition: reposix's whole point is to be invisible to whatever harness the user picked. Build *recipes* (milestone-plan.md §v0.12.0) for existing harnesses, not a competing harness.

---

## 5. Recommended next-3-month plan

Sequenced by value × reachability against the v0.11.0 "Performance & Sales Assets" milestone in `.planning/PROJECT.md`. The owner already has v0.11.0 scaffolded around bench harness + doc-polish backlog + helper-multi-backend-dispatch fix + `IssueId→RecordId` rename.

**Month 1 (v0.11.0 ship — already in flight).**
- Land what's planned: bench harness (Phase 46), MCP-equivalent baseline (Phase 47), recorded asciinema + blog draft (Phase 48), coverage ratchet (Phase 49), 9 major + 17 minor doc-clarity findings, helper-multi-backend-dispatch carry-forward, `IssueId→RecordId` rename.
- **Add to v0.11.0:** §3c **token-cost ledger** (~2 days). It folds naturally into Phase 46/47's bench work — once we can count tokens for the bench, we can count them for every helper invocation. Promotes the `reposix-vs-mcp` table cells from "characterized" to "measured" without any docs work, which is OP-1 ground-truth gold.

**Month 2 (v0.11.5 or early v0.12.0).** Pick **two** innovations:
- §3a **`reposix doctor`**. Lowest-risk, highest leverage. Every confused-user bug report becomes a doctor invocation; every doctor finding teaches reposix something it didn't know. ~2 days. Lands as a new phase in v0.11.5 or first phase of v0.12.0.
- §3b **time-travel via git tags**. Highest originality, ~3 days, no breaking changes. Differentiates reposix from every MCP server out there. Lands as second phase of v0.12.0.

Combined: ~5 engineering days, two surprising features, both compose with everything that came before.

**Month 3 (v0.12.0 second half).**
- Begin agent-SDK integration recipes (milestone-plan.md §v0.12.0). Dogfood the doctor + time-travel work against the recipe CI jobs.
- Assess: if doctor is generating ≥1 fix-this-finding per week, it's earning its keep; if not, prune diagnostics. If time-travel-via-tags is in use by ≥1 external user, prioritise the `reposix log --time-travel` UI; if not, leave it as a power-user feature.

**Why these two innovations specifically for month 2:** Doctor de-risks adoption (§2 future state #5). Time-travel-via-tags creates a pithy talk-track (§2 future state #1, #4) that no competitor has. Together they're cheap, original, and synergistic with the v0.11.0 work already in flight. Other innovations (multi-project helper, OTel, conflict UI, plugin registry) are correct candidates for v0.13.0+ and don't need to be pulled forward.

---

## 6. Originality audit

Honest call on each §3 idea. Categories: **Novel (original to reposix or near-original)**, **Hybrid (well-known idea, novel application)**, **Well-known (table stakes)**.

| Innovation | Category | Why |
|---|---|---|
| §3a `reposix doctor` | Hybrid | Doctor commands are common (`brew doctor`, `flutter doctor`, `docker doctor`). Novel application: agent-era diagnostics where the *fixes* are what matter, not the findings, because an autonomous agent reads the fix string and runs it. |
| §3b time-travel via git tags | **Novel (highest)** | I cannot find prior art for "tag every external sync as a git ref." git-bug uses Lamport timestamps; jirafs has snapshots. Neither exposes per-sync points as first-class git refs an agent can `checkout`. |
| §3c token-cost ledger | Hybrid | Cost telemetry is well-known (LangSmith, Helicone). Novel application: built-in to a git remote helper, persisted in the same SQLite as the audit log, queryable by agents themselves. |
| §3d multi-project helper process | Well-known | Daemon-with-shared-state is standard server design. Listed because the *application* (one git remote helper serving N partial clones) is what milestone-plan.md already commits to; documenting the rationale matters. |
| §3e OpenTelemetry tracing | Hybrid | OTel itself is table-stakes. Novel application: spans carry `taint.origin` + `agent.harness_hint` attributes that make dark-factory queries first-class — no other tracing setup formalises that. |
| §3f conflict resolution UI | Well-known | 3-way merge UIs are old. Field-level diffing of YAML frontmatter is mildly novel but not breakthrough. |
| §3g plugin registry | Well-known | Cargo, npm, pip have registries; this is the same idea. Novelty is the BackendConnector subprocess protocol if we go polyglot. |
| §3h swarm replay | Hybrid | Chaos engineering and load replay are well-known. Novel application: agent-shaped operations as the unit of replay, with deterministic timing for bug reproduction. |
| §3i capability negotiation | Hybrid | API capabilities are well-known. Novel application: injecting valid transitions as YAML comments in the frontmatter (this is from `initial-report.md` §"Validating Workflow Transitions" — credit there). |
| §3j `reposix gc / archive` | Well-known | Cache eviction with LRU/TTL is standard. Listed because users will demand it. |
| §3k streaming push | Well-known | Streaming parsers are standard performance engineering. |
| §3l research paper | **Novel (process)** | The *paper* isn't novel; the *act of formalising the dark-factory pattern as a publishable claim with measurable comparisons* is novel for this project category. Most OSS agent-infra ships marketing slides; almost none submit to peer review. |

**Tally:** 2 Novel · 5 Hybrid · 5 Well-known. The novel ones (§3b, §3l) are the ones to lead with publicly. The hybrids are the ones that compound. The well-knowns are operational must-do — table stakes for the §2 future state where reposix is a credible production substrate.

**Self-check:** is this audit honest? §3b is the call I'd most expect to be wrong about — there may be prior art I haven't found. Recommend a 1-hour literature search before committing publicly to "first to do this." Search terms: "git remote tag every fetch", "promisor remote temporal snapshot", "external-sync git ref."

---

## 7. Open questions for the owner

Five decisions only the owner can make. Pick before v0.12.0 planning starts.

1. **Research paper, yes or no?** §3l. If yes, allocate 4 weeks of writing + figure work after v0.11.0 numbers stabilise. Co-author Simon Willison? If no, deprecate this from §2 future state #4 and rely on conference talks instead.

2. **Plugin registry: Rust-only or polyglot?** §3g + §4 cut #4. Rust-only is safer for the security model (egress allowlist enforcement), polyglot is friendlier for ecosystem growth. Recommendation: Rust-only for v0.14.0; revisit polyglot at v1.0 if ≥3 community members ask for Python connectors. Owner decides timing.

3. **Managed-service future state on the table?** §4 cut #1. Recommendation: not in 2026, re-evaluate at v1.0 with concrete OSS metrics. Owner confirms the cut.

4. **Public roadmap: who owns it once contributors arrive?** Today the owner is `reubenjohn`; `.planning/` is single-author. At ≥10 community contributors, this stops working. Recommendation: split `.planning/` (private, owner-only) from `ROADMAP.md` (public, milestone-level). Owner decides when to flip.

5. **Branding vs MCP — sit alongside, replace, or position above?** v0.10.0 docs commit to "complement, not replace" (banned word: "replace"). The §2 future state #2 ("alongside MCP") is consistent with that. But §3l's research paper would imply *above* MCP for the 80% of operations. Pick one frame and hold it across docs, social, and the paper if it happens. Recommendation: "complement for the 80%, replace nothing." Owner ratifies.

---

## 8. Owner decisions (2026-04-25)

Resolutions on the §7 open questions. These ratify scope and unblock v0.11.0 planning.

1. **Research paper — out of scope for now.** §3l deferred indefinitely; downgrade §2 future-state #4 ("CS-publishable result") to a stretch goal. Conference-talk track stays viable.
2. **Plugin registry — Rust-only for now.** §3g + §4 cut #4 ratified. Subprocess/polyglot connectors stay parked; revisit at v1.0 only if a real ecosystem signal emerges. The egress-allowlist enforcement story remains intact because every connector links the same `reposix_core::http::client()` factory.
3. **Managed service — deferred (may revisit in 2026).** §4 cut #1 stays the default posture. Owner reserves the right to revisit later in 2026 if OSS adoption metrics warrant; until then no architectural decisions assume a hosted offering.
4. **`.planning/` split at >10 contributors — agreed.** Trigger condition stays "≥10 active community contributors." Action when triggered: split `.planning/` (private, owner-only state) from a public `ROADMAP.md` (milestone-level commitments). No work needed today.
5. **Branding posture — "complement for the 80%, replace nothing."** Ratified. This is the canonical phrasing across docs, social, and any future paper. The banned-words linter (`scripts/banned-words-lint.sh` + `.banned-words.toml`) keeps this honest at the doc layer.

These decisions are roadmap inputs, not code changes. Acted on in v0.11.0 planning by:
- Removing §3l from any active phase scope (was already deferred to v0.15.0+).
- Promoting the §3c token-cost ledger and §3a `reposix doctor` to v0.11.0 candidates per §5.
- Holding §3g plugin-registry / subprocess polyglot work parked behind v0.14.0+ gating.
- Adding "complement for the 80%, replace nothing." to the banned-words config as the canonical positioning anchor.

---

*End of brainstorm. ~590 lines + owner decisions. Not roadmap-mutating; the v0.11.0 ROADMAP entry is the binding artifact.*
