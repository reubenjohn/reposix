← [back to index](./index.md)

# Cross-cutting investments, Risks, Skills & Sequencing

## 3. Immediate cross-cutting investments

Things that touch every milestone — easy to forget, costly to retrofit.

1. **Code quality ratchet.** Pick up the `simplify` skill (already available) and run it on every phase's changeset. Add per-crate coverage floors via `cargo-llvm-cov`: ratchet to **80% line coverage** by v0.11.0, **85%** by v0.13.0. CI fails on regression. Phase 49 lands the floor; subsequent phases inherit.

2. **Security regression kit.** Codify the threat-model checks into `scripts/security-regression.sh` — one named command that asserts: allowlist enforcement, frontmatter field stripping, bulk-delete cap, append-only audit triggers, `Tainted<T>` compile-fail tests. Runs in CI. Per global OP-4, this replaces ad-hoc bash. Lands in v0.10.0 (tooling phase).

3. **Dogfood loop.** Use reposix's *own* GitHub issues as the authoritative backlog. An autonomous agent (Phase 62 production case study extends this) opens reposix issues for regressions; humans triage them with `cd ~/work/reposix-issues && grep -r "regression" issues/`. Meta but demonstrable; a primary v0.15.0 sales asset.

4. **`reposix-agent-flow` skill, generalized.** Phase 36 ships a project-level skill encoding the dark-factory regression test. Generalize it across milestones: every new feature that affects agent UX adds a new test fixture to the skill's transcript suite. By v0.13.0 the skill exercises clone, lazy fetch, sparse-checkout recovery, conflict rebase, multi-project, and observability tail.

5. **README hero — numbers, not adjectives.** Audit `README.md` for every adjective on the landing page; replace with a measured number. *"Agent fetches an issue in 2 kB"* not *"fast and light."* *"p50 80 ms"* not *"low latency."* Lands in Phase 41 (hero rewrite); enforced by a regex-based linter in Phase 40 tooling.

6. **`doc-clarity-review` on every doc page.** Every phase that ships docs runs the skill on each new/changed page in isolation. The skill copies pages to `/tmp` and reviews them context-free — exactly the cold-reader scenario. Phase 45 codifies this as a release gate.

---

## 4. Risks and open questions

1. **Is the MCP-equivalent benchmark fair?** Phase 47 hinges on a meaningful comparison. If we cherry-pick an MCP server with bloated schemas, the numbers prove nothing. **Resolve before v0.11.0:** pick a public MCP server (`mcp-server-github` is the obvious candidate), document the comparison methodology in `docs/benchmarks/methodology.md`, invite skeptics to PR alternative baselines.

2. **Does the blob-limit teaching mechanism survive real-world agent diversity?** ARCH-12 validates this with one dark-factory transcript. Different agent harnesses (Devin, SWE-agent) may handle stderr differently or fail to surface the helper's error message. **Resolve before v0.12.0:** Phase 51's recipe CI jobs are the empirical answer — if Cursor or Aider doesn't surface the blob-limit error in a way the agent can read, the recipe fails CI and we redesign.

3. **How do we handle large binary attachments in partial clone?** Issues with attached PDFs / images / video — lazy-fetch a 10 MB blob blocks the agent loop. Architecture-pivot-summary §7 didn't address this. **Resolve before v0.13.0:** decision between (a) skip-fetch by content-type-and-size on the client, (b) cache-only proxy with download-on-demand, (c) Git LFS-style filter. Likely (a) for v0.13, (c) for v1.0.

4. **Sequencing: do v0.10.0 docs need v0.11.0 numbers?** Phase 41's "vs MCP" table needs real numbers from v0.11.0's benchmark harness. **Resolve before starting v0.10.0:** either (a) seed Phase 41 with placeholder numbers and refresh after v0.11.0, (b) reorder — ship a minimal benchmark in v0.10.0 to populate the table, defer the polished blog-post asset to v0.11.0. (Recommended: option b — ship a single "tokens-to-first-insight" number in v0.10.0; the rest waits.)

5. **Who owns the registry for third-party connectors?** v0.14.0 ecosystem milestone. crates.io has no first-party tagging mechanism we control; a central `reposix-registry.toml` requires governance. **Resolve before starting v0.14.0:** owner picks. Recommendation: start with the crates.io tag `reposix-connector` + GitHub topic; defer central registry until ≥3 third-party connectors exist.

---

## 5. Suggested new skills / tooling

Skill conventions follow Claude Code's `.claude/skills/<name>/SKILL.md`.

- **`reposix-agent-flow`** *(planned in Phase 36)* — extended in v0.13.0 to cover multi-project + observability transcripts.
- **`reposix-perf-smoke`** — runs the v0.11.0 latency harness against all configured backends; diffs results vs the baseline in `docs/benchmarks/v0.11.0-comparison.md`; flags regressions. Gates v0.12+ phases that touch hot paths.
- **`reposix-banned-words`** — enforces P2 progressive-disclosure rules; reads the layer-banned-word list from `docs/.banned-words.toml` and rejects any commit that introduces a banned term above its assigned layer. Replaces the ad-hoc Phase 40 grep.
- **`reposix-dogfood-loop`** — runs a create-edit-close cycle against the project's own GitHub issues (or simulator if `--no-network`). Used by `gsd-autonomous` as a final pre-merge gate. Fulfills cross-cutting investment §3.3.
- **`reposix-recipe-runner`** *(v0.12.0)* — given a recipe directory (`docs/guides/agents/<harness>/`), spins up the harness in a subprocess, runs `run.sh`, diffs against `expected.txt`. The CI test fixture for every Phase 51 / 52 recipe.
- **`reposix-conflict-replay`** *(v0.13.0)* — replays a captured conflict-rebase transcript against a fresh checkout. Used to detect regressions in the helper's `error refs/heads/main fetch first` rendering across git versions.
- **`reposix-changelog-narrate`** *(v0.15.0)* — given a milestone's commit log, drafts the launch-kit blog post + HN post. Owner edits; skill never publishes autonomously.

---

## Sequencing rationale (why this order)

`v0.10.0 docs` first because the architecture pivot ships fresh and a docs-first milestone forces the team to *understand* what shipped before optimizing it. `v0.11.0 benchmarks` second because docs need the numbers, and benchmarks are easier to author when the architecture is freshly in mind. `v0.12.0 integrations` third because integration recipes need both stable transport (v0.9.0) and credible numbers (v0.11.0) before adoption is plausible. `v0.13.0 observability` fourth because by then there are users to give signal — premature observability is bikeshed. `v0.14.0 ecosystem` fifth because plugins need a stable trait surface, which the prior four milestones harden. `v0.15.0 launch` last — by definition.

**What was cut.** Direct user-facing UI was considered (`reposix-web`). Cut: agents don't need it; humans use git. Multi-tenant hosted service was considered. Cut: contradicts local-first thesis. A "real-time collaboration" milestone was considered. Cut: git already does this — the whole point.
