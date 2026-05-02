---
phase: 31
plan: 02
type: execute
wave: 2
depends_on:
  - "31-01"
files_modified:
  - crates/reposix-cache/Cargo.toml
  - crates/reposix-cache/src/lib.rs
  - crates/reposix-cache/src/audit.rs
  - crates/reposix-cache/src/meta.rs
  - crates/reposix-cache/src/db.rs
  - crates/reposix-cache/src/error.rs
  - crates/reposix-cache/src/cache.rs
  - crates/reposix-cache/src/builder.rs
  - crates/reposix-cache/fixtures/cache_schema.sql
  - crates/reposix-cache/tests/materialize_one.rs
  - crates/reposix-cache/tests/audit_is_append_only.rs
  - crates/reposix-cache/tests/egress_denied_logs.rs
  - crates/reposix-cli/src/cache_db.rs
  - crates/reposix-cli/src/lib.rs
  - crates/reposix-cli/src/main.rs
  - crates/reposix-cli/src/refresh.rs
  - crates/reposix-cli/Cargo.toml
autonomous: true
requirements:
  - ARCH-02
  - ARCH-03
tags:
  - rust
  - sqlite
  - audit
  - egress
  - allowlist
  - cache
user_setup: []

must_haves:
  truths:
    - "Every call to `Cache::read_blob(oid)` that materializes a blob writes exactly one row into `audit_events_cache` with `op='materialize'`."
    - "Attempting `UPDATE audit_events_cache SET ts='x'` or `DELETE FROM audit_events_cache` raises `SQLITE_CONSTRAINT` (trigger-enforced append-only)."
    - "Pointing the cache at a backend whose origin is not in `REPOSIX_ALLOWED_ORIGINS` returns `Error::Egress(...)` AND writes an `op='egress_denied'` audit row BEFORE returning the typed error."
    - "`cache.db` file lives at `<cache-path>/cache.db` with permission `0o600` (regression-tested via `fs::metadata`)."
    - "`Cache::read_blob(oid)` returns `reposix_core::Tainted<Vec<u8>>` — the Tainted wrapper is visible in the public signature (verified at compile time by caller sites and later by Plan 03's trybuild fixture)."
    - "`reposix-cache` constructs zero `reqwest::Client` instances (HTTP flows through `BackendConnector` implementations that use `reposix_core::http::client()`)."
    - "`cache_db.rs` has been lifted out of `reposix-cli/src/` — the old path is deleted; `reposix-cli` depends on `reposix-cache` and re-exports `open_cache_db`/`update_metadata` from the new crate so existing CLI call sites continue to compile."
  artifacts:
    - path: "crates/reposix-cache/fixtures/cache_schema.sql"
      provides: "DDL for audit_events_cache, meta, oid_map tables + BEFORE UPDATE/DELETE RAISE triggers + indexes."
      contains: "RAISE(ABORT, 'audit_events_cache is append-only')"
    - path: "crates/reposix-cache/src/db.rs"
      provides: "open_cache_db(path) — opens cache.db with mode=0o600, DEFENSIVE flag, WAL, loads the schema."
      contains: "pub fn open_cache_db"
    - path: "crates/reposix-cache/src/audit.rs"
      provides: "log_materialize, log_egress_denied, log_tree_sync helpers (INSERT-only; best-effort WARN on failure per CONTEXT §Atomicity)."
      contains: "fn log_materialize"
    - path: "crates/reposix-cache/src/meta.rs"
      provides: "get_meta/set_meta single-row helpers + get_issue_for_oid/put_oid_mapping for oid_map."
      contains: "pub fn set_meta"
    - path: "crates/reposix-cache/src/builder.rs"
      provides: "Updated Cache::build_from to insert oid_map rows + tree_sync audit row + meta last_fetched_at update. NEW: async fn read_blob returning Tainted<Vec<u8>> with materialize-audit + egress-denial detection."
      contains: "pub async fn read_blob"
    - path: "crates/reposix-cache/tests/materialize_one.rs"
      provides: "Integration test proving one read_blob call = one blob object + one materialize audit row + correct Tainted<Vec<u8>> return."
      contains: "Tainted"
    - path: "crates/reposix-cache/tests/audit_is_append_only.rs"
      provides: "Integration test proving UPDATE and DELETE on audit_events_cache both return SQLITE_CONSTRAINT."
      contains: "append-only"
    - path: "crates/reposix-cache/tests/egress_denied_logs.rs"
      provides: "Integration test proving a non-allowlisted origin returns Error::Egress AND writes an op=egress_denied row."
      contains: "egress_denied"
  key_links:
    - from: "crates/reposix-cache/src/builder.rs:read_blob"
      to: "crates/reposix-cache/src/audit.rs:log_materialize"
      via: "INSERT after successful blob write"
      pattern: "log_materialize"
    - from: "crates/reposix-cache/src/builder.rs:read_blob"
      to: "reposix_core::error::Error::InvalidOrigin"
      via: "downcast/pattern match on backend error, fire log_egress_denied before returning Error::Egress"
      pattern: "InvalidOrigin"
    - from: "crates/reposix-cache/src/builder.rs:read_blob"
      to: "reposix_core::taint::Tainted"
      via: "return value wrapping"
      pattern: "Tainted::new"
    - from: "crates/reposix-cli"
      to: "crates/reposix-cache"
      via: "Cargo.toml path dep + re-export shim for cache_db callers"
      pattern: "reposix-cache"
---

<objective>
Wire the cache to its three non-negotiable operating-principle hooks: append-only SQLite audit, egress allowlist denial that is audited BEFORE the typed error returns, and `Tainted<Vec<u8>>` public return for `read_blob`. Also implement the actual lazy-blob materialization path (`Cache::read_blob`) that Plan 01 deliberately left unimplemented.

Purpose: ARCH-02 (audit row per materialize + Tainted return + append-only triggers) and ARCH-03 (egress allowlist reuse of `reposix_core::http::client` + `EgressDenied` error + audit row).
Output: `audit_events_cache` + `meta` + `oid_map` tables with append-only triggers, `read_blob` returning `Tainted<Vec<u8>>`, three integration tests (`materialize_one`, `audit_is_append_only`, `egress_denied_logs`), and `cache_db.rs` lifted from `reposix-cli` into `reposix-cache` (RESEARCH §Open Question 1 recommendation — do it now rather than carry the divergence across Phase 33 / 35).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-CONTEXT.md
@.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-RESEARCH.md
@.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-01-SUMMARY.md

@crates/reposix-cache/src/lib.rs
@crates/reposix-cache/src/cache.rs
@crates/reposix-cache/src/builder.rs
@crates/reposix-cache/src/error.rs
@crates/reposix-cache/src/path.rs
@crates/reposix-core/src/audit.rs
@crates/reposix-core/fixtures/audit.sql
@crates/reposix-core/src/http.rs
@crates/reposix-core/src/error.rs
@crates/reposix-core/src/taint.rs
@crates/reposix-cli/src/cache_db.rs
@crates/reposix-cli/src/refresh.rs
@crates/reposix-cli/src/lib.rs
@crates/reposix-cli/src/main.rs

<interfaces>
<!-- Load-bearing interfaces from reposix-core and reposix-cli for Plan 02. -->

From `crates/reposix-core/src/error.rs`:
```rust
pub enum Error {
    InvalidOrigin(String),  // <-- THIS is the variant produced when an allowlist check fails
    Http(reqwest::Error),
    Other(String),
    Yaml(serde_yaml::Error),
    // ...
}
```

The allowlist gate lives in `reposix_core::http::HttpClient::request_with_headers_and_body`:
```rust
if !allowlist.iter().any(|g| g.matches(&parsed)) {
    return Err(Error::InvalidOrigin(parsed.to_string()));
}
```

So when the cache calls `backend.get_issue(id)` against a backend whose origin is not allowlisted, the error that propagates up is `reposix_core::Error::InvalidOrigin(...)`. Plan 02's `read_blob` MUST detect this variant BEFORE wrapping it in `Error::Backend(String)` (which is lossy) and fire an `op=egress_denied` audit row. Concrete detection: pattern-match on the concrete `reposix_core::Error` variant returned by the backend (propagated via `?` will be a `reposix_core::Error`, then converted). See Action step 4 for the exact shape.

From `crates/reposix-core/src/taint.rs`:
```rust
pub struct Tainted<T>(T);
impl<T> Tainted<T> {
    pub fn new(value: T) -> Self;
    pub fn into_inner(self) -> T;
    pub fn inner_ref(&self) -> &T;
}
```
No `From`, no `Deref`, no `AsRef` — this is the discipline Plan 03's trybuild fixture will lock in.

From `crates/reposix-core/fixtures/audit.sql` — the pattern to IMITATE (not reuse) because the columns are HTTP-shaped and Plan 02's audit has different columns:
```sql
BEGIN;
CREATE TABLE IF NOT EXISTS audit_events ( ... HTTP-shaped columns ... );
DROP TRIGGER IF EXISTS audit_no_update;
CREATE TRIGGER audit_no_update BEFORE UPDATE ON audit_events
    BEGIN SELECT RAISE(ABORT, 'audit_events is append-only'); END;
DROP TRIGGER IF EXISTS audit_no_delete;
CREATE TRIGGER audit_no_delete BEFORE DELETE ON audit_events
    BEGIN SELECT RAISE(ABORT, 'audit_events is append-only'); END;
COMMIT;
```

From `crates/reposix-core/src/audit.rs` (the helpers to reuse for DEFENSIVE flag + schema load):
```rust
pub fn load_schema(conn: &rusqlite::Connection) -> Result<()>;  // schema via reposix_core::audit::SCHEMA_SQL
pub fn enable_defensive(conn: &rusqlite::Connection) -> Result<()>;  // sets SQLITE_DBCONFIG_DEFENSIVE
pub fn open_audit_db(path: &Path) -> Result<rusqlite::Connection>;   // <-- the pattern, but it loads the core schema not ours
```

Plan 02 needs its own `open_cache_db` that does the same DEFENSIVE + WAL dance but loads `cache_schema.sql` (our schema, not `reposix_core::audit::SCHEMA_SQL`).

From `crates/reposix-cli/src/cache_db.rs` (the code being LIFTED):
```rust
pub struct CacheDb(rusqlite::Connection);
pub fn open_cache_db(mount: &Path) -> Result<CacheDb>;
pub fn update_metadata(db: &CacheDb, backend_name: &str, project: &str, last_fetched_at: &str, commit_sha: Option<&str>) -> Result<()>;
// `.reposix/cache.db` (inside a FUSE mount) with 0o600, WAL, EXCLUSIVE.
```

The lift preserves the public API: reposix-cli continues to import `open_cache_db` and `update_metadata` but the module moves into `reposix-cache`. Call sites in `crates/reposix-cli/src/refresh.rs` need `use reposix_cache::cli_compat::{open_cache_db, update_metadata};` or a re-export shim to keep the old `use crate::cache_db::...` path working — either approach is acceptable. RECOMMENDED: delete `crates/reposix-cli/src/cache_db.rs`, add `reposix-cache = { path = "../reposix-cache" }` to `crates/reposix-cli/Cargo.toml`, and re-export from the new crate via a `cli_compat` module (preserves v0.8.0 single-row `refresh_meta` schema for CLI callers independent of the new `cache_events` schema).
</interfaces>
</context>

## Chapters

- **[Task 1A — `cache_schema.sql`, `db.rs`, `audit.rs`, `meta.rs`, `error.rs` (Steps 1–5)](./task-1-schema-A.md)**
- **[Task 1B — `cache.rs`, `builder.rs`, `lib.rs`, `audit_is_append_only` test (Steps 6–10)](./task-1-schema-B.md)**
- **[Task 2 — `Cache::read_blob` + Tainted return + egress audit + `materialize_one` / `egress_denied_logs` tests](./task-2.md)**
- **[Task 3 — Lift `cache_db.rs` from `reposix-cli` into `reposix-cache::cli_compat`](./task-3.md)**
- **[Threat Model, Verification, Success Criteria, Output](./threat-model-verification.md)**
  STRIDE register, Wave 2 verification checklist, success criteria, and output instructions.
