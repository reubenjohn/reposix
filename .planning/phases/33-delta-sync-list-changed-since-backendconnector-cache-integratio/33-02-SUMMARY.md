---
phase: 33
plan: 02
title: "Cache::sync (delta) + helper integration + audit op='delta_sync'"
status: complete
---

# Phase 33 Plan 02 — Summary

## What shipped

`Cache::sync()` — the cache-level entry point that reads
`meta.last_fetched_at`, calls `BackendConnector::list_changed_since`
(landed in Plan 01), eagerly materializes each changed issue's blob,
rebuilds the full tree, commits, and atomically updates the cursor +
oid_map + delta_sync audit row in one SQLite transaction. The helper's
`stateless-connect` handler now calls `cache.sync()` instead of
`cache.build_from()` — so first invocation (no cursor) seeds, every
subsequent invocation does a delta.

## Tasks

- **02-T01** — Schema CHECK widened to include `'delta_sync'` (and the
  helper_* ops the audit module already inserts in best-effort mode).
- **02-T02** — `audit::log_delta_sync_tx` transaction-scoped helper +
  3 unit tests including a rollback-atomicity check.
- **02-T03** — `Cache::sync` 5-step flow with atomic transaction at the end.
- **02-T04** — `crates/reposix-remote/src/stateless_connect.rs` calls
  `cache.sync()` (single tree-sync entry point — every helper invocation
  writes either a `tree_sync` (seed) or `delta_sync` (incremental) audit row).
- **02-T05** — Three integration tests against in-process `reposix-sim`:
  - `delta_sync_updates_only_changed_issue` — headline ground-truth test:
    one mutation produces exactly one changed blob OID; other 4 are
    pin-equal.
  - `delta_sync_empty_delta_still_writes_audit_and_bumps_cursor`.
  - `delta_sync_atomic_on_backend_error_midsync` — proves cursor stays put
    and no `delta_sync` audit row is written when the backend errors.
- **02-T06** — Workspace gate (check / clippy / test) clean.

## Tests added

- `reposix-cache::audit::tests::log_delta_sync_tx_inserts_row`
- `reposix-cache::audit::tests::log_delta_sync_tx_roll_back_does_not_leak_row`
- `reposix-cache::audit::tests::log_delta_sync_tx_handles_null_since`
- `reposix-cache::tests::delta_sync::delta_sync_updates_only_changed_issue`
- `reposix-cache::tests::delta_sync::delta_sync_empty_delta_still_writes_audit_and_bumps_cursor`
- `reposix-cache::tests::delta_sync::delta_sync_atomic_on_backend_error_midsync`

Net new tests: **+6**.

## Key decisions

- **Atomicity at the SQLite layer**: `oid_map` upserts + `last_fetched_at`
  update + `delta_sync` audit row all commit in one `rusqlite::Transaction`.
  Either all three persist or none do.
- **Tree sync is unconditional full** (per CONTEXT.md "Tree sync vs. blob
  materialization (locked)"). Re-list all issues, recompute hashes for
  unchanged items (don't write the blob — lazy), reuse freshly-written
  OIDs for changed items.
- **Empty-delta sync still writes a `delta_sync` audit row** with `bytes=0`.
  Audit history has one row per fetch; the row is the proof of
  liveness/cursor advancement.
- **Helper has ONE tree-sync entry point** now (`cache.sync`). Every
  invocation that reaches the upload-pack tunnel writes either `tree_sync`
  (seed) or `delta_sync` (incremental). No code path bypasses the audit.
- **Seed path forwards to `build_from`** rather than duplicating its
  tree-assembly logic. The two methods share the same tree shape.
- **Integration test uses real `reposix-sim`** (not wiremock) — the
  ground-truth assertion is on git's view of the bare repo, not on
  bookkeeping mocks (per OP-6).
- **Test HTTP goes through `reposix_core::http::client()`** — clippy's
  `disallowed-methods` enforces this, keeping integration tests on the
  same SG-01 allowlist code path as production.

## Commits

- `feat(33-02): extend audit_events_cache CHECK to include 'delta_sync'` — 790dec4
- `feat(33-02): log_delta_sync_tx — transaction-scoped audit helper` — 9be571e
- `feat(33-02): Cache::sync — atomic delta materialization` — dd555c9
- `feat(33-02): helper stateless-connect calls Cache::sync before tunnel` — 0b53d94
- `test(33-02): integration test — end-to-end delta sync against reposix-sim` — 23fac7c

## Verification commands

```bash
cargo check --workspace                                   # exits 0
cargo clippy --workspace --all-targets -- -D warnings     # exits 0
cargo test --workspace                                    # all green
```

## Hand-off to Phase 34

- `Cache::sync()` is the single entry point Phase 34 (helper push) should
  call before any push-side work.
- `SyncReport { changed_ids, since, new_commit }` is exported at the crate
  root (`pub use builder::SyncReport`).
- Audit op `'delta_sync'` is reserved for delta-sync invocations only;
  Phase 34's push side should add a new op (e.g. `'helper_push'`) rather
  than reusing `'delta_sync'`.
- Helper integration point: `stateless_connect.rs` calls `cache.sync()`
  before the upload-pack tunnel. Phase 34 will add a parallel `command=push`
  (or `export` capability) handler that calls `cache.sync()` PRE-push, then
  PATCHes the backend, then writes a `helper_push` audit row.
- No trait surface changes beyond what Plan 01 added.
