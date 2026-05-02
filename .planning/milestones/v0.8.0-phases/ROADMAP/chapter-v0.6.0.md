← [back to index](./index.md)

# Milestone v0.6.0 — Write Path + Full Sitemap

**Goal:** Turn the mount from a read-only navigator into a writable agent workspace.
**Phases:** 16–20 | **Requirements:** REQUIREMENTS.md §v0.6.0

### Phase 16: Confluence write path — update_issue create_issue delete_or_close on ConfluenceBackend plus atlas_doc_format to Markdown round-trip — SHIPPED 2026-04-14 (v0.6.0)

**Goal:** Implement `create_issue`, `update_issue`, `delete_or_close` on `ConfluenceBackend`; ADF↔Markdown converter; client-side audit log via SG-06; ADF read path with storage fallback. Closes REQ WRITE-01..04.
**Requirements**: WRITE-01, WRITE-02, WRITE-03, WRITE-04
**Depends on:** Phase 15
**Plans:** 4/4 plans executed

Plans:
- [x] 16-A-adf-converter.md
- [x] 16-B-write-methods.md
- [x] 16-C-audit-and-integration.md
- [x] 16-D-docs-and-release.md

### Phase 17: Swarm confluence-direct mode — add --mode confluence-direct to reposix-swarm using SimDirectWorkload as template (v0.6.0)

**Goal:** Add a read-only `labels/` symlink overlay to the FUSE mount. Each `labels/<label>/` directory lists all issues carrying that label as symlinks to the canonical bucket file. `spaces/` deferred to Phase 20.
**Requirements**: LABEL-01, LABEL-02, LABEL-03, LABEL-04, LABEL-05
**Depends on:** Phase 16
**Plans:** 2/2 plans complete

Plans:
- [x] TBD (run /gsd-plan-phase 17 to break down) (completed 2026-04-15)

### Phase 18: OP-2 remainder — tree-recursive and mount-root _INDEX.md synthesis extending TreeSnapshot dfs (v0.6.0)

**Goal:** Complete OP-2 by synthesizing `_INDEX.md` at two additional levels: `mount/tree/<subdir>/_INDEX.md` (recursive subtree sitemap via cycle-safe DFS from `TreeSnapshot`) and `mount/_INDEX.md` (whole-mount overview listing all backends, buckets, and top-level entry counts). Combined with Phase 15 bucket-level `_INDEX.md`, agents can `cat` any level of the mount hierarchy.
**Requirements**: INDEX-01, INDEX-02
**Depends on:** Phase 17
**Plans:** 1/2 plans executed

Plans:
- [x] 18-01-PLAN.md — inode constants, InodeKind variants, render functions, FUSE dispatch, 6 unit tests
- [x] 18-02-PLAN.md — workspace green-gauntlet, CHANGELOG, dev smoke script, SUMMARY
- [x] Phase 18: SHIPPED (see 18-01-SUMMARY.md, 18-SUMMARY.md, VERIFICATION.md)

### Phase 19: OP-1 remainder — labels and spaces directory views as read-only symlink overlays for GitHub and Confluence (v0.6.0)

**Goal:** Add a read-only `labels/` symlink overlay to the FUSE mount. Each `labels/<label>/` directory lists all issues carrying that label as symlinks to the canonical bucket file. `spaces/` deferred to Phase 20.
**Requirements**: LABEL-01, LABEL-02, LABEL-03, LABEL-04, LABEL-05
**Depends on:** Phase 18
**Plans:** 2/2 plans executed

Plans:
- [x] 19-A-labels-fuse-impl.md — inode constants, labels.rs module, FUSE dispatch, >= 5 tests
- [x] 19-B-docs-and-release.md — green workspace gauntlet, CHANGELOG, STATE update
- [x] Phase 19: SHIPPED (see 19-A-SUMMARY.md, 19-SUMMARY.md, VERIFICATION.md)

### Phase 20: OP-3 — reposix refresh subcommand and git-diff cache for mount-as-time-machine semantics (v0.6.0)

**Goal:** Add `reposix refresh` CLI subcommand that re-fetches all issues/pages from the backend, writes deterministic `.md` files into the mount's git working tree, and commits them so `git diff HEAD~1` shows what changed at the backend since the last pull. Ships v0.6.0.
**Requirements**: REFRESH-01, REFRESH-02, REFRESH-03, REFRESH-04, REFRESH-05
**Depends on:** Phase 19
**Plans:** 3 plans

Plans:
- [x] 20-A-refresh-cmd.md — refresh.rs + cache_db.rs + main.rs wiring
- [x] 20-B-tests-and-polish.md — integration tests + workspace gate
- [x] 20-C-docs-and-release.md — CHANGELOG + STATE update (SHIPPED)
- [x] Phase 20: SHIPPED (see 20-A-SUMMARY.md, 20-B-SUMMARY.md, 20-C-SUMMARY.md, VERIFICATION.md)


### Phase 26: Docs clarity overhaul (v0.6.0)

**Goal:** Every user-facing Markdown document can be understood in isolation by an LLM agent or human contributor arriving cold — no other files read, no links followed. Uses the `doc-clarity-review` skill to run an isolated subagent review on each doc, collects friction points / unanswered questions / over/under-explained sections, fixes them, then re-reviews to confirm zero critical friction points remain. Also removes stale root-level orphan docs and aligns version numbers across all pages.

**Depends on:** Phase 25
**Requirements**: TBD
**Plans:** 5/5 plans executed

#### Doc inventory (pre-phase assessment)

**Root-level — action required:**

| File | Status | Action |
|------|--------|--------|
| `README.md` | Stale (says v0.3.0, now v0.7+; outdated phase table) | Update + clarity review |
| `MORNING-BRIEF.md` | Obsolete (explicitly covers v0.1/v0.2 era) | Archive to `docs/archive/` or delete |
| `PROJECT-STATUS.md` | Obsolete (v0.1/v0.2 timeline, superseded by HANDOFF) | Archive to `docs/archive/` or delete |
| `HANDOFF.md` | Partially stale (v0.5 era, OP items partially closed) | Update to reflect v0.7 state |
| `AgenticEngineeringReference.md` | Obsolete (redirect stub — canonical copy at `docs/research/`) | Delete |
| `InitialReport.md` | Obsolete (redirect stub — canonical copy at `docs/research/`) | Delete |
| `CHANGELOG.md` | Current | Keep; check accuracy |

**docs/ pages — clarity review + update where stale:**

| File | Status | Notes |
|------|--------|-------|
| `docs/index.md` | Needs update (says v0.4; now v0.7) | Clarity review + version bump |
| `docs/architecture.md` | Current | Clarity review |
| `docs/why.md` | Current (Phase 22 updated headline) | Clarity review |
| `docs/security.md` | Current | Clarity review |
| `docs/demo.md` | Current | Clarity review |
| `docs/demos/index.md` | Current | Clarity review |
| `docs/development/contributing.md` | Current | Clarity review |
| `docs/development/roadmap.md` | Stale (stops at v0.5; OP items not updated) | Update + clarity review |
| `docs/reference/cli.md` | Current | Clarity review |
| `docs/reference/confluence.md` | Current | Clarity review |
| `docs/reference/git-remote.md` | Current | Clarity review |
| `docs/reference/http-api.md` | Current | Clarity review |
| `docs/reference/crates.md` | Current | Clarity review |
| `docs/connectors/guide.md` | Current | Clarity review |
| `docs/decisions/001-github-state-mapping.md` | Current | Clarity review |
| `docs/decisions/002-confluence-page-mapping.md` | Partially stale (part superseded by ADR-003) | Clarify scope + clarity review |
| `docs/decisions/003-nested-mount-layout.md` | Current | Clarity review |
| `docs/research/initial-report.md` | Current | Clarity review |
| `docs/research/agentic-engineering-reference.md` | Current | Clarity review |
| `docs/social/twitter.md` | Archive/social — not a doc page | Skip clarity review |
| `docs/social/linkedin.md` | Archive/social — not a doc page | Skip clarity review |

#### Review methodology

Each doc is reviewed using the `doc-clarity-review` skill:
1. Copy file(s) to a fresh `/tmp/doc-clarity-review-*/` directory.
2. Run `claude -p '<review prompt>' <file>` — a subprocess Claude with zero repo context.
3. The reviewer is instructed: do not follow links, do not gather more context, be unbiased.
4. Collect: friction points, unanswered questions, over-explained sections, under-explained sections, missing references.
5. Having questions about things that are linked elsewhere is *usually* acceptable — the fix is normalizing with links, not repeating content.
6. Fix each doc, then re-run the review to confirm CLEAR verdict.

Plans:
- [x] 26-01-PLAN.md — Root-level housekeeping (delete stubs, archive obsolete, fix README version)
- [x] 26-02-PLAN.md — Clarity review + fix: README.md, HANDOFF.md, docs/index.md, docs/development/roadmap.md
- [x] 26-03-PLAN.md — Clarity review: docs/ core pages (architecture, why, security, demo, contributing)
- [x] 26-04-PLAN.md — Clarity review: reference/, connectors/, decisions/ docs
- [x] 26-05-PLAN.md — Clarity review: research/ docs + final verification + STATE + SUMMARY

---
