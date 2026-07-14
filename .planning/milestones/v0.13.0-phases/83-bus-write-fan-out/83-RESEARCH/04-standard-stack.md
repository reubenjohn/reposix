← [back to index](./index.md) · phase 83 research

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---|---|---|---|
| `std::process::Command` | std | spawn `git push <mirror_remote_name> main` | Idiom established by `bus_handler::precheck_mirror_drift` (P82) — `Command::new("git").args(["ls-remote", "--", mirror_url, "refs/heads/main"]).output()`. Same approach for push. [VERIFIED: `bus_handler.rs:247`] |
| `anyhow::{Context, Result}` | 1.x | error propagation | Helper crate's existing idiom. [VERIFIED: `main.rs:18`, `bus_handler.rs:45`] |
| `wiremock` | 0.6.5 | SoT mock for fault-injection tests | Existing dep; per-route `Mock::given(...).respond_with(ResponseTemplate::new(409))`. [VERIFIED: `tests/push_conflict.rs:31`, `tests/bus_precheck_b.rs:21`] |
| `assert_cmd` | (existing dev-dep) | spawn helper subprocess in tests | Existing idiom — `Command::cargo_bin("git-remote-reposix")` shape across all `tests/bus_*` files. [VERIFIED: `tests/bus_precheck_b.rs:165`] |
| `tempfile` | (existing dev-dep) | working-tree + bare-mirror fixtures | Existing idiom — `tempfile::tempdir()` for both wtree and bare mirror in `make_synced_mirror_fixture`. [VERIFIED: `tests/bus_precheck_b.rs:60`] |
| `reposix-cache` (existing) | path | `write_mirror_head`, `write_mirror_synced_at`, `log_mirror_sync_written`, `write_last_fetched_at`, `log_helper_push_accepted`, new `log_helper_push_partial_fail_mirror_lag` | All shipped P80/P81 except the new partial-fail helper which P83 mints. |

**No new third-party dependencies required.** Phase 83 is pure helper-crate + cache-crate work using crates already in the workspace [VERIFIED: `cargo metadata` need not run; `crates/reposix-remote/Cargo.toml` and `crates/reposix-cache/Cargo.toml` carry everything needed].

**Version verification:** No new versions to verify — every dependency already pinned.

