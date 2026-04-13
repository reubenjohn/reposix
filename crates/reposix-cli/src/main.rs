//! Top-level `reposix` CLI: orchestrates the simulator, FUSE mount, and demo flows.

use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

/// reposix — git-backed FUSE filesystem for autonomous agents.
#[derive(Debug, Parser)]
#[command(name = "reposix", version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Print the version of every reposix binary on PATH.
    Version,
    /// Run the in-process simulator (delegates to `reposix-sim`).
    Sim {
        /// Bind address.
        #[arg(long, default_value = "127.0.0.1:7777")]
        bind: std::net::SocketAddr,
    },
    /// Mount the FUSE filesystem (delegates to `reposix-fuse`).
    Mount {
        /// Mount point.
        mount_point: PathBuf,
        /// Backend origin.
        #[arg(long, default_value = "http://localhost:7777")]
        origin: String,
    },
    /// Run the canonical end-to-end demo: spin sim, mount, edit, push, observe.
    Demo,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Version => {
            println!("reposix {}", env!("CARGO_PKG_VERSION"));
        }
        Cmd::Sim { bind: _ } | Cmd::Mount { .. } | Cmd::Demo => {
            bail!("not yet implemented — phases 2-5 wire these up");
        }
    }
    Ok(())
}
