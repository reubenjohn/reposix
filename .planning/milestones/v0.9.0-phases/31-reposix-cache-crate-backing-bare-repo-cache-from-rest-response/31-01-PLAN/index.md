---
phase: 31
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - crates/reposix-cache/Cargo.toml
  - crates/reposix-cache/src/lib.rs
  - crates/reposix-cache/src/error.rs
  - crates/reposix-cache/src/path.rs
  - crates/reposix-cache/src/cache.rs
  - crates/reposix-cache/src/builder.rs
  - crates/reposix-cache/tests/tree_contains_all_issues.rs
  - crates/reposix-cache/tests/blobs_are_lazy.rs
autonomous: true
requirements:
  - ARCH-01
tags:
  - rust
  - git
  - gix
  - cache
  - scaffold
user_setup: []

must_haves:
  truths:
    - "Crate `reposix-cache` exists as a workspace member and builds cleanly (`cargo check -p reposix-cache` + `cargo clippy -p reposix-cache --all-targets -- -D warnings`)."
    - "`Cache::build_from(&self).await` against a SimBackend seeded with N issues produces a valid bare git repo on disk with a commit on `refs/heads/main`."
    - "After `build_from`, the tree at HEAD contains exactly N entries, one per seeded issue, in the canonical `issues/<id>.md` path."
    - "After `build_from`, no blob objects exist in `.git/objects/` — only tree(s) + commit (lazy-blob invariant)."
    - "Cache path is deterministic: `$XDG_CACHE_HOME/reposix/<backend>-<project>.git` by default, overridable by `REPOSIX_CACHE_DIR` env var."
  artifacts:
    - path: "crates/reposix-cache/Cargo.toml"
      provides: "Crate manifest with gix=0.82.0 pinned, rusqlite, dirs, thiserror, tracing workspace deps, async-trait, tokio, chrono, serde_yaml workspace deps, reposix-core path dep, and dev-deps tempfile+trybuild."
      contains: "gix = \"=0.82.0\""
    - path: "crates/reposix-cache/src/lib.rs"
      provides: "Crate root with forbid(unsafe_code), warn(clippy::pedantic), and re-exports of Cache and Error."
      contains: "#![forbid(unsafe_code)]"
    - path: "crates/reposix-cache/src/cache.rs"
      provides: "Cache struct definition holding backend, project, path, gix::Repository, rusqlite::Connection."
      contains: "pub struct Cache"
    - path: "crates/reposix-cache/src/builder.rs"
      provides: "async fn build_from that lists issues, renders frontmatter, assembles tree, writes commit, updates refs/heads/main without writing blobs."
      contains: "async fn build_from"
    - path: "crates/reposix-cache/src/path.rs"
      provides: "resolve_cache_path(backend, project) honoring REPOSIX_CACHE_DIR then dirs::cache_dir()."
      contains: "pub fn resolve_cache_path"
    - path: "crates/reposix-cache/src/error.rs"
      provides: "thiserror enum with variants Egress(reposix_core::Error), Backend(String), Sqlite(rusqlite::Error), Git(Box<gix::reference::edit::Error>) or equivalent gix error, Io(std::io::Error), UnknownOid(gix::ObjectId), OidDrift { requested, actual, issue_id }, CacheCollision { expected, found }."
      contains: "pub enum Error"
    - path: "Cargo.toml"
      provides: "Workspace adds crates/reposix-cache to members and gix=0.82.0 + dirs=6 to workspace.dependencies."
      contains: "crates/reposix-cache"
  key_links:
    - from: "crates/reposix-cache/src/builder.rs"
      to: "reposix_core::backend::BackendConnector::list_issues"
      via: "async trait dispatch through Arc<dyn BackendConnector>"
      pattern: "list_issues\\("
    - from: "crates/reposix-cache/src/builder.rs"
      to: "reposix_core::issue::frontmatter::render"
      via: "function call producing canonical on-disk bytes"
      pattern: "frontmatter::render"
    - from: "crates/reposix-cache/src/builder.rs"
      to: "gix::Repository"
      via: "edit_tree + commit + edit_reference (or write_object / write_reference analogues)"
      pattern: "gix::"
    - from: "Cargo.toml"
      to: "crates/reposix-cache"
      via: "workspace members array"
      pattern: "reposix-cache"
---

# Phase 31 Plan 01 — reposix-cache crate: backing bare-repo cache from REST response

<objective>
Scaffold the new `reposix-cache` crate and land the foundation of the git-native architecture pivot: a bare-repo builder that turns a `BackendConnector`'s issue list into a real on-disk bare git repo with a populated tree and an empty blob set (blobs are lazy). This is Wave A — the substrate every Wave B/C task (audit/egress wiring, SQLite, trybuild compile-fail) builds against. The executor must also verify the gix 0.82 API surface (`init_bare`, `edit_tree`, `commit`, `edit_reference`) compiles against the pinned version BEFORE the full builder is written, per RESEARCH §Pitfall 4.

Purpose: ARCH-01 establishes the on-disk substrate the `stateless-connect` handler in Phase 32 will tunnel protocol-v2 traffic to. Without this plan, the rest of v0.9.0 has nothing to build on.
Output: A new workspace crate with `Cache` + `Cache::build_from` + deterministic cache path resolution + two integration tests proving tree correctness and blob laziness. No audit writes, no trybuild fixture, no SQLite side tables yet — Wave B and C own those.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-CONTEXT.md
@.planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-RESEARCH.md
@.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md

@Cargo.toml
@crates/reposix-core/src/lib.rs
@crates/reposix-core/src/backend.rs
@crates/reposix-core/src/issue.rs
@crates/reposix-core/src/taint.rs
@crates/reposix-sim/src/lib.rs

<interfaces>
<!-- Load-bearing interfaces extracted from reposix-core. Executor should use these directly. -->

From `crates/reposix-core/src/lib.rs` (re-exports):
```rust
pub use backend::{BackendConnector, BackendFeature, DeleteReason};
pub use error::{Error, Result};
pub use issue::{frontmatter, Issue, IssueId, IssueStatus};
pub use taint::{sanitize, ServerMetadata, Tainted, Untainted};
```

From `crates/reposix-core/src/backend.rs` (the two methods this plan consumes):
```rust
#[async_trait]
pub trait BackendConnector: Send + Sync + 'static {
    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>>;
    async fn get_issue(&self, project: &str, id: IssueId) -> Result<Issue>;
    // ... other methods not used in Plan 01
}
```

From `crates/reposix-core/src/issue.rs`:
```rust
// Canonical on-disk rendering: YAML frontmatter + Markdown body, newline-terminated.
pub fn render(issue: &Issue) -> Result<String>;

pub struct Issue { pub id: IssueId, pub title: String, /* ... */ pub body: String, /* ... */ }
pub struct IssueId(pub u64); // prints as decimal
```

From `crates/reposix-core/src/error.rs` (variants this plan wraps/propagates):
```rust
pub enum Error {
    InvalidOrigin(String),     // from http client — becomes Error::Egress wrapping at plan 02
    Http(reqwest::Error),
    Other(String),             // catch-all; plan 01 maps to Error::Backend(String)
    Yaml(serde_yaml::Error),
    // ... (see crate for full list)
}
pub type Result<T> = std::result::Result<T, Error>;
```

From gix 0.82 (`docs.rs/gix/0.82.0/`) — verify these compile in Task 1, they are the only gix surface area Plan 01 touches:
```rust
// Crate-level
pub fn gix::init_bare(path: impl AsRef<std::path::Path>) -> Result<gix::Repository, gix::init::Error>;

// On gix::Repository
pub fn object_hash(&self) -> gix::hash::Kind;                  // returns Sha1 for v0.9.0
pub fn empty_tree(&self) -> gix::Object<'_>;                   // empty tree OID accessor (check exact name)
pub fn edit_tree(&self, root: gix::ObjectId) -> Result<gix::object::tree::Editor<'_>, _>;
    // Editor has .upsert(path: impl AsRef<bstr::BStr>, kind: tree::EntryKind, oid: ObjectId) -> Result<_>;
    // and .write() -> Result<ObjectId, _>;
pub fn commit(
    &self,
    reference_name: &str,              // e.g. "refs/heads/main"
    message: impl AsRef<str>,
    tree: gix::ObjectId,
    parents: impl IntoIterator<Item = impl Into<gix::ObjectId>>,
) -> Result<gix::ObjectId, gix::object::commit::Error>;        // CONFIRM exact signature in Task 1
pub fn hash_object(
    &self,
    kind: gix::object::Kind,
    data: &[u8],
) -> Result<gix::hash::ObjectId, _>;                           // returns the OID WITHOUT writing — critical for lazy invariant
```

If any gix signature above differs in 0.82.0 from the sketch, Task 1 MUST document the real signature in a code comment at the top of `crates/reposix-cache/src/builder.rs` BEFORE Task 2 proceeds. Do NOT assume; `cargo check` is the ground truth.

From `crates/reposix-cli/src/cache_db.rs` (reference pattern only — NOT lifted in Plan 01):
```rust
// Opens cache.db with mode=0o600, WAL, EXCLUSIVE. This is the pattern Plan 02 will lift.
pub struct CacheDb(rusqlite::Connection);
pub fn open_cache_db(mount: &Path) -> Result<CacheDb>;
```
</interfaces>
</context>

## Chapters

- **[T01 — Scaffold `reposix-cache` crate and smoke-test gix 0.82 API surface](./T01-scaffold-crate.md)** — Add workspace member, create `Cargo.toml` + `lib.rs`, write `gix_api_smoke` test to verify `init_bare`, `edit_tree`, `commit`, and `hash_object` all compile against the pinned 0.82.0.
- **[T02 — Implement `Cache::build_from` (Steps 1–4)](./T02-build-from-step1.md)** — Create `error.rs`, `path.rs`, `cache.rs`, and `builder.rs` (Steps 1–4: the full `build_from` implementation with lazy-blob `compute_blob_oid`).
- **[T02 — Implement `Cache::build_from` (Steps 5–8 + acceptance)](./T02-build-from-step2.md)** — Wire modules in `lib.rs`, create integration tests `tree_contains_all_issues.rs` and `blobs_are_lazy.rs`, run verification (Steps 5–8 + acceptance criteria + verify + done).
- **[Threat model, verification, success criteria, and output](./tail-threat-verification.md)** — STRIDE threat register, phase verification checklist, success criteria checklist, and `<output>` spec for the post-completion summary.
