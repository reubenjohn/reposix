//! `reposix-sim` binary entry point.

#![forbid(unsafe_code)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use reposix_sim::{run, SimConfig};

/// In-process REST API simulator for reposix.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Bind address.
    #[arg(long, default_value = "127.0.0.1:7878")]
    bind: std::net::SocketAddr,

    /// SQLite DB path. Ignored when `--ephemeral` is set.
    #[arg(long, default_value = "runtime/sim.db")]
    db: PathBuf,

    /// Optional seed JSON file (e.g. `crates/reposix-sim/fixtures/seed.json`).
    #[arg(long)]
    seed_file: Option<PathBuf>,

    /// Skip seeding demo data even if `--seed-file` is given.
    #[arg(long)]
    no_seed: bool,

    /// Use an in-memory DB, ignoring `--db`.
    #[arg(long)]
    ephemeral: bool,

    /// Per-agent rate limit (requests per second). Default 100.
    #[arg(long, default_value_t = 100)]
    rate_limit: u32,
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
    if !args.ephemeral {
        if let Some(parent) = args.db.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).ok();
            }
        }
    }
    // `run` returns `reposix_sim::SimError`; the `?` adapts it into
    // `anyhow::Error` via `From<E: std::error::Error + Send + Sync + 'static>`.
    run(SimConfig {
        bind: args.bind,
        db_path: args.db,
        seed: !args.no_seed,
        seed_file: args.seed_file,
        ephemeral: args.ephemeral,
        rate_limit_rps: args.rate_limit,
    })
    .await?;
    Ok(())
}
