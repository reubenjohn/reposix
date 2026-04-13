//! `reposix-swarm` binary entry point.
//!
//! Starts N concurrent client tasks on a Tokio runtime, each running a
//! realistic workload loop against either the simulator (via
//! [`SimBackend`](reposix_core::backend::sim::SimBackend)) or a mounted FUSE
//! tree. Runs for `--duration` seconds, then prints a markdown summary.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::time::Duration;

use anyhow::Result;
use clap::{Parser, ValueEnum};

/// Mode selector. `sim-direct` hammers the simulator via HTTP; `fuse`
/// hammers a mounted FUSE tree via `std::fs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab_case")]
enum Mode {
    /// HTTP to the simulator via `SimBackend`.
    SimDirect,
    /// Real syscalls against a FUSE mount point.
    Fuse,
}

impl Mode {
    fn as_str(self) -> &'static str {
        match self {
            Self::SimDirect => "sim-direct",
            Self::Fuse => "fuse",
        }
    }
}

/// CLI arguments.
#[derive(Debug, Parser)]
#[command(version, about = "Adversarial swarm harness for reposix.")]
struct Args {
    /// Number of concurrent simulated agents.
    #[arg(long, default_value_t = 10)]
    clients: usize,

    /// Total run duration in seconds.
    #[arg(long, default_value_t = 10)]
    duration: u64,

    /// Target: simulator URL (for `sim-direct`) or FUSE mount path (for
    /// `fuse`).
    #[arg(long, default_value = "http://127.0.0.1:7878")]
    target: String,

    /// Project slug used for `list/get/patch`. Defaults to the demo seed.
    #[arg(long, default_value = "demo")]
    project: String,

    /// Mode — `sim-direct` or `fuse`.
    #[arg(long, value_enum, default_value_t = Mode::SimDirect)]
    mode: Mode,

    /// Optional path to the simulator's audit `SQLite` DB. If provided (and
    /// readable at run end), the summary prints the post-run audit row count
    /// so the invariant assertion can fire.
    #[arg(long)]
    audit_db: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let args = Args::parse();
    let _duration = Duration::from_secs(args.duration);

    // Skeleton: parse args, print a stub summary, exit 0.
    // Subsequent commits wire in:
    //   - workload::Workload implementations,
    //   - the shared MetricsAccumulator,
    //   - the deadline-driven JoinSet loop,
    //   - the markdown summary + audit invariant check.
    println!(
        "reposix-swarm skeleton — clients={} duration={}s mode={} target={} project={}",
        args.clients,
        args.duration,
        args.mode.as_str(),
        args.target,
        args.project,
    );
    if let Some(p) = &args.audit_db {
        println!("audit-db: {}", p.display());
    }
    Ok(())
}
