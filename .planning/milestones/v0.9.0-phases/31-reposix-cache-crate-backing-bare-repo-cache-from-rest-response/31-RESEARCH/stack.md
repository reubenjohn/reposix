← [back to index](./index.md)

# Standard Stack, Don't Hand-Roll, and Runtime State Inventory

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `gix` | 0.82.0 (latest, 2026-04-24) | Pure-Rust git object writing: `init_bare`, `write_blob`, `write_object`, `edit_tree`, `commit_as`, `edit_reference` | `[VERIFIED: crates.io API]` Pure Rust — no `libgit2`/`pkg-config` dep (matches workspace policy: `default-features = false` for `fuser` was chosen for the same reason). All required APIs are stable on `Repository`. Used by `cargo` itself (PR #14762 bumped cargo to gix 0.67) so production-tested. |
| `rusqlite` | 0.32 (already in workspace) | Audit + meta DB | `[VERIFIED: workspace Cargo.toml]` Already a workspace dependency with `bundled` feature → no system libsqlite3. The existing `reposix_core::audit` module wraps it; this crate follows the same pattern. |
| `chrono` | 0.4 (already in workspace) | `DateTime<Utc>` for audit `ts`, `last_fetched_at`, commit author timestamps | `[VERIFIED: workspace Cargo.toml]` Project convention is "Times are `chrono::DateTime<Utc>`. No `SystemTime` in serialized form." (CLAUDE.md). |
| `serde_yaml` | 0.9 (already in workspace) | Issue frontmatter rendering — already done by `reposix_core::issue::frontmatter::render` | `[VERIFIED: workspace Cargo.toml]` This crate calls into the existing renderer; doesn't pull serde_yaml as a direct dep. |
| `tokio` | 1 (already in workspace) | Async runtime — `BackendConnector` methods are `async` | `[VERIFIED: workspace Cargo.toml]` Workspace standard. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `dirs` | 6.0.0 (latest) | Resolve `$XDG_CACHE_HOME` for default cache path | `[VERIFIED: crates.io API 2025-01-12]` Smaller than `etcetera`, sufficient for our use (single function call: `dirs::cache_dir()`). Use `etcetera` 0.11.0 only if we need testable injection of `BaseStrategy` — `dirs` returns `Option<PathBuf>` directly, simpler. |
| `thiserror` | 2 (already in workspace) | Typed `reposix_cache::Error` with variants `Egress(InvalidOrigin)`, `Backend(BackendError)`, `Sqlite(rusqlite::Error)`, `Git(gix::ObjectError)`, `Io(std::io::Error)` | Project convention: thiserror in libraries, anyhow at binary boundaries. |
| `async-trait` | 0.1 (already in workspace) | Implementing async methods if any new traits are introduced | Likely unneeded — `Cache` itself can have `async fn` directly since it's a concrete struct, not a trait. |
| `tempfile` | (test-dev only) | Test fixtures: ephemeral cache paths | Already used by `cache_db.rs` tests. |

### Dev-dependencies

| Library | Version | Purpose |
|---------|---------|---------|
| `trybuild` | 1.0.116 (latest, 2026-02-12) | Compile-fail fixture for `Tainted<Vec<u8>>` discipline. `[VERIFIED: crates.io API]` |
| `wiremock` | (current workspace version) | Mock REST backend for tests where SimBackend isn't enough. Likely unneeded if SimBackend covers all scenarios. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `gix` 0.82 | `git2` 0.20.4 (libgit2 bindings) | git2 has more mature object-mutation surface but pulls `libgit2-sys` which links against system libgit2 (or vendored C). This violates the workspace's pure-Rust build-deps stance (the same reason `fuser` was held to `default-features = false` to avoid `libfuse-dev`). gix is sufficient — write_blob/write_object/edit_tree/commit_as/edit_reference all exist. **CITED:** [docs.rs/gix Repository methods](https://docs.rs/gix/latest/gix/struct.Repository.html) |
| `dirs` 6.0.0 | `etcetera` 0.11.0 (more flexible BaseStrategy) | etcetera lets you inject a strategy (good for tests) but `dirs` is simpler for one call site. The cache path is also overridable via `REPOSIX_CACHE_DIR` so test injection can use that env var instead. |
| Single SQLite file | Two files (audit.db + meta.db) | Not worth the complexity — one connection, two tables. Locking semantics for `cache.db` already exist in `crates/reposix-cli/src/cache_db.rs`. |
| Multi-commit history | Single synthesis commit per sync | LOCKED by CONTEXT.md to single-commit. Multi-commit would force commit-graph reasoning. v0.10.0 work. |

**Installation (proposed `crates/reposix-cache/Cargo.toml`):**

```toml
[package]
name = "reposix-cache"
version.workspace = true
edition.workspace = true

[dependencies]
reposix-core = { workspace = true }
tokio = { workspace = true }
async-trait = { workspace = true }
chrono = { workspace = true }
rusqlite = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
gix = "0.82"
dirs = "6"

[dev-dependencies]
tempfile = "3"
trybuild = "1"
```

**Version verification:** `[VERIFIED: crates.io API 2026-04-24]`
- gix 0.82.0 published 2026-04-24
- git2 0.20.4 published 2026-02-02 (alternative, not selected)
- trybuild 1.0.116 published 2026-02-12
- dirs 6.0.0 published 2025-01-12
- etcetera 0.11.0 published 2025-10-28 (alternative, not selected)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Git object format (loose object headers, zlib, sha1/sha256 OID computation) | Custom `write_loose_object` | `gix::Repository::write_blob`, `write_object` | Header format edge cases (`blob <len>\0<bytes>`), zlib parameters, hash algorithm switching (sha1 vs sha256) — all handled by gix. Hand-rolling = guaranteed bugs. |
| Bare-repo init (`HEAD`, `config`, `description`, `objects/info/`) | Custom directory layout | `gix::init_bare` | Many subtle files; `git fsck` is unforgiving. |
| Tree object encoding (sorted entries, mode bits, null-terminated names) | Custom `write_tree` | `gix::Repository::edit_tree` (requires `tree-editor` cargo feature, default in 0.82) | Tree entry sort order is critical (entries with trailing `/` sort differently). gix gets this right. |
| Append-only audit log (BEFORE UPDATE/DELETE triggers, DEFENSIVE flag) | New schema in this crate from scratch | Pattern from `reposix_core::audit` (lift the open-with-DEFENSIVE pattern + write our own DDL with the same trigger structure) | The hardening (M-04, H-02) is already understood in the workspace — `BEGIN…COMMIT` wrapper, `DROP TRIGGER IF EXISTS` for idempotency, `SQLITE_DBCONFIG_DEFENSIVE` to block `writable_schema` attacks. Don't re-discover. |
| HTTP egress allowlist | Custom URL parser + check | `reposix_core::http::client()` returns `HttpClient` which gates every send | Already shipped, already tested (`http_allowlist.rs`), already lint-locked. The cache **must not** even hold a `reqwest::Client`. |
| Tainted/Untainted boundary | New newtype | `reposix_core::Tainted<T>` | Already defined; the discipline (no `Deref`, no `From`) is established. Just use it. |
| XDG cache dir | `std::env::var("XDG_CACHE_HOME")` then fallback to `$HOME/.cache` | `dirs::cache_dir()` | Handles XDG spec, macOS `~/Library/Caches`, Windows `%LOCALAPPDATA%`, falls back to `$HOME/.cache` per spec. (Even though we only target Linux, the right thing is the right thing.) |
| Commit message timestamp formatting | `format!("{}", chrono::Utc::now())` | `chrono::Utc::now().to_rfc3339()` or `.format("%+")` | RFC 3339 / ISO 8601 — what the CONTEXT.md commit-message spec asks for. |

**Key insight:** Every part of this crate has precedent in the workspace OR in stable, well-known external crates. The novel work is the **composition** — gluing `BackendConnector` → render → gix → audit → Tainted return — not any single primitive.

## Runtime State Inventory

This phase introduces NEW on-disk state. There is no rename/refactor migration. But the planner needs to know what state lives where:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | New: `$XDG_CACHE_HOME/reposix/<backend>-<project>.git/` (bare repo objects + refs + config). New: `<that-path>/cache.db` (SQLite — meta, oid_map, audit_events_cache). | New crate writes both; `.gitignore` for `runtime/` already covers test artifacts in this repo. |
| Live service config | None — this crate adds no service. | None. |
| OS-registered state | None. The cache is just files. | None. |
| Secrets/env vars | New env var consumed: `REPOSIX_CACHE_DIR` (override of default cache path). Existing env vars consumed: `REPOSIX_ALLOWED_ORIGINS` (via `reposix_core::http::client()`). | Document both in CHANGELOG `[v0.9.0]` Added section and in `crates/reposix-cache/src/lib.rs` module docs. |
| Build artifacts / installed packages | None. New crate is workspace-internal; not published to crates.io in v0.9.0. | None. |

**Verified by:** `grep -rn "XDG_CACHE_HOME\|dirs::cache_dir" .` (no existing refs in repo) and CONTEXT.md `<specifics>` section (cache path scheme).
