# Milestones — reposix

## v0.8.0 JIRA Cloud Integration (Shipped: 2026-04-16)

**Phases:** 27–29 (3 phases, 9 plans)
**Commits:** `ba6f9bc`..`8eca6a0` (code) + docs/planning
**Timeline:** 2026-04-15 → 2026-04-16
**Tests:** workspace suite green, 0 failures

**Key accomplishments:**

- `IssueBackend` → `BackendConnector` hard rename across all 5 crates + ADR-004 (Phase 27)
- `Issue.extensions: BTreeMap<String, serde_yaml::Value>` for typed backend metadata (Phase 27)
- `reposix-jira` crate: JQL pagination, status-category mapping, subtask hierarchy, JIRA-specific extensions (Phase 28)
- CLI: `list --backend jira`, `mount --backend jira --project <KEY>`; env vars: `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE` (Phase 28)
- ADR-005 (JIRA issue mapping), `docs/reference/jira.md`, `scripts/tag-v0.8.0.sh` (Phase 28)
- JIRA write path: `create_issue` (POST + ADF + issuetype OnceLock cache), `update_issue` (PUT), `delete_or_close` (Transitions API + DELETE fallback) — 31 unit + 5 contract tests (Phase 29)
- Audit log rows for all JIRA mutations; API token never logged (Phase 29)

**UAT:** 9/9 passed (all automated via wiremock + cargo test)

---

## v0.7.0 — Hardening + Confluence Expansion (SHIPPED 2026-04-16)

**Goal:** Harden the platform under real-world load conditions and expand Confluence support beyond pages.

**Shipped:**

- Phase 21: OP-7 hardening bundle — HARD-00 audit (pre-push hook + SSRF tests), HARD-01 ContentionWorkload proving If-Match 409 determinism, HARD-02 `list_issues_strict` + `--no-truncate` 500-page truncation probe, HARD-03 kill-9 chaos audit-log integrity test, HARD-04 macOS CI matrix + hooks CI step (macos-blocked path), HARD-05 tenant-URL redaction.
- Phase 22: OP-8 honest-tokenizer benchmarks — replaced `len(text) // 4` with Anthropic `count_tokens` API; SHA-256 cached fixtures for offline CI; per-backend comparison table; `docs/why.md` headline updated from 91.6% (estimate) to 89.1% (measured).
- Phase 23: OP-9a Confluence comments — `pages/<id>.comments/<comment-id>.md` synthesized subdirs; `ConfluenceBackend::list_comments` + `list_spaces`; `reposix spaces --backend confluence` CLI subcommand.
- Phase 24: OP-9b Confluence whiteboards + attachments + folders — `whiteboards/<id>.json`, `pages/<id>.attachments/<filename>` (sanitized + 50 MiB cap); `translate()` folder-parented hierarchy fix for `tree/` overlay; `list_attachments` + `list_whiteboards` + `download_attachment` methods.
- Phase 25: OP-11 docs reorg — moved `InitialReport.md` + `AgenticEngineeringReference.md` to `docs/research/`; root redirect stubs; Research nav section in mkdocs.yml; workspace version bump `0.6.0 → 0.7.0`.

**Tests:** 397/397 green at Phase 24 close (workspace). Phase 21 green-gauntlet ~355 tests, 10 `#[ignore]`-gated; Phase 22 9/9 pytest cases pass offline.

**UAT:** Phase 23 + Phase 24 each emitted HUMAN-UAT.md items for live-tenant confirmation of `.comments/` and `.attachments/` mount behavior; automated coverage complete.

---

## v0.6.0 — Write Path + Full Sitemap (SHIPPED 2026-04-14)

**Goal:** Turn the mount from a read-only navigator into a writable agent workspace.

**Shipped:**

- Phase 16: Confluence write path — `ConfluenceBackend::create_issue`, `update_issue`, `delete_or_close` against Confluence Cloud REST v2 (`/wiki/api/v2/pages`); ADF↔Markdown converter (`adf.rs`); client-side audit log via SG-06; read path switched to `atlas_doc_format` with `storage` fallback. Closes WRITE-01..04. Version bump `0.5.0 → 0.6.0`.
- Phase 17: Swarm `--mode confluence-direct` — N read-only clients against `ConfluenceBackend` (list + 3×get per cycle); wiremock CI test + `#[ignore]`-gated real-tenant smoke. Closes SWARM-01, SWARM-02.
- Phase 18: OP-2 remainder — `mount/tree/<subdir>/_INDEX.md` recursive DFS sitemap + `mount/_INDEX.md` whole-mount overview; `ROOT_INDEX_INO = 6` + dynamic tree-index inode allocation. Completes OP-2 started in Phase 15.
- Phase 19: OP-1 remainder — `mount/labels/<label>/` read-only symlink overlay pointing to canonical bucket files; slug + dedupe safety; `.gitignore` + `_INDEX.md` updates. Closes LABEL-01..05 (spaces deferred to Phase 20).
- Phase 20: OP-3 `reposix refresh` subcommand — fetches all issues/pages, writes deterministic `.md` files into mount's git tree, creates a commit so `git diff HEAD~1` shows backend-side changes; `.reposix/cache.db` (SQLite WAL + EXCLUSIVE lock, mode 0600); active-mount detection via `.reposix/fuse.pid`. Closes REFRESH-01..05.
- Phase 26: Docs clarity overhaul — cold-reader clarity review of every user-facing Markdown page (README, HANDOFF, docs/index, architecture, why, security, demo, reference/*, connectors/guide, decisions/*, research/*); archived obsolete root files (`MORNING-BRIEF.md`, `PROJECT-STATUS.md`) to `docs/archive/`; deleted redirect stubs (`AgenticEngineeringReference.md`, `InitialReport.md`); version references aligned to v0.7.

**Tests:** 262/262 green at Phase 18 close; Phase 20 workspace gate green (0 failed) with 4 new refresh integration tests.

---

## v0.5.0 — Dynamic sitemap + IssueBackend decoupling (SHIPPED 2026-04-14)

**Goal:** Ship OP-2 partial (`_INDEX.md` in bucket dir) and decouple FUSE write path + git-remote helper from the sim-specific REST layer.

**Shipped:**

- Phase 14 (v0.4.1): Decouple sim REST shape from FUSE write path and git-remote helper — route through `IssueBackend` trait. Deleted `fetch.rs`, `write.rs`, `client.rs` (~830 LoC). 
- Phase 15 (v0.5.0): Dynamic `_INDEX.md` synthesized in FUSE bucket directory — YAML frontmatter + markdown table sitemap, read-only, lazily rendered.

**Tests:** 277/277 green. Clippy clean. CI green.

---

## v0.4.0 — Nested mount layout: pages/ + tree/ symlinks (SHIPPED 2026-04-14)

**Goal:** Ship OP-1 — convert flat `<id>.md` root to two-view layout: writable `pages/` bucket + synthesized read-only `tree/` symlink overlay for Confluence parentId hierarchy.

**Shipped:**

- Phase 13: Nested mount layout, ADR-003, slug deduplication, cycle-safe DFS, integration tests.

**Tests:** 272/272 green.

---

## v0.3.0 — Confluence Cloud read-only adapter (SHIPPED 2026-04-14)

**Goal:** Ship `reposix-confluence` crate implementing `IssueBackend` against Atlassian Confluence Cloud REST v2. CLI dispatch for list + mount.

**Shipped:**

- Phase 11: ConfluenceReadOnlyBackend, wiremock tests, contract tests, parity demo, ADR-002, connector guide.

**Tests:** 193/193 green.

---

## v0.2.0-alpha — GitHub read-only adapter + demo suite (SHIPPED 2026-04-13)

**Goal:** Ship Phase 8 (demo suite restructure + IssueBackend seam + GithubReadOnlyBackend + contract tests).

**Shipped:**

- Phase 8: IssueBackend trait, SimBackend impl, GithubReadOnlyBackend, contract test suite, Tier-1 demos.

---

## v0.1.0 — MVD: Simulator + FUSE + CLI + demo (SHIPPED 2026-04-13)

**Goal:** Simulator-first read-only FUSE mount that an LLM agent can use with `cat`, `grep`, and `git`.

**Shipped:**

- Phase 1: Core contracts + security guardrails
- Phase 2: Simulator + audit log
- Phase 3: Read-only FUSE mount + CLI
- Phase 4: Demo + recording + README
- Phase S: STRETCH — write path + swarm + FUSE-in-CI

**Tests:** 168/168 green. Demo recorded.
