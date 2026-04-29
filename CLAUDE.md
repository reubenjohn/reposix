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
3. **Audit log is non-optional, and lives in TWO append-only tables.** `audit_events_cache` (cache-internal events — blob materialization, gc, helper RPC fetch/push, sync-tag writes) lives in the cache crate (`reposix-cache::audit`); `audit_events` (backend mutations — `create_record` / `update_record` / `delete_or_close`) lives in the core crate (`reposix-core::audit`) and is written by the sim/confluence/jira adapters. A complete forensic query reads both. Either schema missing a row for a network-touching action means the feature isn't done. The dual-table shape is intentional for the v0.11.x line (POLISH2-22 friction row 12); physical unification behind a `dyn AuditSink` trait is deferred to v0.12.0+ (CC-3).
4. **No hidden state.** Cache state, simulator state, and git remote helper state all live in committed-or-fixture artifacts. No "it works in my session" bugs.
5. **Working tree IS a real git checkout.** The whole point of v0.9.0 is that `.git/` is real, not synthetic; `git diff` is the change set by construction, not by emulation. The partial clone (`extensions.partialClone=origin`) makes blobs lazy, but everything else is upstream git.
6. **Real backends are first-class test targets.** Three canonical targets are sanctioned for aggressive testing: **Confluence space "TokenWorld"** (owned by the user; safe to mutate freely), **GitHub repo `reubenjohn/reposix` issues** (ours; safe to create/close issues during tests), and **JIRA project `TEST`** (default key; overridable via `JIRA_TEST_PROJECT` or `REPOSIX_JIRA_PROJECT`). See `docs/reference/testing-targets.md` for env-var setup, owner permission statement, and cleanup procedure. Simulator remains the default (OP-1), but "simulator-only coverage" does NOT satisfy acceptance for transport-layer or performance claims.
7. **Phase-close means catalog-row PASS.** v0.12.0 introduces the
   "verifier subagent grades GREEN" gate per phase close (QG-06).
   No phase ships on the executing agent's word; an unbiased subagent
   reads the catalog rows + the Wave-N verification artifacts with
   zero session context and writes a verdict to
   `quality/reports/verdicts/p<N>/<ts>.md` (or `.planning/verifications/p<N>/VERDICT.md`
   until P57 lands the `quality/reports/` tree). **The verdict is the
   contract** — if RED, the phase loops back to fix the failing rows
   rather than negotiating the catalog down. Meta-rule extension: when
   a release-pipeline regression is fixed, the same PR ships
   container-rehearsal evidence under `.planning/verifications/p56/`
   (or `quality/reports/verifications/release/` after P58).
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
   new dependency introduced, (c) no new file created outside the
   phase's planned set. The +2 reservation is for items that
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

**Any change to `mkdocs.yml`, `docs/**`, or any markdown file inside the
docs tree** MUST be validated before commit by running:

```bash
bash scripts/check-docs-site.sh
```

The script runs `mkdocs build --strict` and fails on any "Syntax error
in text" string in the rendered HTML output (mermaid + admonition +
cross-ref bugs). The pre-push hook enforces it.

**Whole-nav-section playwright walks (not just the file you changed).**
Editing one mermaid block on one page does NOT scope your validation to
that page. The §0.1 mermaid-render misses recurred because validation
was scoped per-file. Rules:

- Touched **one page** in a docs nav section → playwright-walk every
  page in that section.
- Touched `mkdocs.yml`, any `pymdownx.*` config, any file under
  `docs/javascripts/` or `docs/stylesheets/`, or any frontmatter →
  walk the entire site.
- All walks MUST run with the cache disabled (fresh first-load
  semantics). The §0.1.b race condition only manifests on a cold
  page-cache; warm reloads mask it. Use
  `mcp__playwright__browser_navigate` on a clean context, or hard
  reload via `location.reload(true)` before assertions.
- For every page hit by a walk, assert `document.querySelectorAll('pre.mermaid svg').length > 0`
  for any mermaid block, AND `browser_console_messages` contains zero
  rendering errors. Capture artifacts under
  `.planning/verifications/playwright/<section>/<slug>.json`.

`scripts/check-mermaid-renders.sh` (wired into pre-push) automates the
mermaid SVG assertion across every rendered page; do not rely on
`mkdocs build --strict` alone, it never opens a browser.

## Cold-reader pass on user-facing surfaces

Before declaring any user-facing surface shipped — hero copy, install
instructions, headline numbers, benchmark pages, the README, the docs
landing page — dispatch the `doc-clarity-review` skill on the affected
pages with isolated context. Mechanical hooks miss positioning and
freshness misses (the §0.2 install-path lag, the §0.3 version-pinned
filename, the §0.4 missing-from-nav benchmark all slipped past
strict-build + clippy because none of those tools simulate a
cold reader).

The cold-reader pass is the second half of the close-the-loop principle
(global OP #1) for prose: render → reload-as-stranger → ask "does this
land?" before claiming done.

After P61 ships, the cold-reader pass is operationalized via the
`reposix-quality-review` skill at `.claude/skills/reposix-quality-review/`.
Invoke `bash .claude/skills/reposix-quality-review/dispatch.sh --rubric
subjective/cold-reader-hero-clarity` (or `--all-stale` to grade every
stale subjective rubric in parallel) to grade README.md + docs/index.md
hero clarity automatically. The catalog row at
`quality/catalogs/subjective-rubrics.json` enforces "rubric was checked
within 30d" via the freshness-TTL gate (P61 SUBJ-03).

## Freshness invariants

Drift these and the docs lie. Each invariant has a verifier; the §0.8
SESSION-END-STATE framework (`scripts/end-state.py`) is the
machine-readable source of truth and the pre-push hook gates them.

- **No version-pinned filenames** outside `CHANGELOG.md` and
  `.planning/milestones/v*-phases/`. `find docs scripts -type f | grep
  -E 'v[0-9]+\.[0-9]+\.[0-9]+'` returns empty.
- **Install path leads with package manager** the moment a publish
  surface (crates.io, brew tap, binstall) is live for a given crate
  version. Hero on `docs/index.md` and `README.md` MUST show
  `cargo binstall` / `brew install` BEFORE any `git clone … && cargo
  build` snippet.
- **Benchmarks belong in mkdocs nav.** Anything under `benchmarks/` or
  `docs/benchmarks/` MUST appear in `mkdocs.yml` `nav:`. No
  benchmark linked via absolute github URL bypassing the site.
- **GSD planning org is regular.** No loose `*ROADMAP*.md` or
  `*REQUIREMENTS*.md` outside a `*phases/` dir or
  `.planning/archive/`. `find .planning/milestones -maxdepth 2 -name
  '*ROADMAP*' -o -name '*REQUIREMENTS*' | grep -v phases | grep -v
  archive` returns empty.
- **No orphan docs.** Every `docs/**/*.md` either appears in
  `mkdocs.yml` `nav:` OR is explicitly redirect-only (entry in
  `mkdocs.yml` `plugins.redirects.redirect_maps`).

## Subagent delegation rules

Per the user's global OP #2: "Aggressive subagent delegation." Specifics for this project:

- `gsd-phase-researcher` for any "how do I implement X" question that would consume >100 lines of orchestrator context.
- `gsd-planner` for phase planning. Do not write `PLAN.md` by hand.
- `gsd-executor` for phase execution unless the work is trivially small.
- `gsd-code-reviewer` after every phase ships, before declaring done.
- Run multiple subagents in parallel whenever they're operating on disjoint files.
- **Never delegate `gh pr checkout` to a bash subagent without isolation.** Bash subagents share the coordinator's working tree; `gh pr checkout` switches the local branch behind the coordinator's back, which already caused the cherry-pick mess at commit `5a91ae2`. Either spawn a worktree first (`git worktree add /tmp/pr-N pr-N-branch`) and have the subagent `cd` into it, or have the subagent operate inside `/tmp/<branch>-checkout`. The coordinator's checkout is shared state — treat it that way.
- **QG-06 verifier subagent dispatch on every phase close** — see `quality/PROTOCOL.md` § "Verifier subagent prompt template" for the verbatim copy-paste prompt. The executing agent does NOT grade itself.
- **Dispatching subjective rubrics (cold-reader, install-positioning, headline-numbers)** — `reposix-quality-review` skill at `.claude/skills/reposix-quality-review/` (P61 SUBJ-02). Invocation: `bash .claude/skills/reposix-quality-review/dispatch.sh --rubric <id>` / `--all-stale` / `--force`. Path A (Task tool from Claude session) preferred for unbiased grading; Path B (claude -p subprocess) fallback. The cold-reader rubric integrates the existing global `doc-clarity-review` skill via subprocess.
- **Orchestration-shaped phases run at top-level, not under `/gsd-execute-phase`** (added 2026-04-28 for P65). When a phase's work shape is "fan out → gather → interpret → resolve" rather than "write code → run tests → commit," the top-level coordinator IS the executor. `gsd-executor` lacks `Task` and depth-2 spawning is forbidden, so subagent fan-out cannot live inside it. Mark such phases `Execution mode: top-level` in ROADMAP and provide a research brief the orchestrator follows verbatim. P65 (docs-alignment backfill) is the canonical example; future retroactive audits follow the same pattern. Refresh runs on stale docs (`/reposix-quality-refresh <doc>`) are also top-level — pre-push that BLOCKS mid-`gsd-execute-phase` must be resolved by checkpointing the executor and invoking the slash command from a fresh top-level session.

The orchestrator's job is to route, decide, and integrate — not to type code that a subagent could type.

### Meta-rule: when an owner catches a quality miss, fix it twice

When the owner catches a quality issue the agent missed, the fix is
two-fold: (1) fix the issue in the code/docs, and (2) update CLAUDE.md
(and/or the §0.8 SESSION-END-STATE framework) so the next agent's
session reads the tightened rule. Just shipping the fix without
updating the instructions guarantees recurrence. The §0.1-§0.5 misses
all happened because earlier sessions did (1) without (2).

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

## v0.12.0 Quality Gates — milestone shipped 2026-04-27

Framework lives at `quality/{gates,catalogs,reports,runners}/`. Runtime contract: `quality/PROTOCOL.md`. Catalog schema: `quality/catalogs/README.md`. Pivot journal: `quality/SURPRISES.md`.

Per-phase contribution log (P56–P63), container-rehearsal evidence schema, and v0.12.1 carry-forwards archived at `.planning/milestones/v0.12.0-phases/ARCHIVE.md` per the `*-phases/ARCHIVE.md` convention.

Working on a quality-gates task? Read first:
- `quality/PROTOCOL.md` — runtime contract.
- `.planning/research/v0.12.0-{vision-and-mental-model, naming-and-architecture, roadmap-and-rationale, autonomous-execution-protocol}.md` — design rationale.
- `.planning/milestones/v0.12.0-phases/ARCHIVE.md` — historical phase contributions.

### P64 — Docs-alignment dimension framework + skill (added 2026-04-28)

New 9th dimension: **docs-alignment** (claims have tests; hash drift detection). The dimension exists because v0.9.0's git-native pivot silently dropped Confluence page-tree-symlink behavior — no test failed because no test asserted the user-facing surface. Tests were derived from the implementation, not from prose claims.

**v0.12.1 schema bump (P71/W7):** `tests` and `test_body_hashes` are now parallel `Vec<String>` arrays — one row may bind to multiple tests (e.g. JIRA-writes binding to create + update + delete + recovery tests under one claim). Catalog `schema_version` bumped from `"1.0"` to `"2.0"` (commit `d2127c3`); migration script `scripts/migrate-doc-alignment-schema-w7.py`. Parallel-array invariant `tests.len() == test_body_hashes.len()` enforced via `Row::set_tests`. See `quality/catalogs/README.md` § "docs-alignment dimension" for the full row spec.

Wave 1 (catalog-first, `d0d4730`): empty-state catalog `quality/catalogs/doc-alignment.json` (schema_version 1.0; summary block; floor 0.50; rows []), 3 structure-dim freshness rows guarding catalog presence + summary-block-valid + floor-monotonicity, 5 skill files at `.claude/skills/reposix-quality-doc-alignment/` + 2 thin slash-command skills at `.claude/skills/reposix-quality-{refresh,backfill}/` (preflight committed alongside the contract).

Wave 2 (binary surface, `98dcf11` + `86036c5`): NEW workspace crate `crates/reposix-quality/` — self-contained, `#![forbid(unsafe_code)]` + `#![warn(clippy::pedantic)]`, two `[[bin]]` targets (`reposix-quality` umbrella + `hash_test_fn` standalone). Full clap surface: `doc-alignment {bind, propose-retire, confirm-retire, mark-missing-test, plan-refresh, plan-backfill, merge-shards, walk, status}` + generic `run --gate/--cadence` + `verify --row-id` + `walk` alias. `syn`-based hash binary at `quality/gates/docs-alignment/hash_test_fn` hashes Rust function bodies as `to_token_stream()` sha256 (comments + whitespace normalize away). 28 tests (10 unit + 18 integration golden).

Wave 3 (this commit): runner integration via new `docs-alignment/walk` gate row at cadence=pre-push (P0) shelling to `quality/gates/docs-alignment/walk.sh` (release-first, debug-fallback). Two project-wide principles in `quality/PROTOCOL.md`: Principle A (subagents propose with citations; tools validate and mint) and Principle B (tools fail loud, structured, agent-resolvable) with cross-tool examples. Verifier verdict at `quality/reports/verdicts/p64/VERDICT.md` (Path B in-session per P56–P63 precedent).

Slash commands `/reposix-quality-refresh <doc>` and `/reposix-quality-backfill` are top-level only (depth-2 unreachable from inside `gsd-executor`). P65 runs the first backfill; the dimension goes from empty-state to populated then.

### P65 — Docs-alignment backfill, smoking-gun surfaced (added 2026-04-28)

First backfill ran top-level (orchestrator IS executor — `gsd-executor` lacks `Task`). 24-shard `MANIFEST.json` from `plan-backfill` (`docs/**/*.md` + `README.md` + archived `REQUIREMENTS.md` v0.6.0–v0.11.0; ≤3 files/shard, directory-affinity). Three waves of 8 Haiku extractor subagents (~9 min wall-clock); zero `merge-shards` conflicts.

**Final catalog** (commit `a263868`): 388 rows total — 181 `BOUND`, 166 `MISSING_TEST`, 41 `RETIRE_PROPOSED`. `alignment_ratio` 0.466. `summary.floor_waiver` (until 2026-07-31) + `freshness-invariants.json::docs-alignment/walk` row waiver (matched TTL) keep pre-push green while v0.12.1 closes the punch list. The 3 P64 catalog-integrity rows continue to PASS at pre-push and assert the catalog itself stays well-formed.

**Smoking gun confirmed.** `docs/reference/confluence.md` still describes the FUSE mount + page-tree symlink shape removed in v0.9.0; 3 MISSING_TEST rows captured at lines 110-128 (`fuse_mount_symlink_tree`, `fuse_daemon_role`, outdated `v0.4 write path` claim). Closing this cluster is v0.12.1 P71 work — *not* P65 scope.

**Other clusters** (full breakdown in `quality/reports/doc-alignment/backfill-20260428T085523Z/PUNCH-LIST.md`): JIRA shape (`jira.md` Phase 28 RETIRE_PROPOSED — Phase 29 added writes); benchmark headline numbers (20 MISSING_TEST — perf verifiers exist as shell scripts, not Rust tests; MIGRATE-03 carry-forward); connector authoring guide (24 MISSING_TEST — contract assertions in code without named test fns); glossary over-extraction (24 RETIRE_PROPOSED — bulk-confirm review, not 24 tickets); social copy "92%" headline vs measured 89.1% (drift). v0.12.1 P71-P80 stubs in PUNCH-LIST suggest one phase per cluster.

**Subagent surgery** (3 of 24 shards needed orchestrator intervention; documented in SURPRISES.md): shard 016 wrote to live catalog instead of its shard catalog (recovered via jq move + reset); shards 012 + 023 first attempts violated the binary contract (custom JSON schema or incomplete row); both re-dispatched with "MUST USE BINARY" emphasis.

**Schema cross-cuts surfaced** (filed v0.12.1 carry-forward): `bind` writes `Row.source` as `SourceCite` object but `merge-shards` deserializer expected `Source` enum (object-or-array); reconciled via jq transform in orchestrator before merge. Same for `Row.test`: some shards emitted multi-test arrays, binary expects single string. Both inconsistencies are MIGRATE-03 (i) — unify the schema in v0.12.1.

P65 verifier verdict at `quality/reports/verdicts/p65/VERDICT.md` (Path A — top-level orchestrator HAS `Task`). Milestone-close verifier at `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md`. Owner pushes the v0.12.0 tag.

## v0.12.1 — in flight

### P66 — coverage_ratio metric (added 2026-04-28)

The docs-alignment dimension grows a SECOND axis: `coverage_ratio = lines_covered / total_eligible_lines`. The first axis (`alignment_ratio = bound / non-retired`) answers "of the claims we extracted, how many bind to passing tests?"; coverage answers "of the prose we said we'd mine, what fraction did we actually cover?". Together they yield the agent's mental model:

|                  | high alignment       | low alignment        |
|------------------|----------------------|----------------------|
| **high coverage**| ideal                | extracted; unbound   |
| **low coverage** | tested what we found | haven't started      |

Without coverage, an agent could ship high alignment by extracting only easy claims. The new per-file table — `reposix-quality doc-alignment status --top 10` — is the agent's gap-target view: worst-covered files surface first; rows with `row_count == 0 && total_lines > 50` get a "ZERO ROWS" hint as the most actionable miss.

`coverage_floor` ships at 0.10 (low; even sparse mining usually clears it; baseline measured 0.2055 on the 388-row corpus). Ratcheted up by deliberate human commits as v0.12.1 cluster phases (P72+) widen extraction. The walker NEVER auto-tunes the floor. Walker BLOCKs (exit non-zero) when `coverage_ratio < coverage_floor`; recovery move is `/reposix-quality-backfill` to widen extraction OR a deliberate floor-down commit if extraction is genuinely complete.

## Quality Gates — dimension/cadence/kind taxonomy

The v0.12.0 Quality Gates framework lives at `quality/`. Runtime contract:
`quality/PROTOCOL.md`. Every quality check (gate) sits at one
`(dimension, cadence, kind)` coordinate.

**9 dimensions** — the regression classes the project has:

| Dimension | Checks | Home |
|---|---|---|
| code | clippy, fmt, cargo nextest | `quality/gates/code/` (P58) |
| docs-alignment | claims have tests; hash drift detection | `quality/gates/docs-alignment/` (P64 — shipped) |
| docs-build | mkdocs strict, mermaid renders, link resolve, badges resolve | `quality/gates/docs-build/` (P60) |
| docs-repro | snippet extract, container rehearse, tutorial replay | `quality/gates/docs-repro/` (P59) |
| release | gh assets present, brew formula current, crates.io max version, installer bytes | `quality/gates/release/` (P58) |
| structure | freshness invariants, banned words, top-level scope (QG-08) | `quality/gates/structure/` (P57 — shipped) |
| agent-ux | dark-factory regression | `quality/gates/agent-ux/` (P59) |
| perf | latency, token economy | `quality/gates/perf/` (P59 file-relocate; v0.12.1 stub→real) |
| security | allowlist enforcement, audit immutability | `quality/gates/security/` (v0.12.1 carry-forward) |

**6 cadences** — when the gate runs:

`pre-push` (local, every push, <60s, blocking) · `pre-pr` (PR CI, <10min, blocking) · `weekly` (cron, alerting) · `pre-release` (on tag, <15min, blocking) · `post-release` (after assets ship, alerting) · `on-demand` (manual / subagent).

**5 kinds** — how the gate is verified:

`mechanical` (deterministic shell + asserts) · `container` (fresh ubuntu/alpine + post-conditions) · `asset-exists` (HEAD/GET URL + min-bytes) · `subagent-graded` (rubric-driven subagent) · `manual` (human-only with TTL freshness).

**Adding a new gate** is one catalog row + one verifier in the right dimension dir. The runner discovers + composes by tag. No new top-level script, no new pre-push wiring.

**Catalog-first rule.** Every phase's FIRST commit writes the catalog rows that define this phase's GREEN contract; subsequent commits cite the row id. The verifier subagent reads catalog rows that exist BEFORE the implementation lands.

**Verifier subagent dispatch (QG-06).** Phase close MUST dispatch an unbiased subagent that grades catalog rows from artifacts with zero session context. The executing agent does NOT get to talk the verifier out of RED.

**Mandatory CLAUDE.md update per phase (QG-07).** Each phase introducing a new file/convention/gate updates CLAUDE.md in the same PR. **At milestone close, per-phase contribution sections archive to `.planning/milestones/v<X.Y.Z>-phases/ARCHIVE.md`** so CLAUDE.md stays under the progressive-disclosure size budget (~40 KB; enforced by `~/.git-hooks/pre-commit`).

**Meta-rule extension** (when an owner catches a quality miss): fix the issue, update CLAUDE.md, AND **tag the dimension**. The dimension tag tells the next agent which catalog file to add the row to + which `quality/gates/<dim>/` directory the verifier belongs in. This makes ad-hoc fixes structural.

**Cross-references** (do NOT duplicate runtime detail here):

- `quality/PROTOCOL.md` — autonomous-mode runtime contract.
- `quality/catalogs/README.md` — unified catalog schema spec.
- `quality/SURPRISES.md` — append-only pivot journal.
- `.planning/research/v0.12.0-naming-and-architecture.md` — design rationale.
- `.planning/research/v0.12.0-vision-and-mental-model.md` — why this exists.

**Per-dimension `owner_hint`** when an owner catches a miss: see `quality/PROTOCOL.md` § "Failure modes the protocol protects against" for the routing rule. The catalog row is the contract; the verifier is the code; the artifact is the audit; the verdict is the grade.

## Quick links

- `docs/research/initial-report.md` — full architectural argument for git-remote-helper + partial clone.
- `docs/research/agentic-engineering-reference.md` — dark-factory pattern, lethal trifecta, simulator-first.
- `docs/reference/testing-targets.md` — sanctioned real-backend test targets (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`).
- `docs/benchmarks/latency.md` — golden-path latency envelope per backend.
- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` — ratified design doc for the v0.9.0 pivot.
- `.planning/PROJECT.md` — current scope.
- `.planning/STATE.md` — current cursor.
