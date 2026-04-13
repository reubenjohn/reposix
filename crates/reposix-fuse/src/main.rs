//! `reposix-fuse` binary entry point.
//!
//! Usage: `reposix-fuse <mount_point> --backend <origin> [--project <slug>]`.
//! The mount is foreground-blocking — Ctrl-C unmounts.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use reposix_fuse::{Mount, MountConfig};

/// FUSE daemon mounting a reposix backend at a local path.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Mount point (must be an empty directory or non-existent).
    mount_point: PathBuf,

    /// Backend origin (e.g. `http://127.0.0.1:7878`). `--origin` kept as an
    /// alias for backward compatibility.
    #[arg(
        long,
        alias = "origin",
        env = "REPOSIX_BACKEND",
        default_value = "http://127.0.0.1:7878"
    )]
    backend: String,

    /// Project slug. Every issue under this project is presented as a file.
    #[arg(long, default_value = "demo")]
    project: String,

    /// Read-only mount (forward-compat flag; v0.1 is always read-only).
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
    let _mount = Mount::open(&MountConfig {
        mount_point: args.mount_point,
        origin: args.backend,
        project: args.project,
        read_only: args.read_only,
    })?;
    // Block until SIGINT — drop on signal cleans up via fuser's UmountOnDrop.
    tracing::info!("reposix-fuse mounted; press Ctrl-C to unmount");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let _ = tokio::signal::ctrl_c().await;
    });
    tracing::info!("unmounting");
    Ok(())
}
