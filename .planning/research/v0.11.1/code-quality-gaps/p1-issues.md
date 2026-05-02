← [back to index](./index.md)

# P1 issues (should fix v0.12.0)

### P1-1. `Error::Other(String)` overused — 153 occurrences, hollows out the typed-error contract

`crates/reposix-core/src/error.rs:36-37` defines `Error::Other(String)` as an *"escape hatch"*, but it carries half the load of every backend adapter. Examples that should be typed variants:

- `Error::Other("not found: …")` (`sim.rs:110,324`) — should be `Error::NotFound { project, id }`.
- `Error::Other("version mismatch: …")` (`sim.rs:113,328`) — parsed *back* via substring matching in `sim.rs:566-572` and `cache/builder.rs:511-515`. Round-tripping through string.
- `Error::Other("not supported: …")` — read-only-backend disambiguator.

`reposix-cache/src/builder.rs:506-515` calls this out explicitly: *"Phase 33 will tighten to a proper typed error refactor."* — Phase 33 shipped without it. The string-match (`emsg.contains("blocked origin")`) is brittle: a backend that capitalises `Blocked Origin` silently bypasses the egress audit row.

Add `Error::NotFound`, `Error::NotSupported`, `Error::VersionMismatch { current, requested }` in v0.12.0.

### P1-2. `reposix-confluence/src/lib.rs` is **3 973 LOC** — split needed

| Section | Lines | Suggested target |
|---|---|---|
| Credentials, struct definitions, deserialization shapes | 116-540 | `confluence/types.rs` (or split into `creds.rs` + `wire.rs`) |
| `parse_next_cursor`, `status_from_confluence`, `redact_url`, `translate` (DTO → Record) | 542-705 | `confluence/translate.rs` |
| `impl ConfluenceBackend { … }` (768-line block of HTTP plumbing + audit hooks) | 707-1474 | `confluence/client.rs` |
| `impl BackendConnector` | 1476-end | stays in `lib.rs` |

Roughly mirrors the layout `reposix-jira` already needs (its `lib.rs` is 1 940 LOC; same split pattern applies). One large file is fine when readers grep, but rustdoc renders the entire crate index off this and the result is unreadable.

### P1-3. 17 verbatim copies of `expect("cache.db mutex poisoned")` in `reposix-cache`

`crates/reposix-cache/src/cache.rs:153,162,175,192,207,222,231,251,267,288,310` plus `builder.rs:108,222,365,441,485`, `gc.rs:146`, `sync_tag.rs:179` — 17 sites. A poisoned-mutex panic in any abandons whichever caller hit it. A `Cache::with_db<R>(&self, f: impl FnOnce(&Connection) -> R)` helper centralises the expect-message and lets you swap the panic for `Error::Sqlite` recovery later. The `# Panics` doc section on every `log_helper_*` (10 sites) is identical 5-line boilerplate; collapse after extraction.

### P1-4. `reposix-sim` and `reposix-cache::cli_compat` lift retained `anyhow` in library APIs

CLAUDE.md: *"Errors: `thiserror` for typed crate errors, `anyhow` only at binary boundaries."*

Violations in libraries:
- `crates/reposix-sim/src/lib.rs:22` — `use anyhow::Result;` is the public-API return type for `run`, `run_with_listener`, `prepare_state`. These ARE called from `reposix-cli::sim` (`crates/reposix-cli/src/sim.rs`) AND from integration tests, so they qualify as a library surface.
- `crates/reposix-cli/src/cache_db.rs:20` — same. Less egregious because `reposix-cli` is binary-only, but the lib.rs (`crates/reposix-cli/src/lib.rs`) is consumed by `cli/tests/*` so the `pub` surface is still exposed.

Adding a typed `SimError` in `reposix-sim` is ~30 LOC; the convertibility with `anyhow` at the binary boundary is one `#[from]`.

### P1-5. `Error::Other` round-trips (the round-trip even has its own helper)

`crates/reposix-core/src/backend/sim.rs:566-572` *parses RFC-3339 JSON back out of an `Error::Other(format!("version mismatch: {body}"))` string*:

```rust
let tail = msg.strip_prefix("version mismatch: ").expect("prefix …");
let body: serde_json::Value = serde_json::from_str(tail).expect("tail parses as JSON");
```

This is a stringly-typed protocol between sim's HTTP layer and its caller. If `body` ever contains the substring `"version mismatch: "` (an attacker-controlled record body could) the caller mis-parses. Type the variant; remove the parser.

### P1-6. `reposix-remote` has 4 unused Cargo deps

`crates/reposix-remote/Cargo.toml:43-51`:

| Dep | Cargo line | Used in src? |
|---|---|---|
| `tokio` (full features) | 43 | Yes — `tokio::runtime::Runtime` |
| `serde` | 44 | **No** |
| `serde_json` | 45 | **No** |
| `serde_yaml` | 46 | **No** |
| `clap` | 47 | **No** |
| `anyhow` | 50 | Yes |
| `thiserror` | 51 | Yes |

(`grep -rn "serde_json\|serde_yaml\|clap" crates/reposix-remote/src/ → no results.`) Drop the four. Net: ~30s faster `cargo build` and shorter dep tree printed by `cargo tree`.

Cross-check: `reposix-core/Cargo.toml:23` declares `rusqlite = { workspace = true }` but `crates/reposix-core/src/audit.rs:1-50` only uses `rusqlite::Connection` to *fix up* a connection passed in by callers — `rusqlite` is genuinely a dep for the schema-loader API. Keep that one.

### P1-7. `reposix-remote` is a binary-only crate with 26 unnecessary `pub fn`

The crate has no `[lib]` in `Cargo.toml`, only `[[bin]]`. Every `pub` in the modules is then exposed only to siblings — which means `pub` is a no-op on visibility (siblings have access regardless inside the same binary). Yet `crates/reposix-remote/src/pktline.rs` exposes `pub fn read_frame`, `encode_frame`, `write_frame`, `is_want_line`; `crates/reposix-remote/src/protocol.rs` has 7 `pub fn`s; `crates/reposix-remote/src/backend_dispatch.rs` has 4. Two of these (`pub fn write_frame`, `pub fn build_protocol_inner` etc.) carry `#[allow(dead_code)]` annotations confirming there's no external user.

Either:
- Demote everything to `pub(crate)` (right answer for a binary crate), OR
- Add `[lib]` and let the integration tests (`tests/protocol.rs`, `tests/push_conflict.rs`) link the lib directly, replacing `assert_cmd::Command::cargo_bin("git-remote-reposix")` with in-process calls.

Today's compromise (binary + `pub`-on-modules) gets the worst of both worlds.

### P1-8. `Cache::log_*` 13 audit forwarders are mechanical boilerplate

`crates/reposix-cache/src/cache.rs:152-373` defines 13 wrappers, each three lines: lock the mutex with the same `expect` string, then call `crate::audit::log_<X>(&db, &self.backend_name, &self.project, …)`. 220 lines of "adapter to the audit module" mass. Either move the log functions onto `&Cache` directly inside `audit.rs` (no second hop) or generate them with `macro_rules!`.

### P1-9. CLI binary lib re-exports its entire surface as `pub mod`

`crates/reposix-cli/src/lib.rs:15-27`:

```rust
pub mod binpath;
pub mod cache_db;
pub mod cost;
…
pub mod worktree_helpers;
```

Twelve modules, all `pub`, all so integration tests can call them directly. None of them are intended for external use (`reposix-cli` is a binary). External consumers depending on these (e.g. some hypothetical user importing `reposix_cli::doctor::DoctorReport`) lock the maintainer into stability promises that don't apply.

Two clean options:
1. **`pub(crate)` on every module**, expose only what tests need via a `#[cfg(test)] pub mod test_api` re-export.
2. **`pub` but `#[doc(hidden)]`** on every module + a crate-level docstring noting "no semver guarantees on lib surface; use the `reposix` binary".
