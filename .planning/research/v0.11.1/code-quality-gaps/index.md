# Code quality + Rust idioms audit (post-v0.11.0)

> Read-only senior-reviewer pass over `crates/*/src/**/*.rs` (≈25 126 LOC across 9 crates), 2026-04-25 afternoon. Builds on `v0.11.0-CATALOG-v2.md` — that catalog focused on *files to delete* and *FUSE residue*; this one focuses on *code idioms* and *what would block a 1.0 cut*.

## Summary scorecard

| Crate | Major issues | Minor issues | LOC (src) | Notes |
|---|---|---|---|---|
| reposix-core | 2 | 4 | 2 472 | `Error::Other(String)` overuse is the dominant smell; otherwise the most disciplined crate (`Tainted`/`Untainted` is excellent). |
| reposix-cache | 2 | 5 | 2 944 | Two cache.db schemas (`audit_events_cache` vs `audit_events`), 11× repeated `expect("cache.db mutex poisoned")` boilerplate, stringly-typed egress detection in `classify_backend_error`. |
| reposix-cli | 4 | 8 | 4 137 | `cache_path_from_worktree` STILL duplicated 3× post-Phase-51; **JIRA worktrees resolve the wrong cache dir** (correctness bug); `os::unix::fs::OpenOptionsExt` blocks Windows. |
| reposix-remote | 2 | 4 | 2 705 | Binary-only crate with 26 unnecessary `pub fn`; 4 unused Cargo deps; tracing/printf'd diag instead of structured. |
| reposix-confluence | 3 | 3 | 4 940 | **3 973-line `lib.rs`** — the single biggest split-needed file. Layering violation: opens its own `rusqlite::Connection` to write `audit_events` rows. |
| reposix-jira | 2 | 3 | 2 488 | **1 940-line `lib.rs`**; same layering violation as confluence. |
| reposix-github | 1 | 2 | 957 | Cleanest backend; `panic!` in `#[cfg(test)]` only; could move models to `models.rs`. |
| reposix-sim | 1 | 2 | 1 800 | `pub fn run() -> anyhow::Result<()>` leaks anyhow into a library crate. |
| reposix-swarm | 1 | 1 | 1 370 | One FUSE residue line in `metrics.rs`. |

> **9 cleanly-`forbid(unsafe_code)`'d crates**, but `reposix-sim/src/main.rs` lacks the attribute (every other binary entry-point has it). One-line miss.

## Chapters

- [P0 issues (block 1.0)](./p0-issues.md)
- [P1 issues (should fix v0.12.0)](./p1-issues.md)
- [P2 issues (polish)](./p2-issues.md)
- [Patterns to reject in PR review](./pr-review-patterns.md)
- [Cross-cutting refactor opportunities](./cross-cutting.md)
- [What's actually GOOD](./whats-good.md)
