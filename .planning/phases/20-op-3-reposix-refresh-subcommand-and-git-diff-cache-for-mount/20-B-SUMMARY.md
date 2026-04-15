---
phase: 20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount
plan: 20-B
subsystem: reposix-cli
tags: [cli, integration-tests, git, refresh, workspace-gate]
requirements: [REFRESH-01, REFRESH-02, REFRESH-03, REFRESH-04]

dependency_graph:
  requires:
    - reposix-cli::refresh (run_refresh, run_refresh_inner — Wave A)
    - reposix-cli::cache_db (CacheDb — Wave A)
  provides:
    - crates/reposix-cli/tests/refresh_integration.rs (4 integration tests)
    - crates/reposix-cli/src/lib.rs (library target for integration test access)
  affects:
    - crates/reposix-cli/src/refresh.rs (run_refresh_inner extracted, pub)
    - crates/reposix-cli/src/main.rs (mod declarations use lib target)
    - crates/reposix-cli/src/cache_db.rs (doc lint fix)

tech_stack:
  added: []
  patterns:
    - Binary crate lib.rs dual-target pattern for integration test access
    - run_refresh_inner factored fn for network-free testing
    - chrono::DurationRound for second-precision timestamp comparison

key_files:
  created:
    - crates/reposix-cli/src/lib.rs
    - crates/reposix-cli/tests/refresh_integration.rs
  modified:
    - crates/reposix-cli/src/refresh.rs
    - crates/reposix-cli/src/main.rs
    - crates/reposix-cli/src/cache_db.rs

decisions:
  - Added src/lib.rs to expose refresh, list, cache_db modules publicly so
    integration tests (separate compilation unit) can call run_refresh_inner
    directly without subprocess overhead; main.rs imports from the lib target.
  - run_refresh_inner is pub (not pub(crate)) so it is reachable from the
    integration test crate linked against the library target.
  - db parameter is Option<&CacheDb> so inner fn works with no DB in tests.

metrics:
  duration_minutes: 7
  completed_at: "2026-04-15T08:44:51Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 3
---

# Phase 20 Plan B: Integration tests for `reposix refresh` + workspace green-gauntlet — Summary

**One-liner:** 4 integration tests proving the full refresh flow (file write, idempotency, FUSE guard, timestamp) via `run_refresh_inner` with inline issues; workspace gate 0/0/0 (tests/clippy/fmt).

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| B-1 | Integration tests — full refresh flow | `583b12b` | `crates/reposix-cli/tests/refresh_integration.rs`, `src/lib.rs`, `src/refresh.rs` |
| B-2 | Workspace gate — full suite green + clippy clean | `583b12b` | `src/cache_db.rs`, `src/refresh.rs` (lint fixes) |

## What Was Built

### `src/lib.rs` — library target for integration test access

A binary crate (`[[bin]]`) cannot be imported by integration tests (separate compilation units) without a `lib.rs`. Added a minimal `lib.rs` that re-exports `pub mod cache_db`, `pub mod list`, `pub mod refresh`. Updated `main.rs` to import those modules via `use reposix_cli::{list, refresh}` from the lib target instead of declaring them as private `mod` items.

### `tests/refresh_integration.rs` — 4 integration tests

**`refresh_writes_md_files`**
Calls `run_refresh_inner` with a single synthetic `Issue`. Asserts:
- `issues/00000000001.md` exists and starts with `---\n` (YAML frontmatter)
- `git log --oneline` shows exactly 1 commit
- `git log --format=%an` shows author name containing "reposix"

**`refresh_idempotent_no_diff`**
Calls `run_refresh_inner` twice with identical issues. Asserts:
- `git log --oneline` shows exactly 2 commits
- `git diff HEAD~1 -- issues/` is empty (no issue content changed between runs)

**`refresh_fuse_active_guard`**
Writes current process PID to `.reposix/fuse.pid`. Calls `run_refresh` (the public fn, not inner). Asserts error message contains "FUSE mount is active".

**`fetched_at_is_current_timestamp`**
Calls `run_refresh_inner`. Reads `.reposix/fetched_at.txt`. Parses as `chrono::DateTime<Utc>`. Asserts timestamp is within the `[before_secs, after]` window (before truncated to whole seconds to match second-precision RFC3339 format stored on disk).

### `run_refresh_inner` refactor in `refresh.rs`

Split `run_refresh` into:
- `run_refresh` (pub async): guards offline + FUSE, opens DB, fetches network issues, calls inner
- `run_refresh_inner` (pub sync): receives `Vec<Issue>` + `Option<&CacheDb>`, writes files, commits, updates DB if provided

No behaviour change for production callers; `run_refresh` behaviour is identical.

## Verification Results

1. `cargo test -p reposix-cli --test refresh_integration` — 4/4 green
2. `cargo test --workspace --quiet` — 0 failures across all crates
3. `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings/errors
4. `cargo fmt --all --check` — 0 diffs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Binary crate has no lib target — integration tests cannot import modules**
- **Found during:** Task B-1 first compile attempt
- **Issue:** `reposix-cli` is a `[[bin]]` crate with no `[lib]` target. Integration tests in `tests/` compile as a separate crate linked against the library. Without `lib.rs`, `use reposix_cli::refresh` fails to compile.
- **Fix:** Added `src/lib.rs` with `pub mod cache_db; pub mod list; pub mod refresh;`. Updated `main.rs` to use `use reposix_cli::{list, refresh}` from the lib target.
- **Files modified:** `crates/reposix-cli/src/lib.rs` (created), `src/main.rs`

**2. [Rule 1 - Bug] `refresh_writes_md_files` test asserted commit author *email* instead of author *name***
- **Found during:** Task B-1 first test run
- **Issue:** Commit author email is `simulator@test-project` (not containing "reposix"); the author *name* is `reposix`. Used `--format=%ae` (email) but should use `--format=%an` (name).
- **Fix:** Changed `--format=%ae` to `--format=%an` in the assertion.
- **Files modified:** `crates/reposix-cli/tests/refresh_integration.rs`

**3. [Rule 1 - Bug] `fetched_at_is_current_timestamp` timestamp precision mismatch**
- **Found during:** Task B-1 first test run
- **Issue:** `fetched_at.txt` stores second-precision RFC3339 (`2026-04-15T08:41:58Z`) but `before` was captured with nanosecond precision. The truncated-to-second timestamp falls *before* `before` (which has sub-second component > 0), so `ts >= before` is false.
- **Fix:** Truncate `before` to whole seconds via `chrono::DurationRound::duration_trunc` before comparison.
- **Files modified:** `crates/reposix-cli/tests/refresh_integration.rs`

**4. [Rule 1 - Bug] `fuse_inactive_dead_pid` unit test: `identical_match_arms` clippy lint**
- **Found during:** Task B-2 clippy run
- **Issue:** Two match arms (`Ok(false) => {}` and `Err(_) => {}`) have identical empty bodies — `clippy::identical_match_arms` fires.
- **Fix:** Replaced the match with `assert!(!matches!(result, Ok(true)), ...)`.
- **Files modified:** `crates/reposix-cli/src/refresh.rs`

**5. [Rule 1 - Bug] `doc_markdown` clippy lint in `cache_db.rs` test doc comment**
- **Found during:** Task B-2 clippy run
- **Issue:** `NOTE: SQLite WAL ...` in a doc comment triggers `clippy::doc_markdown` (unbackticked identifier).
- **Fix:** Changed to `NOTE: \`SQLite\` WAL ...`
- **Files modified:** `crates/reposix-cli/src/cache_db.rs`

## Known Stubs

None — all four integration tests exercise real refresh logic and real git subprocess output.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced in this plan. Tests use `tempfile::tempdir()` (0700) with synthetic issue data only (T-20B-02 accepted).

## Self-Check: PASSED

- `crates/reposix-cli/src/lib.rs` — FOUND
- `crates/reposix-cli/tests/refresh_integration.rs` — FOUND
- `crates/reposix-cli/src/refresh.rs` — run_refresh_inner pub fn present — FOUND
- Commit `583b12b` — FOUND
- `cargo test --workspace --quiet` — 0 failures
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings
- `cargo fmt --all --check` — 0 diffs
