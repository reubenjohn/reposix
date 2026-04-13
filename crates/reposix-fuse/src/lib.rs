//! FUSE daemon library — see [`Mount`] for the public entry point.
//!
//! The read-only mount presents every issue in a reposix-compatible backend
//! as a single Markdown file at the mount root, named `<zero-padded-id>.md`.
//! All HTTP calls go through the sealed [`reposix_core::http::HttpClient`]
//! (SG-01 allowlist applies), and every fetch is capped at 5 seconds
//! (SG-07) so the kernel cannot hang on a dead backend.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::path::PathBuf;

use anyhow::{Context, Result};
use fuser::{BackgroundSession, MountOption};
use serde::{Deserialize, Serialize};

pub mod fetch;
pub mod fs;
pub mod inode;

pub use fs::ReposixFs;
pub use inode::InodeRegistry;

/// Runtime configuration for a FUSE mount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    /// Where to mount.
    pub mount_point: PathBuf,
    /// Origin of the reposix-compatible backend (e.g. `http://127.0.0.1:7878`).
    pub origin: String,
    /// Project slug. Every issue under this project is presented as a file.
    #[serde(default = "default_project")]
    pub project: String,
    /// Read-only mode. Accepted for forward-compat with Phase S; writes are
    /// always refused in v0.1.
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
    /// Spawn a read-only FUSE mount at `cfg.mount_point` backed by
    /// `cfg.origin`. The mount lives until the returned [`Mount`] is dropped.
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
    pub fn open(cfg: &MountConfig) -> Result<Self> {
        if !cfg.mount_point.exists() {
            std::fs::create_dir_all(&cfg.mount_point)
                .with_context(|| format!("create mount point {}", cfg.mount_point.display()))?;
        }
        let fs = ReposixFs::new(cfg.origin.clone(), cfg.project.clone())?;

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
