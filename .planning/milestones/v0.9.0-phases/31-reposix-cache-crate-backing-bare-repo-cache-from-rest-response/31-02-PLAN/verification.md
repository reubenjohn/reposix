← [back to index](./index.md) · phase 31 plan 02

# Verification and Success Criteria

## Acceptance criteria

All three tasks must satisfy their acceptance criteria before Phase 31 Wave 2 is declared GREEN.

**Task 1 (Schema):**
- `grep -q "CREATE TABLE IF NOT EXISTS audit_events_cache" crates/reposix-cache/fixtures/cache_schema.sql` returns 0.
- `grep -q "RAISE(ABORT, 'audit_events_cache is append-only')" crates/reposix-cache/fixtures/cache_schema.sql` returns 0.
- `grep -q "pub fn open_cache_db" crates/reposix-cache/src/db.rs` returns 0.
- `grep -q "pub fn log_materialize" crates/reposix-cache/src/audit.rs` returns 0.
- `grep -q "pub fn set_meta" crates/reposix-cache/src/meta.rs` returns 0.
- `grep -q "pub mod db" crates/reposix-cache/src/lib.rs` returns 0.
- `grep -q "SQLITE_DBCONFIG_DEFENSIVE" crates/reposix-cache/src/db.rs` returns 0.
- `grep -q "journal_mode = WAL" crates/reposix-cache/src/db.rs` returns 0.
- `cargo test -p reposix-cache --test audit_is_append_only` exits 0.
- `cargo test -p reposix-cache` (full crate) exits 0.
- `cargo clippy -p reposix-cache --all-targets -- -D warnings` exits 0.

**Task 2 (Blob materialization):**
- `grep -qE "pub async fn read_blob\s*\(\s*&self,\s*oid: gix::ObjectId\s*\)\s*->\s*Result<Tainted<Vec<u8>>>" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -q "log_materialize" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -q "log_egress_denied" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -q "Tainted::new" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -q "OidDrift" crates/reposix-cache/src/builder.rs` returns 0.
- `grep -rnE "reqwest::(Client::new|Client::builder|ClientBuilder::new)" crates/reposix-cache/src/` returns empty.
- `cargo test -p reposix-cache --test materialize_one` exits 0.
- `cargo test -p reposix-cache --test egress_denied_logs` exits 0.
- `cargo test -p reposix-cache` (full crate) exits 0.
- `cargo clippy -p reposix-cache --all-targets -- -D warnings` exits 0.

**Task 3 (Lift):**
- `test -f crates/reposix-cache/src/cli_compat.rs` returns 0.
- `! test -f crates/reposix-cli/src/cache_db.rs` returns 0 (file deleted).
- `grep -q "reposix-cache" crates/reposix-cli/Cargo.toml` returns 0.
- `grep -q "cli_compat" crates/reposix-cache/src/lib.rs` returns 0.
- `grep -q "pub struct CacheDb" crates/reposix-cache/src/cli_compat.rs` returns 0.
- `grep -q "pub fn open_cache_db" crates/reposix-cache/src/cli_compat.rs` returns 0.
- `grep -q "pub fn update_metadata" crates/reposix-cache/src/cli_compat.rs` returns 0.
- `grep -q "another refresh is in progress" crates/reposix-cache/src/cli_compat.rs` returns 0.
- `cargo test --workspace` exits 0 (ALL existing CLI tests pass).
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0.

## Automated verification

Run after all three tasks are complete:

```bash
# Task 1 verifier
cargo test -p reposix-cache --test audit_is_append_only && \
cargo test -p reposix-cache && \
cargo clippy -p reposix-cache --all-targets -- -D warnings

# Task 2 verifier
cargo test -p reposix-cache --test materialize_one && \
cargo test -p reposix-cache --test egress_denied_logs && \
! grep -rnE "reqwest::(Client::new|Client::builder|ClientBuilder::new)" crates/reposix-cache/src/

# Task 3 verifier
cargo test --workspace && \
cargo clippy --workspace --all-targets -- -D warnings && \
! test -f crates/reposix-cli/src/cache_db.rs && \
test -f crates/reposix-cache/src/cli_compat.rs
```

All must return exit code 0.

## Manual verification (optional)

1. **Append-only enforcement:** Run the integration test and inspect the test output for successful SQLITE_CONSTRAINT messages.
   ```bash
   cargo test -p reposix-cache --test audit_is_append_only -- --nocapture
   ```

2. **Audit schema:** Open the cache database and inspect the audit tables.
   ```bash
   sqlite3 /tmp/test-cache.db
   > .tables
   > SELECT * FROM audit_events_cache LIMIT 5;
   > SELECT * FROM meta;
   > SELECT * FROM oid_map;
   ```

3. **Egress denial flow:** Trace the test to confirm the audit row fires before the error returns.
   ```bash
   cargo test -p reposix-cache --test egress_denied_logs -- --nocapture --test-threads=1
   ```

4. **CLI re-export:** Confirm the re-export shim works and old imports still compile.
   ```bash
   cargo build -p reposix-cli
   # If this passes, the shim is working.
   ```

## Success condition

All acceptance criteria satisfied. No warnings from clippy. Full workspace test suite green. The three integration tests capture the core truths: audit row per materialize, append-only enforcement, and egress-denial audit-before-error.

## Output for phase close

Create `.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-02-SUMMARY.md` with:

1. Commit SHAs for each of the three task commits.
2. The exact shape of the `InvalidOrigin` detection in `read_blob` (whether it uses `matches!` on the enum variant, `.to_string().contains(...)`, or both) — Plan 03 consumers need to know.
3. Confirmation that the three new integration tests pass, with sample audit-row counts (e.g., "after materialize_one, audit_events_cache had 1 tree_sync + 1 materialize row = 2 rows total").
4. Note whether `reposix-cli`'s `refresh.rs` needed import-path updates or whether the `pub use reposix_cache::cli_compat as cache_db;` shim was sufficient.
5. Any WARN-emitting code paths added to `tracing` (target, fields) so operators know what to grep for.
