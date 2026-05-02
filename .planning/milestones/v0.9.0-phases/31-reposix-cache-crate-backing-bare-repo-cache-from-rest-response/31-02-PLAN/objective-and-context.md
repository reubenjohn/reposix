← [back to index](./index.md) · phase 31 plan 02

# Objective and Execution Context

## Dependencies and references

**Depends on:** Phase 31 Plan 01 (completion required before starting this plan).

**Read in parallel with:**
- `.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-CONTEXT.md`
- `.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-RESEARCH.md`
- `.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-01-SUMMARY.md`

**Reference codebase files:**
- `crates/reposix-cache/src/lib.rs` — current module structure.
- `crates/reposix-cache/src/cache.rs` and `.../builder.rs` — Plan 01 outputs to extend.
- `crates/reposix-cache/src/error.rs` — error types and conversion.
- `crates/reposix-cache/src/path.rs` — cache path helpers.
- `crates/reposix-core/src/audit.rs` — audit base patterns (DEFENSIVE, load_schema).
- `crates/reposix-core/fixtures/audit.sql` — append-only trigger patterns (imitate, don't reuse).
- `crates/reposix-core/src/http.rs` — HTTP client factory reuse.
- `crates/reposix-core/src/error.rs` — `InvalidOrigin` variant for egress-denial detection.
- `crates/reposix-core/src/taint.rs` — `Tainted<T>` wrapper for blob returns.
- `crates/reposix-cli/src/cache_db.rs` — code to be lifted into `reposix-cache::cli_compat`.
- `crates/reposix-cli/src/refresh.rs`, `lib.rs`, `main.rs` — refactor call sites after lift.

## Key interfaces

### Core types from reposix-core

**`Error::InvalidOrigin(String)`** — produced when allowlist check fails in `reposix_core::http::HttpClient::request_with_headers_and_body`. Plan 02's `read_blob` MUST detect this variant BEFORE any generic wrapping and fire an `op=egress_denied` audit row.

**`Tainted<T>`** — zero-copy wrapper with no `From`, no `Deref`, no `AsRef`. Discipline enforced by Plan 03's trybuild fixture. `read_blob` returns `Result<Tainted<Vec<u8>>>`.

### Patterns from reposix-core::audit

- `load_schema(conn)` — loads SQL schema from a constant.
- `enable_defensive(conn)` — sets `SQLITE_DBCONFIG_DEFENSIVE` (blocks `writable_schema` edits).
- `open_audit_db(path)` — pattern for opening with DEFENSIVE + WAL, but loads `reposix_core::audit::SCHEMA_SQL` not ours.

Plan 02 implements its own `open_cache_db` following the same pattern but loading the `cache_schema.sql` fixture.

### Append-only trigger pattern (from reposix-core/fixtures/audit.sql)

```sql
DROP TRIGGER IF EXISTS audit_no_update;
CREATE TRIGGER audit_no_update BEFORE UPDATE ON audit_events_cache
    BEGIN SELECT RAISE(ABORT, 'audit_events_cache is append-only'); END;
DROP TRIGGER IF EXISTS audit_no_delete;
CREATE TRIGGER audit_no_delete BEFORE DELETE ON audit_events_cache
    BEGIN SELECT RAISE(ABORT, 'audit_events_cache is append-only'); END;
```

Imitate this pattern in `cache_schema.sql` with message `'audit_events_cache is append-only'`. Columns differ (HTTP-shaped vs cache-shaped), so the schema is not shared.

### Code lift from reposix-cli/src/cache_db.rs

```rust
pub struct CacheDb(rusqlite::Connection);
pub fn open_cache_db(mount: &Path) -> Result<CacheDb>;
pub fn update_metadata(db: &CacheDb, backend_name: &str, project: &str, 
                       last_fetched_at: &str, commit_sha: Option<&str>) -> Result<()>;
```

Preserves v0.8.0 `refresh_meta` single-row schema (WAL, EXCLUSIVE locking, 0o600 mode) independent of the new v0.9.0 `cache_events` schema. Lift path: copy to `crates/reposix-cache/src/cli_compat.rs`, re-export from `reposix-cache` in `reposix-cli`.

## Success condition

Three core truths verified by integration tests:

1. **Audit row per materialize:** Every `Cache::read_blob(oid)` call writes exactly one `op='materialize'` row (plus one `op='tree_sync'` on build, if applicable).
2. **Append-only triggers:** UPDATE and DELETE on `audit_events_cache` both return `SQLITE_CONSTRAINT` with message `'audit_events_cache is append-only'`.
3. **Egress-denial audit:** Pointing the cache at a non-allowlisted origin causes `read_blob` to return `Error::Egress(...)` AND write an `op='egress_denied'` audit row BEFORE returning.

All three are tested by integration tests in this plan. Full workspace regression (`cargo test --workspace`) must pass.
