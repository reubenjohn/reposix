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
- ⏳ **v0.9.0 Architecture Pivot — Git-Native Partial Clone** — Phases 31–TBD (in progress, started 2026-04-24)
- 📋 **v0.10.0 Docs & Narrative** — Phase 30 (deferred from v0.9.0 — docs must describe the new architecture)

## Phases

## v0.9.0 Architecture Pivot — Git-Native Partial Clone (IN PROGRESS)

**Motivation:** The FUSE-based design is fundamentally slow (every `cat`/`ls` triggers a live REST API call) and doesn't scale (10k Confluence pages = 10k API calls on directory listing). FUSE also has operational pain: fusermount3, /dev/fuse permissions, WSL2 quirks, pkg-config/libfuse-dev build dependencies. Research confirmed that git's built-in partial clone + the existing `git-remote-reposix` helper can replace FUSE entirely, giving agents a standard git workflow with zero custom CLI awareness required.

**Research:** See `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` (canonical design document), `partial-clone-remote-helper-findings.md` (transport layer POC), `push-path-stateless-connect-findings.md` (write path POC), `sync-conflict-design.md` (sync model). POC code in `poc/` subdir.

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

**Phases:** TBD — to be planned via `/gsd-plan-phase` after research docs are reviewed. Estimated 3–5 phases:
1. `stateless-connect` in `git-remote-reposix` + backing bare-repo cache
2. Push-time conflict detection + delta sync
3. CLI flow change (`reposix init` replacing `reposix mount`)
4. Delete `reposix-fuse` + update all tests/docs/CI
5. Integration testing + hardening

---

## v0.10.0 Docs & Narrative (DEFERRED)

> **Why deferred:** Phase 30 rewrites the hero, tutorial, how-it-works, and architecture pages. All content assumes FUSE. Executing Phase 30 before the architecture pivot would produce docs that are immediately obsolete. Phase 30's plans will need revision to describe the git-native design instead.

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
