# §3 Innovations (continued: 3g–3m) — [index](./index.md)

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
