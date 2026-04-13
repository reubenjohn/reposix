//! FUSE daemon library.
//!
//! Task 1 scope: inode registry + fetch helpers. The [`ReposixFs`]
//! implementation and [`Mount`] public entry point land in Task 2.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod fetch;
pub mod inode;

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

/// Placeholder mount handle. Filled in by Task 2.
#[derive(Debug)]
pub struct Mount {
    _cfg: MountConfig,
}

impl Mount {
    /// Open a mount. Task 1 keeps the old skeleton signature; Task 2 wires
    /// the real fuser `BackgroundSession`.
    ///
    /// # Errors
    /// Returns any I/O error from validating the mount point.
    pub fn open(cfg: MountConfig) -> Result<Self> {
        if !cfg.mount_point.exists() {
            std::fs::create_dir_all(&cfg.mount_point)?;
        }
        Ok(Self { _cfg: cfg })
    }
}
