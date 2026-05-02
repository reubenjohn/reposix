# Post-v0.9.0 Milestone Plan — Draft

**Status:** Research draft. Owner reviews and promotes to ROADMAP.md / MILESTONES.md.
**Author context:** Drafted 2026-04-24, immediately after v0.9.0 architecture pivot was scoped (phases 31–36). v0.9.0 ships the transport. Everything below is what makes it *land*.
**Source-of-truth inputs:** `architecture-pivot-summary.md`, `REQUIREMENTS.md` (ARCH-01..19), `notes/phase-30-narrative-vignettes.md`, `PROJECT.md` core-value statement.

---

## Chapters

- [§3–6: Cross-cutting investments, Risks, Skills, Sequencing rationale](./chapter-2.md) — Everything after the milestones list: immediate cross-cutting investments (§3), risks and open questions (§4), suggested new skills/tooling (§5), and sequencing rationale.

---

## 1. Opening — End-vision statement

**The agent experience in v1.0.** A developer drops `cargo install reposix` (or grabs a release tarball) into their agent harness — Claude Code, Cursor, Aider, raw `claude` SDK, doesn't matter. Five minutes later, the agent has run `reposix init github::acme/backlog ~/work/acme` and is filing JIRA tickets discovered from `grep -r "TODO" ~/src`, opening GitHub issues for failing CI runs in `gh run view` output, and routing Confluence page edits through `git push`. The agent never reads an OpenAPI schema, never enumerates an MCP tool tree, never sees the word "endpoint." Every operation it performs is one its training data already knows: `cat`, `sed`, `git commit`, `git push`. When the backend rejects a write, the agent reads `! [remote rejected] main -> main (fetch first)` and runs `git pull --rebase` — because that is what its priors tell it to do, not because reposix taught it.

**The repo to a cold human visitor in v1.0.** Landing page loads in under a second. Hero is a 12-second asciinema — no marketing copy, just keystrokes: `reposix init`, `cat`, `sed`, `git push`. Below the fold: a measured numbers table — *"reposix vs MCP vs raw REST: 2 kB vs 150 kB vs 47 kB tokens-to-first-insight, p50 latency 80 ms vs 1.4 s vs 320 ms"* — sourced from a benchmark harness in CI, not a marketing deck. A "How it works" section opens like a watch-back — three pages (cache, git, trust) each with one mermaid diagram, playwright-screenshot-verified. A dozen integration recipes for Claude Code / Cursor / Aider / Continue / Devin / SWE-agent / raw-API are each backed by a CI job that runs them against the simulator on every push. A `BackendConnector` crate guide invites third parties to wire Linear, Notion, Asana — which is the architectural argument: POSIX-over-REST is the substrate, not the integration set.

---

## 2. Proposed milestones

Sequenced so each milestone unlocks the next: **docs make the repo legible → benchmarks make it credible → integrations make it adopted → observability makes it operable → ecosystem makes it durable → launch ships it.** Phases numbered ≥40 to leave headroom for v0.9.0 phase decimals (e.g. 36.1 hotfixes).

### v0.10.0 — "Docs & Narrative Shine"

**Thesis.** A cold visitor understands reposix in 10 seconds and runs the tutorial in 5 minutes. The architecture pivot becomes a story, not a code change.

**Success gate.**
- `mkdocs build --strict` green; banned-word linter passing on every page.
- A first-time human reader (proxied via `doc-clarity-review` skill) can state the value prop within 10s of arriving and complete the tutorial without backtracking.
- Playwright screenshots committed for landing + how-it-works + tutorial at desktop (1280px) and mobile (375px).
- Zero references to deleted concepts (`FUSE`, `inode`, `daemon`, `mount`, `fusermount`) anywhere in `docs/` or `README.md`.

**Phases.**

- **Phase 40 — Tooling + skeleton.** Vale + banned-word linter wired to pre-commit and CI. 14 page skeletons + 2 `.gitkeep` so the new nav doesn't dangle. Inherits Phase 30's plan structure (`30-01-PLAN.md` → `40-01-PLAN.md`) but rewrites every banned-word list entry — `FUSE`, `inode`, `daemon`, `mount` are gone; new banned-above-Layer-3 list is `partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2`.
- **Phase 41 — Hero + mental model + vs-MCP.** Hero rewrite of `docs/index.md` — issues are files, edit them, `git push`. Three-up value props enforce P1 (complement, not replace — the word "replace" stays banned in hero copy). "Mental model in 60 seconds" page reframed for git-native: *clone = snapshot · frontmatter = schema · `git push` = sync verb*. "reposix vs MCP / SDKs" comparison table, populated from v0.11.0's benchmark harness output (back-edge dependency — see Risks §4.2).
- **Phase 42 — How-it-works carver.** Three pages: **The cache layer** (`reposix-cache` crate, blob-on-demand, audit), **The git layer** (`stateless-connect` + `export`, partial clone, sparse-checkout teaching mechanism), **The trust model** (taint typing, allowlist, append-only audit, blob-limit guardrail). One mermaid diagram per page, rendered via mcp-mermaid, screenshotted via playwright. Content sourced from `architecture-pivot-summary.md` §2–4.
- **Phase 43 — Tutorial + guides.** Five-minute first-run tutorial — `reposix init sim::demo /tmp/repo`, `cat`, edit, `git commit`, `git push`. Runs end-to-end as a CI test fixture (the doc IS the test). "Write your own connector" guide rewritten against current `BackendConnector` trait. "Integrate with your agent" reframed as a pointer to v0.12.0's recipe set.
- **Phase 44 — Nav restructure + theme + grep-audit.** `mkdocs.yml` per Diátaxis (Home / How it works / Guides / Reference / Decisions / Research). mkdocs-material theme tuning. Grep-audit of `docs/` and `README.md` for stale FUSE references. Delete obsolete pages (`architecture.md`, `security.md` merged into how-it-works/trust-model). Per OP-4: the doc tree must match shipped reality.
- **Phase 45 — Verification + cold-reader review + screenshots.** `doc-clarity-review` skill run on every page in isolation. 14 playwright screenshots committed. Banned-word linter green. CHANGELOG `[v0.10.0]` finalized. Tag script `scripts/tag-v0.10.0.sh`.

### v0.11.0 — "Performance & Sales Assets"

**Thesis.** The numbers table on the landing page is real, reproducible, and devastating to MCP. reposix's value prop becomes empirical, not rhetorical.

**Success gate.**
- A `cargo run -p reposix-bench` produces a JSON+Markdown report comparing reposix / raw REST / MCP-equivalent across at least: tokens-to-first-insight, wall-clock for the canonical "find issues mentioning X, comment on each" task, cost-per-task at standard model prices.
- Numbers committed to `docs/benchmarks/v0.11.0-comparison.md`.
- An asciinema recording is embedded on the landing page, sourced from a committed script (no hand-edited frames).
- A 1500-word blog-post draft lives in `docs/social/blog-posts/v0.11.0-the-2kb-issue.md`.

**Phases.**

- **Phase 46 — Bench harness foundation.** New crate `crates/reposix-bench` consumes ARCH-17's latency artifact and adds: token counting (via `tiktoken-rs` or honest-tokenizer), cost-modelling, and a "comparable task" runner that performs the same workflow via three substrates (reposix git, raw REST SDK, MCP-equivalent tool — gated by whether one exists for the backend).
- **Phase 47 — MCP-equivalent baseline.** Wire up `mcp-server-github` (or a forked variant) as the comparison baseline. Document fairness assumptions (which MCP server, what schemas loaded, what model). Risks §4.1: the comparison must be fair or the entire claim collapses.
- **Phase 48 — Recorded asciinema + blog draft.** Hero asciinema — `reposix init` → grep → edit → push, single take, 12 seconds, generated from a committed `scripts/demos/hero.sh`. Blog post draft: "How we made an issue tracker fit in 2 kB." Includes the numbers table and the architecture-pivot story.
- **Phase 49 — Coverage ratchet to 80%.** Per cross-cutting investment §3.1: add `tarpaulin` or `cargo-llvm-cov` to CI, set per-crate floors (`reposix-cache` 90%, `reposix-remote` 85%, `reposix-cli` 70%), make CI fail on regression. The `simplify` skill runs over the changeset.

### v0.12.0 — "Agent-SDK Integration Guides"

**Thesis.** A user who already runs Claude Code / Cursor / Aider / Continue / Devin / SWE-agent / a raw SDK loop can adopt reposix in five minutes by copy-pasting a recipe. Each recipe is tested in CI.

**Success gate.**
- At least four recipes exist, each with: setup snippet, working agent loop, "gotcha" callout, asciinema/video, and a CI job that runs the recipe end-to-end against the simulator.
- A user-proxy review (run via `doc-clarity-review`) confirms each recipe is self-sufficient — a fresh reader can run it without consulting other docs.

**Phases.**

- **Phase 50 — Recipe scaffolding.** Define the recipe schema: `docs/guides/agents/<harness>/{recipe.md, run.sh, expected.txt}`. Each `run.sh` is a CI fixture. Add `.github/workflows/agent-recipes.yml` that fans out one job per recipe.
- **Phase 51 — Tier-1 recipes (candidate set: Claude Code, Cursor, Aider, raw Anthropic SDK).** Each recipe demonstrates the dark-factory flow from ARCH-12: agent given only `reposix init` + a goal, completes the task via pure git/POSIX. Cursor and Aider recipes use their respective configuration formats; Claude Code uses a SKILL. Raw SDK recipe is a 50-line Python script.
- **Phase 52 — Tier-2 recipes (candidate set: Continue, SWE-agent, Devin, OpenAI assistants).** Lower-priority — only those whose harnesses can be exercised in CI without paid credentials. Devin may stay as a hand-curated guide without a CI job; document the constraint.
- **Phase 53 — "Gotchas" page + troubleshooting matrix.** Common failure modes: blob-limit refusal misread as a bug, `git pull --rebase` loop on perpetual conflicts, `REPOSIX_ALLOWED_ORIGINS` misconfiguration, credential env var name mismatches. Each gotcha has a recipe-specific callout and a troubleshooting matrix entry.

### v0.13.0 — "Observability & Multi-Repo"

**Thesis.** A user running reposix at any non-trivial scale (multiple projects, multiple agents, real backends) can see what's happening in real time. "Audit log" stops being a ground-truth artifact you `sqlite3` after the fact and becomes a live signal.

**Success gate.**
- Helper emits OpenTelemetry traces (configurable via `OTEL_EXPORTER_OTLP_ENDPOINT`); a sample dashboard JSON ships in `docs/reference/observability/`.
- `reposix tail` streams audit events from the SQLite WAL in real time (think `journalctl -f`).
- A single `git-remote-reposix` process can serve multiple projects from one helper invocation (cache shared between projects on the same backend); CI test asserts cross-project isolation despite shared process.

**Phases.**

- **Phase 54 — OTel spans on cache + helper hot paths.** `tracing` + `tracing-opentelemetry` integration. Spans on every blob materialization, every `command=fetch`, every push attempt. Sampling configurable.
- **Phase 55 — `reposix tail` subcommand.** Streams audit table inserts (SQLite `update_hook` or polling fallback). Default human-readable, `--json` for piping. Dogfoodable for the Phase 56 dashboard.
- **Phase 56 — Multi-project helper process.** One `git-remote-reposix` invocation can serve `reposix::github/repo-a` and `reposix::github/repo-b` from one cache directory. Cross-project isolation enforced at the cache-key level. Required for v0.14.0 plugin contributions where one helper hosts many backends.
- **Phase 57 — Project dashboard page.** Static page (or simple WASM) rendering audit-log rollups: pushes/day, blob-fetch rate, p99 latency by op, top contributors. Backed by the `reposix tail --json` stream.

### v0.14.0 — "Plugin Ecosystem" *(candidate, sequence-flexible)*

**Thesis.** Third parties can write a `BackendConnector` for any REST-addressable system in an afternoon, distribute it as a crate, and publish it via a registry — proving the architectural argument that POSIX-over-REST is a substrate, not three integrations.

**Success gate.**
- Connectors guide rewritten as a from-scratch tutorial; sample crate `reposix-connector-template` published.
- One reference third-party-style connector lands as a separate crate (candidate: Linear / Notion / Asana — owner picks based on dogfooding need).
- A registry-discovery mechanism (whether crates.io tag, GitHub topic, or a `reposix-registry.toml`) is documented.

**Phases.** *(left intentionally light — owner shapes after v0.13 ships)*

- **Phase 58 — `BackendConnector` API stabilization.** Lock the trait surface; document semver guarantees. May require a 0.1 → 1.0 transition for `reposix-core`.
- **Phase 59 — `reposix-connector-template` crate.** Cookiecutter-style — a working connector with TODO markers, contract tests, and CI. Owner picks the seed backend.
- **Phase 60 — Registry mechanism.** Decide between: crates.io tag (`reposix-connector`), GitHub topic, central `reposix-registry.toml`, or all three.

### v0.15.0 — "Launch Readiness" *(candidate)*

**Thesis.** reposix is ready for HN, Twitter, an Anthropic-blog cross-post, and a production case study from the owner's own dogfooding.

**Success gate.**
- HN/Twitter launch kit committed (`docs/social/launch-v1.0/`); blog post finalized.
- Production case study from the owner's own usage of reposix (meta-dogfooding §3.3) committed.
- Security audit sign-off from a fresh `security-review` skill pass.
- Versioned-stability commitment: `reposix-core` ≥ 1.0, semver guarantees documented, deprecation policy written.

**Phases.** *(left light)*

- **Phase 61 — Security audit + threat-model refresh.** `security-review` skill run end-to-end. Threat model updated for git-native architecture (the `research/threat-model-and-critique.md` revision deferred from v0.9.0 §7 open-question 3).
- **Phase 62 — Production case study.** Document the owner's own dogfooding: how reposix is used to triage reposix's own GitHub issues. Numbers + transcript.
- **Phase 63 — Launch kit + 1.0 cut.** Tag v1.0.0. Versioned-stability commitment. HN post. Twitter thread. Anthropic-blog cross-post draft.
