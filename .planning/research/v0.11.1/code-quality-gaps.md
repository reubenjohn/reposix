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

---

## P0 issues (block 1.0)

### P0-1. `cache_path_from_worktree` still duplicated 3× after the Phase 51 consolidation

The previous catalog's #6 finding ("Consolidate the four `cache_path_from_worktree` triplets") only got *partly* shipped. The shared module exists at `crates/reposix-cli/src/worktree_helpers.rs:76` and the `pub fn cache_path_from_worktree` is now there, but each consumer added its own thin "+exists check" wrapper rather than using the shared one — so we have FOUR functions named `cache_path_from_worktree` in the crate:

| File | Definition | Action |
|---|---|---|
| `crates/reposix-cli/src/worktree_helpers.rs:76` | Canonical `pub fn` | KEEP |
| `crates/reposix-cli/src/gc.rs:166` | `fn cache_path_from_worktree` (shadowed in module) — shells to `resolve_cache_dir` then bails if dir missing | DELETE — fold the existence check into a single wrapper or just inline at call site |
| `crates/reposix-cli/src/tokens.rs:69` | Same shape | DELETE |
| `crates/reposix-cli/src/cost.rs:282` | Same shape | DELETE |

Each wrapper is 7-9 lines that differ only in the bail message. Promote to a single `cache_path_from_worktree_existing(work) -> Result<PathBuf>` in `worktree_helpers` accepting an `existence_msg: &str`, or just call `worktree_helpers::cache_path_from_worktree` then `if !p.exists() { bail!(...) }` at the 3 call sites. Net delete: ~25 LOC.

Why P0: a senior reviewer who hovers over `gc.rs:166` and sees a function with the EXACT NAME of the canonical helper will assume the canonical one was deprecated. It's "not finishing the refactor" residue.

### P0-2. `worktree_helpers::backend_slug_from_origin` is **wrong for JIRA worktrees** (correctness bug)

`crates/reposix-cli/src/worktree_helpers.rs:51-61` maps any `atlassian.net` origin to `"confluence"`:

```rust
} else if origin.contains("atlassian.net") {
    // sim and confluence/jira can share atlassian.net; we pick "confluence"
    // as a default and the user fixes if wrong. Best-effort.
    "confluence".to_string()
}
```

The docstring (`worktree_helpers.rs:46-48`) claims *"the worktree-side helpers don't see the `/jira/` vs `/confluence/` URL marker (it's discarded before `remote.origin.url` is stored)"* — this is **factually incorrect**. `crates/reposix-cli/src/init.rs:73,89` writes the marker into `remote.origin.url`:

```rust
"reposix::https://{tenant}.atlassian.net/confluence/projects/{project}"
"reposix::https://{instance}.atlassian.net/jira/projects/{project}"
```

`reposix_core::split_reposix_url` slices on `/projects/`, so `spec.origin` retains `/confluence` or `/jira` (the trailing slash is consumed by the marker). The data IS available; the helper just throws it away. Result: every JIRA worktree, when run through `reposix gc`, `reposix tokens`, `reposix cost`, `reposix history`, or `reposix doctor`, resolves to a **`confluence-<project>.git`** cache dir instead of `jira-<project>.git`. These 5 subcommands silently report the wrong cache (or worse, blow up reporting "no cache").

**Cross-check:** `crates/reposix-remote/src/backend_dispatch.rs:116-168` correctly inspects the marker for the helper itself — so `git fetch`/`git push` do the right thing. The bug is purely in the CLI-side worktree helper which short-circuits on the host name.

**Fix:** add `if origin.contains("/jira") { return "jira" }` before the atlassian.net branch. Three-line patch + test. The existing test at `worktree_helpers.rs:99-105` only covers the bare `atlassian.net` host and locks in the wrong behaviour — strengthen it.

### P0-3. `reposix-cache` and `reposix-cli` use unix-only `OpenOptionsExt` without `cfg(unix)` gate

`crates/reposix-cache/src/db.rs:17` and `crates/reposix-cli/src/cache_db.rs:17` both:

```rust
use std::os::unix::fs::OpenOptionsExt as _;
```

…and call `.mode(0o600)` on the open builder. The workspace `Cargo.toml:30` lists `x86_64-pc-windows-msvc` in `cargo-dist` targets. This is a hard build error on Windows — every binary release for Windows currently fails to compile (or the cargo-dist Windows lane is silently skipping these crates). Either:

1. Drop Windows from dist targets and document Linux/macOS only (matches the dependency tree's `rustix` use in `reposix-cli/Cargo.toml:47`), or
2. Add `#[cfg(unix)] use std::os::unix::fs::OpenOptionsExt as _;` and `#[cfg(unix)] .mode(0o600)`, with an explanatory comment about ACL fallback on Windows.

The simulator's audit DB schema lives in `reposix-core::audit` and faces the same thing implicitly via its consumers. This is a pre-1.0 cut decision: keep Windows as a release platform, or drop it from dist.

### P0-4. **Two parallel audit-log schemas**: `audit_events_cache` (canonical) vs `audit_events` (per-backend, layering violation)

The CLAUDE.md is explicit: *"Audit log is non-optional. Every network-touching action … gets a row in the simulator's SQLite audit table."* But there are TWO tables:

| Table | Owner | Inserters |
|---|---|---|
| `audit_events_cache` | `reposix-cache` (`crates/reposix-cache/src/audit.rs`) — schema in `crates/reposix-cache/fixtures/cache_schema.sql` | `reposix-cache::builder`, `reposix-cache::gc`, helper `stateless-connect` path |
| `audit_events` | `reposix-core::audit` (`crates/reposix-core/fixtures/audit.sql`) | `reposix-sim::middleware::audit`, `reposix-confluence::lib::record_audit` (`crates/reposix-confluence/src/lib.rs:1097-1113`), `reposix-jira::lib` (`crates/reposix-jira/src/lib.rs:407-425`) |

The confluence and jira backends carry their own `Option<Arc<Mutex<Connection>>>` and write `audit_events` rows directly — **a layering violation** because the cache crate is supposed to be the single audit owner per CLAUDE.md. This forces both backend crates to depend on `rusqlite + sha2 + hex` (`crates/reposix-confluence/Cargo.toml:29-32`, `crates/reposix-jira/Cargo.toml:29-31`), which is otherwise out of scope for an HTTP adapter.

Pick one model:
- Move the per-backend write hook into `reposix-cache::audit` so backend crates don't open SQLite connections.
- OR explicitly endorse the dual-schema design in `reposix-core/src/audit.rs` module docs and remove the "cache is the canonical audit" framing.

Either way, the current state has a `reposix doctor` that only checks `audit_events_cache` (line `crates/reposix-cli/src/doctor.rs:489-545`) and has zero coverage of the `audit_events` table the backends write to.

---

## P1 issues (should fix v0.12.0)

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

---

## P2 issues (polish)

### P2-1. FUSE-era doc-comment residue still present

The CATALOG-v2 #10 sweep ("Sweep ~12 FUSE-era doc comments") was missed. Surviving sites:

| File:line | Comment |
|---|---|
| `crates/reposix-core/src/lib.rs:3` | `"This crate is the seam between the simulator … the FUSE daemon, the git remote helper …"` |
| `crates/reposix-core/src/backend.rs:7,9,22,72,106` | "FUSE daemon historically", "FUSE layer", "fuse, remote", "Used by FUSE", "will be used by the FUSE daemon" |
| `crates/reposix-core/src/backend/sim.rs:499,629-632,909` | comments referencing `crates/reposix-fuse/...` (deleted crate) |
| `crates/reposix-core/src/path.rs:3,13` | "FUSE boundary (Phase 3)", "FUSE `tree/` overlay" |
| `crates/reposix-core/src/project.rs:8` | "render it as a directory name in the FUSE mount" |
| `crates/reposix-confluence/src/lib.rs:433,1484` | "FUSE `read()` callback", "used by FUSE for the `tree/` overlay" |
| `crates/reposix-remote/src/fast_import.rs:5` | "the SAME function the FUSE read path uses" |
| `crates/reposix-swarm/src/metrics.rs:238-240` | `Mode name (sim-direct or fuse). … FUSE mount path)` (misleading — `Mode::Fuse` was deleted) |

All cosmetic but rustdoc renders this verbatim — it's the first thing a new reader sees.

### P2-2. Stale phase markers + version markers (v0.1.5 / v0.2 / "Plan 02 / Phase 31")

29 references to specific phase numbers and pre-1.0 versions in source comments (`grep -rn "Phase 13\|Phase 31\|POLISH-\|v0\.[12345]\b" crates/*/src/`). Examples:
- `crates/reposix-core/src/backend.rs:2` — "v0.1.5 ships [`sim::SimBackend`] only; v0.2 will add a `GithubReadOnlyBackend` in `crates/reposix-github`." — both shipped long ago.
- `crates/reposix-cache/src/lib.rs:17` — "Audit log, tainted-byte discipline, and egress allowlist enforcement land in Plan 02 and Plan 03 of Phase 31." — they landed.
- `crates/reposix-cli/src/refresh.rs:155` — `// TODO(Phase-21): populate commit_sha …` — Phase 21 shipped without populating it; either populate or delete the TODO.

Any phase marker that's been completed should either be deleted or downgraded to "(see git log; …)" framing.

### P2-3. Stringly-typed egress detection in `Cache::classify_backend_error`

`crates/reposix-cache/src/builder.rs:510-524`:

```rust
let is_egress = matches!(e, reposix_core::Error::InvalidOrigin(_))
    || emsg.contains("blocked origin")
    || emsg.contains("invalid origin")
    || emsg.contains("allowlist");
```

Comment admits *"both typed and stringly to handle backend adapters that wrap the core error in `Error::Other(String)`"*. Rooted in P1-1 (Error::Other overuse). Once the typed variants land, the substring branches go.

### P2-4. `reposix-sim/src/main.rs` lacks `#![forbid(unsafe_code)]`

`crates/reposix-sim/src/main.rs` is the only binary-entry point in the workspace without the attribute. Every other `lib.rs` and `main.rs` has it. One-line fix at line 1.

### P2-5. `reposix-cli/src/main.rs` `#[allow(clippy::too_many_lines)]` on `main` is fine — but the file is at 420 LOC and growing

The function is fine because it's a clap-derive dispatch. But every new subcommand adds another arm. Consider extracting each arm body into a `dispatch_<verb>(args)` function in `main.rs` so `main()` becomes a strict 80-line match-and-await. Pure refactor; no behaviour change.

### P2-6. `Cache::log_token_cost` and friends only WARN on SQL failure, but the WARN goes to a target other tests don't filter on

`crates/reposix-cache/src/audit.rs:33-36` (and 14 sister sites) emits `tracing::warn!(target: "reposix_cache::audit_failure", …)`. Integration tests (e.g. `crates/reposix-cache/tests/audit_is_append_only.rs`) check the *table* state, not the WARN events. A test that wants to assert "audit was best-effort skipped on a poisoned cache" has no hook today. Add a `cfg(test)` global counter or expose a `Metrics::audit_failures()` getter.

### P2-7. `reposix-cli::tokens::with_commas` and `reposix-cli::cost::with_commas` are byte-identical 16-line functions

`crates/reposix-cli/src/tokens.rs:267-283` and `crates/reposix-cli/src/cost.rs:221-237`. Pull into `worktree_helpers.rs` (or a new `reposix-cli::format` module). Same pattern as the four `cache_path_from_worktree` triplets.

### P2-8. `reposix-confluence::ConfPage` etc. derive `Debug` on response shapes containing potentially sensitive content

`crates/reposix-confluence/src/lib.rs:228 (ConfPage), 268 (ConfPageBody), 281 (ConfBodyStorage)` — `#[derive(Debug, Deserialize)]`. If anyone ever logs a `ConfPage` at debug level, the body content (which may contain auth tokens pasted by the user, secrets, etc.) lands in logs. The `ConfluenceCreds` struct has the manual-Debug redaction discipline (line 123); apply it consistently to wire types that carry user content.

### P2-9. `pub fn parse_next_cursor` and `pub fn basic_auth_header` are crate-public despite no external consumer

`crates/reposix-confluence/src/lib.rs:542,558` — both `pub fn`, both used only inside the file. `pub(crate)` if you need them from `tests/`, or move into `impl ConfluenceBackend` as private methods. Same for `crates/reposix-jira/src/lib.rs:957,971`.

---

## Patterns I'd reject in a PR review

1. **`expect("checked")` after `std::env::var`.** `crates/reposix-remote/src/backend_dispatch.rs:253-254,268-269` reads `std::env::var("ATLASSIAN_API_KEY").expect("checked")` after a separate `collect_missing` validation. The TOCTOU window here is microscopic but it's a smell — pass the value through from the validation call site instead. Pattern occurs 4×. Ten minutes to refactor.

2. **`unwrap()` on `parts.last_mut()` after a non-empty proof above it.** `crates/reposix-cli/src/cost.rs:230` and `crates/reposix-cli/src/tokens.rs:276`. Either prove non-empty in the type system (`Vec` → `Vec1` or `[String; N]`) or comment why. Today it's two unsupported-by-doc `unwrap`s in user-facing paths.

3. **`println!` in subcommand modules instead of `writeln!(out, …)`**: every CLI subcommand prints directly to stdout. That makes them un-testable for output without `assert_cmd` subprocess shelling. Plumbing a `&mut dyn Write` through the public API would cost ~20 LOC per subcommand and unlock fast unit-level output tests.

4. **`.to_string()` on path components instead of `&str`.** `crates/reposix-cli/src/worktree_helpers.rs:51-61` returns `String` from `backend_slug_from_origin` even though every value is a `&'static str`. Should be `&'static str`. Same in `GcStrategy::slug` (`crates/reposix-cache/src/gc.rs:62-69`) — already `&'static str`, the right pattern. Be consistent.

5. **Pulling `parking_lot` AND `std::sync::Mutex` in the same workspace.** `crates/reposix-cache/src/cache.rs:5` uses `std::sync::Mutex`; `crates/reposix-confluence/src/lib.rs:75`, `crates/reposix-github/src/lib.rs:60`, `crates/reposix-sim/src/state.rs:10`, `crates/reposix-swarm/src/sim_direct.rs:17` use `parking_lot::Mutex`. Pick one. Workspace `Cargo.toml:91` already has `parking_lot = "0.12"` at workspace level — converge there.

6. **Audit-row INSERT statements as inline string literals scattered across 18 files.** Every `crate::audit::log_*` function in `reposix-cache/src/audit.rs` (16 fns) hand-writes the INSERT. Plus `confluence/src/lib.rs:1097` and `jira/src/lib.rs:411` write their own. Promote to a single `INSERT_AUDIT_ROW: &str = ...;` constant per (table, columns) tuple; the SQL is part of the schema contract, not code.

---

## Cross-cutting refactor opportunities

### CC-1. Promote `cache_path_from_worktree` (the canonical one) to `reposix-cache`

`reposix-cache::path::resolve_cache_path` already takes `(backend, project)`. The CLI wrapper `worktree_helpers::cache_path_from_worktree` reads `remote.origin.url` + `parse_remote_url` + `backend_slug_from_origin`. The *whole pipeline* is generic and belongs in `reposix-cache` as `Cache::path_for_worktree(work: &Path) -> Result<PathBuf>`. The `reposix-cli` layer disappears. Same logic also wanted by the `reposix-remote` helper for the dark-factory rebase teaching path.

### CC-2. Two cache.db schemas → one

See P0-4. `reposix-cli/src/cache_db.rs` (`refresh_meta` schema) and `reposix-cache/src/db.rs` (`audit_events_cache + meta + oid_map`) coexist on disk at different paths. The `refresh_meta` shape is one row of `(backend, project, last_fetched_at, commit_sha)` — that is a `meta` table by another name. Migrate `refresh.rs` to use the cache crate's `meta::set_meta('refresh.last_fetched_at', …)`, delete `cache_db.rs`. Cuts ~250 LOC + one SQLite file per worktree.

### CC-3. Backend-write-side audit hook → trait method, not per-backend SQLite connection

Each backend that mutates external state (`confluence::create_record`, `jira::create_record`, etc.) should take an `&dyn AuditSink` instead of an `Option<Arc<Mutex<Connection>>>`. The simulator's `audit_events` table writer becomes one impl; the cache's `audit_events_cache` writer becomes another. Backends drop `rusqlite/sha2/hex` deps entirely.

### CC-4. Single `# Panics` doc cluster

`crates/reposix-cache/src/cache.rs` has 13 audit-forwarder methods with identical 5-line `# Panics` docs. After P1-8 (`with_db` helper), one module-level `## Panics` note replaces all of them.

---

## What's actually GOOD

A senior reviewer should also call out the parts that are well-built — they're not the squeaky wheel, but they're the load-bearing assets you don't want to disturb:

1. **`Tainted<T>` / `Untainted<T>` discipline** in `crates/reposix-core/src/taint.rs`. Newtype with no `Deref`, `Untainted::new` is `pub(crate)`, the *only* legal upcast is `sanitize`. Compile-fail fixtures (`crates/reposix-cache/tests/compile_fail.rs:22`) lock the invariant. Genuinely well-designed for a security-critical seam.

2. **HTTP allowlist gate** (`crates/reposix-core/src/http.rs`). The `HttpClient` newtype hides `reqwest::Client` behind a private field. The workspace `clippy.toml` (`disallowed-methods = ["reqwest::Client::new", "reqwest::Client::builder", "reqwest::ClientBuilder::new"]`) enforces this at compile time. Defence-in-depth done right.

3. **`reposix-cache::error::Error`** (`crates/reposix-cache/src/error.rs`). Eight typed variants with no `Other(String)` escape hatch. `CacheCollision { expected, found }` and `OidDrift { requested, actual, issue_id }` carry structured payload — exactly the pattern `reposix-core::Error` should adopt.

4. **Audit append-only trigger** (`crates/reposix-core/src/audit.rs:35`, schema in `crates/reposix-core/fixtures/audit.sql`). Schema-level `BEFORE UPDATE/DELETE` triggers raise `RAISE(ABORT, …)`. Tested at `crates/reposix-core/tests/audit_schema.rs:128`. The runtime guarantee matches the docstring.

5. **`BackendConnector` trait dyn-compatibility proof** (`crates/reposix-core/src/backend.rs:247`). One `#[allow(dead_code)] fn _assert_dyn_compatible(_: &dyn BackendConnector) {}` line costs nothing and breaks the build immediately on any future violation. Idiomatic Rust pattern.

6. **`SyncTag` time-travel** (`crates/reposix-cache/src/sync_tag.rs`) — `parse_sync_tag_timestamp`, `format_sync_tag_slug`, deterministic ordering, fully tested at `crates/reposix-cache/tests/sync_tags.rs`. Replayable artefact, no implicit state.

7. **Doctor's check-isolation pattern** (`crates/reposix-cli/src/doctor.rs:336-940`). Each `check_*` function returns `DoctorFinding` and is independently testable. `tests/doctor.rs` exercises ~16 of the 18 checks separately. Adding a check is one new function; it never explodes the call-site complexity.

8. **`#![deny(clippy::print_stdout)]` in `reposix-remote/src/main.rs:10`**. The remote helper's protocol depends on stdout being only protocol-frames; an accidental future `println!` is now a compile error rather than a corrupted protocol stream. This is exactly the right tool for the right invariant.

9. **`gix::ObjectId`-typed throughout the cache crate** (`crates/reposix-cache/src/builder.rs:22,353`). Never stringly-typed except at the SQL boundary (where it's `oid.to_hex().to_string()` once). The SHA1 vs SHA256 future-proofing falls out of `repo.object_hash()`.

10. **`#[non_exhaustive]` on `BackendFeature` and `DeleteReason`** (`crates/reposix-core/src/backend.rs:51,83`). Adding a variant is non-breaking. Future-proof at zero cost.

---

*Compiled by reading every `crates/*/src/**/*.rs` and spot-checking `tests/cli.rs`, `tests/agent_flow.rs`, `tests/contract.rs`. No `cargo` or `rustc` invocations per CLAUDE.md RAM rule.*
