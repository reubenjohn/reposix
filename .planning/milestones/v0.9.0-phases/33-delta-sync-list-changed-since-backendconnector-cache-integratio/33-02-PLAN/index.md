---
phase: 33
plan: 02
title: "Cache::sync (delta) + helper integration + audit op='delta_sync'"
wave: 2
depends_on: [33-01]
requirements: [ARCH-07]
files_modified:
  - crates/reposix-cache/fixtures/cache_schema.sql
  - crates/reposix-cache/src/audit.rs
  - crates/reposix-cache/src/meta.rs
  - crates/reposix-cache/src/cache.rs
  - crates/reposix-cache/src/builder.rs
  - crates/reposix-cache/src/lib.rs
  - crates/reposix-cache/tests/delta_sync.rs
  - crates/reposix-remote/src/stateless_connect.rs
autonomous: true
mode: standard
---

# Phase 33 Plan 02 — `Cache::sync` + helper integration

<objective>
Add `Cache::sync()` to the `reposix-cache` crate: read `meta.last_fetched_at`,
call `BackendConnector::list_changed_since(project, last_fetched_at)`, fetch
each changed issue via `get_issue`, materialize into the bare repo (new or
updated blob + new tree + new commit), then within **one SQLite transaction**
update `meta.last_fetched_at`, upsert the delta'd `oid_map` rows, and write an
`op='delta_sync'` audit row. Wire the helper's `stateless-connect` entry point
(currently calls `cache.build_from()` every time — see
`crates/reposix-remote/src/stateless_connect.rs:98`) to call `cache.sync()`
instead when `meta.last_fetched_at` is present (delta path), falling through
to `build_from()` on first open (seed path).
</objective>

## Chapters

- [T01 — Extend schema CHECK constraint for `op='delta_sync'`](./t01-schema.md) — Edit `cache_schema.sql` to add `'delta_sync'` to the `op IN (...)` CHECK constraint.
- [T02 — `audit::log_delta_sync` helper + unit test](./t02-audit.md) — Add the transaction-scoped `log_delta_sync_tx` function and unit tests proving atomicity.
- [T03 — `Cache::sync` — atomic delta materialization](./t03-cache-sync.md) — Implement `Cache::sync` with `SyncReport`, the full delta-sync algorithm, and atomic SQLite transaction.
- [T04 — Wire helper to call `Cache::sync` on `stateless-connect`](./t04-helper-wire.md) — Replace the direct `cache.build_from()` call in the helper with `cache.sync()`.
- [T05 — Integration test: end-to-end delta sync against SimBackend](./t05-integration-test.md) — Three integration tests covering delta path, empty delta, and atomicity on backend error.
- [T06 — Workspace gate](./t06-workspace-gate.md) — Run `cargo check`, `clippy`, and `cargo test` across the workspace; fix residual lint/test failures.

## Dependencies

- Depends on Plan 01 (`33-01`) which adds `BackendConnector::list_changed_since`.
- Requirement ARCH-07.

## Canonical refs

- `.planning/phases/33-.../33-CONTEXT.md` §Atomic ordering (locked).
- `.planning/phases/33-.../33-CONTEXT.md` §Tree sync vs. blob materialization (locked).
- `crates/reposix-cache/src/builder.rs` — existing `build_from` and `read_blob` for the tree-assembly pattern.
- `crates/reposix-cache/src/audit.rs` — existing audit log patterns.
- `crates/reposix-cache/src/meta.rs` — existing meta get/set + oid_map helpers.
- `crates/reposix-cache/fixtures/cache_schema.sql` — `audit_events_cache.op CHECK` constraint to extend.
- `crates/reposix-remote/src/stateless_connect.rs:93-99` — tree-sync call site to rewire.
