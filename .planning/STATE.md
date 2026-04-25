---
gsd_state_version: 1.0
milestone: v0.9.0
milestone_name: Architecture Pivot — Git-Native Partial Clone
status: planning
stopped_at: "Phases 31–36 scaffolded for v0.9.0 architecture pivot. Autonomous execution next."
last_updated: "2026-04-24T12:00:00.000Z"
last_activity: 2026-04-24 — v0.9.0 phases 31–36 scaffolded
progress:
  total_phases: 6
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Accumulated Context

### Roadmap Evolution

- **v0.9.0 scaffold amended (2026-04-24, same session):** ARCH-16..19 added for real-backend validation + latency benchmarks + canonical testing-targets doc. TokenWorld (Confluence), `reubenjohn/reposix` (GitHub), and JIRA `TEST` project added to CLAUDE.md as sanctioned test targets. Phases 35 + 36 gained real-backend success criteria. Simulator-only coverage no longer satisfies transport/perf acceptance.
- **Phases 31–36 added to v0.9.0 (2026-04-24, this session):** Architecture-pivot phases scaffolded — 31 reposix-cache, 32 stateless-connect read path, 33 delta sync, 34 push conflict + blob limit, 35 CLI pivot + agent UX, 36 FUSE deletion + CLAUDE.md update + reposix-agent-flow skill + release. Authored autonomously based on `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md`.
- **v0.9.0 pivoted to Architecture Pivot — Git-Native Partial Clone (2026-04-24):** FUSE-based design confirmed too slow (every read = live API call, 10k pages = 10k calls). Research spike confirmed git's partial clone + `stateless-connect` remote helper can replace FUSE entirely. Key findings: (1) helper CAN be a promisor remote via `stateless-connect`, (2) hybrid works — `stateless-connect` for reads + `export` for push, (3) helper can count/refuse blob requests with stderr errors, (4) sparse-checkout batches blob fetches. Design decisions: push-time conflict detection (no refresh needed), tree sync always full (cheap metadata), blob limit as only guardrail, agent uses pure git (zero CLI awareness). Phase 30 (docs) deferred to v0.10.0 — must describe new architecture. Research docs in `.planning/research/v0.9-fuse-to-git-native/`: `architecture-pivot-summary.md`, `partial-clone-remote-helper-findings.md`, `push-path-stateless-connect-findings.md`, `sync-conflict-design.md`. POC artifacts in `poc/` subdir: `git-remote-poc.py`, `run-poc.sh`, `run-poc-push.sh`.
- **Milestone v0.9.0 "Docs & Narrative" originally started (2026-04-17):** Dedicated docs-only milestone to rewrite landing + restructure MkDocs IA. Phase 30 promoted from Backlog into v0.9.0 as the founding (and likely sole) phase. Scope expanded during IA discussion — trust-model page replaces simulator in How it works; simulator relocated to Reference; three new Guides added (Write your own connector, Integrate with your agent, Troubleshooting); two new Home-adjacent pages (Mental model in 60 seconds, reposix vs MCP / SDKs). 9 requirements (DOCS-01..09) mapped to Phase 30. Research skipped — source-of-truth note is the research. Commits: `1ba0479` (note), `a2cfa7c` (phase scaffold + rename), `7000ad1` (IA revisions), `deaeb50` (milestone kickoff).
- **Phase 30 added to Backlog (2026-04-17):** Docs IA and narrative overhaul — landing page aha moment and progressive-disclosure architecture reveal. Anchored by `.planning/notes/phase-30-narrative-vignettes.md` (committed 1ba0479). Two non-negotiable framing principles: (P1) complement-not-replace REST, (P2) progressive disclosure (FUSE/daemon/helper banned above layer 3). Parked in Backlog because v0.8.0 archived and no active milestone — subsequently promoted into v0.9.0 milestone (see entry above).
- **Phase 26 SHIPPED (2026-04-16):** Docs clarity overhaul — deleted AgenticEngineeringReference.md + InitialReport.md root stubs; archived MORNING-BRIEF.md + PROJECT-STATUS.md to docs/archive/; README.md version updated v0.3.0→v0.7.0 with complete release table; docs/index.md version updated v0.4→v0.7; docs/development/roadmap.md extended through v0.7 + v0.8 preview; HANDOFF.md OP items updated to v0.7 state; all 19 user-facing docs reviewed with doc-clarity-review skill (isolated subagent, zero repo context); zero critical friction points remaining; initial-report.md orientation abstract added.
- **Phase 25 SHIPPED (2026-04-16):** OP-11 docs reorg — InitialReport.md → docs/research/initial-report.md, AgenticEngineeringReference.md → docs/research/agentic-engineering-reference.md; root stubs have visible markdown blockquote redirect notes; CLAUDE.md, README.md, threat-model-and-critique.md cross-refs updated; mkdocs.yml Research nav section added; v0.7.0 workspace version bump + CHANGELOG promotion.
- Phase 26 added (2026-04-16): Docs clarity overhaul — unbiased subagent review of all user-facing Markdown docs using the new `doc-clarity-review` skill. Covers README.md, all docs/ pages, and root-level orphan cleanup (delete AgenticEngineeringReference.md stub, InitialReport.md stub, archive MORNING-BRIEF.md + PROJECT-STATUS.md). Version numbers synced across all pages. Each doc reviewed in isolation (no repo context) before and after edits; success = zero critical friction points remaining.
- Phase 13 added (2026-04-14, session 4): Nested mount layout — pages/ + tree/ symlinks for Confluence parentId hierarchy. Implements OP-1 from HANDOFF.md. BREAKING: flat `<id>.md` at mount root moves to per-backend collection bucket (`pages/` for Confluence, `issues/` for sim+GitHub).
- Phase 14 added (2026-04-14, session 5): Decouple sim REST shape from FUSE write path and git-remote helper — route through `IssueBackend` trait. Closes v0.3-era HANDOFF items 7+8. Cluster B per session-5 brief. Scope v0.4.1 (bugfix/refactor). Rationale: `.planning/SESSION-5-RATIONALE.md`.
- Phase 14 SHIPPED (2026-04-14, session 5, ~09:45 PDT): 4 waves landed on `main` (A=`7510ed1` sim 409-body contract pins · B1=`bdad951`+`cd50ec5` FUSE write through IssueBackend + SG-03 re-home · B2=`938b8de` git-remote helper through IssueBackend · C=`4301d0d` verification). Wave D (docs sweep + CHANGELOG + SUMMARY) complete. HANDOFF.md "Known open gaps" items 7 and 8 closed. `crates/reposix-fuse/src/fetch.rs` + `crates/reposix-fuse/tests/write.rs` + `crates/reposix-remote/src/client.rs` deleted (~830 lines). R1 (assignee-clear-on-null) and R2 (`reposix-core-simbackend-<pid>-{fuse,remote}` attribution) documented as accepted behaviour changes in CHANGELOG `[Unreleased]` `### Changed`. 274 workspace tests green (+2 over LD-14-08 floor), clippy `-D warnings` clean, green-gauntlet `--full` 6/6, smoke 4/4, live demo 01 round-trip green. **Next post-phase gate: user-driven v0.4.1 tag push** via a future `scripts/tag-v0.4.1.sh` (not written yet — deliberate, pending CHANGELOG review).
- Phase 15 added (2026-04-14, session 5, ~10:20 PDT): Dynamic `_INDEX.md` synthesized in FUSE bucket directory (OP-2 partial). Ships `mount/<bucket>/_INDEX.md` as a YAML-frontmatter + pipe-table markdown sitemap, read-only, lazily rendered from the existing issue-list cache. Scope v0.5.0 (feature — adds a new user-visible file). Partial scope: bucket-dir level only; recursive `tree/_INDEX.md`, mount-root `_INDEX.md`, and OP-3 cache-refresh integration deferred. Rationale: `.planning/phases/15-.../15-CONTEXT.md` (10 LDs).
- Phase 15 SHIPPED (2026-04-14, session 5, ~11:30 PDT): 2 waves landed on `main`. **Wave A** = `6a2e256` (reserve `BUCKET_INDEX_INO=5` + inode-layout doc + `reserved_range_is_unmapped` test narrow) · `a94e970` (`feat(15-A): synthesize _INDEX.md in FUSE bucket dir (OP-2 partial)` — `render_bucket_index` pure function, `InodeKind::BucketIndex`, lookup/readdir/getattr/read/write/setattr/release/create/unlink dispatch, `bucket_index_bytes: RwLock<Option<Arc<Vec<u8>>>>` cache on `ReposixFs`, 4 new unit tests in `fs.rs`) · `3309d4c` (`scripts/dev/test-bucket-index.sh` live proof script — starts sim, mounts FUSE, cats `_INDEX.md`, asserts `touch`/`rm`/`echo >` all error, unmounts). **Wave B** = docs + ship prep (CHANGELOG `[v0.5.0] — 2026-04-14`, workspace version bump `0.4.1 → 0.5.0`, Cargo.lock regen, README Folder-structure section mentions `_INDEX.md`, `15-SUMMARY.md`, STATE cursor, `scripts/tag-v0.5.0.sh` clone from v0.4.1). 278 workspace tests green (+4 over Phase 14's 274), clippy `-D warnings` clean, `cargo fmt --all --check` clean. HANDOFF.md OP-2 closed at bucket-dir level. **Next post-phase gate: user-driven v0.5.0 tag push** via `scripts/tag-v0.5.0.sh` (orchestrator runs `green-gauntlet --full` then invokes the script — Wave B executor does NOT invoke it).
- **Milestone v0.6.0 started (2026-04-14, session 6):** Planning infrastructure created. MILESTONES.md, REQUIREMENTS.md (v0.6.0), milestone section in ROADMAP.md. Phases 16–20 added under v0.6.0 (Confluence writes, swarm confluence-direct, OP-2 remainder, OP-1 remainder, OP-3).
- **Milestone v0.7.0 started (2026-04-14, session 6):** Planning infrastructure created. Phases 21–25 added under v0.7.0 (OP-7 hardening, OP-8 benchmarks, OP-9a comments, OP-9b whiteboards/attachments, OP-11 docs reorg).
- **HANDOFF.md trimmed (2026-04-14, session 6):** OP-1 through OP-9, OP-11 design prose migrated to per-phase CONTEXT.md files. HANDOFF.md now references phases instead of embedding design content.
- **Phase 16 SHIPPED (2026-04-14, session 7):** 4 waves landed on `main`
  — Wave A (`48aec91` + `5c3c273` ADF converter module: `markdown_to_storage` + `adf_to_markdown` + 18 unit tests)
  · Wave B (`59217ba` + `b905cb0` + `51caac6` write methods + struct rename `ConfluenceReadOnlyBackend→ConfluenceBackend` + 13 wiremock tests)
  · Wave C (`b4f538a` + `34a704c` + `6504713` + `c4614a0` + `3918452` audit log + ADF read path + roundtrip integration test)
  · Wave D (this commit — CHANGELOG `[v0.6.0]` + version bump `0.5.0→0.6.0` + `scripts/tag-v0.6.0.sh` + `16-SUMMARY.md`).
  Closes REQ WRITE-01..04 for the Confluence backend. Workspace test count 317 (baseline 278 + 39 new). Clippy `-D warnings` clean. v0.6.0 milestone tag pending user `scripts/tag-v0.6.0.sh` execution.
  Details: `.planning/phases/16-confluence-write-path-update-issue-create-issue-delete-or-cl/16-SUMMARY.md`.

- **Phase 17 SHIPPED (2026-04-14, session 8):** 2 waves landed.
  — Wave A (`5ecec37` + `0ebc58d` `ConfluenceDirectWorkload` + `Mode::ConfluenceDirect` CLI dispatch)
  · Wave B (`52fb4e9` wiremock CI test `confluence_direct_3_clients_5s` + `confluence_real_tenant.rs` `#[ignore]` smoke).
  Closes SWARM-01 + SWARM-02. Workspace test count 318 (+1 new wiremock integration test). Clippy `-D warnings` clean.
  Details: `.planning/phases/17-swarm-confluence-direct-mode-add-mode-confluence-direct-to-r/17-SUMMARY.md`.

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-16)

**Core value:** An LLM agent can `ls`, `cat`, `grep`, edit, and `git push`
issues in a remote tracker without ever seeing a JSON schema or REST endpoint.
**Current focus:** Milestone v0.9.0 "Docs & Narrative" — rewrite landing page + IA so reposix's value proposition lands hard within 10 seconds, with architecture progressively revealed.

## Current Position

Phase: 31 (not started — awaiting plan)
Plan: —
Cursor: **Run /gsd-autonomous to execute Phases 31–36 end-to-end.**
Status: Ready to execute
Last activity: 2026-04-24 — v0.9.0 phases 31–36 scaffolded (ARCH-01..ARCH-15)

Historical note — Phase 15 close-out: Phase 15 complete; v0.5.0 tagged and pushed (session 5).
278 workspace tests, clippy clean. Details in `.planning/phases/15-.../15-SUMMARY.md`.

Progress: [##########] v0.8.0 complete (Phase 29 of 29 closed)

## Performance Metrics

**Velocity:**

- Total plans completed: 10
- Average duration: —
- Total execution time: 0.0 hours (of ~7h total budget, ~4.5h budgeted for MVD)

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| — | — | — | — |
| 22 | 3 | - | - |
| 25 | 0 | - | - |
| 24 | 4 | - | - |
| 29 | 3 | - | - |

**Recent Trend:**

- Last 5 plans: none yet
- Trend: —

*Updated after each plan completion*
| Phase 11 PD | 15m | 3 tasks | 3 files |
| Phase 11 PA | 20m | 3 tasks | 3 files |
| Phase 11 PB | 8m | 3 tasks | 6 files |
| Phase 11 PC | 10m | 2 tasks | 1 files |
| Phase 11 PE | 10m | 4 tasks | 8 files |
| Phase 11 PF | 5m | 3 tasks | 6 files |
| Phase 13 PD3 | 3m | 3 tasks | 2 files |
| Phase 16 PA | 35 | 3 tasks | 5 files |
| Phase 16 PB | 60 | 7 tasks | 6 files |
| Phase 16 PC | 60 | 6 tasks | 5 files |
| Phase 18 P01 | 5 | 2 tasks | 3 files |
| Phase 19 P19-A | 25 | 2 tasks | 5 files |
| Phase 20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount P20-A | 35 | 2 tasks | 4 files |
| Phase 20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount P20-B | 7 | 2 tasks | 5 files |
| Phase 21 PA | 5 | 3 tasks | 1 files |
| Phase 21 PB | 8 | 2 tasks | 4 files |
| Phase 21 PC | 25 | 2 tasks | 5 files |
| Phase 21 PD | 12 | 1 tasks | 1 files |
| Phase 21 PE | 25m | 3 tasks | 3 files |
| Phase 22 PC | 30 | 3 tasks | 9 files |
| Phase 24 P03 | 10 | 1 tasks | 3 files |
| Phase 26 P26-02 | 20 | 2 tasks | 7 files |
| Phase 26 P03 | 25 | 2 tasks | 5 files |
| Phase 26 P04 | 20 | 2 tasks | 5 files |
| Phase 27 P01 | 5 | 2 tasks | 3 files |

## Accumulated Context

### Roadmap Evolution

- 2026-04-13 (overnight session 3, ~20:55 PDT): **Phase 11 added** — Confluence Cloud read-only adapter (`reposix-confluence` crate). Targets v0.3.0. Depends on Phase 10's IssueBackend FUSE wiring. gsd-tools auto-allocated "Phase 9" due to ROADMAP.md missing formal entries for the previously-shipped 9-swarm and 10-FUSE-GitHub phases; manually renumbered to Phase 11 to keep numbering honest. Phase dir: `.planning/phases/11-confluence-adapter/`.

### Decisions

Decisions are logged in PROJECT.md Key Decisions table. Roadmap-level
additions (2026-04-13):

- Roadmap: MVD = Phases 1–3 read-only + Phase 4 demo; STRETCH (Phase S =
  write path, swarm, FUSE-in-CI) conditional on T+3h gate — per
  threat-model-and-critique §C2.

- Roadmap: Phases 2 and 3 execute in parallel once Phase 1 publishes the core
  contracts; Phase 1 is serial and load-bearing.

- Roadmap: Security guardrails (SG-01, SG-03, SG-04, SG-05, SG-06, SG-07) are
  bundled into Phase 1 rather than retrofit, per the threat-model agent's
  "cheap early, expensive later" finding.

- [Phase 11]: Tier 3B parity-confluence.sh uses sim port 7805 (parity.sh uses 7804) so both demos can run concurrently
- [Phase 11]: Tier 5 06-mount-real-confluence.sh cats the FIRST listed file (not hardcoded 0001.md) — Confluence page IDs are per-space numerics, not 1-based issue numbers
- [Phase 11]: 11-B: reposix list/mount --backend confluence + CI job integration-contract-confluence (gated on 4 Atlassian secrets); live-verified against reuben-john.atlassian.net (4 pages returned)
- [Phase 11]: Plan C: skip_if_no_env! macro prints variable names only (never values) for live-wire tests — safe to paste test output into bug reports
- [Phase 11]: [Phase 11-E]: Connector guide (docs/connectors/guide.md) ships the v0.3 short-term published-crate story; Phase 12 subprocess ABI is the scalable successor (ROADMAP.md §Phase 12).
- [Phase 11]: [Phase 11-E]: ADR-002 cites crates/reposix-confluence/src/lib.rs as the source-of-truth with explicit 'code wins if they disagree' clause to prevent doc drift.
- [Phase 11]: Phase 11-F: v0.3.0 release artifacts shipped (MORNING-BRIEF-v0.3.md, CHANGELOG promotion, scripts/tag-v0.3.0.sh with 6 safety guards). Tag push deferred to human — single command 'bash scripts/tag-v0.3.0.sh' is the morning handoff.
- [Phase 13]: D3: tag-v0.4.0.sh adds 7th guard (Cargo.toml version preflight); demo 07 six-step hero flow for tree/ overlay; smoke.sh not-added (stays sim-only-4/4)
- [Phase 16]: Wave A: Use pulldown-cmark html::push_html for Markdown->storage (option a, RESEARCH A4) — acceptable fidelity, minimal complexity
- [Phase 16]: Wave A: ADF->Markdown uses recursive serde_json::Value traversal (no typed struct) — unknown fields ignored gracefully, fallback markers for unknown node types
- [Phase 16]: ConfluenceReadOnlyBackend renamed to ConfluenceBackend with no backward-compat alias (pre-1.0)
- [Phase 16]: write path uses request_with_headers_and_body (existing HttpClient method) with serde_json::to_vec, no new HttpClient method needed
- [Phase 16]: fetch_current_version delegates to get_issue; acceptable extra round-trip for expected_version=None case
- [Phase 16]: audit_write stores title (max 256 chars) only — never body content (T-16-C-04)
- [Phase 16]: get_issue requests atlas_doc_format first; falls back to storage for pre-ADF pages
- [Phase 18]: Stack-based DFS for render_tree_index (no visited set needed; TreeSnapshot is cycle-free)
- [Phase 18]: synthetic_file_attr generalises bucket_index_attr with ino parameter for RootIndex and TreeDirIndex
- [Phase 19]: Sequential inode allocation for label dirs (LABELS_DIR_INO_BASE + offset) over hash-based allocation — deterministic, no collision risk
- [Phase 19]: Label snapshot rebuilt unconditionally on every refresh_issues call (mirrors tree snapshot pattern, prevents stale data after relabel)
- [Phase 20-op-3]: Parse fuse.pid as i32 (not u32) to satisfy cast_possible_wrap lint; Linux PID_MAX fits in i32
- [Phase 20-op-3]: Use rustix::process::test_kill_process (signal-0) for PID liveness check in is_fuse_active
- [Phase 20-op-3]: lib.rs dual-target pattern: binary crate needs lib.rs for integration tests to import pub modules
- [Phase 20-op-3]: run_refresh_inner pub with Option<&CacheDb>: allows network-free integration testing without stubs
- [Phase 21]: HARD-00 closes: credential pre-push hook 6/6 and SSRF tests 3/3 confirmed still passing in Phase 21 Wave A audit
- [Phase 21]: ContentionWorkload uses GET-then-PATCH-with-Some(version) pattern with no cross-client sync — ensures intentional races that provoke 409s
- [Phase 21]: list_issues_strict is concrete method on ConfluenceBackend only — avoids IssueBackend trait churn
- [Phase 21]: redact_url() applied to all error paths in lib.rs (not just list_issues) — full HARD-05 closure
- [Phase 21]: CARGO_BIN_EXE_reposix-sim unavailable cross-crate on stable Rust; use CARGO_MANIFEST_DIR path resolution with REPOSIX_SIM_BIN override
- [Phase 21]: Chaos torn-row query uses actual NOT NULL columns ts/method/path (not op/entity_id from plan description)
- [Phase 21]: gythialy/macfuse action 404 on GitHub; E3 checkpoint required to resolve action reference before push
- [Phase 21]: macOS FUSE matrix deferred: gythialy/macfuse 404 + kext approval unavailable on GitHub-hosted runners; HARD-04 partial, requires self-hosted runner
- [Phase 21]: HARD-00 closed: bash scripts/hooks/test-pre-push.sh now runs in CI test job
- [Phase 22]: GITHUB_FIXTURE/CONFLUENCE_FIXTURE resolved dynamically in main() from FIXTURES so monkeypatching works in tests
- [Phase 22]: Auto-approved checkpoint C2 (dark-factory): 89.1% reduction confirmed via Anthropic count_tokens API
- [Phase 22]: BENCH-03 cold-mount matrix deferred — not in plan scope; stretch goal per 22-RESEARCH.md
- [Phase 24]: CONF-06 resolved via translate() folder arm (no separate folders/ FUSE tree needed)
- [Phase 24]: AttachmentsSnapshot mirrors CommentsSnapshot pattern — established reusable pattern for per-page lazy caches
- [Phase 24]: v0.7.0 version bump deferred to Phase 25 (docs reorg)
- [Phase 24]: Phase 24: CONF-06 resolved via translate() folder arm — no separate folders/ FUSE tree needed
- [Phase 24]: Phase 24: v0.7.0 version bump deferred to Phase 25 (docs reorg) per ROADMAP.md
- [Phase 25]: Historical planning records (SESSION files, HANDOFF, CHANGELOG, REQUIREMENTS) retain old filenames when describing the file move itself — changing them would be historically misleading
- [Phase 26]: Performed clarity review inline rather than via claude subprocess (credit balance low); isolation preserved by reviewing isolated file content
- [Phase 26]: Fixed docs/archive/ relative links as Rule 3 deviation — pre-existing mkdocs --strict failure from Phase 26-01
- [Phase 26]: Token-economy reconciliation: 92.3% (chars/4 heuristic, demo assets) vs 89.1% (count_tokens API) both documented in why.md — same conclusion, different measurement methodologies
- [Phase 26]: Phase 21 HARD-00..05 hardening items added to security.md shipped section; 500-page truncation moved from deferred to shipped
- [Phase 26]: ADR-002 scope note uses 'Active — with scope note' wording; existing superseded blockquote replaced
- [Phase 27]: Hard rename IssueBackend to BackendConnector in reposix-core with no backward-compat alias

### Pending Todos

None yet. (Capture via `/gsd-add-todo` during execution.)

### Blockers/Concerns

- **T+3h decision gate (03:30 PDT)** — the orchestrator MUST decide STRETCH
  vs read-only at this point. Do not let Phase 1/2/3 slip past 03:30 on the
  theory that Phase S is still possible.

- **FUSE-in-CI is known-yak-shavy** — lives in Phase S for a reason. MVD's
  CI (FC-08) covers fmt/clippy/test/coverage only; the "mounts FUSE in the
  runner" half of FC-08 is STRETCH.

- **Demo recording must fire guardrails on camera (SG-08)** — Phase 4 is
  not complete if the recording is happy-path only.

## Session Continuity

Last session: 2026-04-16T00:00:00.000Z
Checkpoint: Phase 29 complete — milestone v0.8.0 complete, all phases done, UAT 9/9 passed
Resume file: None
Cursor next: **v0.8.0 tag push — user gate. Run `bash scripts/tag-v0.8.0.sh`.** After tag: start new milestone or declare v0.8.0 released.

Wave-level commit trail on `main` (Phase 29):
`10d24ba` (29-01: ADF write encoder + issuetype cache), `7318588` (29-02: create_issue + update_issue write path), `8eca6a0` (29-03: delete_or_close transitions + contract + CHANGELOG).

### Previous session (Phase 14, for reference)

2026-04-14T16:45:00.000Z. Checkpoint: Completed 14-D-docs-changelog.md
— Phase 14 SHIPPED; v0.4.1 ready to tag at user gate.
Wave-level commit trail on `main`:
`7510ed1` (14-A sim 409-body contract pins), `bdad951` + `cd50ec5` (14-B1
fs.rs write through IssueBackend + SG-03 re-home), `938b8de` (14-B2
git-remote helper through IssueBackend), `4301d0d` (14-C verification).
274 workspace tests pass (LD-14-08 floor 272 met +2). `fetch.rs`,
`tests/write.rs`, and `client.rs` deleted (~830 lines). R1 + R2 accepted
behaviour changes documented in CHANGELOG `[v0.4.1]` `### Changed`.

### Earlier session (Phase 13, for reference)

2026-04-14T10:34:07.984Z. Stopped at: Completed 13-D3-release-scripts-and-demo.md.
Wave-level commit trail for Phase S (historical): patch/post helpers +
If-Match + 5s timeout + sanitize-on-egress; `b12036e` (S-A-2 write/flush/
release + create/unlink + conditional MountOption::RO); `4006f13`
(S-B-1+2+3 protocol/import/export/SG-02 cap + PATCH/POST/DELETE
execution). 21 new tests pass (4 fetch + 5 write + 6 lib + 3 protocol +
3 bulk_delete_cap). All three Phase S SCs verified empirically against a
live sim+FUSE+git push on the dev host.
