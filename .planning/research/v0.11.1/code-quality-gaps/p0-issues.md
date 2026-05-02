← [back to index](./index.md)

# P0 issues (block 1.0)

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
