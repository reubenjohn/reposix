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
- **QG-06 verifier subagent dispatch on every phase close** — see `quality/PROTOCOL.md` § "Verifier subagent prompt template" for the verbatim copy-paste prompt. The executing agent does NOT grade itself.

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

## Quality Gates — dimension/cadence/kind taxonomy (added P57)

The v0.12.0 Quality Gates framework lives at `quality/`. Runtime contract:
`quality/PROTOCOL.md`. Every quality check (gate) sits at one
`(dimension, cadence, kind)` coordinate.

**8 dimensions** — the regression classes the project has:

| Dimension | Checks | Home |
|---|---|---|
| code | clippy, fmt, cargo nextest | `quality/gates/code/` (P58) |
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

**Mandatory CLAUDE.md update per phase (QG-07).** This rule itself. Each phase introducing a new file/convention/gate updates CLAUDE.md in the same PR.

**Meta-rule extension** (when an owner catches a quality miss): fix the issue, update CLAUDE.md, AND **tag the dimension**. The dimension tag tells the next agent which catalog file to add the row to + which `quality/gates/<dim>/` directory the verifier belongs in. This makes ad-hoc fixes structural.

**Cross-references** (do NOT duplicate runtime detail here):

- `quality/PROTOCOL.md` — autonomous-mode runtime contract.
- `quality/catalogs/README.md` — unified catalog schema spec.
- `quality/SURPRISES.md` — append-only pivot journal.
- `.planning/research/v0.12.0-naming-and-architecture.md` — design rationale.
- `.planning/research/v0.12.0-vision-and-mental-model.md` — why this exists.

**Per-dimension `owner_hint`** when an owner catches a miss: see `quality/PROTOCOL.md` § "Failure modes the protocol protects against" for the routing rule. The catalog row is the contract; the verifier is the code; the artifact is the audit; the verdict is the grade.

### P58 — Release dimension live (added 2026-04-27)

The release dimension is now actively enforcing.

| Verifier | Catalog rows | Cadence |
|---|---|---|
| `quality/gates/release/gh-assets-present.py` | `release/gh-assets-present` | weekly |
| `quality/gates/release/installer-asset-bytes.py` | `install/curl-installer-sh`, `install/powershell-installer-ps1`, `install/build-from-source` | weekly |
| `quality/gates/release/brew-formula-current.py` | `release/brew-formula-current`, `install/homebrew` | weekly |
| `quality/gates/release/crates-io-max-version.py` | `release/crates-io-max-version/<crate>` (8 rows; `reposix-swarm` excluded — `publish=false`) | weekly |
| `quality/gates/release/cargo-binstall-resolves.py` | `release/cargo-binstall-resolves` | post-release |

**Cadence wiring.** `.github/workflows/quality-weekly.yml` (cron Monday 09:00 UTC + workflow_dispatch) drives weekly. `.github/workflows/quality-post-release.yml` (workflow_run on `release` + workflow_dispatch) drives post-release. Both pass `GH_TOKEN: ${{ github.token }}` so verifiers calling `gh` CLI auth correctly.

**QG-09 P58 GH Actions badge.** README.md and docs/index.md link the live workflow status alongside the existing CI badge: `https://github.com/reubenjohn/reposix/actions/workflows/quality-weekly.yml/badge.svg`. The shields.io endpoint badge ships in P60 alongside mkdocs publishing badge.json.

**Code dimension absorption (P58 Wave C).** `quality/gates/code/` ships three verifiers: `clippy-lint-loaded.sh` (SIMPLIFY-04 migration of `scripts/check_clippy_lint_loaded.sh`); `check-fixtures.py` (SIMPLIFY-05 Option A migration of `scripts/check_fixtures.py`); `ci-job-status.sh` (POLISH-CODE P58-stub thin gh-CLI wrapper backing `code/cargo-test-pass` + `code/cargo-fmt-clean`). Both old script paths DELETED — caller analysis returned zero shell/yaml/toml callers.

**Orphan-scripts ledger.** `quality/catalogs/orphan-scripts.json` is empty after Wave E removed the `release/crates-io-max-version` waiver row. The dimension provides active enforcement now. SIMPLIFY-12 P63 audits this file at milestone close — goal is to keep it empty.

**Recovery patterns** (the regressions this dimension catches):
- `release/brew-formula-current` RED with stale version: `gh workflow run release.yml --ref reposix-cli-vX.Y.Z` (P56 SURPRISES.md row 2 latest-pointer pattern).
- `release/gh-assets-present` RED with missing assets: same recovery (release.yml didn't fire on the latest tag).
- `release/cargo-binstall-resolves` PARTIAL: documented expected per P56 SURPRISES.md row 3 + waived until 2026-07-26; MIGRATE-03 v0.12.1 ships the binstall metadata fix.

**Meta-rule extension (P58).** When a release-pipeline regression is fixed, the same PR ships container-rehearsal evidence under `quality/reports/verifications/release/`. The artifact JSON IS the proof; the verifier subagent reads it.

**Cross-references** (do NOT duplicate runtime detail here):
- `quality/gates/release/README.md` — release-dim verifier table + conventions.
- `quality/gates/code/README.md` — code-dim verifier table + SIMPLIFY-04/05 absorption record.
- `quality/catalogs/release-assets.json` — 15-row catalog (P58 active enforcement).
- `quality/catalogs/code.json` — 4-row catalog (clippy + fixtures PASS; test + fmt WAIVED until P63).

### P59 — Docs-repro + agent-ux + perf-relocate dimensions live (added 2026-04-27)

Three more dimensions land. The docs-repro dimension is the deepest (9 rows + drift detector); agent-ux is intentionally sparse (1 row); perf is file-relocate-only at v0.12.0.

**Docs-repro home:** `quality/gates/docs-repro/`. 3 verifiers + 1 manual-spec checker:

| Verifier | Catalog rows | Cadence |
|---|---|---|
| `snippet-extract.py` (--list / --check / --write-template) | `docs-repro/snippet-coverage` | pre-push |
| `container-rehearse.sh <id>` | 4 example container rows + `docs-repro/tutorial-replay` | post-release |
| `tutorial-replay.sh` (SIMPLIFY-06; `scripts/repro-quickstart.sh` deleted) | `docs-repro/tutorial-replay` | post-release |
| `manual-spec-check.sh <id>` | `docs-repro/example-03-claude-code-skill` | on-demand |

The 4 container rows + tutorial-replay are WAIVED until 2026-05-12 — the example scripts assume an external simulator that the container does not bring up; sim-inside-container plumbing is post-v0.12.0 work. Snippet-coverage row PASS (drift detector); example-03-claude-code-skill PASS (manual-spec-check).

**Agent-ux home:** `quality/gates/agent-ux/`. Intentionally sparse — dark-factory regression is the only gate at v0.12.0; perf and security stubs land v0.12.1 per MIGRATE-03.

| Verifier | Catalog row | Cadence |
|---|---|---|
| `dark-factory.sh sim` (SIMPLIFY-07; migrated from `scripts/dark-factory-test.sh`) | `agent-ux/dark-factory-sim` | pre-pr |

The v0.9.0 dark-factory invariant (helper stderr-teaching strings on conflict + blob-limit paths) is preserved verbatim. `.github/workflows/ci.yml` invokes the canonical path explicitly; `scripts/dark-factory-test.sh` survives as a 7-line shim per OP-5 reversibility (CLAUDE.md "Local dev loop" command keeps working unchanged).

**Perf home (relocate-only):** `quality/gates/perf/`. File-relocate stubs only at v0.12.0; full gate logic deferred to v0.12.1 stub per MIGRATE-03. 3 catalog rows all WAIVED until 2026-07-26.

| Source | Migrated to | Cadence | Status |
|---|---|---|---|
| `scripts/latency-bench.sh` | `quality/gates/perf/latency-bench.sh` | weekly | WAIVED v0.12.1 |
| `scripts/bench_token_economy.py` | `quality/gates/perf/bench_token_economy.py` | weekly | WAIVED v0.12.1 |
| `scripts/test_bench_token_economy.py` | `quality/gates/perf/test_bench_token_economy.py` | (test) | n/a |

`benchmarks/fixtures/*` stays in place (test inputs, not gates). The 2 perf shims at `scripts/{latency-bench.sh, bench_token_economy.py}` exec/subprocess.run the canonical paths; the test file is renamed-only (no shim — pytest auto-discovers).

**SIMPLIFY-06/07/11 absorption record:**
- SIMPLIFY-06: `scripts/repro-quickstart.sh` DELETED (no callers; tutorial-replay.sh ports the 7-step assertion shape verbatim).
- SIMPLIFY-07: `scripts/dark-factory-test.sh` SHIM (CLAUDE.md "Local dev loop" command + 14 doc/example refs keep working unchanged).
- SIMPLIFY-11: 3 perf scripts MIGRATED via git mv; 2 shims at old paths; test file deleted.

**Recovery patterns:**
- snippet drift detected: `python3 quality/gates/docs-repro/snippet-extract.py --write-template <derived-id>` → paste output into `quality/catalogs/docs-reproducible.json`.
- container rehearsal RED: read `quality/reports/verifications/docs-repro/<row-id>.json`; `stderr` field has the docker output.
- dark-factory regression RED: run `bash quality/gates/agent-ux/dark-factory.sh sim` with `set -x`; the v0.9.0 invariant is the teaching-strings assertion.
- docker-absent (local dev): the runner emits stderr warning + non-fatal exit; container rows grade WAIVED. CI dispatch is the actual rehearsal.

**Cross-references** (do NOT duplicate runtime detail here):
- `quality/gates/docs-repro/README.md` — docs-repro verifier table + conventions.
- `quality/gates/agent-ux/README.md` — agent-ux thin home + intentional sparsity note.
- `quality/gates/perf/README.md` — perf relocate-only stubs + waiver explanation.
- `quality/PROTOCOL.md` — runtime contract (cadences + waiver protocol + verifier-subagent template).
- MIGRATE-03 in `.planning/REQUIREMENTS.md` — v0.12.1 carry-forward (perf full implementation + container sim plumbing).

### P60 — Docs-build dimension live + composite cutover (added 2026-04-27)

Docs-build dimension lands with 4 verifiers backing 4 catalog rows in `quality/catalogs/docs-build.json` plus the back-compat `structure/badges-resolve` row in `quality/catalogs/freshness-invariants.json` (P57 pre-anchored; P60 implements via shared verifier). Same wave hard-cuts the pre-push hook to a runner one-liner and supplants the green-gauntlet composite.

| Verifier | Catalog row(s) | Cadence |
|---|---|---|
| `quality/gates/docs-build/mkdocs-strict.sh` | `docs-build/mkdocs-strict` | pre-push |
| `quality/gates/docs-build/mermaid-renders.sh` | `docs-build/mermaid-renders` | pre-push |
| `quality/gates/docs-build/link-resolution.py` | `docs-build/link-resolution` | pre-push |
| `quality/gates/docs-build/badges-resolve.py` | `docs-build/badges-resolve` + `structure/badges-resolve` | pre-push + weekly |

**SIMPLIFY-08/09/10 absorption (3 closures in one phase):**
- **SIMPLIFY-08**: 3 git-mv migrations preserve history — `scripts/check-docs-site.sh` → `mkdocs-strict.sh`; `scripts/check-mermaid-renders.sh` → `mermaid-renders.sh`; `scripts/check_doc_links.py` → `link-resolution.py`. Thin shims at old paths per OP-5; pre-push hook + CI workflows continue working unchanged through migration.
- **SIMPLIFY-09**: `scripts/green-gauntlet.sh` rewritten as a thin shim that delegates to `python3 quality/runners/run.py --cadence pre-pr`. The 3 modes (`--quick`, default, `--full`) collapse to runner cadence calls.
- **SIMPLIFY-10**: `scripts/hooks/pre-push` body collapsed from 229 lines (6 chained verifiers) to 40 lines total / 10 body lines — a cred-hygiene wrapper invocation (P0 fail-fast on the stdin push-ref ranges) followed by a single `exec python3 quality/runners/run.py --cadence pre-push`. Warm-cache profile 5.3s; well under the 60s pivot threshold so cargo fmt + clippy stay routed THROUGH the runner via the Wave D code-dimension wrappers.

**BADGE-01 closure**: `quality/gates/docs-build/badges-resolve.py` HEADs every badge URL extracted from `README.md` + `docs/index.md` (8 unique URLs) and asserts HTTP 200 + Content-Type matches `image/*` OR `application/json`. Stdlib-only `urllib.request`; ~165 lines. Wave C handled the QG-09 chicken-and-egg via `WAVE_F_PENDING_URLS` (skip-and-log); Wave F cleared the set after the publish landed.

**QG-09 P60 closure**: `docs/badge.json` seeded from `quality/reports/badge.json` and auto-included in mkdocs build → `https://reubenjohn.github.io/reposix/badge.json` resolves. `README.md` (8 badges) + `docs/index.md` (3 badges) gain `![Quality](https://img.shields.io/endpoint?url=...)`. The Quality (weekly) GH Actions badge from P58 stays alongside; the two convey complementary signals (workflow status vs catalog rollup). GH Pages publish completed within ~90s of the Wave F push commit.

**POLISH-DOCS-BUILD broaden-and-deepen (Wave G)**: 4 cadences GREEN at sweep entry (zero RED to fix; Waves A-F left the dimension pristine). 19 PASS pre-push, 1+2 WAIVED pre-pr, 14+3+2 weekly, 6 WAIVED post-release. New artifact `quality/runners/check_p60_red_rows.py` reads the 3 P60-relevant catalogs and reports per-row grades for the 8 P60-touched rows — promoted from ad-hoc bash per CLAUDE.md §4.

**Recovery patterns:**
- mkdocs --strict RED: read `/tmp/check-docs-site*/build.log`; common causes are broken cross-refs, admonition syntax, mermaid HTML-entity escaping, missing nav entry.
- mermaid-renders RED: refresh artifacts via `mcp__playwright__*` walks per CLAUDE.md "Whole-nav-section playwright walks"; fresh-context (cache disabled) walks only.
- link-resolution RED with >5 broken: fix worst, file `.planning/notes/v0.12.1-link-backlog.md` for rest, do NOT block phase close.
- badges-resolve PARTIAL during GH Pages propagation lag: P2 row; doesn't block; document timing in SUMMARY.

**Cross-references** (do NOT duplicate runtime detail here):
- `quality/gates/docs-build/README.md` — docs-build verifier table + conventions.
- `quality/PROTOCOL.md` — cadence routing + verifier-subagent template + waiver protocol.
- `quality/SURPRISES.md` — P60 entries (mkdocs auto-include, hook one-liner, Wave G zero-RED).
- MIGRATE-03 in `.planning/REQUIREMENTS.md` — v0.12.1 carry-forward (auto-sync `docs/badge.json` from `quality/reports/badge.json` on every runner emit).

## Quick links

- `docs/research/initial-report.md` — full architectural argument for git-remote-helper + partial clone.
- `docs/research/agentic-engineering-reference.md` — dark-factory pattern, lethal trifecta, simulator-first.
- `docs/reference/testing-targets.md` — sanctioned real-backend test targets (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`).
- `docs/benchmarks/latency.md` — golden-path latency envelope per backend.
- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` — ratified design doc for the v0.9.0 pivot.
- `.planning/PROJECT.md` — current scope.
- `.planning/STATE.md` — current cursor.
