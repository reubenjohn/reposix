← [back to index](./index.md)

# Task 3: Lift `cache_db.rs` from `reposix-cli` into `reposix-cache` (RESEARCH §Open Question 1)

**Files:**
```
crates/reposix-cli/Cargo.toml,
crates/reposix-cli/src/cache_db.rs,
crates/reposix-cli/src/lib.rs,
crates/reposix-cli/src/main.rs,
crates/reposix-cli/src/refresh.rs,
crates/reposix-cache/src/cli_compat.rs,
crates/reposix-cache/src/lib.rs
```

**Read first:**
```
crates/reposix-cli/src/cache_db.rs,
crates/reposix-cli/src/lib.rs,
crates/reposix-cli/src/main.rs,
crates/reposix-cli/src/refresh.rs,
crates/reposix-cli/Cargo.toml,
crates/reposix-cache/src/lib.rs
```

## Behavior

- `crates/reposix-cache/src/cli_compat.rs` exists and exposes `CacheDb`, `open_cache_db`, `update_metadata` with the SAME public signatures the old `crates/reposix-cli/src/cache_db.rs` had. The module's body is the VERBATIM content of the old cache_db.rs (preserves the v0.8.0 `refresh_meta` single-row table + WAL+EXCLUSIVE + 0o600 behavior).
- `crates/reposix-cli/src/cache_db.rs` is DELETED.
- `crates/reposix-cli/Cargo.toml` gets `reposix-cache = { path = "../reposix-cache" }` in `[dependencies]`.
- `crates/reposix-cli/src/lib.rs` and `crates/reposix-cli/src/main.rs` have their `mod cache_db;` and `pub mod cache_db;` lines removed (if present) and replaced with `use reposix_cache::cli_compat as cache_db;` re-exports where necessary, OR the call sites in `crates/reposix-cli/src/refresh.rs` import directly from `reposix_cache::cli_compat`.
- `cargo test --workspace` still passes — including the CLI's existing `open_creates_schema`, `update_metadata_roundtrip`, `lock_conflict_returns_error`, `open_is_idempotent` tests (they move WITH the code to the new crate).

## Action

Step 1 — Create `crates/reposix-cache/src/cli_compat.rs` by copying `crates/reposix-cli/src/cache_db.rs` VERBATIM. The file header doc comment should be updated to say:
```rust
//! CLI-compat shim: the `reposix refresh` subcommand's metadata DB.
//!
//! This module was lifted from `crates/reposix-cli/src/cache_db.rs` in
//! Phase 31 Plan 02 (RESEARCH §Open Question 1) so the single v0.8.0
//! `refresh_meta` schema has one home instead of drifting across two
//! crates during v0.9.0. The public surface is unchanged — callers that
//! previously did `use crate::cache_db::{CacheDb, open_cache_db, update_metadata}`
//! now do `use reposix_cache::cli_compat::{CacheDb, open_cache_db, update_metadata}`.
//!
//! Note: this is intentionally SEPARATE from the new `reposix_cache::db`
//! module which owns the v0.9.0 `cache_schema.sql`. The CLI's refresh
//! subcommand will migrate to the new schema in Phase 35 (CLI pivot); until
//! then, the two coexist.
```

All function bodies and tests are copied as-is. No changes to signatures.

Step 2 — Register the new module in `crates/reposix-cache/src/lib.rs`:
```rust
pub mod cli_compat;
```
(Place it after `pub mod cache;` alphabetically.)

Step 3 — Add `reposix-cache` as a dep to `crates/reposix-cli/Cargo.toml`. Locate the `[dependencies]` table and add (alphabetically if possible):
```toml
reposix-cache = { path = "../reposix-cache" }
```

Step 4 — Delete `crates/reposix-cli/src/cache_db.rs` entirely. Use `rm` or `git rm`.

Step 5 — Update the two files that module-declare or use `cache_db`:

5a. In `crates/reposix-cli/src/lib.rs`, find the `pub mod cache_db;` (or `mod cache_db;`) declaration and REPLACE with a re-export shim that keeps the old path alive:
```rust
/// Re-export shim: `cache_db` module moved to `reposix_cache::cli_compat`
/// in Phase 31. This alias keeps `reposix_cli::cache_db::{...}` working
/// for any external caller (though there should be none — it was never
/// a public API).
pub use reposix_cache::cli_compat as cache_db;
```

5b. In `crates/reposix-cli/src/main.rs`, if there is a `mod cache_db;` line, DELETE it (main binaries can pull the re-export through lib.rs if the crate has one, or directly `use reposix_cache::cli_compat as cache_db;` if not).

5c. In `crates/reposix-cli/src/refresh.rs`, find `use crate::cache_db::...` imports and adjust. If Step 5a kept the alias path alive, they continue to work. Otherwise change to `use reposix_cache::cli_compat::{CacheDb, open_cache_db, update_metadata};`.

Step 6 — Run the lifted tests (they moved with the code and now live at `crates/reposix-cache/src/cli_compat.rs` as `#[cfg(test)] mod tests { ... }` inside the module):
```bash
cargo test -p reposix-cache cli_compat   # cargo narrows to the module's tests
```

Step 7 — Full workspace regression:
```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The existing CLI tests referencing `cache_db::open_cache_db` / `cache_db::update_metadata` / the four named tests must all still pass.

## Acceptance Criteria

- `test -f crates/reposix-cache/src/cli_compat.rs` returns 0.
- `! test -f crates/reposix-cli/src/cache_db.rs` returns 0 (file deleted).
- `grep -q "reposix-cache" crates/reposix-cli/Cargo.toml` returns 0.
- `grep -q "cli_compat" crates/reposix-cache/src/lib.rs` returns 0.
- `grep -q "pub struct CacheDb" crates/reposix-cache/src/cli_compat.rs` returns 0.
- `grep -q "pub fn open_cache_db" crates/reposix-cache/src/cli_compat.rs` returns 0.
- `grep -q "pub fn update_metadata" crates/reposix-cache/src/cli_compat.rs` returns 0.
- `grep -q "another refresh is in progress" crates/reposix-cache/src/cli_compat.rs` returns 0 (the BUSY-message preserved).
- `cargo test --workspace` exits 0 (ALL existing CLI tests including `open_creates_schema`, `update_metadata_roundtrip`, `lock_conflict_returns_error`, `open_is_idempotent` still pass — now from their new home).
- `cargo clippy --workspace --all-targets -- -D warnings` exits 0.

## Verify

```
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && ! test -f crates/reposix-cli/src/cache_db.rs && test -f crates/reposix-cache/src/cli_compat.rs
```

## Done

cache_db.rs has one home in `reposix-cache::cli_compat`; all four existing CLI tests pass from the new crate; `reposix-cli` depends on `reposix-cache` and continues to compile and run. No regression in the full workspace suite.
