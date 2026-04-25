# Roadmap: reposix

## Milestones

- ✅ **v0.1.0 MVD** — Phases 1-4, S (shipped 2026-04-13) · [archive](milestones/v0.8.0-ROADMAP.md)
- ✅ **v0.2.0-alpha** — Phase 8: GitHub read-only adapter (shipped 2026-04-13)
- ✅ **v0.3.0** — Phase 11: Confluence Cloud read-only adapter (shipped 2026-04-14)
- ✅ **v0.4.0** — Phase 13: Nested mount layout pages/+tree/ (shipped 2026-04-14)
- ✅ **v0.5.0** — Phases 14-15: IssueBackend decoupling + bucket _INDEX.md (shipped 2026-04-14)
- ✅ **v0.6.0** — Phases 16-20: Write path + full sitemap (shipped 2026-04-14)
- ✅ **v0.7.0** — Phases 21-26: Hardening + Confluence expansion + docs (shipped 2026-04-16)
- ✅ **v0.8.0 JIRA Cloud Integration** — Phases 27-29 (shipped 2026-04-16)
- ✅ **v0.9.0 Architecture Pivot — Git-Native Partial Clone** — Phases 31–36 (shipped 2026-04-24) · [archive](milestones/v0.9.0-ROADMAP.md)
- 📋 **v0.10.0 Docs & Narrative Shine** — planning (see `.planning/research/v0.10.0-post-pivot/milestone-plan.md`; revives Phase 30's docs scope against the new git-native architecture)

## Phases

## v0.10.0 Docs & Narrative Shine (PLANNING)

> **Status:** scoping. The architecture pivot shipped in v0.9.0 (2026-04-24); v0.10.0 ports the deferred Phase 30 docs work onto the git-native design and adds tutorial / how-it-works / mental-model pages around the new flow. See `.planning/research/v0.10.0-post-pivot/milestone-plan.md` for the proposed phase breakdown (Phases 40–45).

**Thesis.** A cold visitor understands reposix in 10 seconds and runs the tutorial in 5 minutes. The architecture pivot becomes a story, not a code change.

**Carry-forward from v0.9.0 (tech debt):** Helper hardcodes `SimBackend` in the `stateless-connect` handler — documented in `.planning/v0.9.0-MILESTONE-AUDIT.md` §5. Resolution scheduled before v0.11.0 benchmark commits (track as a hotfix or v0.11.0 prereq).

<details>
<summary>✅ v0.9.0 Architecture Pivot (Phases 31–36) — SHIPPED 2026-04-24</summary>

## v0.9.0 Architecture Pivot — Git-Native Partial Clone

**Motivation:** The FUSE-based design is fundamentally slow (every `cat`/`ls` triggers a live REST API call) and doesn't scale (10k Confluence pages = 10k API calls on directory listing). FUSE also has operational pain: fusermount3, /dev/fuse permissions, WSL2 quirks, pkg-config/libfuse-dev build dependencies. Research confirmed that git's built-in partial clone + the existing `git-remote-reposix` helper can replace FUSE entirely, giving agents a standard git workflow with zero custom CLI awareness required.

**Research:** See `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (canonical design document), `partial-clone-remote-helper-findings.md` (transport layer POC), `push-path-stateless-connect-findings.md` (write path POC), `sync-conflict-design.md` (sync model). POC code in `poc/` subdir (`git-remote-poc.py`, `run-poc.sh`, `run-poc-push.sh`, trace logs).

**Key design decisions:**
- DELETE `crates/reposix-fuse` entirely; drop `fuser` dependency
- ADD `stateless-connect` capability to `git-remote-reposix` for partial-clone reads
- KEEP `export` capability for push (hybrid confirmed working in POC)
- ADD `reposix-cache` crate: backing bare-repo cache built from REST responses
- Agent UX is pure git: `git clone`, `cat`, `git push` — zero reposix CLI awareness
- Push-time conflict detection: helper checks backend state at push time, rejects with standard git error
- Blob limit guardrail: helper refuses to serve >N blobs, error message teaches agent to use sparse-checkout
- Tree sync always full (cheap metadata); blob materialization is the only limited/lazy operation
- Delta sync via `since` queries (all backends support this natively)

**Phases (31–36):**
1. Phase 31 — `reposix-cache` crate (bare-repo cache from REST responses, audit + tainted + allowlist)
2. Phase 32 — `stateless-connect` capability in `git-remote-reposix` (read path; protocol-v2 tunnel)
3. Phase 33 — Delta sync (`list_changed_since` on `BackendConnector` + cache integration)
4. Phase 34 — Push path (conflict detection + blob limit + frontmatter allowlist)
5. Phase 35 — CLI pivot (`reposix init`) + dark-factory agent UX validation
6. Phase 36 — FUSE deletion + CLAUDE.md update + `reposix-agent-flow` skill + release

### Phase 31: `reposix-cache` crate — backing bare-repo cache from REST responses (v0.9.0)

**Goal:** Land the foundation crate that materializes REST API responses into a real on-disk bare git repo. The cache is the substrate every later phase builds on. Operating-principle hooks for this phase: **audit log non-optional** (one row per blob materialization); **tainted-by-default** (cache returns `Tainted<Vec<u8>>` — the type system encodes the trust boundary); **egress allowlist** (no new HTTP client construction outside `reposix_core::http::client()`); **simulator-first** (every test in this crate runs against `SimBackend`). Per project CLAUDE.md "Subagent delegation rules": use `gsd-phase-researcher` for any "how do I build a bare git repo from raw blobs in Rust" question — non-trivial, easy to over-research in the orchestrator.

**Requirements:** ARCH-01, ARCH-02, ARCH-03

**Depends on:** (nothing — foundation phase)

**Success criteria:**
1. `cargo build -p reposix-cache` and `cargo clippy -p reposix-cache --all-targets -- -D warnings` clean.
2. Given a `SimBackend` seeded with N issues, `reposix_cache::Cache::build_from(backend)` produces a valid bare git repo on disk containing N blobs (lazy — only materialized on demand) and a tree object that lists every issue path.
3. Audit table contains exactly one `op="materialize"` row per blob materialization (test seeds N issues, materializes M blobs, asserts `count(*) == M`).
4. Cache returns blob bytes wrapped in `reposix_core::Tainted<Vec<u8>>`; a compile-fail test asserts that calling `egress::send(blob)` without `sanitize` is a type error.
5. Egress allowlist test: pointing the cache at a backend whose origin is not in `REPOSIX_ALLOWED_ORIGINS` returns an error and writes an audit row with `op="egress_denied"`.
6. SQLite audit table is append-only — `BEFORE UPDATE/DELETE RAISE` trigger asserted by integration test.

**Context anchor:** `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` §2 (How it works), §5 (Add — `reposix-cache` crate), §6 (What stays the same — `BackendConnector` trait reused), and §7 open question 2 (atomicity of REST write + bare-repo cache update — implementation note for this phase).

**Plans:** 3 plans across 3 waves

- [ ] 31-01-PLAN.md — Wave 1: reposix-cache crate scaffold + gix 0.82 smoke + Cache::build_from with lazy tree (ARCH-01)
- [ ] 31-02-PLAN.md — Wave 2: cache_schema.sql + audit/db/meta modules + Cache::read_blob (Tainted + egress-denial audit) + lift cache_db.rs from reposix-cli (ARCH-02, ARCH-03)
- [ ] 31-03-PLAN.md — Wave 3: trybuild compile-fail fixtures — Tainted→Untainted + Untainted::new pub(crate) locks (ARCH-02)

### Phase 32: `stateless-connect` capability in `git-remote-reposix` (read path) (v0.9.0)

**Goal:** Port the Python POC's `stateless-connect` handler to Rust inside `crates/reposix-remote/`. Tunnel protocol-v2 traffic to the Phase 31 cache so `git clone --filter=blob:none reposix::sim/proj-1 /tmp/clone` works end-to-end with lazy blob loading. The existing `export` capability for push must keep working in the same binary (hybrid). Operating-principle hooks: **subagent delegation per project CLAUDE.md** — use `gsd-phase-researcher` for the protocol-v2 stateless-connect Rust port (non-trivial; three protocol gotchas from POC must be encoded correctly or git misframes the next request); **ground truth obsession** — verify against a real `git clone` run, not against unit-test mocks; **close the feedback loop** — capture a fresh trace log analogous to POC `poc-helper-trace.log` and commit it under `.planning/research/v0.9-fuse-to-git-native/rust-port-trace.log`.

**Requirements:** ARCH-04, ARCH-05

**Depends on:** Phase 31

**Success criteria:**
1. `git clone --filter=blob:none reposix::sim/proj-1 /tmp/clone` succeeds with all blobs missing (assertable via `git rev-list --objects --missing=print --all`).
2. Lazy blob fetch on `git cat-file -p <oid>` hits the backend exactly once per OID (idempotent — second `cat-file` is local-only; assertable via audit-row count).
3. `git checkout origin/main` after `git sparse-checkout set issues/PROJ-24*` batches blob fetches into a single `command=fetch` RPC (assertable: helper records exactly one `command=fetch` audit row with multiple `want` lines, not N rows with one `want` each).
4. Refspec namespace is `refs/heads/*:refs/reposix/*` (regression test that `refs/heads/*:refs/heads/*` would cause empty-delta bug per POC).
5. The same helper binary still services `git push` via `export` (hybrid POC parity). Existing v0.8.0 push tests pass unchanged.
6. Three protocol gotchas (initial advert no `0002`; subsequent responses DO need `0002`; binary stdin throughout) are covered by named tests.

**Context anchor:** architecture-pivot-summary §3 (Confirmed Technical Findings — `stateless-connect`, transport routing, three protocol gotchas, refspec namespace). POC artifacts: `.planning/research/v0.9-fuse-to-git-native/poc/git-remote-poc.py`, `poc-helper-trace.log`, `run-poc.sh`.

### Phase 33: Delta sync — `list_changed_since` on `BackendConnector` + cache integration (v0.9.0)

**Goal:** Add incremental backend queries so `git fetch` after a backend mutation transfers only the changed issue's tree+blob, not the whole project. Wire `last_fetched_at` (already present in `crates/reposix-cli/src/cache_db.rs`) into the new `reposix-cache` crate, and update it atomically with each delta sync. Operating-principle hooks: **simulator-first** (sim respects `since` query param; all delta-sync tests use sim); **audit log non-optional** (one audit row per delta-sync invocation); **ground truth obsession** (test asserts that after a single backend mutation, exactly one issue's blob OID changes — not all of them).

**Requirements:** ARCH-06, ARCH-07

**Depends on:** Phase 31, Phase 32

**Success criteria:**
1. `BackendConnector::list_changed_since(timestamp) -> Vec<IssueId>` defined on the trait and implemented for `SimBackend`, `GithubBackend`, `ConfluenceBackend`, `JiraBackend`. Each backend uses its native incremental query (`?since=`, JQL `updated >=`, CQL `lastModified >`).
2. `SimBackend` REST surface respects a `since` query parameter (if absent, returns all — backwards compatible).
3. After `agent_a` mutates issue `proj-1/42` on the simulator and `agent_b` runs `git fetch origin`, `git diff --name-only origin/main` returns exactly `issues/42.md`. Other blob OIDs are unchanged.
4. Tree sync is unconditional (not gated by `REPOSIX_BLOB_LIMIT`); the limit only applies to blob materialization.
5. Cache update + `last_fetched_at` write happen in one SQLite transaction (kill-9 chaos test asserts no divergent state — borrows the Phase 21 HARD-03 chaos pattern).
6. One audit row per delta-sync invocation: `(ts, backend, project, since_ts, items_returned, op="delta_sync")`.

**Context anchor:** architecture-pivot-summary §4 (Sync and Conflict Model — delta sync via `since` queries, fetch flow, agent-sees-changes-via-pure-git). Existing `cache_db.rs` `refresh_meta` row is the storage location for `last_fetched_at`.

### Phase 34: Push path — conflict detection + blob limit guardrail (v0.9.0)

**Goal:** Make the `export` handler conflict-aware and the `stateless-connect` handler scope-bounded. Push-time conflict detection rejects stale-base pushes with a canned `fetch first` git status so agents experience the standard "git pull --rebase, retry" cycle without learning anything new. Blob-limit guardrail caps `command=fetch` size so a runaway `git grep` cannot melt API quotas — and the stderr message names `git sparse-checkout` so an unprompted agent self-corrects (dark-factory pattern). Operating-principle hooks: **tainted-by-default** (frontmatter sanitize step is the explicit `Tainted -> Untainted` conversion); **audit log non-optional** (every push attempt — accept and reject — gets an audit row); **ROI awareness** (blob-limit error message is the cheapest possible regression net for "agent does naive `git grep`").

**Requirements:** ARCH-08, ARCH-09, ARCH-10

**Depends on:** Phase 32

**Success criteria:**
1. Stale-base push: agent pushes a commit whose base differs from the current backend version. Helper emits `error refs/heads/main fetch first` (canned status, git renders the standard "perhaps a `git pull` would help" hint) and a detailed diagnostic via stderr through `diag()`. Reject path drains the incoming stream and never touches the bare cache (no partial state — assertable: `git fsck` clean after reject).
2. Successful push: REST writes apply, bare-repo cache updates, helper emits `ok refs/heads/main`. REST + cache update is atomic (kill-9 between REST and cache leaves state consistent — same chaos pattern as Phase 33).
3. Frontmatter field allowlist: an issue body with `version: 999999` in frontmatter does not change the server version; `id`, `created_at`, `updated_at` are likewise stripped. Asserted by named test.
4. Blob limit: a `command=fetch` request with > `REPOSIX_BLOB_LIMIT` `want` lines (default 200) is refused. Helper's stderr message is verbatim: `error: refusing to fetch <N> blobs (limit: <M>). Narrow your scope with \`git sparse-checkout set <pathspec>\` and retry.`
5. `REPOSIX_BLOB_LIMIT` env var is read at helper startup; integration test asserts that setting it to `5` causes a 6-want fetch to fail and a 5-want fetch to succeed.
6. Audit row for every push attempt, accept and reject: `(ts, backend, project, ref, files_touched, decision, reason)`.

**Context anchor:** architecture-pivot-summary §3 ("Helper can count want lines and refuse", "Push rejection format", "Conflict detection happens inside `handle_export`"), §4 ("Blob limit as teaching mechanism"), §7 open question 2 (REST + cache atomicity). POC artifacts: `.planning/research/v0.9-fuse-to-git-native/poc/git-remote-poc.py` (push reject path), `poc-push-trace.log`.

### Phase 35: CLI pivot — `reposix init` replacing `reposix mount` + agent UX validation (v0.9.0)

**Goal:** Replace the `reposix mount` command with `reposix init <backend>::<project> <path>` (which `git init`s, configures `extensions.partialClone`, sets the remote URL, and runs `git fetch --filter=blob:none origin`). Then run the dark-factory acceptance test: a fresh subprocess agent with no reposix CLI awareness completes a clone -> grep -> edit -> commit -> push -> conflict -> pull --rebase -> push cycle against the simulator without invoking any `reposix` subcommand other than `init`. The dark-factory regression must run against BOTH the simulator AND at least one real backend: Confluence "TokenWorld" space, GitHub `reubenjohn/reposix` issues, or JIRA project `TEST` (credentials permitting). Latency for each step of the golden path (clone, first-blob, sparse-batched checkout, edit, push, conflict, pull-rebase, push-again) is captured and asserted against soft thresholds. Operating-principle hooks: **agent UX = pure git** (zero in-context learning required); **close the feedback loop** (acceptance test runs in CI and on local dev via the Phase 36 skill); **ground truth obsession** (the agent's transcript is captured as a test fixture so regressions are visible in `git diff`); **real backends are first-class test targets** (per project CLAUDE.md OP-6 — simulator-only coverage does NOT satisfy transport/perf acceptance).

**Requirements:** ARCH-11, ARCH-12, ARCH-16, ARCH-17 (capture)

**Depends on:** Phase 31, Phase 32, Phase 33, Phase 34

**Success criteria:**
1. `reposix init sim::proj-1 /tmp/repo` produces a directory containing a valid partial-clone working tree (`git rev-parse --is-inside-work-tree` returns true; `git config remote.origin.url` returns `reposix::sim/proj-1`; `git config extensions.partialClone` is set; `.git/objects` has tree objects but no blob objects until `git checkout` runs).
2. `reposix mount` is removed from the CLI; running it prints a helpful migration message pointing at `reposix init`.
3. CHANGELOG `[v0.9.0]` section documents the breaking CLI change with a migration note (`reposix mount /path` -> `reposix init <backend>::<project> /path`).
4. README.md updated to use `reposix init` everywhere.
5. **Dark-factory regression test (the headline acceptance test):** a subprocess Claude (or scripted shell agent acting as one) given ONLY a `reposix init` command + a goal ("find issues mentioning 'database' and add a TODO comment to each") completes the task using pure git/POSIX tools. The transcript exercises:
   - `cat`, `grep -r`, edit, `git add`, `git commit`, `git push` — happy path.
   - Conflict path: a second writer mutates one of the agent's target issues mid-flight; agent sees `! [remote rejected]`, runs `git pull --rebase`, retries `git push`, succeeds.
   - Blob-limit path: a naive `git grep` triggers the Phase 34 blob-limit error; agent reads the error message, runs `git sparse-checkout set issues/PROJ-24*`, retries, succeeds.
6. The transcript above is committed as a test fixture so any regression that breaks the dark-factory flow shows up in `git diff`.
7. Real-backend integration run passes against ≥1 of {Confluence TokenWorld, GitHub `reubenjohn/reposix`, JIRA `TEST`} when credentials present. Falls back to `#[ignore]` skip when absent, with a clear WARN that the v0.9.0 claim is unverified for that backend.
8. Latency captured for each golden-path step (clone, first-blob, sparse-batched checkout, edit, push, conflict, pull-rebase, push-again); written to `docs/benchmarks/v0.9.0-latency.md`. Soft thresholds asserted (sim cold clone < 500ms, real backend < 3s); regressions flagged but not CI-blocking.
9. `docs/reference/testing-targets.md` created documenting the three canonical targets (TokenWorld, `reubenjohn/reposix`, JIRA `TEST`) with env-var setup and the explicit "go crazy, it's safe" permission statement from the owner.

**Context anchor:** architecture-pivot-summary §4 ("Agent UX: pure git, zero in-context learning", "Blob limit as teaching mechanism"), §5 (Change — CLI flow). The acceptance test is the operationalization of architecture-pivot-summary §4's "agent learns from any tool error" claim. Project CLAUDE.md OP-6 (real backends as first-class test targets) defines the canonical TokenWorld / `reubenjohn/reposix` / JIRA `TEST` targets exercised here.

### Phase 36: FUSE deletion + CLAUDE.md update + `reposix-agent-flow` skill + final integration tests + release (v0.9.0)

**Goal:** Demolish FUSE entirely and ship v0.9.0. Per OP-4 self-improving infrastructure: **this phase updates project CLAUDE.md and adds the `reposix-agent-flow` skill — agent grounding must ship in lockstep with code**. There can be no window where CLAUDE.md describes deleted code, and no window where the project lacks the dark-factory regression skill that the v0.9.0 architecture is supposed to enable. Operating-principle hooks: **self-improving infrastructure (OP-4)** — CLAUDE.md + skill ship together with FUSE deletion; **close the feedback loop (OP-1)** — `gh run view` on the release tag must show green CI without the `apt install fuse3` step; **reversibility enables boldness (OP-5)** — execute via `gsd-pr-branch` or worktree so a botched FUSE deletion can be reverted in one move.

**Requirements:** ARCH-13, ARCH-14, ARCH-15, ARCH-17 (artifact), ARCH-18, ARCH-19

**Depends on:** Phase 35

**Success criteria:**
1. `crates/reposix-fuse/` is deleted (zero references in `cargo metadata --format-version 1` output).
2. `fuser` is removed from every `Cargo.toml` in the workspace (assertable: `grep -r '\bfuser\b' Cargo.toml crates/*/Cargo.toml` returns empty).
3. `cargo check --workspace && cargo clippy --workspace --all-targets -- -D warnings` clean.
4. CI workflow updated: drops `cargo test --features fuse-mount-tests`, drops `apt install fuse3`, drops `/dev/fuse` requirement. `gh run view` on the resulting commit shows green.
5. Project `CLAUDE.md` fully rewritten for git-native architecture per ARCH-14: no v0.9.0-in-progress banner — replaced with steady-state "Architecture (git-native partial clone)" section; FUSE references purged from elevator pitch, Operating Principles, Workspace layout, Tech stack, Commands, Threat model. `git grep -i 'fuser\|fusermount\|fuse-mount-tests\|reposix mount' CLAUDE.md` returns empty.
6. Skill `reposix-agent-flow` created at `.claude/skills/reposix-agent-flow/SKILL.md` with frontmatter `name: reposix-agent-flow` and `description: <one-line description referencing the dark-factory regression test>`. Skill body documents the test pattern and references architecture-pivot-summary §4. Skill is invoked from CI (release-gate job) and from local dev (`/reposix-agent-flow`).
7. `scripts/tag-v0.9.0.sh` created mirroring `scripts/tag-v0.8.0.sh` (6 safety guards minimum: clean tree, on `main`, version match in `Cargo.toml`, CHANGELOG `[v0.9.0]` exists, tests green, signed tag).
8. CHANGELOG `[v0.9.0]` section is finalized with all six phases summarized + breaking-change migration note (`reposix mount` -> `reposix init`).
9. Phase 35's dark-factory regression test (now invoked via the new skill) passes against the post-deletion codebase.
10. CI jobs `integration-contract-{confluence,github,jira}-v09` green on main (or `pending-secrets` when creds unavailable). Each job runs the ARCH-16 smoke suite and uploads latency rows as a run artifact.
11. Benchmark artifact `docs/benchmarks/v0.9.0-latency.md` includes a sim column AND at least one real-backend column (TokenWorld / `reubenjohn/reposix` / JIRA `TEST`). Soft thresholds documented; regressions flagged inline.
12. CLAUDE.md "Commands you'll actually use" section gains a "Testing against real backends" block naming TokenWorld / `reubenjohn/reposix` / JIRA `TEST` with env-var setup. CLAUDE.md OP-6 cross-references `docs/reference/testing-targets.md`.

**Context anchor:** architecture-pivot-summary §5 (Delete — `crates/reposix-fuse`, `fuser` dependency), §9 (Milestone Impact). Project `CLAUDE.md` "Subagent delegation rules" section. User global `CLAUDE.md` OP-4 "Self-improving infrastructure".

</details>

---

## v0.10.0 Docs & Narrative (carries Phase 30 forward against the new architecture)

> **Status:** Phase 30 was originally scoped against the FUSE design. Now that v0.9.0 has shipped the git-native architecture, Phase 30 (and the broader Docs & Narrative Shine milestone, Phases 40–45 per `.planning/research/v0.10.0-post-pivot/milestone-plan.md`) will execute against the new flow. Banned-word linter rules will be revised: `FUSE`, `inode`, `daemon`, `mount`, `fusermount` removed; new banned-above-Layer-3 list = `partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2`.

### Phase 30: Docs IA and narrative overhaul — landing page aha moment and progressive-disclosure architecture reveal (v0.10.0)

**Goal:** Rewrite the landing page and restructure the MkDocs nav so reposix's value proposition lands hard within 10 seconds of a cold reader arriving, with technical architecture progressively revealed in a "How it works" section rather than leaked above the fold. Expand the site from a correct reference tree into a substrate story that explains *why*, *how*, and *how to extend*.

**Requirements:** DOCS-01, DOCS-02, DOCS-03, DOCS-04, DOCS-05, DOCS-06, DOCS-07, DOCS-08, DOCS-09

**Depends on:** v0.9.0 architecture pivot (Phase 30 plans need revision to describe git-native design, not FUSE)

**Success criteria:**
1. A cold reader arriving at the MkDocs site can state reposix's value proposition within 10 seconds (validated via user-proxy review).
2. P2 banned terms (FUSE, inode, daemon, helper, kernel, mount, syscall) do not appear above the "How it works" layer in any page (banned-word linter PASS).
3. `mkdocs build --strict` returns green.
4. Playwright screenshots of landing + how-it-works + tutorial pages at desktop (1280px) and mobile (375px) widths are committed to the phase SUMMARY.
5. `gh run view` shows green CI on the milestone commit.

**Context anchor:** `.planning/notes/phase-30-narrative-vignettes.md` (source of truth for narrative intent, framing principles P1/P2, hero vignette, IA sketch). `.planning/phases/30-.../CONTEXT.md` summarizes for the planner.

**Plans:** 9 plans across 5 waves (NEED REVISION — currently describe FUSE architecture)

- [ ] 30-01-PLAN.md — Wave 0: Vale + tooling install + CI integration + pre-commit hook + structure/screenshot/mermaid scripts (DOCS-09)
- [ ] 30-02-PLAN.md — Wave 0: Page skeletons (14 new pages + 2 .gitkeep) so Wave 1 nav doesn't dangle (DOCS-02, DOCS-03, DOCS-04, DOCS-05, DOCS-06)
- [ ] 30-03-PLAN.md — Wave 1: Hero rewrite of docs/index.md + mental-model + vs-mcp-sdks filled (DOCS-01, DOCS-03)
- [ ] 30-04-PLAN.md — Wave 1: mkdocs.yml nav restructure + theme tuning + social plugin (DOCS-07, DOCS-08)
- [ ] 30-05-PLAN.md — Wave 2: How-it-works carver (filesystem + git + trust-model) from architecture.md + security.md (DOCS-02)
- [ ] 30-06-PLAN.md — Wave 1: Tutorial author + end-to-end runner against simulator (DOCS-06)
- [ ] 30-07-PLAN.md — Wave 2: Guides (write-your-own-connector move + integrate-with-your-agent + troubleshooting) + reference/simulator fill (DOCS-04, DOCS-05)
- [ ] 30-08-PLAN.md — Wave 3: Grep-audit + delete architecture.md/security.md/demo.md/connectors/ + update README + clean mkdocs.yml not_in_nav (DOCS-07)
- [ ] 30-09-PLAN.md — Wave 4: Verification (mkdocs/vale/tutorial/structure) + 14 playwright screenshots + doc-clarity-review cold-reader + CHANGELOG v0.9.0 + SUMMARY (DOCS-01..09)

<details>
<summary>✅ v0.1.0–v0.7.0 (Phases 1–26) — SHIPPED 2026-04-13 through 2026-04-16</summary>

All details archived in `.planning/milestones/v0.8.0-ROADMAP.md`.

Summary:
- Phases 1–4, S: MVD simulator + FUSE + CLI + demo + write path + swarm
- Phase 8: GitHub read-only adapter + IssueBackend trait + contract tests
- Phase 11: Confluence Cloud read-only adapter + wiremock + ADR-002
- Phase 13: Nested mount layout (pages/ + tree/ symlinks)
- Phase 14: IssueBackend decoupling (FUSE write path + git-remote)
- Phase 15: Dynamic _INDEX.md in bucket dir (OP-2 partial)
- Phase 16: Confluence write path (create/update/delete + ADF↔Markdown)
- Phase 17: Swarm confluence-direct mode
- Phase 18: OP-2 remainder (tree-recursive + mount-root _INDEX.md)
- Phase 19: OP-1 remainder (labels/ symlink overlay)
- Phase 20: OP-3 (reposix refresh subcommand + git-diff cache)
- Phase 21: OP-7 hardening (contention swarm, truncation probe, chaos audit, macFUSE CI)
- Phase 22: OP-8 honest-tokenizer benchmarks
- Phase 23: OP-9a Confluence comments (pages/<id>.comments/)
- Phase 24: OP-9b Confluence whiteboards/attachments/folders
- Phase 25: OP-11 docs reorg (research/ migration)
- Phase 26: Docs clarity overhaul (doc-clarity-review, version sync)

</details>

<details>
<summary>✅ v0.8.0 JIRA Cloud Integration (Phases 27–29) — SHIPPED 2026-04-16</summary>

### Phase 27: Foundation — `IssueBackend` → `BackendConnector` rename + `Issue.extensions` field (v0.8.0)

**Goal:** Hard rename `IssueBackend` → `BackendConnector` across all crates + ADR-004. Add `Issue.extensions: BTreeMap<String, serde_yaml::Value>` for typed backend metadata.
**Plans:** 3/3 plans complete

- [x] 27-01-PLAN.md — IssueBackend → BackendConnector rename (SHIPPED)
- [x] 27-02-PLAN.md — BackendConnector rename propagation across workspace (SHIPPED)
- [x] 27-03-PLAN.md — Issue.extensions field + ADR-004 + v0.8.0 + CHANGELOG (SHIPPED)

### Phase 28: JIRA Cloud read-only adapter (`reposix-jira`) (v0.8.0)

**Goal:** First-class JIRA Cloud backend. JQL pagination, status-category mapping, subtask hierarchy, JIRA-specific extensions, CLI dispatch, contract tests, ADR-005, docs/reference/jira.md.
**Plans:** 3/3 plans complete

- [x] 28-01-PLAN.md — JiraBackend core adapter + 17 tests (SHIPPED)
- [x] 28-02-PLAN.md — CLI integration + contract tests (SHIPPED)
- [x] 28-03-PLAN.md — ADR-005 + jira.md + CHANGELOG v0.8.0 + tag script (SHIPPED)

### Phase 29: JIRA write path — `create_issue`, `update_issue`, `delete_or_close` via Transitions API (stretch) (v0.8.0)

**Goal:** Complete the JIRA write path. `create_issue` → POST, `update_issue` → PUT, `delete_or_close` → Transitions API with DELETE fallback. Audit log for all mutations.
**Requirements:** JIRA-06
**Plans:** 3/3 plans complete

- [x] 29-01-PLAN.md — ADF helpers + create_issue (SHIPPED)
- [x] 29-02-PLAN.md — update_issue + audit rows (SHIPPED)
- [x] 29-03-PLAN.md — delete_or_close transitions + contract test (SHIPPED)

</details>

## Backlog

### Phase 999.1: Follow-up — missing SUMMARY.md files from prior phases (BACKLOG)

**Goal:** Resolve plans that ran without producing summaries during earlier phase executions
**Deferred at:** 2026-04-16 during /gsd-next advancement to /gsd-verify-work (Phase 29 → milestone completion)
**Plans:**
- [ ] Phase 16: 16-D-docs-and-release (ran, no SUMMARY.md)
- [ ] Phase 17: 17-A-workload-and-cli (ran, no SUMMARY.md)
- [ ] Phase 17: 17-B-tests-and-docs (ran, no SUMMARY.md)
- [ ] Phase 18: 18-02 (ran, no SUMMARY.md)
- [ ] Phase 21: 21-A-audit (ran, no SUMMARY.md)
- [ ] Phase 21: 21-B-contention (ran, no SUMMARY.md)
- [ ] Phase 21: 21-C-truncation (ran, no SUMMARY.md)
- [ ] Phase 21: 21-D-chaos (ran, no SUMMARY.md)
- [ ] Phase 21: 21-E-macos (ran, no SUMMARY.md)
- [ ] Phase 22: 22-A-bench-upgrade (ran, no SUMMARY.md)
- [ ] Phase 22: 22-B-fixtures-and-table (ran, no SUMMARY.md)
- [ ] Phase 22: 22-C-wire-docs-ship (ran, no SUMMARY.md)
- [ ] Phase 25: 25-02 (ran, no SUMMARY.md)
- [ ] Phase 27: 27-02 (ran, no SUMMARY.md)
