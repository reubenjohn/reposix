---
phase: 20-op-3-reposix-refresh-subcommand-and-git-diff-cache-for-mount
plan: 20-A
subsystem: reposix-cli
tags: [cli, sqlite, git, refresh, cache]
requirements: [REFRESH-01, REFRESH-03, REFRESH-04, REFRESH-05]

dependency_graph:
  requires: []
  provides:
    - reposix-cli::cache_db (CacheDb, open_cache_db, update_metadata)
    - reposix-cli::refresh (RefreshConfig, run_refresh, is_fuse_active, git_refresh_commit)
    - reposix CLI Refresh subcommand
  affects:
    - crates/reposix-cli/src/main.rs (Cmd enum extended)
    - crates/reposix-cli/Cargo.toml (chrono dep added)

tech_stack:
  added:
    - chrono (workspace dep, now explicit in reposix-cli/Cargo.toml)
  patterns:
    - SQLite WAL + EXCLUSIVE locking for advisory concurrency guard
    - rustix::process::test_kill_process for signal-0 PID liveness check
    - std::process::Command for git subprocess orchestration
    - reposix_core::frontmatter::render for deterministic .md serialization

key_files:
  created:
    - crates/reposix-cli/src/cache_db.rs
    - crates/reposix-cli/src/refresh.rs
  modified:
    - crates/reposix-cli/src/main.rs
    - crates/reposix-cli/Cargo.toml

decisions:
  - Parse fuse.pid as i32 (not u32) to avoid cast_possible_wrap lint; Linux
    PID_MAX fits in i32 (max 4_194_304).
  - Use rustix::process::test_kill_process (signal-0) for PID liveness rather
    than /proc/<pid>/status — cleaner API, same safety guarantee.
  - conn() method on CacheDb is #[cfg(test)] only; non-test code uses the
    public update_metadata free function, avoiding dead_code lint.
  - OpenOptions uses .truncate(false) explicitly — we never want to wipe an
    existing DB file on open.

metrics:
  duration_minutes: 35
  completed_at: "2026-04-15T08:37:44Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 2
---

# Phase 20 Plan A: `reposix refresh` subcommand + `CacheDb` SQLite metadata store — Summary

**One-liner:** `reposix refresh` subcommand with WAL-locked SQLite metadata store, signal-0 FUSE guard, deterministic `.md` rendering via `frontmatter::render`, and `git init`/`add`/`commit` subprocess to produce a time-machine snapshot commit.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| A-1 | `cache_db.rs` — SQLite metadata store | `2be3b45` | `crates/reposix-cli/src/cache_db.rs` |
| A-2 | `refresh.rs` + `main.rs` wiring | `83f6fa3` | `crates/reposix-cli/src/refresh.rs`, `main.rs`, `Cargo.toml` |

## What Was Built

### `cache_db.rs`

- `CacheDb` newtype wrapping `rusqlite::Connection` — holds WAL EXCLUSIVE lock for its lifetime.
- `open_cache_db(mount: &Path) -> Result<CacheDb>`: creates `.reposix/` dir, pre-creates `cache.db` with `0o600` permissions, opens with WAL + EXCLUSIVE mode, applies `refresh_meta` schema.
- `update_metadata(db, backend_name, project, last_fetched_at, commit_sha)`: `INSERT OR REPLACE` single-row sentinel.
- `map_busy`: maps `SQLITE_BUSY` → human error "another refresh is in progress".
- 4 unit tests: `open_creates_schema`, `update_metadata_roundtrip`, `lock_conflict_returns_error`, `open_is_idempotent`.

### `refresh.rs`

- `RefreshConfig` struct + `backend_label()` method.
- `run_refresh(cfg)`: guards `--offline` (error) and live FUSE mount, opens cache DB, fetches issues via backend dispatch, writes `<bucket>/<padded-id>.md`, writes `fetched_at.txt` + `.reposix/.gitignore`, calls `git_refresh_commit`, updates metadata, prints summary.
- `is_fuse_active(mount)`: reads `.reposix/fuse.pid`, uses `rustix::process::test_kill_process` (signal-0) to check liveness; ESRCH → stale pid → inactive.
- `git_refresh_commit(mount, bucket, author, message)`: `git -C <mount> init` (idempotent), selective `git add`, `git commit --allow-empty` with explicit `GIT_AUTHOR_*` / `GIT_COMMITTER_*` env vars for bare CI.
- 4 unit tests: `fuse_active_with_live_pid`, `fuse_inactive_no_pid_file`, `fuse_inactive_dead_pid`, `git_refresh_commit_creates_commit`.

### `main.rs` additions

- `mod refresh;` declaration.
- `Cmd::Refresh` variant with `mount_point`, `--origin`, `--project`, `--backend`, `--offline` flags.
- Dispatch arm calling `refresh::run_refresh(RefreshConfig { … }).await`.

## Verification Results

1. `cargo test -p reposix-cli -- cache_db` — 4/4 green
2. `cargo test -p reposix-cli -- refresh` — 4/4 green
3. `cargo clippy -p reposix-cli -- -D warnings` — zero warnings/errors
4. `cargo check --workspace` — clean
5. `reposix refresh --help` — shows subcommand with all flags documented

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `conn()` caused dead_code lint in non-test builds**
- **Found during:** Task 2 clippy pass
- **Issue:** `conn()` was declared `pub` but only used inside `#[cfg(test)]` blocks.
- **Fix:** Added `#[cfg(test)]` attribute to `conn()` method.
- **Files modified:** `crates/reposix-cli/src/cache_db.rs`

**2. [Rule 1 - Bug] `OpenOptions` missing `.truncate(false)` triggered `suspicious_open_options` lint**
- **Found during:** Task 2 clippy pass
- **Issue:** Opening a file with `.create(true)` without explicit `.truncate()` is flagged.
- **Fix:** Added `.truncate(false)` — correct semantics: never wipe an existing DB on open.
- **Files modified:** `crates/reposix-cli/src/cache_db.rs`

**3. [Rule 1 - Bug] `u32 as i32` cast triggered `cast_possible_wrap` lint**
- **Found during:** Task 2 clippy pass
- **Issue:** PID was parsed as `u32` then cast to `i32` for `rustix::process::Pid::from_raw`.
- **Fix:** Parse directly as `i32` — Linux PID_MAX (4,194,304) fits in i32; doc comment explains.
- **Files modified:** `crates/reposix-cli/src/refresh.rs`

**4. [Rule 1 - Bug] Two `SQLite` occurrences in doc comments missing backticks (`doc_markdown` lint)**
- **Found during:** Task 2 clippy pass
- **Fix:** Wrapped as `` `SQLite` `` in module doc and `open_cache_db` error doc.
- **Files modified:** `crates/reposix-cli/src/cache_db.rs`

**5. [Rule 2 - Missing dep] `chrono` not in `reposix-cli/Cargo.toml`**
- **Found during:** Task 2 compile
- **Issue:** `refresh.rs` uses `chrono::Utc::now()` but `chrono` was not listed as a direct dep of `reposix-cli`.
- **Fix:** Added `chrono.workspace = true` to `crates/reposix-cli/Cargo.toml`.
- **Files modified:** `crates/reposix-cli/Cargo.toml`

## Known Stubs

None — all required functionality is implemented. The `--offline` flag is intentionally stubbed with a clear error message ("offline mode is not yet implemented for refresh; Phase 21") per the plan spec.

## Threat Flags

No new threat surface beyond what the plan's threat model covers. All T-20A mitigations are implemented:
- T-20A-01: `--author` passed as single `--author=VALUE` arg to `Command::args` (no shell interpolation).
- T-20A-02: `cache.db` created with `0o600` via `OpenOptions::mode(0o600)`.
- T-20A-05: `SQLITE_BUSY` → clear error "another refresh is in progress" (no blocking wait).

## Self-Check: PASSED

- `crates/reposix-cli/src/cache_db.rs` — FOUND
- `crates/reposix-cli/src/refresh.rs` — FOUND
- Commit `2be3b45` — FOUND (feat(20-A-task1))
- Commit `83f6fa3` — FOUND (feat(20-A))
- All 11 unit tests pass, clippy clean, workspace check clean
