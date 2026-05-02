← [back to index](./index.md)

# Task 2: Implement `Cache::build_from` — tree construction with lazy blobs (Steps 1–4)

Continued in [Steps 5–8 + acceptance criteria](./T02-build-from-step2.md).

<task type="auto" tdd="true">
  <name>Task 2: Implement `Cache::build_from` — tree construction with lazy blobs</name>
  <files>
    crates/reposix-cache/src/error.rs,
    crates/reposix-cache/src/path.rs,
    crates/reposix-cache/src/cache.rs,
    crates/reposix-cache/src/builder.rs,
    crates/reposix-cache/src/lib.rs,
    crates/reposix-cache/tests/tree_contains_all_issues.rs,
    crates/reposix-cache/tests/blobs_are_lazy.rs
  </files>
  <read_first>
    crates/reposix-cache/Cargo.toml,
    crates/reposix-cache/src/lib.rs,
    crates/reposix-cache/tests/gix_api_smoke.rs,
    crates/reposix-core/src/backend.rs,
    crates/reposix-core/src/issue.rs,
    crates/reposix-sim/src/lib.rs,
    crates/reposix-cli/src/cache_db.rs,
    .planning/phases/31-reposix-cache-crate-backing-bare-repo-cache-from-rest-response/31-RESEARCH.md
  </read_first>
  <behavior>
    - `Cache::open(backend, project)` resolves a deterministic path from `(backend_name, project)` via `REPOSIX_CACHE_DIR` env var falling back to `dirs::cache_dir()/reposix/`. Creates the parent dir, `gix::init_bare`s the bare repo, stores `backend`, `project`, `path`, `repo` in the struct.
    - `Cache::build_from(&self).await` calls `backend.list_issues(project)`, renders each issue to canonical bytes via `reposix_core::issue::frontmatter::render`, computes (but does not persist as standalone objects beyond the tree walk) each blob OID, writes a tree with entries `issues/<id>.md -> <blob_oid>`, writes a commit on `refs/heads/main` with message `sync(<backend>:<project>): <N> issues at <ISO8601>`, returns the commit OID.
    - After `build_from` completes, `git ls-tree -r refs/heads/main | wc -l` equals the number of seeded issues (verified by gix-equivalent code or a `gix::Repository::find_tree` walk).
    - After `build_from` completes, no blob objects live in `.git/objects/` for the issue bodies — only the tree and commit objects. (Implementation note: `gix::Repository::write_blob` DOES persist the blob. Plan 01's builder must use `gix::Repository::hash_object(Kind::Blob, &bytes)` to compute the OID without persisting, OR use `gix::objs::compute_hash` — whichever the Task 1 smoke-test verified exists in 0.82. If only `write_blob` exists and there is no standalone hash-only API, fall back to the `git-hash` crate via gix re-export; the key invariant is "no loose/pack blob objects in `.git/objects/` after build_from" — assert this explicitly in the `blobs_are_lazy` test.)
    - HEAD is explicitly set to `refs/heads/main` after init (defends against user `init.defaultBranch = master` leaking in from `~/.gitconfig`, per RESEARCH §Pitfall 5).
    - Integration tests cover (a) N=1, (b) N=10 seeded issues — tree entry count matches, tree paths are exactly `issues/<id>.md`, and no blob objects are present in `.git/objects/`.
  </behavior>
  <action>
    Step 1 — Create `crates/reposix-cache/src/error.rs`:
    ```rust
    //! Typed error for the reposix-cache crate.

    use thiserror::Error;

    /// Errors produced by the reposix-cache crate.
    #[derive(Debug, Error)]
    pub enum Error {
        /// I/O failure (directory creation, file open).
        #[error("io: {0}")]
        Io(#[from] std::io::Error),

        /// Backend (`BackendConnector`) returned an error. In Plan 02 the
        /// `InvalidOrigin` sub-variant will be split out as `Egress` so the
        /// audit layer can distinguish egress denial from other failures.
        #[error("backend: {0}")]
        Backend(String),

        /// gix operation failed (init, write_object, edit_tree, commit,
        /// edit_reference). Boxed to keep the error variant small.
        #[error("git: {0}")]
        Git(String),

        /// Rendering issue to canonical on-disk bytes failed.
        #[error("render: {0}")]
        Render(#[from] reposix_core::Error),

        /// Plan 01 does NOT use rusqlite yet; variant added in Plan 02.
        /// Reserved here so downstream consumers see stable variants.
        #[error("sqlite: {0}")]
        Sqlite(String),

        /// Cache path belongs to a different `(backend, project)` than the
        /// one passed to `Cache::open`. Raised in Plan 02 once the `meta`
        /// table is wired; scaffolded here.
        #[error("cache collision: expected {expected}, found {found}")]
        CacheCollision { expected: String, found: String },
    }

    /// Alias for this crate's `Result`.
    pub type Result<T> = std::result::Result<T, Error>;

    impl From<gix::init::Error> for Error {
        fn from(e: gix::init::Error) -> Self { Self::Git(e.to_string()) }
    }
    impl From<rusqlite::Error> for Error {
        fn from(e: rusqlite::Error) -> Self { Self::Sqlite(e.to_string()) }
    }
    ```

    If the gix error variants needed are different from `gix::init::Error` (builder uses `edit_tree`, `commit`, `find_reference`, etc.), add stringly-typed `From` impls using `to_string()` rather than boxing the concrete error; we prefer the smaller enum over exhaustive type-level propagation for a Phase 31 foundation. Plan 02 can tighten if needed.

    Step 2 — Create `crates/reposix-cache/src/path.rs`:
    ```rust
    //! Deterministic cache path resolution. No hidden state (OP-4).

    use std::path::PathBuf;

    use crate::error::{Error, Result};

    /// Environment variable that overrides the default cache directory root.
    pub const CACHE_DIR_ENV: &str = "REPOSIX_CACHE_DIR";

    /// Resolve the on-disk bare-repo path for (backend, project).
    ///
    /// Precedence:
    /// 1. `REPOSIX_CACHE_DIR` env var, if set and non-empty.
    /// 2. `dirs::cache_dir()` (XDG on Linux, `~/Library/Caches` on macOS,
    ///    `%LOCALAPPDATA%` on Windows).
    /// 3. Error if neither is available (no `$HOME`, no `XDG_CACHE_HOME`).
    ///
    /// The returned path is `<root>/reposix/<backend>-<project>.git`.
    ///
    /// # Errors
    /// Returns [`Error::Io`] if no cache root is discoverable.
    pub fn resolve_cache_path(backend: &str, project: &str) -> Result<PathBuf> {
        let root = std::env::var(CACHE_DIR_ENV)
            .ok()
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(dirs::cache_dir)
            .ok_or_else(|| {
                Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "no cache dir: set REPOSIX_CACHE_DIR or ensure XDG_CACHE_HOME/HOME is set",
                ))
            })?;
        // Safe filename: the simulator uses `simulator` for its backend_name
        // and project slugs are validated upstream; we do NOT re-validate here.
        Ok(root.join("reposix").join(format!("{backend}-{project}.git")))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn env_var_wins() {
            let tmp = tempfile::tempdir().unwrap();
            let prev = std::env::var(CACHE_DIR_ENV).ok();
            // SAFETY: this test is the only caller; no other test reads this env.
            std::env::set_var(CACHE_DIR_ENV, tmp.path());
            let p = resolve_cache_path("sim", "proj-1").unwrap();
            assert_eq!(p, tmp.path().join("reposix").join("sim-proj-1.git"));
            match prev {
                Some(v) => std::env::set_var(CACHE_DIR_ENV, v),
                None => std::env::remove_var(CACHE_DIR_ENV),
            }
        }
    }
    ```

    Step 3 — Create `crates/reposix-cache/src/cache.rs`:
    ```rust
    //! The `Cache` struct. Holds the backend, project, and gix bare repo.

    use std::path::PathBuf;
    use std::sync::Arc;

    use reposix_core::BackendConnector;

    use crate::error::{Error, Result};
    use crate::path::resolve_cache_path;

    /// Backing bare-repo cache for one `(backend, project)` tuple.
    ///
    /// Created via [`Cache::open`]. Call [`Cache::build_from`] to populate
    /// the tree; call [`Cache::read_blob`] (Plan 02) to materialize a blob
    /// on demand.
    pub struct Cache {
        pub(crate) backend: Arc<dyn BackendConnector>,
        pub(crate) backend_name: String,
        pub(crate) project: String,
        pub(crate) path: PathBuf,
        pub(crate) repo: gix::Repository,
    }

    impl Cache {
        /// Open (or create) the cache at the deterministic path for
        /// `(backend_name, project)`.
        ///
        /// Side effects: `std::fs::create_dir_all` on the parent, and
        /// `gix::init_bare` on the target. Idempotent — re-opening an
        /// existing cache is a no-op for the filesystem and simply rebinds
        /// the `gix::Repository` handle.
        ///
        /// # Errors
        /// - [`Error::Io`] for directory creation failure or no discoverable
        ///   cache root.
        /// - [`Error::Git`] if `gix::init_bare` fails.
        pub fn open(
            backend: Arc<dyn BackendConnector>,
            backend_name: impl Into<String>,
            project: impl Into<String>,
        ) -> Result<Self> {
            let backend_name = backend_name.into();
            let project = project.into();
            let path = resolve_cache_path(&backend_name, &project)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let repo = gix::init_bare(&path).map_err(|e| Error::Git(e.to_string()))?;
            Ok(Self { backend, backend_name, project, path, repo })
        }

        /// Return the on-disk path to the bare repo.
        #[must_use]
        pub fn repo_path(&self) -> &std::path::Path {
            &self.path
        }

        /// Return the backend name (useful for audit log rows in Plan 02).
        #[must_use]
        pub fn backend_name(&self) -> &str { &self.backend_name }

        /// Return the project (useful for audit log rows in Plan 02).
        #[must_use]
        pub fn project(&self) -> &str { &self.project }
    }
    ```

    Step 4 — Create `crates/reposix-cache/src/builder.rs`:
    ```rust
    //! Tree construction and (Plan 02) lazy blob materialization.

    use chrono::Utc;
    use reposix_core::issue::frontmatter;

    use crate::cache::Cache;
    use crate::error::{Error, Result};

    impl Cache {
        /// Sync the tree from the backend and commit to `refs/heads/main`.
        ///
        /// Does NOT materialize blobs — the returned commit references blob
        /// OIDs that are only persisted on demand via `read_blob` (Plan 02).
        ///
        /// Commit message format: `sync(<backend>:<project>): <N> issues at <ISO8601>`.
        ///
        /// # Errors
        /// - [`Error::Backend`] if `list_issues` fails.
        /// - [`Error::Render`] if frontmatter rendering fails for any issue.
        /// - [`Error::Git`] if any gix operation fails.
        pub async fn build_from(&self) -> Result<gix::ObjectId> {
            let issues = self
                .backend
                .list_issues(&self.project)
                .await
                .map_err(|e| Error::Backend(e.to_string()))?;

            // Render each issue, compute OID WITHOUT writing the blob.
            // The tree references each blob_oid; the blob itself is persisted
            // only when `read_blob(oid)` is called.
            //
            // NOTE: The exact API for "compute OID without persist" in gix
            // 0.82 is one of:
            //   - `gix::Repository::hash_object(Kind::Blob, bytes)` (if present)
            //   - `gix::objs::compute_hash(object_hash, Kind::Blob, bytes)`
            //     (crate-level helper — available via gix reexport)
            // Task 1's smoke test confirmed the exact shape; use whichever
            // the smoke test validated. If both are absent, fall back to
            // `repo.write_blob(bytes)` (persists) and ACCEPT that Plan 01
            // persists blobs — but this violates the lazy invariant, so
            // the `blobs_are_lazy` test will fail and the executor must
            // loop back with a real hash-only path.
            let mut entries: Vec<(String, gix::ObjectId)> = Vec::with_capacity(issues.len());
            for issue in &issues {
                let rendered = frontmatter::render(issue)?;
                let bytes = rendered.into_bytes();
                let oid = compute_blob_oid(&self.repo, &bytes)?;
                let path = format!("issues/{}.md", issue.id.0);
                entries.push((path, oid));
            }

            // Assemble tree.
            let empty = gix::ObjectId::empty_tree(self.repo.object_hash());
            let mut editor = self
                .repo
                .edit_tree(empty)
                .map_err(|e| Error::Git(e.to_string()))?;
            for (path, oid) in &entries {
                editor
                    .upsert(path.as_str(), gix::object::tree::EntryKind::Blob, *oid)
                    .map_err(|e| Error::Git(e.to_string()))?;
            }
            let tree_oid = editor.write().map_err(|e| Error::Git(e.to_string()))?;

            // Commit.
            let msg = format!(
                "sync({}:{}): {} issues at {}",
                self.backend_name,
                self.project,
                entries.len(),
                Utc::now().to_rfc3339()
            );
            let commit_oid = self
                .repo
                .commit(
                    "refs/heads/main",
                    msg,
                    tree_oid,
                    std::iter::empty::<gix::ObjectId>(),
                )
                .map_err(|e| Error::Git(e.to_string()))?;

            // Explicitly point HEAD at refs/heads/main (Pitfall 5).
            // gix::Repository::head_name() / set_head() — API name may differ;
            // using a file-level write is the guaranteed-portable fallback.
            let head_path = self.path.join("HEAD");
            std::fs::write(&head_path, "ref: refs/heads/main\n")?;

            Ok(commit_oid.detach())
        }
    }

    /// Compute blob OID without writing to `.git/objects`.
    ///
    /// Plan 01 lazy-blob invariant: the tree references OIDs the cache has
    /// NOT persisted as objects. Phase 32 (stateless-connect helper) will
    /// call `Cache::read_blob(oid)` to materialize.
    fn compute_blob_oid(repo: &gix::Repository, bytes: &[u8]) -> Result<gix::ObjectId> {
        // Prefer the hash-only API. If gix 0.82 does not expose one, use
        // gix_object::compute_hash via a minimal helper; in the worst case,
        // wrap the bytes with "blob <len>\0" manually and sha1 — but that
        // is exactly the "don't hand-roll git object format" anti-pattern
        // from RESEARCH §Don't Hand-Roll. If you reach that point, STOP
        // and update this plan before proceeding.
        //
        // Example using gix::objs::compute_hash (replace with the exact
        // path Task 1's smoke test validated):
        let hash_kind = repo.object_hash();
        let oid = gix::objs::compute_hash(hash_kind, gix::object::Kind::Blob, bytes)
            .map_err(|e| Error::Git(e.to_string()))?;
        Ok(oid)
    }
    ```
