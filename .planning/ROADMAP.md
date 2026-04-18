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

## Phases

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

### Phase 30: Docs IA and narrative overhaul — landing page aha moment and progressive-disclosure architecture reveal

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 29
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 30 to break down)
