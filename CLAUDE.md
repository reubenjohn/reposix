# CLAUDE.md — reposix project guide

This file is read by every agent (Claude Code, Codex, Cursor, etc.) that opens this repo. It's the local extension of the user's global CLAUDE.md (`~/.claude/CLAUDE.md`) and overrides nothing — it adds project-specific rules.

## Project elevator pitch

reposix exposes REST-based issue trackers (and similar SaaS systems) as a git-native partial clone, served by `git-remote-reposix` from a local bare-repo cache built from REST responses. Agents use `cat`, `grep`, `sed`, and `git` on real workflows — no MCP tool schemas, no custom CLI, no FUSE mount. See `docs/research/initial-report.md` for the architectural argument and `docs/research/agentic-engineering-reference.md` for the dark-factory pattern that motivates the simulator-first approach.

## Architecture (git-native partial clone)

> **Source of truth:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` + `.planning/research/v0.13.0-dvcs/architecture-sketch.md` (DVCS extensions, ratified P78–P88).

The reposix runtime has three pieces:

- **`reposix-cache`** — a real on-disk bare git repo built from REST responses via the `BackendConnector` trait. Materializes blobs lazily; every materialization writes to the `audit_events_cache` SQLite table; bytes return wrapped in `reposix_core::Tainted<Vec<u8>>`.
- **`git-remote-reposix`** — git remote helper. Advertises `stateless-connect` (read path: tunnels protocol-v2 fetch with `--filter=blob:none`) and `export` (push path: fast-import → push-time conflict detection → REST writes). Refspec namespace `refs/heads/*:refs/reposix/*`.
- **`reposix init <backend>::<project> <path>`** — bootstraps a partial-clone working tree (`extensions.partialClone=origin`, `remote.origin.url=reposix::<scheme>://<host>/projects/<project>`).
- **`reposix attach <backend>::<project>`** — adopts an existing checkout (vanilla `git clone` mirror, hand-edited tree, prior `reposix init`) and binds it to a `SoT` backend. Reconciles records by frontmatter `id` (5 cases per architecture-sketch); idempotent on re-attach against the same SoT (Q1.3); rejects different SoT (Q1.2). Sets `extensions.partialClone=reposix` and `remote.reposix.url=reposix::<sot>?mirror=<plain-mirror-url>`.

After bootstrap, agent UX is pure git: `cd <path> && git checkout origin/main && cat issues/<id>.md && grep -r TODO . && <edit> && git add . && git commit && git push`. Zero reposix CLI awareness required beyond `init` / `attach`.

### Load-bearing behaviors

- **Push-time conflict detection.** Helper rejects with the standard git "fetch first" error on remote drift; agent recovers via `git pull --rebase && git push`. v0.13.0 P81 made this efficient via `list_changed_since(cursor)` rather than full list-records; L2/L3 cache-desync hardening defers to v0.14.0.
- **Blob limit.** Helper refuses `command=fetch` requests that would materialize more than `REPOSIX_BLOB_LIMIT` blobs (default 200). The stderr error names `git sparse-checkout` as the recovery move so an unfamiliar agent recovers without prompt engineering.
- **Bus URL form (v0.13.0 P82+).** `reposix::<sot>?mirror=<mirror-url>` fans out to SoT (REST) + mirror (plain `git push` shell-out). Push-only (Q3.4); fetch on a bus URL falls through to the single-backend path. Two prechecks gate the push BEFORE reading stdin: PRECHECK A (`git ls-remote` against the mirror) + PRECHECK B (`list_changed_since` against the SoT cursor). `?` in mirror URLs must be percent-encoded.
- **Bus write fan-out (v0.13.0 P83+).** SoT-first then mirror-best-effort. SoT-success + mirror-success advances both `refs/mirrors/<sot>-head` and `-synced-at`. SoT-success + mirror-FAIL freezes `synced-at` (observable lag) and writes `helper_push_partial_fail_mirror_lag` audit op; helper still acks. SoT-fail never attempts mirror. NO helper-side retry (D-08, Q3.6 RATIFIED). NOT 2PC for cross-action partial state — recovery is next-push reads new SoT via PRECHECK B.
- **Mirror-lag refs.** Cache writes `refs/mirrors/<sot>-head` (direct ref) and `refs/mirrors/<sot>-synced-at` (annotated tag, message body `mirror synced at <RFC3339>`) per successful sync. Refs live in the cache's bare repo; vanilla `git fetch` brings them along. They measure the SoT-edit→mirror-sync gap, NOT current SoT state (Q2.2 doc-clarity contract — full treatment in `docs/concepts/dvcs-topology.md`).
- **Webhook-driven mirror sync (v0.13.0 P84).** Reference GH Action workflow lives in the *mirror* repo (template at `docs/guides/dvcs-mirror-setup-template.yml`; byte-equal copies enforced by `agent-ux/webhook-trigger-dispatch`). Triggers: `repository_dispatch` (event_type=`reposix-mirror-sync`) + cron `*/30 * * * *` literal safety net. Secrets on the mirror repo (`ATLASSIAN_API_KEY` etc.). `cargo binstall reposix-cli` (workspace name is `reposix`; binstall metadata in `crates/reposix-cli/Cargo.toml`). p95 latency target ≤ 120s. Owner walk-through: `docs/guides/dvcs-mirror-setup.md`.

## Operating Principles (project-specific)

The user's global Operating Principles in `~/.claude/CLAUDE.md` are bible. The following are project-specific reinforcements, not replacements:

1. **Simulator is the default / testing backend.** The simulator at `crates/reposix-sim/` is the default backend for every demo, unit test, and autonomous agent loop. Real backends (GitHub via `reposix-github`, Confluence via `reposix-confluence`, JIRA via `reposix-jira`) are guarded by the `REPOSIX_ALLOWED_ORIGINS` egress allowlist and require explicit credential env vars (`GITHUB_TOKEN`, `ATLASSIAN_API_KEY` + `ATLASSIAN_EMAIL` + `REPOSIX_CONFLUENCE_TENANT`, `JIRA_EMAIL` + `JIRA_API_TOKEN` + `REPOSIX_JIRA_INSTANCE`). Autonomous mode never hits a real backend unless the user has put real creds in `.env` AND set a non-default allowlist. This is both a security constraint (fail-closed by default) and the StrongDM dark-factory pattern.
2. **Tainted by default.** Any byte that came from a remote (simulator counts) is tainted. Tainted content must not be routed into actions with side effects on other systems (e.g. don't echo issue bodies into `git push` to remotes outside an explicit allowlist). The lethal-trifecta mitigation matters even against the simulator, because the simulator is *seeded* by an agent and seed data is itself attacker-influenced.
3. **Audit log is non-optional, and lives in TWO append-only tables.** `audit_events_cache` (cache-internal events — blob materialization, gc, helper RPC fetch/push, sync-tag writes, mirror-refs sync writes) lives in the cache crate (`reposix-cache::audit`); `audit_events` (backend mutations — `create_record` / `update_record` / `delete_or_close`) lives in the core crate (`reposix-core::audit`) and is written by the sim/confluence/jira adapters. A complete forensic query reads both. Either schema missing a row for a network-touching action means the feature isn't done. The dual-table shape is intentional; physical unification behind a `dyn AuditSink` trait is deferred.
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
9. **Milestone-close ritual: distill before archiving.** Each
   milestone's `*-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` entries
   AND the autonomous-run session findings get distilled into a new
   section of `.planning/RETROSPECTIVE.md` BEFORE the milestone
   archives — using the existing template (What Was Built / What
   Worked / What Was Inefficient / Patterns Established / Key
   Lessons). Raw intake files travel with the milestone archive into
   `*-phases/`; distilled lessons live permanently and discoverably
   in `RETROSPECTIVE.md`. **Why:** without this step, learnings get
   lost in milestone archives — the +2 phase practice produces signal
   that's worth keeping cross-milestone (failure modes, patterns,
   process gaps) but the raw intake format is too granular for future
   readers to skim. The ratification subagent for milestone-close
   should verify a RETROSPECTIVE.md section exists for the milestone
   and grade RED if it doesn't.

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
- Git: `gix` 0.83 (pinned with `=` because gix is pre-1.0; bumped from 0.82 in P78 — issues #29 + #30 yanked the prior pin). **Runtime requirement: `git >= 2.34`** for `extensions.partialClone` + `stateless-connect`.
- Storage: `rusqlite` 0.32 with `bundled` feature (no system libsqlite3).
- Errors: `thiserror` for typed crate errors, `anyhow` only at binary boundaries.

## Commands you'll actually use

```bash
# Local dev loop
bash scripts/install-hooks.sh                             # one-time after fresh clone (sets core.hooksPath=.githooks)
cargo check --workspace                                   # fast type check
cargo test --workspace                                    # unit + integration tests
cargo clippy --workspace --all-targets -- -D warnings     # CI lint
cargo fmt --all                                           # CI fmt

# Run the stack
cargo run -p reposix-sim                                  # start simulator on :7878
cargo run -p reposix-cli -- init sim::demo /tmp/repo      # bootstrap a partial-clone working tree
cd /tmp/repo && git checkout origin/main                  # agent UX from here is pure git
cat issues/0001.md && grep -ril TODO . && git push        # cat, grep, edit, push

# Attach an existing checkout (vanilla GH-mirror clone or hand-edited tree, v0.13.0+)
git clone git@github.com:org/issues-repo.git /tmp/issues  # vanilla mirror clone (no reposix needed)
cd /tmp/issues
reposix attach sim::demo --remote-name reposix            # build cache from REST; reconcile by frontmatter id; add reposix remote
git push reposix main                                     # push via reposix remote (single-SoT shape; bus URL form requires P82+)

# L1 escape hatch (v0.13.0+): rebuild the cache from REST when a push reject suggests cache desync
reposix sync --reconcile                                  # full list_records walk + cache rebuild (DVCS-PERF-L1-02)

# Bus push (v0.13.0+ P83-01): URL form `reposix::<sot>?mirror=<mirror-url>` recognized + dispatched; cheap prechecks (mirror drift + SoT drift) gate the push BEFORE reading stdin. Write fan-out shipped in P83-01 — SoT-first + mirror-best-effort + lag-tracking (DVCS-BUS-WRITE-01..05); fault-injection coverage lands in P83-02.
git push reposix main                                     # bus push (URL: reposix::<sot>?mirror=<url>; SoT-first + mirror-best-effort + lag-tracking — DVCS-BUS-WRITE-01..05 in v0.13.0)

# Webhook-driven mirror sync (v0.13.0 P84+; mirror repo only)
gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches \
  -f event_type=reposix-mirror-sync                       # manually trigger mirror sync (synthetic; cron is */30min)
bash scripts/webhook-latency-measure.sh                    # owner-runnable n=10 real-TokenWorld latency measurement (gated on v0.13.x release with working binstall — see SURPRISES-INTAKE)

# Dark-factory regression (proves agent UX is pure git, zero in-context learning)
bash scripts/dark-factory-test.sh sim                          # v0.9.0 arm — init + partial-clone + helper teaching strings (local + CI)
bash quality/gates/agent-ux/dark-factory.sh dvcs-third-arm     # v0.13.0 P86 arm — vanilla-clone + reposix attach + bus URL composition + cache audit (local + CI)

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

### Push cadence — per-phase

Every phase closes with `git push origin main` BEFORE the verifier-subagent dispatch. Pre-push gate-passing is part of phase close, not an end-of-session sweep — verifier grades RED if the phase shipped without the push landing. Trivial in-phase chores (typo fix, comment cleanup) ride to origin with the phase's terminal push, not their own round-trip. The pre-commit fmt hook is the secondary safety net at commit time.

### Code conventions

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

## Release pipeline

**`release-plz.toml`** — `git_release_enable = false` at the workspace level so release-plz does NOT create per-package GitHub releases. Why: each per-package release (zero assets) was published *after* the canonical `v*` release and stole the `releases/latest` pointer, 404'ing the user-facing installer URLs and 3 catalog rows (`release/gh-assets-present`, `install/curl-installer-sh`, `install/powershell-installer-ps1`). Per-package tags and crates.io publishes are unaffected. The canonical multi-platform release lives at `.github/workflows/release.yml` (tag `v*`).

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
- **Audit log append-only.** Both `audit_events_cache` and `audit_events` tables (SQLite WAL, no UPDATE/DELETE). Schema split + forensic-query rationale in OP-3.

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
| agent-ux | dark-factory regression (sim arm + DVCS third arm) + reposix-attach + bus URL prechecks + webhook YAML | `quality/gates/agent-ux/` |
| perf | latency, token economy | `quality/gates/perf/` |
| security | allowlist enforcement, audit immutability | `quality/gates/security/` |

**7 cadences** — when the gate runs:

`pre-commit` (local, every commit, <2s, blocking) · `pre-push` (local, every push, <60s, blocking) · `pre-pr` (PR CI, <10min, blocking) · `weekly` (cron, alerting) · `pre-release` (on tag, <15min, blocking) · `post-release` (after assets ship, alerting) · `on-demand` (manual / subagent).

Rows carry `cadences: list[str]` (a single gate may declare multiple cadences and fire at every relevant trigger; see `quality/PROTOCOL.md` § "Latency budgets").

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

## Quick links

- `docs/research/initial-report.md` — full architectural argument for git-remote-helper + partial clone.
- `docs/research/agentic-engineering-reference.md` — dark-factory pattern, lethal trifecta, simulator-first.
- `docs/reference/testing-targets.md` — sanctioned real-backend test targets (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`).
- `docs/benchmarks/latency.md` — golden-path latency envelope per backend.
- `docs/concepts/dvcs-topology.md` (P85) — three roles (SoT-holder / mirror-only consumer / round-tripper) + mirror-lag refs explained; the canonical DVCS mental model.
- `docs/guides/dvcs-mirror-setup.md` (P85) — owner walk-through for webhook + GH Action setup; cron-only fallback (Q4.2); cleanup procedure.
- `docs/guides/troubleshooting.md` § "DVCS push/pull issues" (P85) — bus `fetch first` rejections, attach reconciliation warnings, webhook race conditions, cache-desync recovery.
- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` — ratified design doc for the v0.9.0 pivot.
- `.planning/PROJECT.md` — current scope.
- `.planning/STATE.md` — current cursor.
