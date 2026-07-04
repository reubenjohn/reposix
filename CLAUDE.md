# CLAUDE.md — reposix project guide

This file is read by every agent (Claude Code, Codex, Cursor, etc.) that opens this repo. It's the local extension of the user's global CLAUDE.md (`~/.claude/CLAUDE.md`) and overrides nothing — it adds project-specific rules.

## Project elevator pitch

reposix exposes REST-based issue trackers (and similar SaaS systems) as a git-native partial clone, served by `git-remote-reposix` from a local bare-repo cache built from REST responses. Agents use `cat`, `grep`, `sed`, and `git` on real workflows — no MCP tool schemas, no custom CLI, no FUSE mount. See `docs/research/initial-report.md` for the architectural argument and `docs/research/agentic-engineering-reference.md` for the dark-factory pattern that motivates the simulator-first approach.

## Architecture (git-native partial clone)

> **Source of truth:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary/index.md` + `.planning/research/v0.13.0-dvcs/architecture-sketch/index.md` (DVCS extensions, ratified P78–P88; both are entry-points + chapter files post-split).

The reposix runtime has three pieces:

- **`reposix-cache`** — a real on-disk bare git repo built from REST responses via the `BackendConnector` trait. Materializes blobs lazily; every materialization writes to the `audit_events_cache` SQLite table; bytes return wrapped in `reposix_core::Tainted<Vec<u8>>`.
- **`git-remote-reposix`** — git remote helper. Advertises `stateless-connect` (read path: tunnels protocol-v2 fetch with `--filter=blob:none`) and `export` (push path: fast-import → push-time conflict detection → REST writes). Refspec namespace `refs/heads/*:refs/reposix/*`.
- **`reposix init <backend>::<project> <path>`** — bootstraps a partial-clone working tree (`extensions.partialClone=origin`, `remote.origin.url=reposix::<scheme>://<host>/projects/<project>`).
- **`reposix attach <backend>::<project>`** — adopts an existing checkout (vanilla `git clone` mirror, hand-edited tree, prior `reposix init`) and binds it to a `SoT` backend (all four backends — sim, GitHub, Confluence, JIRA — real-backend dispatch landed P91 via the shared `backend_dispatch` factory, D91-03). Reconciles records by frontmatter `id` (5 cases per architecture-sketch); idempotent on re-attach against the same SoT (Q1.3); rejects different SoT (Q1.2). Sets `extensions.partialClone=reposix` and `remote.reposix.url=reposix::<sot>?mirror=<plain-mirror-url>`, plus `remote.pushDefault=<remote-name>` so a bare `git push` routes through the SoT bus (attach warns instead of clobbering a `pushDefault` you already set; also warns if `git-remote-reposix` is missing from PATH).

After bootstrap, agent UX is pure git: `cd <path> && git checkout origin/main && cat issues/<id>.md && grep -r TODO . && <edit> && git add . && git commit && git push`. Zero reposix CLI awareness required beyond `init` / `attach`. Record paths are bucket-aware (D91-13): `issues/<id>.md` for sim/GitHub/JIRA, `pages/<id>.md` for Confluence — `reposix_core::path::{record_path, bucket_for_backend}` is the single source of truth, never hand-pick a bucket string.

### Load-bearing behaviors

- **Push-time conflict detection.** Helper rejects with the standard git "fetch first" error on remote drift; agent recovers via `git pull --rebase && git push`. v0.13.0 P81 made this efficient via `list_changed_since(cursor)` rather than full list-records; L2/L3 cache-desync hardening defers to v0.14.0.
- **Blob limit.** Helper refuses `command=fetch` requests that would materialize more than `REPOSIX_BLOB_LIMIT` blobs (default 200). The stderr error names `git sparse-checkout` as the recovery move so an unfamiliar agent recovers without prompt engineering.
- **Bus URL form (v0.13.0 P82+).** `reposix::<sot>?mirror=<mirror-url>` fans out to SoT (REST) + mirror (plain `git push` shell-out). Push-only (Q3.4); fetch on a bus URL falls through to the single-backend path. Two prechecks gate the push BEFORE reading stdin: PRECHECK A (`git ls-remote` against the mirror) + PRECHECK B (`list_changed_since` against the SoT cursor). `?` in mirror URLs must be percent-encoded.
- **Bus write fan-out (v0.13.0 P83+).** SoT-first then mirror-best-effort. SoT-success + mirror-success advances both `refs/mirrors/<sot>-head` and `-synced-at`. SoT-success + mirror-FAIL freezes `synced-at` (observable lag) and writes `helper_push_partial_fail_mirror_lag` audit op; helper still acks. SoT-fail never attempts mirror. NO helper-side retry (D-08, Q3.6 RATIFIED). NOT 2PC for cross-action partial state — recovery is next-push reads new SoT via PRECHECK B.
- **Mirror-lag refs.** Cache writes `refs/mirrors/<sot>-head` (direct ref) and `refs/mirrors/<sot>-synced-at` (annotated tag, message body `mirror synced at <RFC3339>`) per successful sync. Refs live ONLY in the local reposix cache's own bare repo — verified empirically (P91 91-06): they are NEVER pushed to the plain-git mirror (a real bus push leaves the mirror's `git for-each-ref` showing just `refs/heads/main`), and bus remotes omit `stateless-connect` (DVCS-BUS-FETCH-01) so `git fetch <bus-remote>` does not bring them across either. Today's only consumers are the bus push's own reject-hint (reads the ref internally, renders age inline) and manual inspection of the cache's bare repo directly (`git --git-dir=<cache-path> log refs/mirrors/<sot>-synced-at -1`; find `<cache-path>` via `reposix gc`'s printed cache root). They measure the SoT-edit→mirror-sync gap, NOT current SoT state (Q2.2 doc-clarity contract — full treatment in `docs/concepts/dvcs-topology.md`).
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
8. **Plans accommodate surprises (the +2 phase practice).** Every milestone reserves its last two phases as absorption slots: Slot 1 drains `SURPRISES-INTAKE.md` (each entry → RESOLVED | DEFERRED | WONTFIX with SHA or rationale), Slot 2 drains `GOOD-TO-HAVES.md` (XS always closes; M default-defers). Eager-resolution preference: fix inside the discovering phase if < 1 hour and no new dependency; otherwise file to intake — never silently skip, never scope-creep to fit. The surprises-absorption verifier spot-checks prior phases' honesty: an empty intake alongside skipped findings in verdicts is RED. Long-form (slots, failure modes, honesty check): `.planning/PRACTICES.md` § OP-8.
9. **Milestone-close ritual: distill before archiving.** Intake files + autonomous-run findings get distilled into a new `.planning/RETROSPECTIVE.md` section BEFORE the milestone archives; raw intakes travel with the archive. The milestone-close ratification subagent grades RED if the RETROSPECTIVE.md section is missing. Long-form: `.planning/PRACTICES.md` § OP-9.

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
never loose at `.planning/milestones/` top-level (illustrative; repeats per milestone):

```
.planning/milestones/
└── v0.8.0-phases/
    ├── ARCHIVE.md             # condensed milestone log (per POLISH2-21)
    ├── ROADMAP.md             # milestone-level scoping (kept intact)
    ├── REQUIREMENTS.md        # milestone-level scoping (kept intact)
    └── tag-v0.8.0.sh          # historical one-shot release script
```

The `freshness/no-loose-roadmap-or-requirements` claim in
`quality/gates/structure/freshness-invariants.py` enforces this — any `*ROADMAP*.md` or
`*REQUIREMENTS*.md` placed loose at `.planning/milestones/v*.0-*.md`
fails the verifier. New milestones must scaffold their planning docs
inside `*-phases/` from day one.

## Tech stack

- Rust stable (1.82+ via `rust-toolchain.toml`).
- Async: `tokio` 1.
- Web: `axum` 0.7 + `reqwest` 0.12 (rustls only, never openssl-sys).
- Git: `gix` 0.83 (pinned with `=` because gix is pre-1.0). **Runtime requirement: `git >= 2.34`** for `extensions.partialClone` + `stateless-connect`.
- Storage: `rusqlite` 0.32 with `bundled` feature (no system libsqlite3).
- Errors: `thiserror` for typed crate errors, `anyhow` only at binary boundaries.

## Commands you'll actually use

```bash
# Local dev loop (full matrix in CONTRIBUTING.md)
bash scripts/install-hooks.sh                             # one-time after fresh clone (sets core.hooksPath=.githooks)
cargo check --workspace && cargo test --workspace         # type check + tests (see Build memory budget below)
cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all

# Run the stack
cargo run -p reposix-sim                                  # start simulator on :7878
cargo run -p reposix-cli -- init sim::demo /tmp/repo      # bootstrap a partial-clone working tree
cd /tmp/repo && git checkout origin/main                  # agent UX from here is pure git
cat issues/0001.md && grep -ril TODO . && git push        # cat, grep, edit, push

# Attach an existing checkout (vanilla GH-mirror clone or hand-edited tree, v0.13.0+; Pattern C, matches
# docs/concepts/dvcs-topology.md — clone target IS the mirror of the attach SoT, not an unrelated placeholder)
git clone git@github.com:reubenjohn/reposix-tokenworld-mirror.git /tmp/issues  # vanilla mirror clone (no reposix needed)
cd /tmp/issues
cargo binstall reposix-cli reposix-remote                 # two binaries: `reposix` CLI + the `git-remote-reposix` helper git shells out to
reposix attach confluence::TokenWorld                      # build cache from REST; reconcile by frontmatter id; add bus remote (default name `reposix`); sets remote.pushDefault=reposix (warns instead of clobbering a pushDefault you set yourself)
git push                                                   # pushDefault routes the bare `git push` through the bus: SoT-first + mirror-best-effort fan-out — see § Load-bearing behaviors

# L1 escape hatch (v0.13.0+): rebuild the cache from REST when a push reject suggests cache desync
reposix sync --reconcile                                  # full list_records walk + cache rebuild (DVCS-PERF-L1-02)

# Webhook-driven mirror sync (v0.13.0 P84+; mirror repo only)
gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches \
  -f event_type=reposix-mirror-sync                       # manually trigger mirror sync (synthetic; cron is */30min)
bash scripts/webhook-latency-measure.sh                    # owner-runnable n=10 real-TokenWorld latency measurement (gated on v0.13.x release with working binstall — see SURPRISES-INTAKE)

# Dark-factory regression (proves agent UX is pure git, zero in-context learning)
bash quality/gates/agent-ux/dark-factory.sh sim                # v0.9.0 arm — init + partial-clone + helper teaching strings (local + CI)
bash quality/gates/agent-ux/dark-factory.sh dvcs-third-arm     # v0.13.0 P86 arm — vanilla-clone + reposix attach + bus URL composition + cache audit (local + CI)

# Testing against real backends — env-var setup, per-backend export blocks, and the
# dark_factory_real_{confluence,github,jira} invocations live in docs/reference/testing-targets.md.
bash scripts/preflight-real-backends.sh                    # exit 0 = sanctioned targets reachable; exit 1 = auth/network gap; exit 2 = no creds
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

**Milestone-close additionally requires** `python3 quality/runners/run.py --cadence pre-release-real-backend` exit 0 (see § "Subagent delegation rules" for the non-skippable 9th-probe contract).

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

Any change to `mkdocs.yml` or `docs/**` MUST pass `bash quality/gates/docs-build/mkdocs-strict.sh` before commit (pre-push enforces). Mermaid SVG assertions: `quality/gates/docs-build/mermaid-renders.sh` (also pre-push). For playwright walk rules and scoping (which pages to re-check after a change), see `/reposix-quality-doc-alignment` skill.

## Cold-reader pass on user-facing surfaces

Before declaring any user-facing surface shipped (hero copy, install instructions, headline numbers, README, docs landing page), dispatch `/doc-clarity-review` on the affected pages. For automated rubric grading, use `/reposix-quality-review` (`--rubric <id>` / `--all-stale`). The catalog at `quality/catalogs/subjective-rubrics.json` enforces 30-day freshness TTL.

## Freshness invariants

All invariants are enforced by `quality/runners/verdict.py` (pre-push hook; on-demand: `python3 quality/runners/verdict.py session-end`). When a push is blocked, read the error — it names the violated invariant and the fix. The invariants: no version-pinned filenames, install path leads with package manager, benchmarks in mkdocs nav, no loose ROADMAP/REQUIREMENTS outside `*-phases/`, no orphan docs.

## Release pipeline

**`release-plz.toml`** — `git_release_enable = false` at the workspace level so release-plz does NOT create per-package GitHub releases: each zero-asset per-package release used to steal the `releases/latest` pointer, 404'ing installer URLs and 3 release/install catalog rows. Per-package tags and crates.io publishes are unaffected. The canonical multi-platform release lives at `.github/workflows/release.yml` (tag `v*`).

## Subagent delegation rules

Per the user's global OP #2: "Aggressive subagent delegation." Specifics for this project:

- `gsd-phase-researcher` for any "how do I implement X" question that would consume >100 lines of orchestrator context.
- `gsd-planner` for phase planning. Do not write `PLAN.md` by hand.
- `gsd-executor` for phase execution unless the work is trivially small.
- `gsd-code-reviewer` after every phase ships, before declaring done.
- Run multiple subagents in parallel whenever they're operating on disjoint files.
- **Never delegate `gh pr checkout` to a bash subagent without isolation.** Bash subagents share the coordinator's working tree; `gh pr checkout` switches the local branch behind the coordinator's back, which already caused the cherry-pick mess at commit `5a91ae2`. Either spawn a worktree first (`git worktree add /tmp/pr-N pr-N-branch`) and have the subagent `cd` into it, or have the subagent operate inside `/tmp/<branch>-checkout`. The coordinator's checkout is shared state — treat it that way.
- **Verifier subagent on every phase close** — see "Verifier subagent dispatch" in Quality Gates section + `quality/PROTOCOL.md` § "Verifier subagent prompt template".
- **Dispatching subjective rubrics (cold-reader, install-positioning, headline-numbers)** — `/reposix-quality-review` skill. Invocation: `bash .claude/skills/reposix-quality-review/dispatch.sh --rubric <id>` / `--all-stale` / `--force`. Path A (Task tool from Claude session) preferred for unbiased grading; Path B (claude -p subprocess) fallback. `subjective/dvcs-cold-reader` is now dispatch-wired for real grading (P90 90-03; previously fell through to a Path-B stub). `agent-ux/dvcs-third-arm` is pure shell asserts and was reclassified `kind: mechanical` (P90 D90-10) — it was never subjective grading despite the earlier `subagent-graded` tag.
- **Orchestration-shaped phases run at top-level, not under `/gsd-execute-phase`.** When a phase's work shape is "fan out → gather → interpret → resolve" rather than "write code → run tests → commit," the top-level coordinator IS the executor. `gsd-executor` lacks `Task` and depth-2 spawning is forbidden, so subagent fan-out cannot live inside it. Mark such phases `Execution mode: top-level` in ROADMAP and provide a research brief the orchestrator follows verbatim. Docs-alignment backfill is the canonical example; retroactive audits follow the same pattern. Refresh runs on stale docs (`/reposix-quality-refresh <doc>`) are also top-level — pre-push that BLOCKS mid-`gsd-execute-phase` must be resolved by checkpointing the executor and invoking the slash command from a fresh top-level session.
- **The milestone-close 9th probe (RBF-FW-03) is non-skippable.** Any milestone-close ritual missing `python3 quality/runners/run.py --cadence pre-release-real-backend` exit 0 grades the milestone RED. The probe runs the vision litmus test against the sanctioned real backend (TokenWorld for v0.13.0); the catalog row `agent-ux/milestone-close-vision-litmus-real-backend` carries `blast_radius: P0` and NEVER carries a `waiver` block (the deferral-loop guard, anti-C7). Its verifier is `quality/gates/agent-ux/milestone-close-vision-litmus.sh`; the verdict skeleton lives at `quality/dispatch/milestone-close-verdict.md`. The SLOT reads NOT-VERIFIED (never FAIL, never skip-as-pass) via two paths — env unset → runner short-circuits (`_realbackend.is_skipped`); env set + substrate absent → script exits 75, which the runner maps to NOT-VERIFIED (`_realbackend.map_exit_code_to_status`). Exit-code + OD-2 hard-RED conventions: PROTOCOL.md § "Verifier exit-code conventions" + § "Milestone-close 9th probe". The script stopped being an unconditional exit-75 stub in P91 (D91-06): it now really executes the vanilla-clone + `reposix attach confluence::TokenWorld` + edit + commit + `git push` flow against real TokenWorld and asserts the 5 T2 pass boxes + dual-table audit — it PASSED 11/11 asserts in the P91 close run.

**Ownership charter for dispatched subagents.** Every subagent (executor, verifier, researcher, code-reviewer) that touches a real surface owns it, not just its acceptance criteria: (1) acceptance criteria are the floor, not the ceiling — done means "I'd defend this in review as excellent," not "plan executed"; (2) noticing is a deliverable — every report names what it noticed near its work (lying doc claims, tests that don't assert what their names promise, error messages that don't teach recovery, dead code, stale comments, missing edge cases), and an empty noticing section from code-touching work is itself a red flag (mirrors the verifier honesty check); (3) eager-fix or file, never silently skip — `<1h` + no new dependency → fix in place, else → `SURPRISES-INTAKE`/`GOOD-TO-HAVES` with severity + sketch (OP-8); (4) verify against reality — run the thing, render the page, hit the backend; a claim without an artifact is not done (OP-1); (5) north star — polish for adoption: would a skeptical dev hitting this surface for the first time come away impressed? (Owner mandate OD-3, 2026-07-03.)

The orchestrator's job is to route, decide, and integrate — not to type code that a subagent could type.

### Meta-rule: when an owner catches a quality miss, fix it twice

When the owner catches a quality issue the agent missed, the fix is
two-fold: (1) fix the issue in the code/docs, and (2) update CLAUDE.md
(and/or the §0.8 SESSION-END-STATE framework) so the next agent's
session reads the tightened rule. Just shipping the fix without
updating the instructions guarantees recurrence.

## Threat model

This project is a textbook lethal-trifecta machine: private data (issue bodies in the working tree) + untrusted input (every body/comment/title is attacker-influenced) + exfiltration paths (`git push` to arbitrary remotes; helper + cache make outbound HTTP). The mandatory, tested cuts: `REPOSIX_ALLOWED_ORIGINS` egress allowlist through the single `reposix_core::http::client()` factory (clippy `disallowed_methods` bans direct `reqwest::Client::new()`); bytes-in-bytes-out export path with `Tainted<T>` → `sanitize()` as the only escape hatch; frontmatter field allowlist stripping server-controlled fields on inbound writes; append-only dual audit tables (OP-3); and the env-gated `pre-release-real-backend` cadence that makes allowlist enforcement testable end-to-end (fails closed, see § "Subagent delegation rules").

Full per-cut table with code locations: `docs/how-it-works/trust-model.md`. Red-team analysis: `research/threat-model-and-critique.md`.

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
| agent-ux | dark-factory regression (sim arm + DVCS third arm, both `kind: mechanical`) + reposix-attach + bus URL prechecks + webhook YAML + test-name-vs-asserts honesty gate | `quality/gates/agent-ux/` |
| perf | latency, token economy | `quality/gates/perf/` |
| security | allowlist enforcement, audit immutability | `quality/gates/security/` |

**8 cadences** — when the gate runs:

`pre-commit` (local, every commit, <2s, blocking) · `pre-push` (local, every push, <60s, blocking) · `pre-pr` (PR CI, <10min, blocking) · `weekly` (cron, alerting) · `pre-release` (on tag, <15min, blocking) · `post-release` (after assets ship, alerting) · `on-demand` (manual / subagent) · `pre-release-real-backend` (local + milestone-close, env-gated, mandatory at tag-time, blocking; default-skips to NOT-VERIFIED when `REPOSIX_ALLOWED_ORIGINS` + backend creds are unset — never skip-counts-as-pass, per PROTOCOL.md OD-2).

Rows carry `cadences: list[str]` (a single gate may declare multiple cadences and fire at every relevant trigger; see `quality/PROTOCOL.md` § "Latency budgets").

**6 kinds** — how the gate is verified:

`mechanical` (deterministic shell + asserts) · `container` (fresh ubuntu/alpine + post-conditions) · `asset-exists` (HEAD/GET URL + min-bytes) · `subagent-graded` (rubric-driven subagent) · `manual` (human-only with TTL freshness) · `shell-subprocess` (real subprocess against a real binary/backend + a transcript artifact at `quality/reports/transcripts/<row-slug>-<RFC3339>.txt` recording argv + env_keys [variable NAMES only — no `=value`, security] + cwd + exit_code + stdout/stderr blocks). The worked example (`quality/gates/agent-ux/shell-subprocess-example.sh`) invokes the local `target/debug/reposix` binary when present and falls back to `bash --version` for CI portability; transport-claim rows MUST invoke a real reposix binary or backend endpoint. Verifier grading contract: PROTOCOL.md § "For rows with `kind: shell-subprocess`".

**Honesty rules (P90 RBF-FW-06..12 — full detail in `quality/PROTOCOL.md` § "Honesty rules").** `minted_at` (RFC3339, write-once) is the audit-cutoff anchor on every row minted from P90 onward; `claim_vs_assertion_audit` validation anchors on it when present, falling back to `last_verified` for legacy (pre-P90) rows — the legacy exemption retires at P95 RBF-D-06. Transport/perf rows carry `coverage_kind` (`real-backend` or `WAIVED + until_date`, never a bare PASS-with-comment) with a `transport_claim` tri-state opt-out; enforcement is hard for `minted_at`-bearing rows, RAISE-only for pre-P90 legacy rows. A missing verifier script flips a row to `NOT-VERIFIED` unconditionally (never preserves prior PASS) with a distinct `error: verifier-not-found` artifact marker so a deploy glitch reads differently from a real regression. An env-gated skip (`pre-release-real-backend` without creds) also fails closed to `NOT-VERIFIED`, but preserves the prior real grade in `last_real_grade`/`last_real_verified` plus a `skip_reason: env-missing` marker — history survives, ground truth is never silently lost (OD-2-safe). `kind: shell-subprocess` transcripts are written fresh at grade time (not build time). The `agent-ux/test-name-vs-asserts` gate (F-K4b) requires every `expected.asserts` entry to map to at least one `asserts_passed` string on `minted_at`-bearing rows — vocabulary mismatch grades RED (closes the p86 F6 "test name lies about what it checks" class). Milestone-close dispatches two independent honesty spot-checks from `quality/dispatch/`: `absorption-honesty-spot-check.md` (F-K5, every no-intake phase, spot-check author ≠ orchestrator) and `milestone-adversarial.md` (RBF-FW-12, fresh subagent grades whether each row's assertion would falsify its own description; artifact at `quality/reports/verifications/adversarial/<milestone>.json`; verdict blocks GREEN on any failed row audit).

**Structure-dimension gates (P89 RBF-FW-04 + RBF-FW-05).** `quality/gates/structure/banned-production-tokens.sh` blocks `\bP\d{2,3}-\d+\b` phase-ID tokens in `crates/**/*.rs` outside `tests/` and `target/`, with per-line `// banned-words: ok` allowlist markers for justified exceptions (pre-commit + pre-push). `quality/gates/structure/deferral-pointer-linter.sh` cross-references `not yet wired in P\d+` / `lands? (alongside|in) P\d+` / `substrate-gap-deferred` strings in `crates/` against PLAN-artifact existence under `.planning/phases/N-*/` — accepting BOTH flat (`N-NN-PLAN.md`) and post-split (`N-NN-PLAN/*.md` dir) layouts (pre-push). PNN extraction is phrase-scoped, not line-scoped: a phase number in an adjacent allowlist marker (e.g. `// banned-words: ok — P91 …`) is NOT treated as a deferral target. A deferral pattern that matches with no PNN suffix (bare `// substrate-gap-deferred`) ALSO BLOCKs — every deferral pointer must name a real downstream phase. Both catalogs live in `quality/catalogs/freshness-invariants.json` (wrapper `"dimension": "structure"`; there is no `structure.json`). Note `crates/reposix-cli/src/sync.rs`'s `P82+` is deliberately NOT matched by the banned-token regex (no `-NN` suffix) — the deferral-pointer linter owns that shape.

**Banned-token regex scope (P89 RBF-FW-04):** the `\bP\d{2,3}-\d+\b` regex is a deliberate trade-off, not a comprehensive phase-prefix scrubber. It **CATCHES** v0.13+ phase numbers (`P79-02`, `P83-01`, `P150-01`) — the F-K class the owner repeatedly catches in user-facing stderr. It **INTENTIONALLY MISSES** v0.8/v0.9-era audit IDs `P\d-\d` (`P1-1`, `P0-2` in `crates/reposix-core/src/error.rs` — legitimate code-quality references, not phase IDs) and generic prose that happens to embed such a token (an NLP-grade classifier would be needed). **Forward convention:** new audit-ID schemes SHOULD adopt `P\d{2,3}-` numbering (so accidental leaks are caught) OR a distinct prefix (`AUD-1`, `QA-2-3`) the framework won't ban. The same rationale lives in the script header for in-context discoverability.

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
- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary/index.md` — ratified design doc for the v0.9.0 pivot (entry-point + chapters post-split).
- `.planning/PROJECT.md` — current scope.
- `.planning/STATE.md` — current cursor.
