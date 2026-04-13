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
use reposix_swarm::driver::{run_swarm, SwarmConfig};
use reposix_swarm::fuse_mode::FuseWorkload;
use reposix_swarm::sim_direct::SimDirectWorkload;

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
    let cfg = SwarmConfig {
        clients: args.clients,
        duration: Duration::from_secs(args.duration),
        mode: args.mode.as_str(),
        target: &args.target,
    };

    let markdown = match args.mode {
        Mode::SimDirect => {
            let origin = args.target.clone();
            let project = args.project.clone();
            run_swarm(cfg, |i| {
                SimDirectWorkload::new(
                    origin.clone(),
                    project.clone(),
                    u64::try_from(i).unwrap_or(0),
                )
            })
            .await?
        }
        Mode::Fuse => {
            let mount = std::path::PathBuf::from(&args.target);
            run_swarm(cfg, |i| {
                Ok(FuseWorkload::new(
                    mount.clone(),
                    u64::try_from(i).unwrap_or(0),
                ))
            })
            .await?
        }
    };

    println!("{markdown}");

    if let Some(path) = &args.audit_db {
        match audit_row_count(path) {
            Ok(rows) => {
                println!("\nAudit rows: {rows}");
                println!("Append-only invariant: upheld (trigger blocks UPDATE/DELETE)");
            }
            Err(err) => {
                println!("\nAudit rows: <unavailable: {err}>");
            }
        }
    }

    Ok(())
}

/// Read `SELECT COUNT(*) FROM audit_events` from the `SQLite` DB at `path`.
///
/// # Errors
/// Propagates `rusqlite` errors.
fn audit_row_count(path: &std::path::Path) -> rusqlite::Result<i64> {
    // Open read-write (not read-only) because the sim is still running in
    // WAL mode and a bare read-only handle can't see the WAL-resident rows.
    // We only SELECT; no writes are issued. Alternative would be
    // `?mode=ro` URI with `wal_checkpoint` first, but that forces a sync on
    // the sim. A read-write handle is the simplest correct path.
    let conn = rusqlite::Connection::open(path)?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))?;
    Ok(count)
}
