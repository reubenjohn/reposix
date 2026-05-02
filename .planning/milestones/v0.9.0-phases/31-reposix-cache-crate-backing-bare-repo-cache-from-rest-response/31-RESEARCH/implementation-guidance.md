← [back to index](./index.md)

# Common Pitfalls and Code Examples

## Common Pitfalls

### Pitfall 1: Computing blob OID without writing, then writing in a different order
**What goes wrong:** Tree references blob OID `X`; later `read_blob` writes the bytes and gets OID `Y` because frontmatter rendering was non-deterministic (e.g., `BTreeMap` iteration order changes, or a timestamp leaked in).
**Why it happens:** `frontmatter::render` is supposed to be pure but `Issue.extensions: BTreeMap` is the only field whose iteration order affects YAML output, and BTreeMap is deterministic by key — so this is unlikely. But `chrono::Utc::now()` MUST NOT appear inside `render()`.
**How to avoid:** Add a unit test in this phase asserting `frontmatter::render(&issue) == frontmatter::render(&issue.clone())` — bytewise. (Not a regression test on `reposix-core`; a contract assertion the cache crate relies on.)
**Warning signs:** `Error::OidDrift` firing in production. If it ever fires, look at non-determinism in `render` first.

### Pitfall 2: Audit row write fails after blob write
**What goes wrong:** Blob is in `.git/objects/`, but `audit_events_cache` row never wrote. Subsequent compliance audits show fewer rows than blobs, looking like a security event.
**Why it happens:** SQLite `BUSY` from a concurrent reader; disk full; transient FS error.
**How to avoid:** CONTEXT.md decision: "audit failure logs WARN but still returns bytes — audit failure must not poison the user flow, but should be visible." Use `tracing::warn!` with target `reposix_cache::audit_failure`, include the issue_id and OID, and add a metric counter `audit_failures_total`. Document this trade-off in module docs so reviewers don't think it's an oversight.
**Warning signs:** WARN log lines with target `reposix_cache::audit_failure` in production logs. Operators should treat persistent occurrence as a P1.

### Pitfall 3: Tainted bytes leak via panic / unwrap
**What goes wrong:** `Cache::read_blob` returns `Tainted<Vec<u8>>`. Caller does `cache.read_blob(oid).await?.into_inner()` and the bytes flow into `git push` to a non-allowlisted remote.
**Why it happens:** Tainted is enforced by the type system, but `into_inner()` is `pub` and unconditional. The compile-fail fixture in this phase only catches the *type* contract, not the *flow* contract.
**How to avoid:** This phase ships the type-level guard (the compile-fail fixture ensures `egress::send(tainted)` doesn't compile). Flow-level guard is Phase 34's job (sanitize-on-push frontmatter allowlist). Document the boundary explicitly: "this phase makes Tainted impossible to misuse *implicitly*; phase 34 makes Tainted impossible to misuse *explicitly* without a documented sanitize step."
**Warning signs:** Code reviews finding `into_inner()` calls in non-cache crates. The crate's own consumers (Phase 32 helper) should call `inner_ref()` and pipe bytes directly to git's stdout, never `into_inner()` followed by an HTTP send.

### Pitfall 4: gix API stability across versions
**What goes wrong:** `gix` is pre-1.0 (issue #470 explicitly tracks 1.0 readiness). Method signatures for `commit_as` / `edit_tree` / `edit_reference` shift between minor versions.
**Why it happens:** Maintainers' explicit policy until 1.0.
**How to avoid:** Pin `gix = "=0.82.0"` (with `=`) in this crate's Cargo.toml until v0.10.0 stabilizes the broader cache API. Wave A's first task should be a smoke test that compiles `gix::init_bare` + `write_blob` + `commit` against the actually-resolved version before the rest of the builder is written.
**Warning signs:** `cargo update` output mentions `gix v0.82 → v0.83` and immediately something stops compiling.

### Pitfall 5: Misconfiguring the bare-repo's HEAD
**What goes wrong:** `gix::init_bare` may default HEAD to `refs/heads/master` or `refs/heads/main` depending on the user's `init.defaultBranch` git config. The Phase 32 helper expects `refs/heads/main`.
**Why it happens:** gix respects local git config when present.
**How to avoid:** Explicitly set HEAD to `refs/heads/main` after init via `repo.edit_reference` (or write `HEAD` file directly if gix doesn't expose this — fallback). Add a test: `assert_eq!(cache.repo.head_ref()?.name(), "refs/heads/main")`.
**Warning signs:** Phase 32 integration tests fail with "ref does not exist" on a freshly-built cache.

### Pitfall 6: Concurrent build_from and read_blob
**What goes wrong:** Two callers race: caller A is in the middle of `build_from` (rewriting the tree), caller B calls `read_blob(some_oid)` against an OID that's about to be evicted from the new tree.
**Why it happens:** v0.9.0 has no global lock around the cache.
**How to avoid:** v0.9.0 scope (CONTEXT.md): single-writer, no concurrent build_from. Document this constraint. Phase 33 (delta sync) will introduce a serialization story (likely SQLite EXCLUSIVE WAL lock per the existing `cache_db.rs` pattern).
**Warning signs:** This is a deferred concern; not worth defending in v0.9.0.

## Code Examples

### Example 1: Audit table DDL (lifts the `reposix_core::audit` pattern)

```sql
-- Source: pattern from crates/reposix-core/fixtures/audit.sql (verified)
-- This file: crates/reposix-cache/fixtures/cache_schema.sql

BEGIN;

CREATE TABLE IF NOT EXISTS audit_events_cache (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    ts            TEXT    NOT NULL,                                -- ISO 8601 UTC
    op            TEXT    NOT NULL CHECK (op IN ('materialize','egress_denied','tree_sync')),
    backend       TEXT    NOT NULL,
    project       TEXT    NOT NULL,
    issue_id      TEXT,                                            -- nullable for tree_sync
    oid           TEXT,                                            -- nullable for tree_sync (or hex blob OID)
    bytes         INTEGER,                                         -- blob size (materialize) or item count (tree_sync)
    reason        TEXT                                             -- nullable; populated for egress_denied
);

CREATE TABLE IF NOT EXISTS meta (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS oid_map (
    oid       TEXT PRIMARY KEY,
    issue_id  TEXT NOT NULL,
    backend   TEXT NOT NULL,
    project   TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_oid_map_issue
    ON oid_map(backend, project, issue_id);

DROP TRIGGER IF EXISTS audit_cache_no_update;
CREATE TRIGGER audit_cache_no_update BEFORE UPDATE ON audit_events_cache
    BEGIN
        SELECT RAISE(ABORT, 'audit_events_cache is append-only');
    END;

DROP TRIGGER IF EXISTS audit_cache_no_delete;
CREATE TRIGGER audit_cache_no_delete BEFORE DELETE ON audit_events_cache
    BEGIN
        SELECT RAISE(ABORT, 'audit_events_cache is append-only');
    END;

COMMIT;
```

### Example 2: trybuild compile-fail fixture for Tainted discipline

```rust
// File: crates/reposix-cache/tests/compile-fail/tainted_blob_into_egress.rs
//
// This file MUST fail to compile. The .stderr sibling captures the expected
// error so trybuild's UI test framework asserts the diagnostic shape.

use reposix_core::Tainted;

// A fake "egress" sink that demands an Untainted<Vec<u8>>.
// The real Phase 34 push path will look like this.
fn egress_send(_bytes: reposix_core::Untainted<Vec<u8>>) {}

fn main() {
    let tainted: Tainted<Vec<u8>> = Tainted::new(vec![1, 2, 3]);
    // The next line MUST NOT compile: there is no From<Tainted<Vec<u8>>>
    // for Untainted<Vec<u8>>, and Untainted::new is pub(crate).
    egress_send(tainted);
}
```

```rust
// File: crates/reposix-cache/tests/compile_fail.rs

#[test]
fn tainted_blob_cannot_flow_to_egress() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/tainted_blob_into_egress.rs");
}
```

### Example 3: Cache::open + build_from skeleton (informational — not a copy-paste contract)

```rust
// crates/reposix-cache/src/cache.rs

use std::path::PathBuf;
use std::sync::Arc;

use reposix_core::{BackendConnector, Tainted};

pub struct Cache {
    pub(crate) backend: Arc<dyn BackendConnector>,
    pub(crate) project: String,
    pub(crate) path: PathBuf,
    pub(crate) repo: gix::Repository,
    pub(crate) db: rusqlite::Connection,
}

impl Cache {
    /// Open or create a cache at the deterministic path for (backend, project).
    ///
    /// # Errors
    /// - I/O failure during directory creation
    /// - gix::init_bare failure
    /// - SQLite open failure
    pub fn open(
        backend: Arc<dyn BackendConnector>,
        project: impl Into<String>,
    ) -> Result<Self> { /* ... */ }

    /// Sync the tree from the backend. Writes a synthesis commit to refs/heads/main.
    /// Does NOT materialize blobs.
    ///
    /// # Errors
    /// Any backend / git / sqlite error. See `Error`.
    pub async fn build_from(&self) -> Result<gix::ObjectId> { /* ... */ }

    /// Materialize a single blob by OID. Tainted because the bytes came from
    /// the backend.
    ///
    /// # Errors
    /// - Unknown OID
    /// - Backend egress denied (audit row written first)
    /// - OID drift between requested and actual
    pub async fn read_blob(&self, oid: gix::ObjectId) -> Result<Tainted<Vec<u8>>> { /* ... */ }
}
```
