← [back to index](./index.md)

# Patterns I'd reject in a PR review

1. **`expect("checked")` after `std::env::var`.** `crates/reposix-remote/src/backend_dispatch.rs:253-254,268-269` reads `std::env::var("ATLASSIAN_API_KEY").expect("checked")` after a separate `collect_missing` validation. The TOCTOU window here is microscopic but it's a smell — pass the value through from the validation call site instead. Pattern occurs 4×. Ten minutes to refactor.

2. **`unwrap()` on `parts.last_mut()` after a non-empty proof above it.** `crates/reposix-cli/src/cost.rs:230` and `crates/reposix-cli/src/tokens.rs:276`. Either prove non-empty in the type system (`Vec` → `Vec1` or `[String; N]`) or comment why. Today it's two unsupported-by-doc `unwrap`s in user-facing paths.

3. **`println!` in subcommand modules instead of `writeln!(out, …)`**: every CLI subcommand prints directly to stdout. That makes them un-testable for output without `assert_cmd` subprocess shelling. Plumbing a `&mut dyn Write` through the public API would cost ~20 LOC per subcommand and unlock fast unit-level output tests.

4. **`.to_string()` on path components instead of `&str`.** `crates/reposix-cli/src/worktree_helpers.rs:51-61` returns `String` from `backend_slug_from_origin` even though every value is a `&'static str`. Should be `&'static str`. Same in `GcStrategy::slug` (`crates/reposix-cache/src/gc.rs:62-69`) — already `&'static str`, the right pattern. Be consistent.

5. **Pulling `parking_lot` AND `std::sync::Mutex` in the same workspace.** `crates/reposix-cache/src/cache.rs:5` uses `std::sync::Mutex`; `crates/reposix-confluence/src/lib.rs:75`, `crates/reposix-github/src/lib.rs:60`, `crates/reposix-sim/src/state.rs:10`, `crates/reposix-swarm/src/sim_direct.rs:17` use `parking_lot::Mutex`. Pick one. Workspace `Cargo.toml:91` already has `parking_lot = "0.12"` at workspace level — converge there.

6. **Audit-row INSERT statements as inline string literals scattered across 18 files.** Every `crate::audit::log_*` function in `reposix-cache/src/audit.rs` (16 fns) hand-writes the INSERT. Plus `confluence/src/lib.rs:1097` and `jira/src/lib.rs:411` write their own. Promote to a single `INSERT_AUDIT_ROW: &str = ...;` constant per (table, columns) tuple; the SQL is part of the schema contract, not code.
