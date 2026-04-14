---
gsd_state_version: 1.0
milestone: v0.4
milestone_name: target)
status: complete
stopped_at: Completed 15-B-docs-changelog-summary-state-tag.md — Phase 15 SHIPPED; v0.5.0 ready to tag at user gate.
last_updated: "2026-04-14T18:30:00.000Z"
last_activity: "2026-04-14 11:30 PDT — Phase 15 Wave B docs + CHANGELOG [v0.5.0] + SUMMARY + STATE cursor + tag script complete."
progress:
  total_phases: 9
  completed_phases: 0
  total_plans: 0
  completed_plans: 19
  percent: 100
---

# Project State

## Accumulated Context

### Roadmap Evolution

- Phase 13 added (2026-04-14, session 4): Nested mount layout — pages/ + tree/ symlinks for Confluence parentId hierarchy. Implements OP-1 from HANDOFF.md. BREAKING: flat `<id>.md` at mount root moves to per-backend collection bucket (`pages/` for Confluence, `issues/` for sim+GitHub).
- Phase 14 added (2026-04-14, session 5): Decouple sim REST shape from FUSE write path and git-remote helper — route through `IssueBackend` trait. Closes v0.3-era HANDOFF items 7+8. Cluster B per session-5 brief. Scope v0.4.1 (bugfix/refactor). Rationale: `.planning/SESSION-5-RATIONALE.md`.
- Phase 14 SHIPPED (2026-04-14, session 5, ~09:45 PDT): 4 waves landed on `main` (A=`7510ed1` sim 409-body contract pins · B1=`bdad951`+`cd50ec5` FUSE write through IssueBackend + SG-03 re-home · B2=`938b8de` git-remote helper through IssueBackend · C=`4301d0d` verification). Wave D (docs sweep + CHANGELOG + SUMMARY) complete. HANDOFF.md "Known open gaps" items 7 and 8 closed. `crates/reposix-fuse/src/fetch.rs` + `crates/reposix-fuse/tests/write.rs` + `crates/reposix-remote/src/client.rs` deleted (~830 lines). R1 (assignee-clear-on-null) and R2 (`reposix-core-simbackend-<pid>-{fuse,remote}` attribution) documented as accepted behaviour changes in CHANGELOG `[Unreleased]` `### Changed`. 274 workspace tests green (+2 over LD-14-08 floor), clippy `-D warnings` clean, green-gauntlet `--full` 6/6, smoke 4/4, live demo 01 round-trip green. **Next post-phase gate: user-driven v0.4.1 tag push** via a future `scripts/tag-v0.4.1.sh` (not written yet — deliberate, pending CHANGELOG review).
- Phase 15 added (2026-04-14, session 5, ~10:20 PDT): Dynamic `_INDEX.md` synthesized in FUSE bucket directory (OP-2 partial). Ships `mount/<bucket>/_INDEX.md` as a YAML-frontmatter + pipe-table markdown sitemap, read-only, lazily rendered from the existing issue-list cache. Scope v0.5.0 (feature — adds a new user-visible file). Partial scope: bucket-dir level only; recursive `tree/_INDEX.md`, mount-root `_INDEX.md`, and OP-3 cache-refresh integration deferred. Rationale: `.planning/phases/15-.../15-CONTEXT.md` (10 LDs).
- Phase 15 SHIPPED (2026-04-14, session 5, ~11:30 PDT): 2 waves landed on `main`. **Wave A** = `6a2e256` (reserve `BUCKET_INDEX_INO=5` + inode-layout doc + `reserved_range_is_unmapped` test narrow) · `a94e970` (`feat(15-A): synthesize _INDEX.md in FUSE bucket dir (OP-2 partial)` — `render_bucket_index` pure function, `InodeKind::BucketIndex`, lookup/readdir/getattr/read/write/setattr/release/create/unlink dispatch, `bucket_index_bytes: RwLock<Option<Arc<Vec<u8>>>>` cache on `ReposixFs`, 4 new unit tests in `fs.rs`) · `3309d4c` (`scripts/dev/test-bucket-index.sh` live proof script — starts sim, mounts FUSE, cats `_INDEX.md`, asserts `touch`/`rm`/`echo >` all error, unmounts). **Wave B** = docs + ship prep (CHANGELOG `[v0.5.0] — 2026-04-14`, workspace version bump `0.4.1 → 0.5.0`, Cargo.lock regen, README Folder-structure section mentions `_INDEX.md`, `15-SUMMARY.md`, STATE cursor, `scripts/tag-v0.5.0.sh` clone from v0.4.1). 278 workspace tests green (+4 over Phase 14's 274), clippy `-D warnings` clean, `cargo fmt --all --check` clean. HANDOFF.md OP-2 closed at bucket-dir level. **Next post-phase gate: user-driven v0.5.0 tag push** via `scripts/tag-v0.5.0.sh` (orchestrator runs `green-gauntlet --full` then invokes the script — Wave B executor does NOT invoke it).

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-13)

**Core value:** An LLM agent can `ls`, `cat`, `grep`, edit, and `git push`
issues in a remote tracker without ever seeing a JSON schema or REST endpoint.
**Current focus:** **v0.1 SHIPPED.** All 4 MVD phases + STRETCH Phase S complete.
Demo verified end-to-end on dev host 04:59 PDT. CI green.

## Current Position

Phase: **15 SHIPPED** (latest on the v0.4/v0.5 track; session 5 OP-2 partial).
Plan: 2 waves complete (A = fs impl + tests, B = docs + ship prep — see
`.planning/phases/15-dynamic-index-md-synthesized-in-fuse-bucket-directory-op-2-p/15-SUMMARY.md`).
Cursor: **Phase 15 complete; v0.5.0 ready to tag at user gate.** Next
post-phase step is orchestrator-driven: run `bash
scripts/green-gauntlet.sh --full` (6/6 gates), then `bash
scripts/tag-v0.5.0.sh` (7 safety guards). The Wave B executor has NOT
run the tag script per orchestrator brief. Historical cursor: Phase 14
close-out at 2026-04-14 09:45 PDT — v0.4.1 ready-to-tag —
`.planning/phases/14-.../14-SUMMARY.md`.
Status: SC-15-01 through SC-15-08, SC-15-10 all PASS in Wave A+B; SC-15-09
(green-gauntlet --full) gated on orchestrator. 278 workspace tests pass
(+4 over Phase 14's 274), clippy `-D warnings` clean, `cargo fmt --all
--check` clean, `cargo check --workspace --offline` clean after version
bump, live transcript from `scripts/dev/test-bucket-index.sh` shows 6
seeded issues rendering as YAML frontmatter + pipe-table with valid
columns and `touch`/`rm`/`echo >` all surfacing EROFS/EACCES. HANDOFF.md
OP-2 closed at bucket-dir level.
Last activity: 2026-04-14 11:30 PDT — Phase 15 Wave B close-out.
Historical note — previous `Current Position` entries (Phase 4
close-out at 2026-04-13 05:00 PDT, v0.1 ship) are preserved in
`.planning/phases/04-demo-recording-readme/04-DONE.md`.

Progress: [██████████] 100% (Phases 1, 2, 3, S, 4 all done)

## Performance Metrics

**Velocity:**

- Total plans completed: 0
- Average duration: —
- Total execution time: 0.0 hours (of ~7h total budget, ~4.5h budgeted for MVD)

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| — | — | — | — |

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

Last session: 2026-04-14T18:30:00.000Z
Stopped at: Completed 15-B-docs-changelog-summary-state-tag.md — Phase
15 SHIPPED; v0.5.0 ready to tag at user gate.
Wave-level commit trail on `main` (Phase 15):
`6a2e256` (15-A inode.rs: reserve BUCKET_INDEX_INO=5 + doc + test
narrow), `a94e970` (15-A fs.rs: render_bucket_index + BucketIndex inode
dispatch + 4 new unit tests), `3309d4c` (15-A
scripts/dev/test-bucket-index.sh live proof), `c3d2901` (15-B
CHANGELOG [v0.5.0] + workspace version bump 0.4.1 -> 0.5.0 + README
mention), `f43f0e5` (15-B 15-SUMMARY.md + STATE.md cursor), `ceec233`
(15-B scripts/tag-v0.5.0.sh).
278 workspace tests pass (+4 vs Phase 14 baseline of 274). Clippy
`-D warnings` clean, `cargo fmt --all --check` clean. Cargo.toml
workspace version bumped `0.4.1 → 0.5.0`; Cargo.lock regenerated via
`cargo check --workspace --offline`.
Resume file: None (phase self-contained).
Cursor next: **v0.5.0 tag push** (orchestrator gate — run `bash
scripts/green-gauntlet.sh --full`, then `bash scripts/tag-v0.5.0.sh`).
After tag: pick the next session-5 stretch goal — Cluster C (swarm
confluence-direct), tree-level `_INDEX.md` recursive synthesis
(remaining half of OP-2), OP-3 (`reposix refresh` cache-refresh
subcommand), OP-7 SSRF + contention probes, or Cluster A (Confluence
writes — unblocked by Phase 14).

### Previous session (Phase 14, for reference)

2026-04-14T16:45:00.000Z. Stopped at: Completed 14-D-docs-changelog.md
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
