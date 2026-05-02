# §3 Original Innovations — [index](./index.md)

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

*Continued in [innovations-g-m.md](./innovations-g-m.md) — §3g through §3m.*
