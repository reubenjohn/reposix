//! The [`Cache`] struct. Holds the backend, project, gix bare repo, and
//! `cache.db` connection.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use reposix_core::BackendConnector;

use crate::db::open_cache_db;
use crate::error::{Error, Result};
use crate::meta;
use crate::path::resolve_cache_path;

/// Backing bare-repo cache for one `(backend, project)` tuple.
///
/// Created via [`Cache::open`]. Call [`Cache::build_from`] to populate
/// the tree; call [`Cache::read_blob`] to materialize a blob on demand.
pub struct Cache {
    pub(crate) backend: Arc<dyn BackendConnector>,
    pub(crate) backend_name: String,
    pub(crate) project: String,
    pub(crate) path: PathBuf,
    pub(crate) repo: gix::Repository,
    /// Wrapped in [`Mutex`] because [`rusqlite::Connection`] is not
    /// [`Send`]-safe across `await` points; interior mutability lets
    /// the async methods acquire the lock, do a short SQL call, and
    /// drop it before awaiting.
    pub(crate) db: Mutex<rusqlite::Connection>,
}

impl Cache {
    /// Open (or create) the cache at the deterministic path for
    /// `(backend_name, project)`.
    ///
    /// Side effects: [`std::fs::create_dir_all`] on the parent,
    /// [`gix::init_bare`] on the target, and [`open_cache_db`] on
    /// `<cache-path>/cache.db`. Idempotent — re-opening an existing
    /// cache rebinds the handles without touching content.
    ///
    /// On second and subsequent opens, the `meta` table is consulted
    /// for an `identity` row; if present and mismatched with the
    /// caller's `(backend_name, project)`, returns
    /// [`Error::CacheCollision`]. On first open the identity is
    /// written.
    ///
    /// # Errors
    /// - [`Error::Io`] for directory creation failure or no
    ///   discoverable cache root.
    /// - [`Error::Git`] if `gix::init_bare` fails.
    /// - [`Error::Sqlite`] if the cache DB cannot be opened or its
    ///   schema cannot be loaded.
    /// - [`Error::CacheCollision`] if the cache belongs to a
    ///   different `(backend, project)` tuple.
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

        // cache.db lives inside the bare repo dir so a single path
        // scheme covers both git state and cache state.
        let db = open_cache_db(&path)?;

        // Identity check: Plan 02 writes on first open, errors on
        // mismatch. Phase 33 may refine the semantics (e.g. wipe +
        // re-seed).
        let expected = format!("{backend_name}:{project}");
        if let Some(found) = meta::get_meta(&db, "identity")? {
            if found != expected {
                return Err(Error::CacheCollision { expected, found });
            }
        } else {
            meta::set_meta(&db, "identity", &expected)?;
        }

        Ok(Self {
            backend,
            backend_name,
            project,
            path,
            repo,
            db: Mutex::new(db),
        })
    }

    /// On-disk path to the bare repo (the `<backend>-<project>.git` dir).
    #[must_use]
    pub fn repo_path(&self) -> &std::path::Path {
        &self.path
    }

    /// Backend name (written into audit rows).
    #[must_use]
    pub fn backend_name(&self) -> &str {
        &self.backend_name
    }

    /// Project slug (written into audit rows).
    #[must_use]
    pub fn project(&self) -> &str {
        &self.project
    }
}
