# reposix

## What This Is

A git-backed FUSE filesystem that exposes REST APIs (issue trackers, knowledge bases, ticketing systems) as a POSIX directory tree, so autonomous LLM agents can use `cat`, `grep`, `sed`, and `git` instead of MCP tool schemas. Targets the "dark factory" agent-engineering pattern: when no human reads the code, agents need substrates that match their pre-training distribution.

## Core Value

**An LLM agent can `ls`, `cat`, `grep`, edit, and `git push` issues in a remote tracker without ever seeing a JSON schema or REST endpoint.** Everything else (multi-backend support, simulators, RBAC, conflict resolution) is in service of that single experience.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- ✓ **RENAME-01: `IssueBackend` → `BackendConnector` trait rename** — Phase 27
- ✓ **EXT-01: `Issue.extensions` field** — Phase 27
- ✓ **JIRA-01: `reposix-jira` crate — read-only `BackendConnector` impl** — Phase 28
- ✓ **JIRA-02: JQL pagination + status-category mapping + subtask hierarchy** — Phase 28
- ✓ **JIRA-03: JIRA-specific `extensions` in frontmatter** — Phase 28
- ✓ **JIRA-04: CLI dispatch** — Phase 28
- ✓ **JIRA-05: Tests + docs + ADRs** — Phase 28
- ✓ **JIRA-06 (stretch): JIRA write path** — Phase 29. `create_issue` (POST), `update_issue` (PUT), `delete_or_close` (Transitions API + DELETE fallback). 31 unit tests + 5-arm contract test. Audit log for all mutations.
- ✓ **ARCH-01: `crates/reposix-cache/` builds bare git repo from REST** — Phase 31
- ✓ **ARCH-02: cache returns `Tainted<Vec<u8>>` + audit row per blob materialization + append-only triggers** — Phase 31
- ✓ **ARCH-03: cache enforces `REPOSIX_ALLOWED_ORIGINS` via single `reposix_core::http::client()` factory** — Phase 31
- ✓ **ARCH-04: `git-remote-reposix` advertises `stateless-connect` (hybrid with `export`)** — Phase 32
- ✓ **ARCH-05: three protocol-v2 framing gotchas covered + refspec namespace `refs/heads/*:refs/reposix/*`** — Phase 32
- ✓ **ARCH-06: `BackendConnector::list_changed_since` on all 4 backends + sim `?since=` REST param** — Phase 33
- ✓ **ARCH-07: atomic delta-sync transaction (cache + last_fetched_at + audit row in one tx)** — Phase 33
- ✓ **ARCH-08: push-time conflict detection (`error refs/heads/main fetch first`, cache untouched on reject)** — Phase 34
- ✓ **ARCH-09: blob-limit guardrail with verbatim `git sparse-checkout` teaching string + audit row** — Phase 34
- ✓ **ARCH-10: frontmatter field allowlist on push (`id`/`created_at`/`version`/`updated_at` stripped)** — Phase 34
- ✓ **ARCH-11: `reposix init <backend>::<project> <path>` replaces `reposix mount`** — Phase 35
- ✓ **ARCH-12: dark-factory pure-git agent UX validated against sim** — Phase 35 (real-backend exercise `pending-secrets`)
- ✓ **ARCH-13: `crates/reposix-fuse/` + `fuser` dependency + `fuse-mount-tests` feature gate purged** — Phase 36
- ✓ **ARCH-14: project `CLAUDE.md` rewritten to steady-state git-native architecture (no FUSE residue)** — Phase 36
- ✓ **ARCH-15: `.claude/skills/reposix-agent-flow/SKILL.md` ships with dark-factory regression invocation** — Phase 36
- ✓ **ARCH-16: real-backend smoke harness wired (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`)** — Phase 35 (test infra; `pending-secrets`)
- ✓ **ARCH-17: golden-path latency captured + `docs/benchmarks/v0.9.0-latency.md` artifact** — Phase 35 (capture) + Phase 36 (artifact); sim column populated
- ✓ **ARCH-18: `docs/reference/testing-targets.md` documents the three sanctioned targets** — Phase 36
- ✓ **ARCH-19: three CI integration-contract-*-v09 jobs defined** — Phase 36 (`pending-secrets` until secrets decrypt)
- ✓ **DOCS-01: `docs/index.md` hero — V1 vignette + 3 measured numbers; ≤ 250 above-fold words** — Phase 40
- ✓ **DOCS-02: How-it-works trio (`filesystem-layer`, `git-layer`, `trust-model`); each one mermaid diagram** — Phase 41
- ✓ **DOCS-03: Two concept pages (mental-model-in-60-seconds, reposix-vs-mcp-and-sdks)** — Phase 40
- ✓ **DOCS-04: Three guides (write-your-own-connector, integrate-with-your-agent, troubleshooting)** — Phase 42
- ✓ **DOCS-05: Simulator relocated to `docs/reference/simulator.md`** — Phase 42
- ✓ **DOCS-06: 5-minute first-run tutorial verified by `scripts/tutorial-runner.sh`** — Phase 42
- ✓ **DOCS-07: MkDocs nav restructured per Diátaxis** — Phase 43
- ✓ **DOCS-08: mkdocs-material theme tuning + README hero rewritten** — Phase 43 (linter wiring) + Phase 45 (README)
- ✓ **DOCS-09: Banned-words linter `scripts/banned-words-lint.sh` + `docs/.banned-words.toml` + pre-commit + CI + `reposix-banned-words` skill** — Phase 43
- ✓ **DOCS-10: 16-page cold-reader clarity audit; zero critical friction points** — Phase 44 (2 critical fixed, 1 escalated to Phase 45 and closed there)
- ✓ **DOCS-11: README points at mkdocs site; CHANGELOG `[v0.10.0]` finalized** — Phase 45 (playwright screenshots deferred to v0.11.0 — cairo system libs unavailable on dev host)

### Active

**Functional core**
- [ ] **Simulator-first architecture.** A standalone HTTP fake server that mimics issue-tracker semantics (rate limits, 409 conflicts, workflow rules, RBAC). Serves as the dev/test substrate so we never need real credentials to validate end-to-end behavior.
- [ ] **Issues as Markdown + YAML frontmatter.** Each issue is a single `.md` file. Metadata (status, assignee, labels) lives in YAML frontmatter; body is free text.
- [ ] **FUSE mount with full read+write.** `getattr`, `readdir`, `read`, `write`, `create`, `unlink`. Backed by an async client to the simulator (or any compatible backend).
- [ ] **`git-remote-reposix` helper.** Standard git remote helper protocol. `git push` translates diffs to API calls. Conflicts surface as native git merge conflicts.
- [ ] **Working CLI orchestrator.** `reposix sim`, `reposix mount`, `reposix demo` — single binary, ergonomic UX.
- [ ] **Audit log of every network-touching action.** SQLite, queryable. Non-optional per OP #6 (ground truth).
- [ ] **Adversarial swarm harness.** A small load generator that spawns N agent-shaped clients hammering the FUSE mount against the simulator. Channels the StrongDM "10k agent QA team" pattern at miniature scale.
- [ ] **Working CI on GitHub Actions.** Lint, test, integration test that actually mounts the FUSE in the runner. Codecov coverage. Badges in README.
- [ ] **Demo-ready by 2026-04-13 morning.** README + asciinema/script(1) recording + walkthrough doc that lets the user re-run end-to-end in <5 minutes.

**Security guardrails (from threat-model audit — non-negotiable)**
- [ ] **Outbound HTTP allowlist.** All HTTP clients (FUSE daemon + remote helper) refuse origins not in `REPOSIX_ALLOWED_ORIGINS` (default `http://127.0.0.1:*`, `http://localhost:*`). Enforced in a single `reposix_core::http::client()` factory — no other code constructs `reqwest::Client`. Test: a write that attempts to `git push` to `https://attacker.example.com/projects/x` returns `EPERM` and is logged.
- [ ] **Bulk-delete cap on push.** Any single `git push` that deletes >5 issues is rejected with a clear error. Defends against a `rm -rf` on the mount point cascading into a `DELETE` storm.
- [ ] **Server-controlled frontmatter fields are immutable from clients.** `id`, `created_at`, `version`, and `updated_at` are stripped from inbound writes (FUSE write path + push diff path) before serialization. Test: an attacker-authored issue body with `version: 999999` does not update the server version.
- [ ] **Filename derivation never uses titles.** Files are named `<id>.md` (zero-padded to 4 digits for v0.1). Titles are body-only. FUSE rejects path components containing `/`, `\0`, `.`, `..` with `EINVAL`.
- [ ] **Tainted-content typing.** Bytes that came from a remote (network or simulator) are wrapped in `reposix_core::Tainted<T>`. Functions that perform side-effects on other systems (egress HTTP, file write outside the mount) accept only `Untainted<T>`. Conversion goes through an explicit `sanitize` step that strips the immutable fields above. The type system enforces what the prose promises.
- [ ] **Audit log is append-only.** SQLite `audit` table has no UPDATE or DELETE triggers permitted; CI test asserts `pragma table_info` and a `BEFORE UPDATE/DELETE RAISE` trigger exists.
- [ ] **FUSE never blocks the kernel forever.** All upstream HTTP calls have a 5-second timeout; on timeout the daemon returns `EIO` (per `docs/research/initial-report.md` §"Graceful Degradation via POSIX Errors"), never hangs.
- [ ] **Demo recording must show guardrails firing.** The asciinema/script recording includes at least one allowlist refusal and one 409-conflict-as-merge-conflict resolution. A demo that only shows happy-path is dishonest about what reposix is.

*(JIRA integration v0.8.0 — all requirements shipped; see Validated above)*

### Out of Scope

- **Real Jira/GitHub/Confluence credentials in v0.1** — Simulator first. Real backends bolt on once the substrate is proven. Avoids credential exposure during overnight autonomous build. *(JIRA credentials now planned for v0.8.0 — see JIRA-01…JIRA-06 above.)*
- **Windows / macOS support in v0.1** — FUSE on Linux only. macOS via macFUSE is a follow-up; Windows needs a different VFS layer entirely.
- **Web UI** — agents don't need it; humans use the CLI + the underlying git repo.
- **Multi-tenant hosted service** — local-first only. The whole point is the agent talks to the local FS.
- **Pickle/binary serialization** — JSON + YAML only. Per simon-willison-style auditability.
- **Eager full sync of remote state** — lazy, on-demand fetches with caching. A naïve `grep -r` must not melt API quotas (per `docs/research/initial-report.md` §rate-limiting).

## Context

- **Why this exists.** From `docs/research/agentic-engineering-reference.md`: MCP burns 100k+ tokens on schema discovery before the first useful operation. POSIX is in the model's pre-training. A `cat /mnt/jira/PROJ-123.md` operation is ~2k tokens of context vs ~150k for the equivalent MCP-mediated read.
- **Reference materials.** `docs/research/initial-report.md` (architecture deep-dive on FUSE + git-remote-helper for agentic tooling) and `docs/research/agentic-engineering-reference.md` (Simon Willison interview distillation: dark factory pattern, lethal trifecta, simulator-first).
- **Inspiration projects.** `~/workspace/token_world` (Python, knowledge-graph as ground truth, CI discipline). `~/workspace/theact` (small-model RPG engine, observability tooling). `~/workspace/reeve_bot` (production Telegram bot stack).
- **Threat model.** This project is a textbook lethal trifecta: private remote data + untrusted ticket text + git-push exfiltration. Mitigations are first-class: tainted-content marking, audit log, no auto-push to unauthorized remotes, RBAC → POSIX permission translation.

## Constraints

- **Tech stack**: Rust 1.82+, Cargo workspace. No libfuse-dev linking — `fuser` with `default-features = false` so we don't need apt packages we can't install. Async via Tokio, HTTP via axum/reqwest, FFI via libc.
- **Timeline**: Demo by 2026-04-13 ~08:00 PDT. Hard limit. Project kicked off 2026-04-13 ~00:30 PDT. ~7 hours of autonomous build time.
- **Compatibility**: Linux only for v0.1 (FUSE3 + FUSE2 both available on host). CI runs on `ubuntu-latest`.
- **Security**: Cannot store real credentials. Cannot interact with private services. Simulator is the only backend until human review.
- **Dependencies**: Only crates that compile without `pkg-config` or system dev headers (we lack passwordless sudo on dev host). `fuser` default-features=false. `rusqlite` with `bundled` feature. `reqwest` with `rustls-tls` (no openssl-sys).
- **Ground truth**: All state in committed artifacts. Simulator audit DB committed to `runtime/` only as test fixtures, not source of truth.
- **Egress safety**: The single `reposix_core::http::client()` factory is the only legal way to construct an HTTP client in this workspace. Direct `reqwest::Client::new()` calls are denied by clippy lint (`clippy::disallowed_methods` configured in `clippy.toml`). Every HTTP request honors `REPOSIX_ALLOWED_ORIGINS`.
- **Decision deadline**: At local time 03:30 (T+3h from kickoff) the orchestrator MUST decide: shipping the full-write demo, or pivoting to read-only-mount + remote-helper for a credible minimum-viable demo. No sunk-cost grinding past 03:30 on a path that won't land by 07:30.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust over Python | Production substrate; FUSE perf matters under swarm load; user explicitly chose Rust | — Pending |
| Simulator-first, not real backend | StrongDM pattern: real APIs rate-limit a swarm to death; simulator is free to hammer; also avoids credential risk in autonomous mode | — Pending |
| `fuser` crate, `default-features = false` | No libfuse-dev / pkg-config available; runtime uses fusermount binary which is present | — Pending |
| `rusqlite` with `bundled` | Avoids needing libsqlite3-dev | — Pending |
| Workspace with 5 crates (`-core`, `-sim`, `-fuse`, `-remote`, `-cli`) | Clear separation of concerns; each crate independently testable; `-core` isolates types from binaries | — Pending |
| Issues as `.md` + YAML frontmatter | Matches `docs/research/initial-report.md` §"Modeling Hierarchical Directory Structures"; agents already understand the format | — Pending |
| `git-remote-helper` protocol over custom sync | Leverages git's conflict resolution (OP: ground truth, simon willison §5.2 lethal trifecta argues git semantics > JSON conflict synthesis) | — Pending |
| Public GitHub repo `reubenjohn/reposix` | User authorized; CI must run; demo must be shareable | — Pending |
| Auto/YOLO mode, coarse granularity, all workflow gates on | User asked for max autonomy + GSD discipline; coarse phases fit 7-hour window | — Pending |
| Skip GSD discuss step | User instruction (~12:55 AM): "do all the gsd planning, exec, review, etc, just without the discuss steps" | — Pending |
| Lethal-trifecta cuts are first-class requirements, not afterthoughts | Threat-model subagent flagged egress + bulk-delete + tainted typing as ship-blockers; safer to bake in than retrofit | — Pending |

## Current Milestone: v0.11.0 — Performance & Sales Assets (planning)

**Goal:** Land the helper-multi-backend-dispatch fix carried forward from v0.9.0 (so `stateless-connect` actually routes by URL host, not always to `SimBackend`); ship `cargo run -p reposix-bench` for honest per-backend latency numbers; close the 9 major + 17 minor doc-clarity findings logged in `.planning/notes/v0.11.0-doc-polish-backlog.md`; capture the playwright screenshots deferred from v0.10.0 once cairo libs are available; and complete the IssueId→RecordId refactor that's in flight on a parallel runner.

**Carry-forward from v0.10.0 (tech debt):**
- Playwright screenshots (DOCS-11 success criterion 4) — `scripts/take-screenshots.sh` stub names the contract.
- 9 major + 17 minor doc-clarity findings — `.planning/notes/v0.11.0-doc-polish-backlog.md`.
- `scripts/tag-v0.10.0.sh` — owner gate; not yet authored.

**Carry-forward from v0.9.0 (still open):**
- Helper hardcodes `SimBackend` in `stateless-connect` handler — must land before any v0.11.0 benchmark commits.

---

## Previously Validated Milestone: v0.10.0 — Docs & Narrative Shine (SHIPPED 2026-04-25)

**Goal:** Make the v0.9.0 architecture pivot legible. A cold visitor understands reposix's value proposition within 10 seconds of landing on the docs site, runs the 5-minute first-run tutorial against the simulator, and ends with a real edit committed and pushed via `reposix init` + standard git. The architecture pivot becomes a *story* (cache layer / git layer / trust model — three pages, each with one mermaid diagram rendered via mcp-mermaid and a playwright screenshot), not a code change.

**Target features (Phases 40–45 — see `.planning/ROADMAP.md` for the full breakdown, `.planning/research/v0.10.0-post-pivot/milestone-plan.md` for the design authority):**

- **Phase 40** — Hero rewrite of `docs/index.md` + two home-adjacent concept pages (`mental-model-in-60-seconds`, `reposix-vs-mcp-and-sdks`). Hero numbers sourced from `docs/benchmarks/v0.9.0-latency.md` (`8 ms` get-issue, `24 ms` `reposix init` cold, `9 ms` list, `5 ms` capabilities probe).
- **Phase 41** — How-it-works trio (`filesystem-layer`, `git-layer`, `trust-model`); each page one mermaid diagram + playwright screenshot, **reframed for git-native** (no FUSE / inode / daemon vocabulary above Layer 4).
- **Phase 42** — 5-minute first-run tutorial verified by `scripts/tutorial-runner.sh` (the doc IS the test). Three guides: write-your-own-connector, integrate-with-your-agent (pointer page; v0.12.0 ships full recipes), troubleshooting. Simulator relocated to Reference.
- **Phase 43** — MkDocs nav restructured per Diátaxis. mkdocs-material theme tuning. `scripts/banned-words-lint.sh` + `docs/.banned-words.toml` + `.claude/skills/reposix-banned-words/SKILL.md` ship together (institutional memory of P2 framing rules per OP-4, replacing ad-hoc grep).
- **Phase 44** — `doc-clarity-review` skill run as a release gate over every user-facing page (cold-reader scenario in isolation, OP-6); zero critical friction points required to ship.
- **Phase 45** — README hero rewrite (every adjective replaced with a measured number from `docs/benchmarks/v0.9.0-latency.md` or v0.9.0 audit/threat-model), CHANGELOG `[v0.10.0]`, social cards, playwright screenshots committed for landing + how-it-works + tutorial. `scripts/tag-v0.10.0.sh` mirrors v0.9.0 tag script (≥6 safety guards).

**Non-negotiable framing principles** (carried over from `.planning/notes/phase-30-narrative-vignettes.md` — these constraints apply to every DOCS-NN requirement):

- **P1 — complement, not replace.** reposix does not replace REST APIs. REST stays. reposix absorbs the ceremony around the 80% of tracker operations agents do constantly. The word "replace" is banned from hero and value-prop copy. Acceptable verbs: *complement, absorb, subsume, lift, erase the ceremony, no new vocabulary*.
- **P2 — progressive disclosure: phenomenology before implementation.** Layer 1 (hero, first 10 seconds): what the user experiences — issues are files, edit them, `git push`. Layer 2 (just below the fold): minimum mechanism to make the experience make sense. Layer 3 (How it works): technical reveal earned. Layer 4 (Reference / ADRs / Research): full depth. Banned-above-Layer-3 list is **revised for git-native**: `FUSE`, `fusermount`, `kernel`, `syscall` (all retained), plus the new git-native jargon `partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2` (new — these were introduced by v0.9.0 and would leak architecture above Layer 3 if unchecked).
- **Numbers, not adjectives.** Every adjective on the README hero and `docs/index.md` hero is replaced with a measured number sourced from a committed artifact (`docs/benchmarks/v0.9.0-latency.md`, v0.9.0 audit, threat-model). Enforced by `scripts/banned-words-lint.sh` (Phase 43).
- **Self-improving infrastructure (OP-4).** Banned-words linter, `reposix-banned-words` skill, and `tutorial-runner.sh` are committed code, not session memory. Per project CLAUDE.md: ad-hoc bash is a missing-tool signal — every grep that asserts a layer rule becomes a reviewable artifact.

**Source of truth:** `.planning/research/v0.10.0-post-pivot/milestone-plan.md` (design authority, drafted 2026-04-24). Original narrative IA: `.planning/notes/phase-30-narrative-vignettes.md` (framing principles + hero vignette V1 — must be revised for git-native architecture by Phase 41 carving).

**Carry-forward from v0.9.0 (tech debt):** Helper hardcodes `SimBackend` in the `stateless-connect` handler — documented in `.planning/v0.9.0-MILESTONE-AUDIT.md` §5 and the v0.9.0 ROADMAP archive. Resolution scheduled before any v0.11.0 ("Performance & Sales Assets") benchmark commits, NOT v0.10.0 (out-of-scope per active requirements section).

---

## Previously Validated Milestone: v0.9.0 — Architecture Pivot — Git-Native Partial Clone (SHIPPED 2026-04-24)

**Goal:** Replace the FUSE virtual filesystem with git's built-in partial clone mechanism. The `git-remote-reposix` helper becomes a promisor remote tunnelling protocol-v2 traffic to a local bare-repo cache built from REST responses. Agents interact with the project using only standard git commands — no reposix-specific CLI awareness. FUSE is deleted entirely; `crates/reposix-fuse/` is removed and the `fuser` dependency is purged. The pivot is operationalized across six phases (31–36) covering cache foundation, read transport, delta sync, push path, CLI pivot, and the demolition+release cycle.

**Target features:**
- DELETE `crates/reposix-fuse/` entirely; drop `fuser` dependency; remove `fuse-mount-tests` feature gate; CI no longer needs `apt install fuse3` or `/dev/fuse`.
- ADD `crates/reposix-cache/` — a new crate that materializes REST API responses into a local bare git repo. Lazy blobs, full tree, audit row per blob materialization, `Tainted<Vec<u8>>` return type, egress allowlist enforced.
- ADD `stateless-connect` capability to `git-remote-reposix` (alongside existing `export`) — protocol-v2 tunnel from git into the cache. Refspec namespace `refs/heads/*:refs/reposix/*`. Three protocol gotchas from POC encoded correctly in Rust.
- ADD `BackendConnector::list_changed_since(timestamp)` trait method + per-backend implementations (sim `?since=`, GitHub `?since=`, JIRA JQL `updated >=`, Confluence CQL `lastModified >`). Delta sync transfers only changed issues.
- ADD push-time conflict detection inside the `export` handler — emits canned `error refs/heads/main fetch first` so agents experience standard `git pull --rebase, retry` UX. Reject path drains the stream and never touches the bare cache.
- ADD blob-limit guardrail — helper counts `want` lines per `command=fetch`, refuses if > `REPOSIX_BLOB_LIMIT` (default 200), with a stderr error message that names `git sparse-checkout` so unprompted agents self-correct (dark-factory pattern).
- ADD frontmatter field allowlist on the push path — server-controlled fields (`id`, `created_at`, `version`, `updated_at`) stripped before REST.
- CHANGE CLI: `reposix mount <path>` -> `reposix init <backend>::<project> <path>` (runs `git init`, configures `extensions.partialClone`, sets remote URL, runs `git fetch --filter=blob:none origin`). Breaking change documented in CHANGELOG.
- ADD project Claude Code skill `reposix-agent-flow` at `.claude/skills/reposix-agent-flow/SKILL.md` — encodes the dark-factory autonomous-agent regression test (subprocess agent given only `reposix init` + a goal completes the work via pure git/POSIX). Invoked from CI and local dev.
- UPDATE project `CLAUDE.md` in lockstep with FUSE deletion (Phase 36) — no window where grounding describes deleted code; FUSE references purged from elevator pitch, Operating Principles, Workspace layout, Tech stack, Commands, Threat model.

**Non-negotiable framing principles** (carried over from CLAUDE.md project Operating Principles — these constraints apply to every ARCH-NN requirement):

- **Simulator-first.** Every demo, unit test, and autonomous agent loop runs against `reposix-sim` by default. Real backends (GitHub, Confluence, JIRA) require explicit credentials AND a non-default `REPOSIX_ALLOWED_ORIGINS`. Autonomous mode never hits a real backend.
- **Tainted-by-default.** Every byte materialized into the bare-repo cache or returned to git originated from a remote (real or simulator). The cache returns `Tainted<Vec<u8>>`; conversion to `Untainted` is the explicit `sanitize` step where the frontmatter field allowlist is enforced.
- **Audit log non-optional.** Every blob materialization, every `command=fetch`, every `export` push (accept and reject) writes a row to the SQLite audit table. SQLite WAL append-only — no UPDATE or DELETE on audit rows.
- **No hidden state.** Cache state and helper state live in committed-or-fixture artifacts. No "it works in my session" bugs.
- **Working tree = real git repo.** The mount point is no longer synthetic — it is a true git working tree backed by `.git/objects` (partial clone, blobs lazy). `git diff` is the canonical change set, by construction.
- **Self-improving infrastructure (OP-4).** Project `CLAUDE.md` and Claude Code skills ship in lockstep with the code that invalidates them. Phase 36 explicitly bundles agent-grounding updates with FUSE deletion.

**Source of truth:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (canonical design doc, 440 lines, ratified 2026-04-24). Supporting POC artifacts in `.planning/research/v0.9-fuse-to-git-native/poc/`.

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-25 — Milestone v0.10.0 SHIPPED (Docs & Narrative Shine, Phases 40–45); v0.11.0 Performance & Sales Assets planning started*
