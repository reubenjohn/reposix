//! `reposix-sim` binary entry point.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use reposix_sim::{run, SimConfig};

/// In-process REST API simulator for reposix.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Bind address (default 127.0.0.1:7777).
    #[arg(long, default_value = "127.0.0.1:7777")]
    bind: std::net::SocketAddr,

    /// SQLite audit log path. Use `:memory:` for ephemeral.
    #[arg(long, default_value = "runtime/sim.db")]
    db: PathBuf,

    /// Skip seeding demo data.
    #[arg(long)]
    no_seed: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let args = Args::parse();
    if let Some(parent) = args.db.parent() {
        if !parent.as_os_str().is_empty() && parent != std::path::Path::new(":memory:") {
            std::fs::create_dir_all(parent).ok();
        }
    }
    run(SimConfig {
        bind: args.bind,
        db_path: args.db,
        seed: !args.no_seed,
    })
    .await
}
