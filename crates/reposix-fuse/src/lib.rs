//! FUSE daemon library — see [`Mount`] for the public entry point.
//!
//! The read-only mount presents every issue in a reposix-compatible backend
//! as a single Markdown file at the mount root, named `<zero-padded-id>.md`.
//! Read-path I/O now flows through the [`reposix_core::IssueBackend`] trait
//! (Phase 10 rewire), so the same FUSE daemon can serve the simulator or a
//! real GitHub repo. Every backend call is still wrapped in a 5-second
//! [`tokio::time::timeout`] (SG-07) so the kernel cannot hang on a dead
//! backend. Write-path callbacks still speak to the simulator's REST shape
//! directly via [`fetch`] (v0.3 will lift these onto the trait).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use fuser::{BackgroundSession, MountOption};
use reposix_core::IssueBackend;
use serde::{Deserialize, Serialize};

pub mod fetch;
pub mod fs;
pub mod inode;
pub mod tree;

pub use fs::ReposixFs;
pub use inode::InodeRegistry;
pub use tree::{TreeSnapshot, TREE_DIR_INO_BASE, TREE_ROOT_INO, TREE_SYMLINK_INO_BASE};

/// Runtime configuration for a FUSE mount.
///
/// The `origin` field is retained because the write-path callbacks
/// (`release` → PATCH, `create` → POST) still speak the simulator's REST
/// shape directly via [`fetch`]. Read-path callbacks (`readdir`, `read`,
/// `lookup`) route through the [`IssueBackend`] passed to [`Mount::open`],
/// which is how `--backend github` works end-to-end.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    /// Where to mount.
    pub mount_point: PathBuf,
    /// Origin of the reposix-compatible REST backend used by the write path
    /// (e.g. `http://127.0.0.1:7878`). Ignored by the read path.
    pub origin: String,
    /// Project slug (sim) or `owner/repo` (github). Passed to every
    /// `IssueBackend` call.
    #[serde(default = "default_project")]
    pub project: String,
    /// Read-only mode. When `true` the kernel refuses writes at the VFS
    /// layer and the write-path callbacks never fire.
    pub read_only: bool,
}

fn default_project() -> String {
    "demo".to_owned()
}

/// A running FUSE mount. Dropping unmounts via fuser's `UmountOnDrop`.
#[derive(Debug)]
pub struct Mount {
    _session: BackgroundSession,
}

impl Mount {
    /// Spawn a FUSE mount at `cfg.mount_point` whose read path is served by
    /// `backend`. The mount lives until the returned [`Mount`] is dropped.
    ///
    /// # Errors
    /// Returns an error if:
    /// - the mount point cannot be created,
    /// - the [`ReposixFs`] fails to construct (HTTP client init, runtime),
    /// - `fuser::spawn_mount2` fails (kernel refused the mount, e.g. a
    ///   missing `/dev/fuse` or a stale existing mount at the target).
    ///
    /// # Security
    /// The `allow_other` mount option is intentionally OFF (SG: keep the
    /// mount single-user). `MountOption::AutoUnmount` is also off: fuser
    /// 0.17 refuses `AutoUnmount` when `SessionACL == Owner`, and
    /// broadening ACL to satisfy `AutoUnmount` would violate the
    /// no-allow-other invariant. Unmounting is driven by (a) dropping
    /// this `Mount` struct (fuser's `UmountOnDrop`) and (b) the CLI's
    /// `MountProcess` watchdog (`fusermount3 -u <mount>`) as belt-and-
    /// suspenders.
    pub fn open(cfg: &MountConfig, backend: Arc<dyn IssueBackend>) -> Result<Self> {
        if !cfg.mount_point.exists() {
            std::fs::create_dir_all(&cfg.mount_point)
                .with_context(|| format!("create mount point {}", cfg.mount_point.display()))?;
        }
        let fs = ReposixFs::new(backend, cfg.origin.clone(), cfg.project.clone())?;

        // Phase S: `MountOption::RO` is conditional. When `cfg.read_only` is
        // true we mount RO (the kernel refuses writes at the VFS layer before
        // they reach our callbacks); when false the write path is live.
        let mut options = vec![
            MountOption::FSName("reposix".to_owned()),
            MountOption::Subtype("reposix".to_owned()),
            MountOption::DefaultPermissions,
        ];
        if cfg.read_only {
            options.push(MountOption::RO);
        }
        // `fuser::Config` is `#[non_exhaustive]`, so we can't use a
        // struct-literal update. Start from `default()` and mutate in
        // place.
        let mut config = fuser::Config::default();
        config.mount_options = options;
        let session = fuser::spawn_mount2(fs, &cfg.mount_point, &config)
            .with_context(|| format!("spawn_mount2 at {}", cfg.mount_point.display()))?;
        Ok(Self { _session: session })
    }
}
