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

## v0.12.0 Quality Gates — phase log

The v0.12.0 milestone migrates ad-hoc `scripts/check-*.sh` and the
conflated `scripts/end-state.py` to a dimension-tagged Quality
Gates system at `quality/{gates,catalogs,reports,runners}/`. **The
framework itself ships in P57** — until then, the catalog format
lives in `.planning/docs_reproducible_catalog.json` (ACTIVE-V0; P57
migrates it to the unified schema). Read
`.planning/research/v0.12.0-{vision-and-mental-model,
naming-and-architecture, roadmap-and-rationale,
autonomous-execution-protocol}.md` before working on any v0.12.0
phase. Read `quality/SURPRISES.md` (append-only pivot journal — seeded
in P56) at the start of every phase to avoid repeating dead ends.
Future phases follow `quality/PROTOCOL.md` (lands P57 — autonomous-mode
runtime contract).

### P56 contribution — release pipeline + install-evidence pattern

`.github/workflows/release.yml` now fires on **both** `v*` AND
`reposix-cli-v*` tag globs (Option A from
`.planning/research/v0.12.0-install-regression-diagnosis.md`),
because release-plz cuts per-crate tags. Archive filenames use the
`v${version}` form (e.g. `reposix-v0.11.3-x86_64-unknown-linux-musl.tar.gz`)
regardless of which tag triggered the workflow; the GH Release object
still uses the actual tag name (e.g. `reposix-cli-v0.11.3`) so
release-plz's existing release object is the one that gets assets
attached. **If you edit release.yml,** preserve the
`release_tag`/`version`/`version_tag` distinction in the `plan` job's
outputs — collapsing them re-introduces the regression. The
install-path contract lives in
`.planning/docs_reproducible_catalog.json` until P58 splits it into
`quality/catalogs/install-paths.json`.

P56 shipped these new artifacts (referenced by future phases):

- `scripts/p56-rehearse-curl-install.sh` — ubuntu:24.04 container rehearsal for the curl one-liner.
- `scripts/p56-rehearse-cargo-binstall.sh` — rust:1.82-slim container rehearsal for `cargo binstall`.
- `scripts/p56-asset-existence.sh` — generic HEAD/range asset reachability check.
- `scripts/p56-validate-install-evidence.py` — install-evidence JSON validator (Wave 4 verifier subagent re-runs this).
- `.planning/verifications/p56/install-paths/<id>.json` — per-row evidence files.
- `.planning/docs_reproducible_catalog.json` — ACTIVE-V0 catalog seeded with 5 install rows + 6 docs/tutorial/benchmark rows.

### Container-rehearsal evidence schema (P56 → P58 forward path)

Every install-path verifier writes JSON to
`.planning/verifications/p56/install-paths/<id>.json` with the shape:

```jsonc
{
  "claim_id": "install/<slug>",
  "phase": "p56",
  "verifier_kind": "asset-existence | container-rehearsal | shell-against-checkout",
  "verified_at": "<RFC3339-UTC>",
  "container_image": "<image:tag>",     // present when verifier_kind contains container-rehearsal
  "container_image_digest": "<sha256:…>",// optional but recommended
  "verifier_script": "scripts/<committed-script>.sh",
  "asserts": { /* per-row assertion booleans + observed values */ },
  "evidence": { /* container_log_path, log excerpts, exit codes, observed bytes */ },
  "status": "PASS | FAIL | PARTIAL | NOT-VERIFIED"
}
```

Future-shape note: P58 ports this schema to `quality/reports/verifications/release/`
under the unified Quality Gates layout. New install-path or release-pipeline
verifiers should write the same shape there from day one — same `claim_id`,
same `asserts`/`evidence` discipline.

### Meta-rule (extension of OP-1 close-the-loop)

When a release-pipeline regression is fixed, the same PR ships
container-rehearsal evidence under `.planning/verifications/p56/`
(or `quality/reports/verifications/release/` after P58). "Fix landed
green" without a re-run-from-scratch evidence JSON is incomplete —
release.yml can land green and still surface trigger-semantics gaps
the next phase has to journal.

### Carry-forward to v0.12.1 (tracked under MIGRATE-03)

Four items discovered in P56 deferred to v0.12.1 carry-forward:

1. **release-plz GITHUB_TOKEN-pushed tags don't trigger downstream workflows** —
   GH loop-prevention rule. Workaround: `gh workflow run --ref <tag>` (workflow_dispatch).
   Fix path: release-plz workflow uses fine-grained PAT or adds a post-tag dispatch step.
2. **install/cargo-binstall metadata misaligned** with release.yml archive shape —
   ~10 LOC `[package.metadata.binstall]` fix in `crates/reposix-cli/Cargo.toml` +
   `crates/reposix-remote/Cargo.toml`; row stays PARTIAL until v0.12.1.
3. **Rust 1.82 MSRV can't `cargo install reposix-cli` from crates.io** because
   transitive `block-buffer-0.12.0` requires `edition2024` (unstable on 1.82).
   Orthogonal MSRV bug — fix is either cap dep at <0.12 or raise MSRV to 1.85.
4. **Latest-pointer caveat** — `releases/latest/download/...` follows release
   recency; release-plz cuts per-crate releases; a non-cli release published
   after the cli release moves the pointer and re-breaks the curl URL until the
   next cli release. Long-term fix: `gh release create --latest` to pin the
   pointer, or configure release-plz to publish reposix-cli last.

Cross-references:

- `.planning/REQUIREMENTS.md` ## v0.12.0 (active milestone scope, MIGRATE-03 carry-forward list)
- `.planning/ROADMAP.md` ## v0.12.0 Quality Gates (PLANNING) (Phases 56-63)
- `quality/PROTOCOL.md` (lands P57 — autonomous-mode runtime contract)
- `quality/SURPRISES.md` (append-only pivot journal; ≤200 lines, archives at 200)

## Quick links

- `docs/research/initial-report.md` — full architectural argument for git-remote-helper + partial clone.
- `docs/research/agentic-engineering-reference.md` — dark-factory pattern, lethal trifecta, simulator-first.
- `docs/reference/testing-targets.md` — sanctioned real-backend test targets (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`).
- `docs/benchmarks/latency.md` — golden-path latency envelope per backend.
- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` — ratified design doc for the v0.9.0 pivot.
- `.planning/PROJECT.md` — current scope.
- `.planning/STATE.md` — current cursor.
