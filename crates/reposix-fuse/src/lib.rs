//! FUSE daemon library — see [`Mount`] for the public entry point.
//!
//! # Status
//! Skeleton. Implementation lands in phase 3.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Runtime configuration for a FUSE mount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    /// Where to mount.
    pub mount_point: PathBuf,
    /// Origin of the reposix-compatible backend (e.g. `http://localhost:7777`).
    pub origin: String,
    /// Read-only mode. Even if the backend permits writes, refuse them locally.
    pub read_only: bool,
}

/// Placeholder mount handle. Phase 3 fills this in with a real fuser background session.
#[derive(Debug)]
pub struct Mount {
    _cfg: MountConfig,
}

impl Mount {
    /// Open a mount. Currently returns a placeholder; phase 3 will wire fuser.
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
