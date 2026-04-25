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
        let mut repo = gix::init_bare(&path).map_err(|e| Error::Git(e.to_string()))?;

        // Provide a default committer/author identity so `Cache::build_from`
        // can produce a commit even when the host has no `user.name` /
        // `user.email` configured (typical of CI runners). The
        // `noreply.invalid` domain is RFC-2606 reserved and signals
        // "not a real address" to anyone reading the log. User-level
        // git config (e.g. `~/.gitconfig`) is loaded BEFORE this fallback
        // is applied, but we only set values if the keys are absent —
        // so a configured `user.name` / `user.email` still wins.
        if repo.committer().is_none() {
            let mut snap = repo.config_snapshot_mut();
            if snap.string(gix::config::tree::User::NAME).is_none() {
                snap.set_value(&gix::config::tree::User::NAME, "reposix-cache")
                    .map_err(|e| Error::Git(format!("set user.name: {e}")))?;
            }
            if snap.string(gix::config::tree::User::EMAIL).is_none() {
                snap.set_value(
                    &gix::config::tree::User::EMAIL,
                    "reposix-cache@noreply.invalid",
                )
                .map_err(|e| Error::Git(format!("set user.email: {e}")))?;
            }
            snap.commit().map_err(|e| Error::Git(e.to_string()))?;
        }

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

    /// Write an `op='helper_connect'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_connect(&self, service: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_connect(&db, &self.backend_name, &self.project, service);
    }

    /// Write an `op='helper_advertise'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_advertise(&self, bytes: u32) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_advertise(&db, &self.backend_name, &self.project, bytes);
    }

    /// Write an `op='helper_fetch'` audit row. Best-effort. The `stats`
    /// payload is produced by the `reposix-remote::stateless_connect`
    /// handler; we accept a by-value wrapper (via a structural trait)
    /// so the two crates don't need to share a type. See the
    /// `HelperFetchRecord` trait below.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_fetch<R: HelperFetchRecord>(&self, stats: &R) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_fetch(
            &db,
            &self.backend_name,
            &self.project,
            stats.command(),
            stats.want_count(),
            stats.request_bytes(),
            stats.response_bytes(),
        );
    }

    /// Write an `op='helper_fetch_error'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_fetch_error(&self, exit_code: i32, stderr_tail: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_fetch_error(
            &db,
            &self.backend_name,
            &self.project,
            exit_code,
            stderr_tail,
        );
    }

    /// Write an `op='blob_limit_exceeded'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_blob_limit_exceeded(&self, want_count: u32, limit: u32) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_blob_limit_exceeded(
            &db,
            &self.backend_name,
            &self.project,
            want_count,
            limit,
        );
    }

    /// Write an `op='helper_push_started'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_push_started(&self, ref_name: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_push_started(&db, &self.backend_name, &self.project, ref_name);
    }

    /// Write an `op='helper_push_accepted'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_push_accepted(&self, files_touched: u32, summary: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_push_accepted(
            &db,
            &self.backend_name,
            &self.project,
            files_touched,
            summary,
        );
    }

    /// Write an `op='helper_push_rejected_conflict'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_push_rejected_conflict(
        &self,
        issue_id: &str,
        local_version: u64,
        backend_version: u64,
    ) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_push_rejected_conflict(
            &db,
            &self.backend_name,
            &self.project,
            issue_id,
            local_version,
            backend_version,
        );
    }

    /// Write an `op='helper_push_sanitized_field'` audit row. Best-effort.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned.
    pub fn log_helper_push_sanitized_field(&self, issue_id: &str, field: &str) {
        let db = self.db.lock().expect("cache.db mutex poisoned");
        crate::audit::log_helper_push_sanitized_field(
            &db,
            &self.backend_name,
            &self.project,
            issue_id,
            field,
        );
    }
}

/// Structural accessor for a helper-fetch RPC-turn record. Implemented
/// by `reposix-remote::stateless_connect::RpcStats` — we use a trait
/// so that crate does not depend on a `reposix-cache`-defined struct
/// and vice-versa, keeping the cache crate free of
/// transport-layer concepts.
pub trait HelperFetchRecord {
    /// The `command=<word>` keyword if extracted from the first data
    /// frame (`fetch`, `ls-refs`, etc.). `None` if not a recognizable
    /// protocol-v2 command.
    fn command(&self) -> Option<&str>;
    /// Count of `want ` lines observed in the request.
    fn want_count(&self) -> u32;
    /// Total request bytes written to `upload-pack` stdin.
    fn request_bytes(&self) -> u32;
    /// Total response bytes read from `upload-pack` stdout.
    fn response_bytes(&self) -> u32;
}
