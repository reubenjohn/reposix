---
phase: 20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount
plan: 20-C
subsystem: reposix-cli
tags: [cli, changelog, release, docs, state]
requirements: [REFRESH-01, REFRESH-02, REFRESH-03, REFRESH-04, REFRESH-05]

dependency_graph:
  requires:
    - reposix-cli::refresh (Wave A)
    - reposix-cli::cache_db (Wave A)
    - crates/reposix-cli/tests/refresh_integration.rs (Wave B)
  provides:
    - CHANGELOG.md [Unreleased] Phase 20 OP-3 entry
    - STATE.md Phase 20 SHIPPED cursor
    - ROADMAP.md Phase 20 all three plans checked
  affects:
    - CHANGELOG.md
    - .planning/STATE.md
    - .planning/ROADMAP.md

tech_stack:
  added: []
  patterns:
    - CHANGELOG [Unreleased] append pattern (no version promotion — v0.6.0 already set by Phase 16)

key_files:
  created:
    - .planning/phases/20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount/20-C-SUMMARY.md
  modified:
    - CHANGELOG.md
    - .planning/STATE.md
    - .planning/ROADMAP.md

decisions:
  - CHANGELOG entry added under [Unreleased] (not a new version header) because workspace
    version 0.6.0 was already promoted in Phase 16 Wave D; promoting again would duplicate
    the version tag. Phase 20 changes are part of the same v0.6.0 milestone.
  - Workspace Cargo.toml version not bumped (already 0.6.0 from Phase 16; no version
    change required for documentation-only wave).

metrics:
  duration_minutes: 5
  completed_at: "2026-04-15T00:00:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 3
---

# Phase 20 Plan C: CHANGELOG + STATE update (release finalization) — Summary

**One-liner:** Workspace CI gate confirmed clean (test/clippy/fmt), Phase 20 OP-3 `reposix refresh` entry added under CHANGELOG `[Unreleased]`, STATE.md cursor advanced to Phase 20 SHIPPED + Milestone v0.6.0 complete, ROADMAP.md Phase 20 all three plans marked done.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| C-1 | Workspace gate + CHANGELOG Phase 20 entry | `4012f38` | `CHANGELOG.md` |
| C-2 | STATE.md + ROADMAP.md updates + SUMMARY | this commit | `.planning/STATE.md`, `.planning/ROADMAP.md`, `20-C-SUMMARY.md` |

## What Was Built

### CHANGELOG.md

Added `### Added — Phase 20: OP-3 — reposix refresh subcommand + git-diff cache` section under `[Unreleased]`, covering:
- `reposix refresh` subcommand (REFRESH-01..05): deterministic `.md` write, git commit, FUSE guard, `fetched_at.txt`, `--offline` forward-compat stub.
- `cache_db` module: SQLite WAL + EXCLUSIVE lock at `<mount>/.reposix/cache.db` (0600).

No version header promoted — `[v0.6.0]` was already set by Phase 16 Wave D. Phase 20 additions remain under `[Unreleased]` until the human-gate `scripts/tag-v0.6.0.sh` executes.

### STATE.md

- `stopped_at` updated: "Phase 20 SHIPPED. Milestone v0.6.0 complete (Phases 16-20). Phases 21+ are v0.7.0 scope."
- `current_phase` cursor updated: Phase 20 SHIPPED, Phase 21 (OP-7 hardening) is next.
- `Plan` count updated: 3 of 3 waves complete.

### ROADMAP.md

Phase 20 plan list updated — all three plans marked `[x]`:
- `[x] 20-A-refresh-cmd.md`
- `[x] 20-B-tests-and-polish.md`
- `[x] 20-C-docs-and-release.md`

## Phase 20 Complete — What Was Shipped (Waves A+B+C)

| Wave | Commit(s) | Key Deliverables |
|------|-----------|-----------------|
| A | `2be3b45`, `83f6fa3` | `cache_db.rs` (SQLite WAL store), `refresh.rs` (run_refresh + is_fuse_active + git_refresh_commit), `main.rs` Cmd::Refresh wiring |
| B | `583b12b` | `lib.rs` dual-target, 4 integration tests (file write, idempotency, FUSE guard, timestamp), workspace green-gauntlet 0/0/0 |
| C | `4012f38`, this | CHANGELOG entry, STATE cursor, ROADMAP plans checked, SUMMARY |

**New files:** `crates/reposix-cli/src/refresh.rs`, `crates/reposix-cli/src/cache_db.rs`, `crates/reposix-cli/src/lib.rs`, `crates/reposix-cli/tests/refresh_integration.rs`

**New public API:** `run_refresh`, `run_refresh_inner`, `is_fuse_active`, `git_refresh_commit`, `CacheDb::open`, `update_metadata`

## Verification Results

1. `cargo test --workspace --quiet` — 0 failures
2. `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings/errors
3. `cargo fmt --all --check` — 0 diffs
4. `grep "0.6.0" Cargo.toml` — version present
5. `grep "reposix refresh" CHANGELOG.md` — entry present
6. STATE.md `stopped_at` contains "Phase 20 SHIPPED"

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Deviation] Cargo.toml version already at 0.6.0**
- **Found during:** Task C-1 pre-flight
- **Issue:** The plan spec calls for bumping workspace version to 0.6.0, but Phase 16 Wave D already performed this bump (`0.5.0 → 0.6.0`). Bumping again would be a no-op that creates a misleading commit.
- **Fix:** Skipped the version bump step; verified `version = "0.6.0"` already present; documented in decisions.
- **Files modified:** none (no change needed)

**2. [Rule 1 - Deviation] CHANGELOG entry placed under [Unreleased] not as new [v0.6.0] header**
- **Found during:** Task C-1 CHANGELOG read
- **Issue:** The plan spec says to add a `[v0.6.0]` section, but a `## [v0.6.0] — 2026-04-14` section already exists (Phase 16). Creating a second `[v0.6.0]` would be a malformed CHANGELOG.
- **Fix:** Added Phase 20 content as a new sub-section under existing `[Unreleased]` block, consistent with Phase 19 and Phase 18 entries above it.
- **Files modified:** `CHANGELOG.md`

## Known Stubs

None — all documentation accurately reflects implemented functionality. The `--offline` flag stub is explicitly documented as "deferred to Phase 21" in both CHANGELOG and code.

## Deferred Items

- Offline FUSE read path (Phase 21): `--offline` flag exists but errors immediately.
- `_INDEX.md` committed in refresh (Phase 21): refresh writes issue `.md` files only; `_INDEX.md` re-synthesis on refresh deferred.
- `spaces/` multi-space mount (Phase 21): requires `IssueBackend::list_spaces` API-breaking addition.
- Full SQLite issue cache replacing DashMap (Phase 21+): `cache.db` currently stores metadata only.
- `reposix-cache` as its own crate (Phase 21+): `cache_db.rs` lives in `reposix-cli` for now.

## Threat Flags

None — this plan modifies only documentation and planning artifacts. No new network endpoints, auth paths, file access patterns, or schema changes.

## Self-Check: PASSED

- `CHANGELOG.md` — entry present under [Unreleased] — FOUND
- `.planning/STATE.md` — "Phase 20 SHIPPED" in stopped_at — FOUND
- `.planning/ROADMAP.md` — all three Phase 20 plans marked [x] — FOUND
- Commit `4012f38` — FOUND (docs(20-C-task1))
- `cargo test --workspace --quiet` — 0 failures
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings
- `cargo fmt --all --check` — 0 diffs
