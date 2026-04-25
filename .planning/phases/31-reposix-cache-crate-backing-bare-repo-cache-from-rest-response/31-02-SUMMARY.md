---
phase: 31
plan: 02
status: complete
completed_at: 2026-04-24
---

# Phase 31 Plan 02 — Summary

## Objective achieved

Wired the three non-negotiable operating-principle hooks on the
cache:

1. **Append-only SQLite audit.** `audit_events_cache` table with
   `BEFORE UPDATE` / `BEFORE DELETE` triggers. `cache.db` opened with
   `mode=0o600`, `SQLITE_DBCONFIG_DEFENSIVE`, WAL.
2. **Egress allowlist → audited denial.** `Cache::read_blob` catches
   `reposix_core::Error::InvalidOrigin` (typed + substring fallback),
   fires `op='egress_denied'` audit row BEFORE returning the typed
   `Error::Egress(..)`.
3. **Tainted-by-default return.** `Cache::read_blob` public signature
   is `pub async fn read_blob(&self, oid: gix::ObjectId) ->
   Result<Tainted<Vec<u8>>>`. Plan 03's trybuild fixture will lock
   this mechanically.

Also: the pre-v0.9.0 `reposix-cli::cache_db` module lifted verbatim
into `reposix_cache::cli_compat`.

## Tasks completed

- **Task 1** — `fixtures/cache_schema.sql`, `db.rs`, `audit.rs`,
  `meta.rs`, extended `cache.rs` (DB handle + identity row),
  extended `builder.rs::build_from` (oid_map + tree_sync + meta
  upsert), new `Error` variants (`Egress`, `UnknownOid`, `OidDrift`),
  `audit_is_append_only` integration test + 4 `db::tests::` unit
  tests.
- **Task 2** — `builder.rs::read_blob` (async, returns
  `Tainted<Vec<u8>>`, fires `materialize`/`egress_denied` audit rows,
  OID-drift detection), `materialize_one.rs` (two tests) and
  `egress_denied_logs.rs` (one test with a stub
  `EgressRejectingBackend`).
- **Task 3** — `cli_compat.rs` lift from `reposix-cli`; re-export
  shim preserves the `reposix_cli::cache_db::{...}` path.

## Commits

- `445138c` feat(31-02): wire audit+meta+oid_map SQLite hardening + `read_blob`
- `d9336df` test(31-02): `materialize_one` + `egress_denied_logs` integration tests
- `d379f21` refactor(31-02): lift `cache_db.rs` from `reposix-cli` to `reposix_cache::cli_compat`

## Tests added / moved

New integration tests in `crates/reposix-cache/tests/`:
- `audit_is_append_only.rs` — 1 test.
- `materialize_one.rs` — 2 tests (`read_blob_materializes_exactly_one_and_audits`, `unknown_oid_returns_error`).
- `egress_denied_logs.rs` — 1 test.

New unit tests in `crates/reposix-cache/src/db.rs`:
- `open_creates_cache_db_file`, `open_is_idempotent`,
  `cache_db_has_expected_tables`, `cache_db_has_append_only_triggers`.

Moved from `reposix-cli/src/cache_db.rs` (now at
`reposix-cache/src/cli_compat.rs` tests mod):
- `open_creates_schema`, `update_metadata_roundtrip`,
  `lock_conflict_returns_error`, `open_is_idempotent`.

Total new tests added by Plan 02: **8**.
Total reposix-cache tests now: **13** (was 5 after Plan 01).
Workspace test suite: **452 passed / 0 failed**.

## Audit row counts (from runtime)

- After `materialize_one::read_blob_materializes_exactly_one_and_audits`:
  1 `tree_sync` + 1 `materialize` + (after second read_blob) a
  second `materialize` = 3 rows total.
- After `egress_denied_logs::egress_denied_writes_audit_row_and_returns_egress_error`:
  1 `tree_sync` + 1 `egress_denied` + 0 `materialize` = 2 rows total.

## `InvalidOrigin` detection shape

`Cache::classify_backend_error(&e, issue_id)` — typed AND stringly:

```rust
let is_egress =
    matches!(e, reposix_core::Error::InvalidOrigin(_))
    || emsg.contains("blocked origin")     // display form of InvalidOrigin
    || emsg.contains("invalid origin")     // legacy backends
    || emsg.contains("allowlist");         // extra defensive match
```

Pragmatic v0.9.0 fallback — Phase 33 will tighten via typed error
refactor. Both `build_from` (list_issues path) and `read_blob`
(get_issue path) share this classifier.

## WARN-emitting paths added

All three audit helpers (`log_materialize`, `log_egress_denied`,
`log_tree_sync`) in `crates/reposix-cache/src/audit.rs` emit
`tracing::warn!` with target `reposix_cache::audit_failure` on SQL
failure. Fields always include `backend` and `project`; materialize
adds `issue_id` and `oid`. Persistent WARN at this target = P1.

## Deviations from the plan sketch

1. **Plan sketch** asked for `Error::Sqlite(#[from] rusqlite::Error)`
   but error messages are built via `format!(...).Sqlite(String)`
   for flexibility. Plan 01 already landed the `From<rusqlite::Error>`
   converter that yields `Error::Sqlite(e.to_string())`, so this path
   preserves every existing call-site pattern.
2. **`reposix-cli/src/refresh.rs` import paths unchanged** — the
   `pub use reposix_cache::cli_compat as cache_db;` shim in
   `reposix-cli/src/lib.rs` keeps `use crate::cache_db::{...}`
   imports working. Task 3 acceptance criterion satisfied without
   touching any refresh call site.
3. **`tempfile = "3"`** added to reposix-cache dev-deps only (NOT
   runtime). The unit tests in `path::tests` and `db::tests` are all
   `#[cfg(test)]` — consumers do not need tempfile.
4. **`anyhow`** is a runtime dep of reposix-cache (scoped to
   `cli_compat.rs`'s `Result<T, anyhow::Error>` surface). Documented
   inline in the Cargo.toml comment; the rest of the crate is
   `thiserror`-only per project convention.

## Notes for Plan 03

- `Tainted<Vec<u8>>` is now the `Cache::read_blob` public return.
  The compile-fail fixture's sink must accept `Untainted<Vec<u8>>`
  and the fixture must pass `Tainted<Vec<u8>>` — mismatched-types at
  compile time.
- Plan 03's `sink::sink_egress(_: Untainted<Vec<u8>>)` — the
  privileged stub. Phase 34 will flesh out the body; the function is
  `#[doc(hidden)]` and lives in a new `crates/reposix-cache/src/sink.rs`.
- `reposix_core::Untainted::new` is `pub(crate)` (confirmed in
  `crates/reposix-core/src/taint.rs:60-62`). The privacy-violation
  fixture will call it from outside the core crate and expect `E0624`.
- No new dev-deps needed; `trybuild = "1"` already present.

## Acceptance status

All 15 acceptance criteria across Tasks 1/2/3 satisfied. `cargo
clippy --workspace --all-targets -- -D warnings` clean; `cargo test
--workspace` green (452 passed, 0 failed); zero `reqwest::Client`
constructors in `crates/reposix-cache/src/`.
