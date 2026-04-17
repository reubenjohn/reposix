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

## Upcoming Milestone: v0.6.0 — Write Path + Full Sitemap

**Goal:** Turn the mount from a read-only navigator into a writable agent workspace.

**Target features:**
- Confluence write path (`create_issue` / `update_issue` / `delete_or_close` on `ConfluenceBackend`) + `atlas_doc_format` ↔ Markdown round-trip
- Swarm `--mode confluence-direct` mode using `SimDirectWorkload` as implementation template
- OP-2 remainder: tree-recursive `tree/<subdir>/_INDEX.md` synthesis + mount-root `_INDEX.md`
- OP-1 remainder: `labels/` and `spaces/` directory views as read-only symlink overlays (GitHub + Confluence)
- OP-3: `reposix refresh` subcommand + git-diff cache for mount-as-time-machine semantics

## Upcoming Milestone: v0.8.0 — JIRA Cloud Integration

**Goal:** First-class JIRA Cloud backend with trait rename, extensions field, and full read-only adapter. Write path as stretch.

**Target features:**
- `IssueBackend` → `BackendConnector` rename + `Issue.extensions: BTreeMap<String, String>`
- `reposix-jira` crate: JQL pagination, status-category mapping, subtask hierarchy, JIRA-specific extensions
- CLI: `list --backend jira`, `mount --backend jira --project <KEY>`
- ADR-004 (mapping rationale), `docs/reference/jira.md`
- (Stretch) Write path via Transitions API

## Current Milestone: v0.7.0 — Hardening + Confluence Expansion

**Goal:** Harden the platform under real-world load conditions and expand Confluence support beyond pages.

**Target features:**
- OP-7 hardening bundle: contention swarm (`--contention` mode), 500-page truncation probe + `WARN` + `--no-truncate`, chaos audit-log (kill-9 sim mid-swarm), macFUSE parity CI matrix
- OP-8 honest-tokenizer benchmarks: replace `len/4` with `count_tokens` API, per-backend comparison tables, cold-mount timing, git-push round-trip latency
- OP-9a: Confluence comments exposed as `pages/<id>.comments/<comment-id>.md`
- OP-9b: Confluence whiteboards, attachments, and folders
- OP-11: Docs reorg — `InitialReport.md` + `AgenticEngineeringReference.md` → `docs/research/` + root cleanup

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
*Last updated: 2026-04-16 — Milestone v0.8.0 complete (Phase 29 shipped, all JIRA requirements validated)*
