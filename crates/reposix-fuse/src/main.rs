//! `reposix-fuse` binary entry point.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use reposix_fuse::{Mount, MountConfig};

/// FUSE daemon mounting a reposix backend at a local path.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Mount point (must be empty directory or non-existent).
    mount_point: PathBuf,

    /// Backend origin (e.g. http://localhost:7777).
    #[arg(long, env = "REPOSIX_ORIGIN", default_value = "http://localhost:7777")]
    origin: String,

    /// Read-only mount.
    #[arg(long)]
    read_only: bool,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let args = Args::parse();
    let _mount = Mount::open(MountConfig {
        mount_point: args.mount_point,
        origin: args.origin,
        read_only: args.read_only,
    })?;
    tracing::warn!("FUSE mount skeleton — phase 3 wires the real fuser session");
    Ok(())
}
