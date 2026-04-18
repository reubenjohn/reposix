---
phase: 23
plan: "02"
subsystem: reposix-cli
tags: [cli, confluence, spaces, subcommand]
dependency_graph:
  requires: ["23-01"]
  provides: ["reposix spaces subcommand", "ConfluenceBackend::list_spaces CLI surface"]
  affects: ["crates/reposix-cli/src/main.rs", "crates/reposix-cli/src/list.rs", "crates/reposix-cli/src/lib.rs"]
tech_stack:
  added: []
  patterns: ["pub(crate) visibility promotion", "clap Subcommand derive", "ListBackend reuse across modules"]
key_files:
  created:
    - crates/reposix-cli/src/spaces.rs
  modified:
    - crates/reposix-cli/src/list.rs
    - crates/reposix-cli/src/lib.rs
    - crates/reposix-cli/src/main.rs
decisions:
  - "Default --backend for spaces is confluence (not sim) since only confluence is supported"
  - "render_spaces_table is a private fn; only run() is pub async — matches list.rs pattern"
  - "read_confluence_env promoted to pub(crate) rather than duplicated; DRY"
metrics:
  duration: "~15 minutes"
  completed: "2026-04-16"
  tasks_completed: 2
  files_modified: 4
---

# Phase 23 Plan 02: spaces subcommand Summary

`reposix spaces --backend confluence` CLI subcommand listing all readable Confluence spaces as a pipe-friendly fixed-width table of KEY / NAME / URL.

## What Was Built

- **`crates/reposix-cli/src/spaces.rs`** — new module with `pub async fn run(backend: ListBackend)` entry point. Sim and Github arms bail with clear messages; Confluence arm calls `read_confluence_env()` then `ConfluenceBackend::list_spaces()` and pipes through `render_spaces_table()`.
- **`crates/reposix-cli/src/list.rs`** — `read_confluence_env` and `read_confluence_env_from` promoted from `fn` to `pub(crate) fn` (no signature change, no behaviour change).
- **`crates/reposix-cli/src/lib.rs`** — `pub mod spaces;` added alongside existing module declarations.
- **`crates/reposix-cli/src/main.rs`** — `use reposix_cli::spaces;` import added; `Cmd::Spaces { backend }` variant added with full clap doc comments; dispatch arm `Cmd::Spaces { backend } => spaces::run(backend).await` added.

## CLI Help Output

```
reposix spaces [OPTIONS]

Options:
      --backend <BACKEND>  [default: confluence]
  -h, --help               Print help
```

`reposix --help` shows `spaces` as a registered subcommand. `reposix spaces --help` shows `--backend` flag with all three enum variants.

## Tests Passing

- 3 new unit tests in `spaces::tests`: `render_table_prints_header_and_rows`, `sim_backend_returns_clear_error`, `github_backend_returns_clear_error`
- All 14 `reposix-cli` lib unit tests pass (11 pre-existing + 3 new)
- All integration tests pass (4 in `tests/`)
- `cargo clippy -p reposix-cli --all-targets -- -D warnings` exits 0

## UX Notes vs PATTERNS.md

- Column widths: KEY=12, NAME=30, URL=remaining — matches the "pipe-friendly fixed-width" constraint from CONTEXT.md; same visual style as the issues table in `list.rs` (ID=10, STATUS=12).
- `render_spaces_table` is private (not `pub`) — only `run()` is exported, consistent with `render_table` in `list.rs`.

## Threat Model Compliance

- **T-23-02-01** (env-var names-only): `read_confluence_env` unchanged in behaviour; T-11B-01 test in `list.rs` still passes. Missing-env error from `spaces` confirmed to print names only, no values.
- **T-23-02-02** (redact_url in error paths): inherited from Plan 01's `ConfluenceBackend::list_spaces` implementation.
- **T-23-02-03** (ANSI escapes in space names): accepted risk per threat register; no mitigation in v0.7.0.
- **T-23-02-04** (SSRF): inherited from Plan 01's allowlist enforcement.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. The `spaces::run()` function is fully wired to `ConfluenceBackend::list_spaces`; no placeholder data paths.

## Threat Flags

None. No new network endpoints, auth paths, file access patterns, or schema changes beyond what the plan's threat model already covers.

## Self-Check: PASSED

- `crates/reposix-cli/src/spaces.rs`: FOUND
- `crates/reposix-cli/src/lib.rs` contains `pub mod spaces`: FOUND
- `crates/reposix-cli/src/main.rs` contains `Cmd::Spaces` dispatch arm: FOUND
- Commit `45feb90` (Task 1): FOUND
- Commit `10825ef` (Task 2): FOUND
- 3 spaces unit tests pass: CONFIRMED
- clippy clean: CONFIRMED
