//! The [`Cache`] struct. Holds the backend, project, and gix bare repo.

use std::path::PathBuf;
use std::sync::Arc;

use reposix_core::BackendConnector;

use crate::error::{Error, Result};
use crate::path::resolve_cache_path;

/// Backing bare-repo cache for one `(backend, project)` tuple.
///
/// Created via [`Cache::open`]. Call [`Cache::build_from`] to populate
/// the tree; call `Cache::read_blob` (Plan 02) to materialize a blob
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
    /// Side effects: [`std::fs::create_dir_all`] on the parent, and
    /// [`gix::init_bare`] on the target. Idempotent — re-opening an
    /// existing cache rebinds the [`gix::Repository`] handle without
    /// touching filesystem content.
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
        Ok(Self {
            backend,
            backend_name,
            project,
            path,
            repo,
        })
    }

    /// On-disk path to the bare repo (the `<backend>-<project>.git` dir).
    #[must_use]
    pub fn repo_path(&self) -> &std::path::Path {
        &self.path
    }

    /// Backend name (useful for audit-log rows in Plan 02).
    #[must_use]
    pub fn backend_name(&self) -> &str {
        &self.backend_name
    }

    /// Project slug (useful for audit-log rows in Plan 02).
    #[must_use]
    pub fn project(&self) -> &str {
        &self.project
    }
}
