← [back to index](./index.md) · phase 81 plan 01

## Task 81-01-T03 — `reposix sync --reconcile` CLI subcommand + smoke test

<read_first>
- `crates/reposix-cli/src/main.rs` lines 37-160 (`enum Cmd` — find the
  alphabetical insertion point for `Sync`; existing variants:
  `Sim, Init, Attach, List, Refresh, Spaces, Doctor, History, Log, At, Gc, Tokens, Cost, Version`).
- `crates/reposix-cli/src/main.rs` lines 27-29 (existing
  `use reposix_cli::{...}` — the new `sync` import joins this list).
- `crates/reposix-cli/src/lib.rs` (entire file — 29 lines; the new
  `pub mod sync;` joins alphabetically between `spaces` and `tokens`).
- `crates/reposix-cli/src/refresh.rs` (find the cache-from-worktree
  pattern — `RefreshConfig` + how it resolves the cache).
- `crates/reposix-cli/src/attach.rs` (find the cache-resolution
  pattern used by the P79 attach handler — likely the cleaner model
  for `sync --reconcile`).
- `crates/reposix-cache/src/path.rs` (full file — 8-31 lines;
  `REPOSIX_CACHE_DIR` env + `resolve_cache_path` function).
- `crates/reposix-cache/src/builder.rs` lines 56-90 (`Cache::build_from`
  signature: `pub async fn build_from(&self) -> Result<gix::ObjectId>`;
  the handler calls this directly).
- `crates/reposix-cli/tests/agent_flow.rs` (existing CLI smoke-test
  pattern — sim subprocess + CLI binary against tempdir; the new
  `tests/sync.rs` test mirrors this).
- `crates/reposix-cli/tests/attach.rs` (P79-precedent for cache-state
  assertion patterns).
</read_first>

<action>
Three concerns: new module → clap-derive variant → smoke test → cargo
check + commit.

### 3a. New module — `crates/reposix-cli/src/sync.rs`

Author the new file. Keep it minimal (~50 lines). The handler:

```rust
//! `reposix sync [--reconcile]` — cache reconciliation against the
//! SoT. The L1 conflict-detection escape hatch.
//!
//! Without `--reconcile`, prints a one-line hint pointing at
//! `--reconcile` (NOT an error — `reposix sync` is a v0.13.0+ surface
//! whose bare form is reserved for future flag combinations).
//!
//! Design intent: the bus remote (P82–P83) names this command in its
//! reject-path stderr hints. Renaming or making the bare form error
//! would force a doc-rev cascade.
//!
//! See `.planning/research/v0.13.0-dvcs/architecture-sketch.md
//! § "Performance subtlety: today's `list_records` walk on every push"`.

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::worktree_helpers::resolve_cache_for_worktree;

/// Entrypoint for `reposix sync [--reconcile] [path]`.
///
/// `reconcile=false`: prints a hint pointing at `--reconcile`; exits 0.
/// `reconcile=true`: opens the cache for the working tree at `path`
/// (or cwd) and calls `Cache::build_from` to perform a full
/// `list_records` walk + cache rebuild. The cache's `meta.last_fetched_at`
/// cursor is bumped to `Utc::now()` as a side effect of `build_from`.
///
/// # Errors
/// - I/O when locating the cache / opening the cache
/// - REST when calling `Cache::build_from` against the backend
pub async fn run(reconcile: bool, path: Option<PathBuf>) -> Result<()> {
    if !reconcile {
        println!(
            "reposix sync: pass --reconcile to perform a full \
             list_records walk + cache rebuild (the L1 escape hatch)."
        );
        println!(
            "see: `architecture-sketch.md § Performance subtlety` and \
             docs/concepts/dvcs-topology.md (P85 forthcoming)."
        );
        return Ok(());
    }

    let cwd = match path {
        Some(p) => p,
        None => std::env::current_dir().context("get cwd")?,
    };
    let cache = resolve_cache_for_worktree(&cwd)
        .context("resolve cache for working tree")?;
    let oid = cache
        .build_from()
        .await
        .context("Cache::build_from for --reconcile")?;
    println!(
        "reposix sync: cache rebuilt (synthesis-commit OID = {oid}); \
         meta.last_fetched_at advanced."
    );
    Ok(())
}
```

**Cache-resolution accessor.** `resolve_cache_for_worktree` is a
placeholder — the actual accessor name MUST be confirmed during T03
read_first. Likely candidates (in `crates/reposix-cli/src/`):
- `worktree_helpers.rs` — confirmed module exists per
  `crates/reposix-cli/src/lib.rs:28` (`pub mod worktree_helpers;`).
- `cache_db.rs` — `pub mod cache_db;` per `lib.rs:17`.
- `attach.rs` / `refresh.rs` — internal accessors.

The pattern is: take a working tree directory → derive
`(backend, project)` from the configured remote URL → call
`reposix_cache::path::resolve_cache_path(backend, project)` →
`Cache::open(...)`. T03's executor reads `attach.rs` or `refresh.rs`,
finds the existing helper that performs this resolution, and uses it
verbatim. If no shared helper exists, the inline pattern (3 lines)
inside `run()` is acceptable; do NOT add a new pub helper unless it
clearly factors out duplicated code.

### 3b. `lib.rs` re-export — `crates/reposix-cli/src/lib.rs`

Add `pub mod sync;` to the pub-mod list. Position alphabetically between
`spaces` and `tokens`:

```rust
pub mod attach;
pub mod binpath;
pub mod cache_db;
pub mod cost;
pub mod doctor;
pub mod gc;
pub mod history;
pub mod init;
pub mod list;
pub mod refresh;
pub mod sim;
pub mod spaces;
pub mod sync;          // NEW
pub mod tokens;
pub mod worktree_helpers;
```

### 3c. clap-derive surface — `crates/reposix-cli/src/main.rs`

Two edits:

1. Add `sync` to the existing `use reposix_cli::{...};` line (line 27):

   ```rust
   use reposix_cli::{
       attach, cost, doctor, gc, history, init, list, refresh, sim, spaces, sync, tokens,
   };
   ```

2. Add the `Sync` variant to `enum Cmd` (alphabetical placement — between
   `Spaces` and `Doctor` is cleanest given the existing ordering, OR
   match the visual flow of `Spaces` then a new top-level command;
   pick alphabetical):

   ```rust
   /// On-demand cache reconciliation against the SoT (the L1 escape
   /// hatch per `architecture-sketch.md § Performance subtlety`).
   ///
   /// Without --reconcile, this prints a hint pointing at the flag.
   ///
   /// Examples:
   ///   reposix sync --reconcile
   ///   reposix sync --reconcile /tmp/repo
   Sync {
       /// Force a full list_records walk + cache rebuild (drops L1
       /// cursor; the next push behaves like a first-push).
       #[arg(long)]
       reconcile: bool,
       /// Working-tree directory. Defaults to cwd.
       path: Option<PathBuf>,
   },
   ```

3. Add the match arm in `main`'s match statement (alphabetical placement
   — after `Spaces`):

   ```rust
   Cmd::Sync { reconcile, path } => sync::run(reconcile, path).await,
   ```

### 3d. Smoke test — `crates/reposix-cli/tests/sync.rs`

Author the new test file:

```rust
//! Smoke test for `reposix sync --reconcile` (DVCS-PERF-L1-02).
//!
//! Asserts the command exists, accepts --reconcile, and advances the
//! cache's `meta.last_fetched_at` cursor.

#![allow(clippy::missing_panics_doc)]

use std::sync::Arc;

use reposix_cache::Cache;
use reposix_core::BackendConnector;
use wiremock::MockServer;

mod common;
use common::{sample_issues, seed_mock, sim_backend, CacheDirGuard};

#[tokio::test(flavor = "multi_thread")]
async fn sync_reconcile_advances_cursor() {
    let server = MockServer::start().await;
    let issues = sample_issues("demo", 3);
    seed_mock(&server, "demo", &issues).await;

    let cache_root = tempfile::tempdir().unwrap();
    let _env = CacheDirGuard::new(cache_root.path());
    let backend: Arc<dyn BackendConnector> = sim_backend(&server);
    let cache = Cache::open(backend, "sim", "demo").expect("Cache::open");

    // Seed sync to populate the cursor.
    cache.sync().await.expect("seed sync");
    let t1 = cache
        .read_last_fetched_at()
        .expect("read cursor")
        .expect("cursor present after seed sync");

    // Sleep so RFC3339 second-granularity ticks forward.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    // Run the CLI subcommand. Use the in-process API to avoid spawning
    // a subprocess (faster + deterministic). The `reposix_cli::sync::run`
    // path matches the clap-derive dispatch in `main.rs`.
    let cwd = tempfile::tempdir().unwrap();
    // The handler resolves cache from cwd; we point cwd at a path that
    // resolves to the same cache dir via REPOSIX_CACHE_DIR env (set by
    // CacheDirGuard).
    reposix_cli::sync::run(true, Some(cwd.path().to_path_buf()))
        .await
        .expect("sync --reconcile");

    let t2 = cache
        .read_last_fetched_at()
        .expect("read cursor after sync")
        .expect("cursor present after --reconcile");

    assert!(
        t2 > t1,
        "expected cursor to advance after --reconcile; got t1 = {t1}, t2 = {t2}"
    );
}
```

**`mod common;` reuse (M3 hard-block).** Cargo's test harness does
NOT share `mod common;` across crates — each crate's `tests/` directory
has its own. As of plan-check 2026-05-01, the helpers (`sample_issues`,
`seed_mock`, `sim_backend`, `CacheDirGuard`) live ONLY in
`crates/reposix-cache/tests/common/mod.rs`. CHECK FIRST whether
`crates/reposix-cli/tests/common.rs` (or `tests/common/mod.rs`)
exists AND contains the helpers — `crates/reposix-cli/tests/history.rs:30-145`
defines local copies of `CacheDirGuard`, `sample_issues`, `seed_mock`
inline (each crate's test files re-define them as needed). If
`reposix-cli/tests/common.rs` does NOT have them, EITHER copy them from
`crates/reposix-cache/tests/common/mod.rs` (preferred — authoritative
reference) OR inline the minimal setup directly in `tests/sync.rs`
(faster — but check the existing `history.rs` style first to stay
consistent). DO NOT assume `mod common;` works across crates without
verification.

**`reposix_cli::sync::run` invocation.** The smoke test calls the
in-process API directly (avoiding subprocess overhead). The clap-derive
dispatch in `main.rs` MUST match exactly — `Cmd::Sync { reconcile,
path } => sync::run(reconcile, path).await,` — so the in-process call
exercises the same code path as the user's CLI invocation.

If the in-process call doesn't resolve the cache correctly (cwd
mismatch with `REPOSIX_CACHE_DIR`), fall back to a subprocess
invocation:

```rust
let output = std::process::Command::new(env!("CARGO_BIN_EXE_reposix"))
    .args(["sync", "--reconcile"])
    .arg(cwd.path())
    .env("REPOSIX_CACHE_DIR", cache_root.path())
    .output()
    .expect("run reposix sync --reconcile");
assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
```

Build serially:

```bash
cargo check -p reposix-cli
cargo clippy -p reposix-cli -- -D warnings
cargo nextest run -p reposix-cli --test sync
```

Stage and commit:

```bash
git add crates/reposix-cli/src/sync.rs \
        crates/reposix-cli/src/lib.rs \
        crates/reposix-cli/src/main.rs \
        crates/reposix-cli/tests/sync.rs
git commit -m "$(cat <<'EOF'
feat(cli): reposix sync --reconcile subcommand (DVCS-PERF-L1-02)

- crates/reposix-cli/src/sync.rs (new) — async run(reconcile, path) handler; thin wrapper over Cache::build_from
- Bare `reposix sync` (no flags) prints a one-line hint pointing at --reconcile (NOT an error — bus remote in P82-P83 will name this command in reject-path hints; reserving the bare form for future flag combinations)
- crates/reposix-cli/src/lib.rs — pub mod sync added
- crates/reposix-cli/src/main.rs — Sync { reconcile, path } variant + match arm + use update
- crates/reposix-cli/tests/sync.rs (new) — sync_reconcile_advances_cursor smoke test asserting last_fetched_at advances

Phase 81 / Plan 01 / Task 03 / DVCS-PERF-L1-02.
EOF
)"
```
</action>

<verify>
  <automated>cargo check -p reposix-cli && cargo clippy -p reposix-cli -- -D warnings && cargo nextest run -p reposix-cli --test sync && cargo run -p reposix-cli -- sync --reconcile --help > /dev/null</automated>
</verify>

<done>
- `crates/reposix-cli/src/sync.rs` exists with `pub async fn run(reconcile: bool, path: Option<PathBuf>) -> Result<()>`.
- `crates/reposix-cli/src/lib.rs` declares `pub mod sync;`.
- `crates/reposix-cli/src/main.rs` clap-derive surface includes the
  `Sync { reconcile: bool, path: Option<PathBuf> }` variant + a match
  arm dispatching to `sync::run`.
- `cargo run -p reposix-cli -- sync --reconcile --help` exits 0.
- `cargo run -p reposix-cli -- sync` (no flags) exits 0 and prints
  the hint line.
- `crates/reposix-cli/tests/sync.rs::sync_reconcile_advances_cursor`
  passes (`cargo nextest run -p reposix-cli --test sync`).
- `cargo clippy -p reposix-cli -- -D warnings` exits 0.
- `# Errors` doc on `sync::run`.
- Cargo serialized: T03 cargo invocations run only after T02's commit
  has landed; per-crate fallback used.
</done>

---

