# CLAUDE.md — reposix project guide

This file is read by every agent (Claude Code, Codex, Cursor, etc.) that opens this repo. It's the local extension of the user's global CLAUDE.md (`~/.claude/CLAUDE.md`) and overrides nothing — it adds project-specific rules.

## Project elevator pitch

reposix exposes REST-based issue trackers (and similar SaaS systems) as a git-native partial clone, served by `git-remote-reposix` from a local bare-repo cache built from REST responses. Agents use `cat`, `grep`, `sed`, and `git` on real workflows — no MCP tool schemas, no custom CLI, no FUSE mount. See `docs/research/initial-report.md` for the architectural argument and `docs/research/agentic-engineering-reference.md` for the dark-factory pattern that motivates the simulator-first approach.

## Architecture (git-native partial clone)

> **Source of truth:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (ratified 2026-04-24).

The reposix runtime has three pieces:

- **`reposix-cache`** — a real on-disk bare git repo built from REST responses via the `BackendConnector` trait. The cache produces a fully populated tree (filenames, directory structure, blob OIDs) but materializes blobs lazily — only when the helper requests one on git's behalf. Every materialization writes a row to the SQLite audit table; bytes return wrapped in `reposix_core::Tainted<Vec<u8>>`.
- **`git-remote-reposix`** — a hybrid git remote helper. It advertises `stateless-connect` (read path: tunnels protocol-v2 fetch traffic to the cache's bare repo with `--filter=blob:none`) and `export` (push path: parses the fast-import stream, runs push-time conflict detection against the backend, applies REST writes on success). Refspec namespace is `refs/heads/*:refs/reposix/*`.
- **`reposix init <backend>::<project> <path>`** — bootstraps a partial-clone working tree: `git init`, `extensions.partialClone=origin`, `remote.origin.url=reposix::<scheme>://<host>/projects/<project>`, then `git fetch --filter=blob:none origin`.

After `init`, agent UX is pure git: `cd <path> && git checkout origin/main && cat issues/<id>.md && grep -r TODO . && <edit> && git add . && git commit && git push`. Zero reposix CLI awareness required beyond `init`.

Two guardrails are load-bearing for the dark-factory pattern:

- **Push-time conflict detection.** The helper checks backend state when `git push` runs and rejects with the standard git "fetch first" error if the remote drifted. The agent recovers via `git pull --rebase && git push` — no custom protocol.
- **Blob limit.** The helper refuses `command=fetch` requests that would materialize more than `REPOSIX_BLOB_LIMIT` blobs (default 200), with a stderr error that names `git sparse-checkout` as the recovery move. An agent unfamiliar with reposix observes the error and recovers without prompt engineering.

## Operating Principles (project-specific)

The user's global Operating Principles in `~/.claude/CLAUDE.md` are bible. The following are project-specific reinforcements, not replacements:

1. **Simulator is the default / testing backend.** The simulator at `crates/reposix-sim/` is the default backend for every demo, unit test, and autonomous agent loop. Real backends (GitHub via `reposix-github`, Confluence via `reposix-confluence`, JIRA via `reposix-jira`) are guarded by the `REPOSIX_ALLOWED_ORIGINS` egress allowlist and require explicit credential env vars (`GITHUB_TOKEN`, `ATLASSIAN_API_KEY` + `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT`, `JIRA_EMAIL` + `JIRA_API_TOKEN` + `REPOSIX_JIRA_INSTANCE`). Autonomous mode never hits a real backend unless the user has put real creds in `.env` AND set a non-default allowlist. This is both a security constraint (fail-closed by default) and the StrongDM dark-factory pattern.
2. **Tainted by default.** Any byte that came from a remote (simulator counts) is tainted. Tainted content must not be routed into actions with side effects on other systems (e.g. don't echo issue bodies into `git push` to remotes outside an explicit allowlist). The lethal-trifecta mitigation matters even against the simulator, because the simulator is *seeded* by an agent and seed data is itself attacker-influenced.
3. **Audit log is non-optional, and lives in TWO append-only tables.** `audit_events_cache` (cache-internal events — blob materialization, gc, helper RPC fetch/push, sync-tag writes) lives in the cache crate (`reposix-cache::audit`); `audit_events` (backend mutations — `create_record` / `update_record` / `delete_or_close`) lives in the core crate (`reposix-core::audit`) and is written by the sim/confluence/jira adapters. A complete forensic query reads both. Either schema missing a row for a network-touching action means the feature isn't done. The dual-table shape is intentional; physical unification behind a `dyn AuditSink` trait is deferred.
4. **No hidden state.** Cache state, simulator state, and git remote helper state all live in committed-or-fixture artifacts. No "it works in my session" bugs.
5. **Working tree IS a real git checkout.** The whole point of v0.9.0 is that `.git/` is real, not synthetic; `git diff` is the change set by construction, not by emulation. The partial clone (`extensions.partialClone=origin`) makes blobs lazy, but everything else is upstream git.
6. **Real backends are first-class test targets.** Three canonical targets are sanctioned for aggressive testing: **Confluence space "TokenWorld"** (owned by the user; safe to mutate freely), **GitHub repo `reubenjohn/reposix` issues** (ours; safe to create/close issues during tests), and **JIRA project `TEST`** (default key; overridable via `JIRA_TEST_PROJECT` or `REPOSIX_JIRA_PROJECT`). See `docs/reference/testing-targets.md` for env-var setup, owner permission statement, and cleanup procedure. Simulator remains the default (OP-1), but "simulator-only coverage" does NOT satisfy acceptance for transport-layer or performance claims.
7. **Phase-close means catalog-row PASS.** No phase ships on the executing agent's word. An unbiased verifier subagent grades the catalog rows — if RED, the phase loops back. See "Verifier subagent dispatch" in the Quality Gates section below.
8. **Plans accommodate surprises (the +2 phase practice).** Every
   milestone reserves its **last two phases** as absorption slots for
   what reality surfaces during planned-phase execution:
   - **Slot 1 — Surprises absorption.** Issues a planned phase
     discovered but couldn't fix without doubling its scope. The
     discovering phase appends to
     `.planning/milestones/<v>-phases/SURPRISES-INTAKE.md` (one entry
     per item: severity + what + whf(c)zy-out-of-scope + sketched
     resolution) instead of silently skipping or expanding scope.
     The surprises-absorption phase drains the file: each entry → RESOLVED |
     DEFERRED | WONTFIX, each with a commit SHA or rationale.
   - **Slot 2 — Good-to-haves polish.** Improvements (clarity, perf,
     consistency, grounding) the planned phases observed but didn't
     fold in. Same intake mechanism, separate file
     (`GOOD-TO-HAVES.md`). Sized XS / S / M; XS items always close;
     M items default-defer to next milestone.

   **Eager-resolution preference (load-bearing):** when a planned
   phase observes a surprise or polish item, prefer fixing it inside
   the discovering phase IF (a) < 1 hour incremental work, (b) no
   new dependency introduced. The +2 reservation is for items that
   genuinely don't fit the discovering phase. **What this practice
   prevents:** the "found-it-but-skipped-it" failure mode where good
   signal gets dropped to keep a phase tight; AND the
   "scope-creep-to-fit-the-finding" failure mode where a phase grows
   to twice its planned size to absorb every drift discovered. The
   intake split makes "I saw it, here's what I think, P<last-2> will
   handle it" the default move.

   **Verifier honesty check:** the surprises-absorption phase's
   verifier subagent spot-checks the previous phases' plans + verdicts
   and asks "did this phase honestly look for out-of-scope items?" An
   empty intake is acceptable IF the running phases produced
   "Eager-resolution" decisions in their plans; an empty intake when
   the verdicts show skipped findings is a RED signal. This prevents
   the practice from degrading into a no-op.

   The +2 reservation is in addition to whatever planned phases the
   milestone scopes; if the milestone has 8 planned phases, it
   actually has 10 (planned + 2 reservation). Roadmap entries for the
   reservation phases name them explicitly so they're not omitted by
   accident.

## Workspace layout

```
crates/
├── reposix-core/        # Shared types: Record, Project, RemoteSpec, Error, Tainted<T>.
├── reposix-sim/         # In-process axum HTTP simulator.
├── reposix-cache/       # On-disk bare-repo cache backed by gix; lazy blob materialization.
├── reposix-remote/      # git-remote-reposix binary (stateless-connect + export).
├── reposix-cli/         # Top-level `reposix` CLI (`init`, `sim`, `list`, `refresh`, `spaces`).
├── reposix-github/      # GitHub Issues BackendConnector.
├── reposix-confluence/  # Confluence Cloud BackendConnector.
├── reposix-jira/        # JIRA Cloud BackendConnector.
└── reposix-swarm/       # Multi-agent contention/swarm test harness.

.planning/               # GSD project state. Do not hand-edit; use /gsd-* commands.
docs/                    # User-facing docs (reference, benchmarks, demos, testing-targets).
research/                # Long-form research notes + red-team reports.
runtime/                 # gitignored — local sim DB, scratch working trees.
```

### `.planning/milestones/` convention (HANDOVER §0.5 / Option B)

Per-milestone planning artifacts live INSIDE the matching `*-phases/` dir,
never loose at `.planning/milestones/` top-level:

```
.planning/milestones/
├── v0.8.0-phases/
│   ├── ARCHIVE.md             # condensed milestone log (per POLISH2-21)
│   ├── ROADMAP.md             # milestone-level scoping (kept intact)
│   ├── REQUIREMENTS.md        # milestone-level scoping (kept intact)
│   └── tag-v0.8.0.sh          # historical one-shot release script
├── v0.9.0-phases/
│   ├── ROADMAP.md
│   └── tag-v0.9.0.sh
└── v0.10.0-phases/
    ├── ROADMAP.md
    └── tag-v0.10.0.sh
```

The `freshness/no-loose-roadmap-or-requirements` claim in
`scripts/end-state.py` enforces this — any `*ROADMAP*.md` or
`*REQUIREMENTS*.md` placed loose at `.planning/milestones/v*.0-*.md`
fails the verifier. New milestones must scaffold their planning docs
inside `*-phases/` from day one.

## Tech stack

- Rust stable (1.82+ via `rust-toolchain.toml`).
- Async: `tokio` 1.
- Web: `axum` 0.7 + `reqwest` 0.12 (rustls only, never openssl-sys).
- Git: `gix` 0.82 (pinned with `=` because gix is pre-1.0). **Runtime requirement: `git >= 2.34`** for `extensions.partialClone` + `stateless-connect`.
- Storage: `rusqlite` 0.32 with `bundled` feature (no system libsqlite3).
- Errors: `thiserror` for typed crate errors, `anyhow` only at binary boundaries.

## Commands you'll actually use

```bash
# Local dev loop
cargo check --workspace                                   # fast type check
cargo test --workspace                                    # unit + integration tests
cargo clippy --workspace --all-targets -- -D warnings     # CI lint
cargo fmt --all                                           # CI fmt

# Run the stack
cargo run -p reposix-sim                                  # start simulator on :7878
cargo run -p reposix-cli -- init sim::demo /tmp/repo      # bootstrap a partial-clone working tree
cd /tmp/repo && git checkout origin/main                  # agent UX from here is pure git
cat issues/0001.md && grep -ril TODO . && git push        # cat, grep, edit, push

# Dark-factory regression (proves agent UX is pure git, zero in-context learning)
bash scripts/dark-factory-test.sh sim                     # local + CI

# Testing against real backends — see docs/reference/testing-targets.md for env-var setup.
# Confluence — TokenWorld space (safe to mutate)
export ATLASSIAN_API_KEY=… ATLASSIAN_EMAIL=… REPOSIX_CONFLUENCE_TENANT=reuben-john
export REPOSIX_ALLOWED_ORIGINS='https://reuben-john.atlassian.net'
cargo test -p reposix-cli --test agent_flow_real -- --ignored dark_factory_real_confluence

# GitHub — reubenjohn/reposix issues (safe to mutate)
export GITHUB_TOKEN=…
export REPOSIX_ALLOWED_ORIGINS='https://api.github.com'
cargo test -p reposix-cli --test agent_flow_real -- --ignored dark_factory_real_github

# JIRA — default project key TEST (overridable via JIRA_TEST_PROJECT or REPOSIX_JIRA_PROJECT)
export JIRA_EMAIL=… JIRA_API_TOKEN=… REPOSIX_JIRA_INSTANCE=…
export JIRA_TEST_PROJECT=TEST
cargo test -p reposix-cli --test agent_flow_real -- --ignored dark_factory_real_jira
```

## GSD workflow

This project uses GSD (`get-shit-done`) for planning and execution. Workflow rule:

> **Always enter through a GSD command.** Never edit code outside a GSD-tracked phase or quick.

Entry points:

- `/gsd-quick` — small fix or doc tweak.
- `/gsd-execute-phase <n>` — run a planned phase end-to-end with subagents.
- `/gsd-debug` — investigate a bug.
- `/gsd-progress` — what's the state of the project right now.

The auto-mode bootstrap from 2026-04-13 set `mode: yolo`, `granularity: coarse`, and enabled all workflow gates (research / plan_check / verifier / nyquist / code_review). Do not silently downgrade these.

## Coding conventions

- `#![forbid(unsafe_code)]` in every crate.
- `#![warn(clippy::pedantic)]` in every crate. Allow-list specific lints with rationale; never blanket-allow `pedantic`.
- All public items documented; missing-doc lint is on for `reposix-core`.
- All `Result`-returning functions have a `# Errors` doc section.
- Tests live next to the code (`#[cfg(test)] mod tests`). Integration tests in `tests/`.
- Frontmatter uses `serde_yaml` 0.9 + Markdown body. Never JSON-on-disk for issues.
- Times are `chrono::DateTime<Utc>`. No `SystemTime` in serialized form.

## Build memory budget (load-bearing — read before parallelizing)

**The VM has crashed twice from RAM pressure caused by parallel cargo workspace builds.** The reposix workspace links large crates (`gix` chain, `rusqlite-bundled`, `reqwest+rustls`); a single `cargo check --workspace` peaks rustc + linker around 4–6 GB; `cargo test --workspace` is worse because rustc + N test binaries all link in parallel by default.

**Hard rules — both apply, no exceptions without explicit user override:**

1. **Never run more than one cargo invocation at a time.** This includes `cargo check`, `cargo build`, `cargo test`, `cargo clippy`. If two subagents both need to compile, they run sequentially, not in parallel. Coordinator-level safe rule: at most one phase-executor subagent doing cargo work at a time. Doc-only / planning-only subagents can still run in parallel with one cargo subagent.
2. **Prefer per-crate over workspace-wide:** `cargo check -p reposix-cli` instead of `cargo check --workspace` whenever the change is scoped. The pre-push hook covers the workspace-wide gate; you don't need to re-run it locally per commit.

**Soft rules:**

- `cargo build --jobs 2` (or `CARGO_BUILD_JOBS=2`) is safer than the default `-j$(nproc)` on this machine — set it in `.cargo/config.toml` if you find yourself fighting OOM during a single workspace build.
- `cargo nextest run` materializes test binaries one at a time vs `cargo test`'s parallel link; prefer it for full-workspace test runs.
- Doc-site work (`mkdocs build`, playwright) is cheap on RAM but runs Chromium — combined with cargo, it adds another ~500 MB. Schedule docs work in a no-cargo window.
- `rust-analyzer` in an editor pinned to this repo can use 2–3 GB on its own; close non-active editor instances during heavy build sessions.

**If the VM crashes again:** suspect parallel cargo, suspect rust-analyzer, suspect leftover background processes (`ps aux | grep -E "cargo|rustc"`). Don't blame the linker; blame the orchestration that let two of them run at once.

## Docs-site validation

Any change to `mkdocs.yml` or `docs/**` MUST pass `bash scripts/check-docs-site.sh` before commit (pre-push enforces). Mermaid SVG assertions: `scripts/check-mermaid-renders.sh` (also pre-push). For playwright walk rules and scoping (which pages to re-check after a change), see `/reposix-quality-doc-alignment` skill.

## Cold-reader pass on user-facing surfaces

Before declaring any user-facing surface shipped (hero copy, install instructions, headline numbers, README, docs landing page), dispatch `/doc-clarity-review` on the affected pages. For automated rubric grading, use `/reposix-quality-review` (`--rubric <id>` / `--all-stale`). The catalog at `quality/catalogs/subjective-rubrics.json` enforces 30-day freshness TTL.

## Freshness invariants

All invariants are enforced by `scripts/end-state.py` (pre-push hook). When a push is blocked, read the error — it names the violated invariant and the fix. The invariants: no version-pinned filenames, install path leads with package manager, benchmarks in mkdocs nav, no loose ROADMAP/REQUIREMENTS outside `*-phases/`, no orphan docs.

## Subagent delegation rules

Per the user's global OP #2: "Aggressive subagent delegation." Specifics for this project:

- `gsd-phase-researcher` for any "how do I implement X" question that would consume >100 lines of orchestrator context.
- `gsd-planner` for phase planning. Do not write `PLAN.md` by hand.
- `gsd-executor` for phase execution unless the work is trivially small.
- `gsd-code-reviewer` after every phase ships, before declaring done.
- Run multiple subagents in parallel whenever they're operating on disjoint files.
- **Never delegate `gh pr checkout` to a bash subagent without isolation.** Bash subagents share the coordinator's working tree; `gh pr checkout` switches the local branch behind the coordinator's back, which already caused the cherry-pick mess at commit `5a91ae2`. Either spawn a worktree first (`git worktree add /tmp/pr-N pr-N-branch`) and have the subagent `cd` into it, or have the subagent operate inside `/tmp/<branch>-checkout`. The coordinator's checkout is shared state — treat it that way.
- **Verifier subagent on every phase close** — see "Verifier subagent dispatch" in Quality Gates section + `quality/PROTOCOL.md` § "Verifier subagent prompt template".
- **Dispatching subjective rubrics (cold-reader, install-positioning, headline-numbers)** — `/reposix-quality-review` skill. Invocation: `bash .claude/skills/reposix-quality-review/dispatch.sh --rubric <id>` / `--all-stale` / `--force`. Path A (Task tool from Claude session) preferred for unbiased grading; Path B (claude -p subprocess) fallback.
- **Orchestration-shaped phases run at top-level, not under `/gsd-execute-phase`.** When a phase's work shape is "fan out → gather → interpret → resolve" rather than "write code → run tests → commit," the top-level coordinator IS the executor. `gsd-executor` lacks `Task` and depth-2 spawning is forbidden, so subagent fan-out cannot live inside it. Mark such phases `Execution mode: top-level` in ROADMAP and provide a research brief the orchestrator follows verbatim. Docs-alignment backfill is the canonical example; retroactive audits follow the same pattern. Refresh runs on stale docs (`/reposix-quality-refresh <doc>`) are also top-level — pre-push that BLOCKS mid-`gsd-execute-phase` must be resolved by checkpointing the executor and invoking the slash command from a fresh top-level session.

The orchestrator's job is to route, decide, and integrate — not to type code that a subagent could type.

### Meta-rule: when an owner catches a quality miss, fix it twice

When the owner catches a quality issue the agent missed, the fix is
two-fold: (1) fix the issue in the code/docs, and (2) update CLAUDE.md
(and/or the §0.8 SESSION-END-STATE framework) so the next agent's
session reads the tightened rule. Just shipping the fix without
updating the instructions guarantees recurrence.

## Threat model

This project is a textbook lethal-trifecta machine:

| Leg of trifecta | Where it shows up here |
| --- | --- |
| Private data | The partial-clone working tree exposes issue bodies, internal field values, attachments. |
| Untrusted input | Every issue body / comment / title is attacker-influenced text. |
| Exfiltration | `git push` can target arbitrary remotes; the helper + cache make outbound HTTP. |

Cuts that are mandatory and tested:

- **Outbound HTTP allowlist.** The remote helper (`git-remote-reposix`) and the cache materializer (`reposix-cache`) refuse to talk to any origin not in `REPOSIX_ALLOWED_ORIGINS` (env var, defaults to `http://127.0.0.1:*` only). All HTTP construction goes through the single `reposix_core::http::client()` factory; clippy's `disallowed_methods` catches direct `reqwest::Client::new()` call sites.
- **No shell escape from `export` / cache writes.** Bytes-in-bytes-out; no rendering, no template expansion. The `Tainted<T>` → `Untainted<T>` conversion is the explicit `sanitize` step where escaping happens.
- **Frontmatter field allowlist.** Server-controlled fields (`id`, `created_at`, `version`, `updated_at`) cannot be overridden by client writes; they are stripped on the inbound `export` path before the REST call.
- **Audit log is append-only — and dual-schema by design.** SQLite WAL, no UPDATE/DELETE on either audit table. `audit_events_cache` (in `reposix-cache::audit`) records every blob materialization, helper fetch turn, helper push (accept and reject), gc eviction, and sync-tag write. `audit_events` (in `reposix-core::audit`, written by the sim/confluence/jira adapters) records every backend-side mutating REST call. Forensic queries that need both layers (e.g., "which JIRA write came from which `git push`?") read both tables. See OP-3 above for why the split is intentional.

See `research/threat-model-and-critique.md` (produced by red-team subagent) for the full analysis.

## What to do when context fills

If you (the agent) notice this CLAUDE.md getting hard to keep in working memory:

1. Read `.planning/STATE.md` first — it's the entry point.
2. Read the most recent `.planning/phases/*/PLAN.md`.
3. Skim `git log --oneline -20` to know what's recently shipped.
4. Don't read this file linearly; grep for the section you need.

## Quality Gates — dimension/cadence/kind taxonomy

The Quality Gates framework lives at `quality/`. Runtime contract: `quality/PROTOCOL.md`. Catalog schema: `quality/catalogs/README.md`. Pivot journal: `quality/SURPRISES.md`. Historical build narrative: `.planning/milestones/v0.12.0-phases/archive/` (per-phase files).

Working on a quality-gates task? Read `quality/PROTOCOL.md` first.

**9 dimensions** — the regression classes the project has:

| Dimension | Checks | Home |
|---|---|---|
| code | clippy, fmt, cargo nextest | `quality/gates/code/` |
| docs-alignment | claims have tests; hash drift detection | `quality/gates/docs-alignment/` |
| docs-build | mkdocs strict, mermaid renders, link resolve, badges resolve | `quality/gates/docs-build/` |
| docs-repro | snippet extract, container rehearse, tutorial replay | `quality/gates/docs-repro/` |
| release | gh assets present, brew formula current, crates.io max version, installer bytes | `quality/gates/release/` |
| structure | freshness invariants, banned words, top-level scope | `quality/gates/structure/` |
| agent-ux | dark-factory regression | `quality/gates/agent-ux/` |
| perf | latency, token economy | `quality/gates/perf/` |
| security | allowlist enforcement, audit immutability | `quality/gates/security/` |

**6 cadences** — when the gate runs:

`pre-push` (local, every push, <60s, blocking) · `pre-pr` (PR CI, <10min, blocking) · `weekly` (cron, alerting) · `pre-release` (on tag, <15min, blocking) · `post-release` (after assets ship, alerting) · `on-demand` (manual / subagent).

**5 kinds** — how the gate is verified:

`mechanical` (deterministic shell + asserts) · `container` (fresh ubuntu/alpine + post-conditions) · `asset-exists` (HEAD/GET URL + min-bytes) · `subagent-graded` (rubric-driven subagent) · `manual` (human-only with TTL freshness).

**Adding a new gate** is one catalog row + one verifier in the right dimension dir. The runner discovers + composes by tag. No new top-level script, no new pre-push wiring.

**Catalog-first rule.** Every phase's FIRST commit writes the catalog rows that define this phase's GREEN contract; subsequent commits cite the row id. The verifier subagent reads catalog rows that exist BEFORE the implementation lands.

**Verifier subagent dispatch.** Phase close MUST dispatch an unbiased subagent that grades catalog rows from artifacts with zero session context. The executing agent does NOT grade itself — see `quality/PROTOCOL.md` § "Verifier subagent prompt template" for the verbatim prompt.

**CLAUDE.md stays current.** Each phase introducing a new file/convention/gate updates CLAUDE.md in the same PR. The update means *revising existing sections* to reflect the new state — not appending a narrative. Ask: "if an agent opens this file cold, what do they need to know?" and edit the document to say that, wherever it naturally fits.

**Meta-rule extension** (when an owner catches a quality miss): fix the issue, update CLAUDE.md, AND **tag the dimension**. The dimension tag routes to the right catalog file + `quality/gates/<dim>/` verifier directory.

### Docs-alignment dimension

Binary: `reposix-quality doc-alignment {bind, propose-retire, confirm-retire, mark-missing-test, plan-refresh, plan-backfill, merge-shards, walk, status}`. Run `status --top 10` for gap targeting. Pre-push gate: `quality/gates/docs-alignment/walk.sh`. Full spec: `quality/catalogs/README.md` § "docs-alignment dimension".

Two axes: `alignment_ratio` (bound / non-retired) and `coverage_ratio` (lines_covered / total_eligible). Walker BLOCKs when either drops below floor; recovery is `/reposix-quality-backfill`.

**Slash commands** (top-level only — unreachable from inside `gsd-executor`):
- `/reposix-quality-refresh <doc>` — refresh stale rows for one doc.
- `/reposix-quality-backfill` — full extraction across docs/ + archived REQUIREMENTS.md.

## v0.12.1 — in flight

### P72 — Lint-config invariants

Verifier home: `quality/gates/code/lint-invariants/` (sub-area README at `quality/gates/code/lint-invariants/README.md`). 8 shell verifiers bind 9 `MISSING_TEST` rows in `quality/catalogs/doc-alignment.json` covering README + `docs/development/contributing.md` workspace-level invariants.

| Verifier                    | Catalog row(s)                                                                                          |
| --------------------------- | ------------------------------------------------------------------------------------------------------- |
| `forbid-unsafe-code.sh`     | `README-md/forbid-unsafe-code` + `docs-development-contributing-md/forbid-unsafe-per-crate` (D-01) |
| `rust-msrv.sh`              | `README-md/rust-1-82-requirement`                                                                       |
| `tests-green.sh`            | `README-md/tests-green` (compile-only, D-05)                                                            |
| `errors-doc-section.sh`     | `docs-development-contributing-md/errors-doc-section-required` (clippy lint, D-07)                      |
| `rust-stable-channel.sh`    | `docs-development-contributing-md/rust-stable-no-nightly`                                               |
| `cargo-check-workspace.sh`  | `docs-development-contributing-md/cargo-check-workspace-available`                                      |
| `cargo-test-count.sh`       | `docs-development-contributing-md/cargo-test-133-tests` (>= 368 floor, re-measured P72 per D-06)        |
| `demo-script-exists.sh`     | `docs-development-contributing-md/demo-script-exists`                                                   |

Prose updated: `docs/development/contributing.md:20` re-measured to `>= 368 test binaries` BEFORE the bind (D-06 audit trail). Verifiers minted via `bind` per `quality/PROTOCOL.md` § "Subagents propose; tools validate and mint" (Principle A). Cargo invocations serialized by the runner per CLAUDE.md "Build memory budget" (D-04).

### P73 — Connector contract gaps

Closed 4 `MISSING_TEST` rows asserting connector authoring + JIRA-shape contracts. Two new wiremock-based Rust tests live next to existing per-crate contract tests:

- `crates/reposix-confluence/tests/auth_header.rs::auth_header_basic_byte_exact`
- `crates/reposix-github/tests/auth_header.rs::auth_header_bearer_byte_exact`
- `crates/reposix-jira/tests/list_records_excludes_attachments.rs::list_records_excludes_attachments_and_comments`

The auth-header tests use `wiremock::matchers::header(K, V)` (returns `HeaderExactMatcher`, NOT `HeaderRegexMatcher`) for byte-exact assertion — the canonical idiom for any future connector contract test of this kind. Plan-time prose cited `header_exact` as the function name; the actual public API in wiremock 0.6.5 is `header(K, V)` (same byte-exact semantics). The JIRA test asserts at the **rendering boundary** (Record.body markdown + Record.extensions allowlist), not at the JSON parse layer — that's where the deferral in `docs/decisions/005-jira-issue-mapping.md:79-87` is observable to a downstream consumer.

The `real-backend-smoke-fixture` row was a pure rebind to the existing `crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_confluence` `#[ignore]` smoke (TokenWorld is sanctioned for free mutation per `docs/reference/testing-targets.md`).

The stale `docs/benchmarks/token-economy.md:23-28` JIRA row was resolved via path (a) per D-05: prose updated to acknowledge the adapter shipped v0.11.x, plus a 5-line shell verifier at `quality/gates/docs-alignment/jira-adapter-shipped.sh` asserting the manifest exists. Bench-number re-measurement remains deferred to perf-dim P67.

No SURPRISES-INTAKE / GOOD-TO-HAVES entries appended (no out-of-scope items observed during execution; no auth-header bug surfaced in either backend; OP-8 honesty check intentional).

See: `quality/PROTOCOL.md` § "Principle A"; CLAUDE.md "Build memory budget" (D-09 sequential per-crate cargo).

## Quick links

- `docs/research/initial-report.md` — full architectural argument for git-remote-helper + partial clone.
- `docs/research/agentic-engineering-reference.md` — dark-factory pattern, lethal trifecta, simulator-first.
- `docs/reference/testing-targets.md` — sanctioned real-backend test targets (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`).
- `docs/benchmarks/latency.md` — golden-path latency envelope per backend.
- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` — ratified design doc for the v0.9.0 pivot.
- `.planning/PROJECT.md` — current scope.
- `.planning/STATE.md` — current cursor.
