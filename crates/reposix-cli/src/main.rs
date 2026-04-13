//! Top-level `reposix` CLI: orchestrates the simulator, FUSE mount, and
//! demo flows.
//!
//! Subcommands:
//! - `reposix sim` — run the Phase-2 simulator as a child process.
//! - `reposix mount <dir> --backend <origin> --project <slug>` — mount
//!   the FUSE daemon at `<dir>`.
//! - `reposix demo` — end-to-end orchestration (sim → mount → ls/cat/grep
//!   → audit tail → cleanup).
//! - `reposix version` — print the version.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
// Pass-by-value in `run` shims matches the clap-derive call sites cleanly
// and means we don't have to juggle reference lifetimes in the top-level
// dispatcher. Not load-bearing performance.
#![allow(clippy::needless_pass_by_value)]

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod binpath;
mod demo;
mod list;
mod mount;
mod sim;

/// reposix — git-backed FUSE filesystem for autonomous agents.
#[derive(Debug, Parser)]
#[command(name = "reposix", version, about, subcommand_required = true)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Run the Phase-2 REST simulator (delegates to `reposix-sim`).
    Sim {
        /// Bind address.
        #[arg(long, default_value = "127.0.0.1:7878")]
        bind: String,
        /// `SQLite` audit DB path. Ignored when `--ephemeral` is set.
        #[arg(long, default_value = "runtime/sim.db")]
        db: PathBuf,
        /// Optional seed JSON file.
        #[arg(long)]
        seed_file: Option<PathBuf>,
        /// Skip seeding demo data.
        #[arg(long)]
        no_seed: bool,
        /// Use an in-memory DB.
        #[arg(long)]
        ephemeral: bool,
        /// Per-agent rate limit (requests per second).
        #[arg(long, default_value_t = 100)]
        rate_limit: u32,
    },
    /// Mount the FUSE filesystem (delegates to `reposix-fuse`).
    ///
    /// `--backend sim` (default) speaks the reposix simulator's REST shape
    /// at `--origin`. `--backend github` mounts real `api.github.com`
    /// issues for `--project owner/repo`; requires
    /// `REPOSIX_ALLOWED_ORIGINS` to include `https://api.github.com` and
    /// optionally `GITHUB_TOKEN` for the 5000 req/hr ceiling.
    Mount {
        /// Mount point.
        mount_point: PathBuf,
        /// Simulator REST origin. Used when `--backend sim` (default);
        /// ignored for `--backend github`.
        #[arg(long, default_value = "http://127.0.0.1:7878")]
        origin: String,
        /// Project slug (sim) or `owner/repo` (github).
        #[arg(long, default_value = "demo")]
        project: String,
        /// Which backend to speak. `sim` keeps v0.1 behaviour; `github`
        /// mounts real GitHub Issues end-to-end.
        #[arg(long, value_enum, default_value_t = list::ListBackend::Sim)]
        backend: list::ListBackend,
        /// Read-only flag (forward-compat; v0.1 sim mount is always
        /// read-write, github is forced read-only).
        #[arg(long)]
        read_only: bool,
    },
    /// Run the canonical end-to-end demo: spawn sim → mount → run
    /// scripted ls/cat/grep → tail audit log → clean up.
    Demo {
        /// Stay up after scripted steps until Ctrl-C — useful for
        /// asciinema recording where the human narrates.
        #[arg(long)]
        keep_running: bool,
    },
    /// List issues for a project by calling the backend's `list_issues`
    /// method and dumping the result as JSON (default) or a pretty table.
    ///
    /// `--backend sim` (default) hits the in-process simulator at `--origin`.
    /// `--backend github` hits real `https://api.github.com` for the public
    /// repo named by `--project` (e.g. `octocat/Hello-World`); requires
    /// `REPOSIX_ALLOWED_ORIGINS` to include `https://api.github.com` and
    /// optionally `GITHUB_TOKEN` for the 1000 req/hr ceiling.
    List {
        /// Project slug (sim) or `owner/repo` (github).
        #[arg(long, default_value = "demo")]
        project: String,
        /// Sim backend origin. Ignored for `--backend github`.
        #[arg(long, default_value = "http://127.0.0.1:7878")]
        origin: String,
        /// Which backend to query.
        #[arg(long, value_enum, default_value_t = list::ListBackend::Sim)]
        backend: list::ListBackend,
        /// Output format.
        #[arg(long, value_enum, default_value_t = list::ListFormat::Json)]
        format: list::ListFormat,
    },
    /// Print the version.
    Version,
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
            Ok(())
        }
        Cmd::Sim {
            bind,
            db,
            seed_file,
            no_seed,
            ephemeral,
            rate_limit,
        } => sim::run(&bind, db, seed_file, no_seed, ephemeral, rate_limit),
        Cmd::Mount {
            mount_point,
            origin,
            project,
            backend,
            read_only: _,
        } => mount::run(mount_point, origin, project, backend),
        Cmd::Demo { keep_running } => demo::run(keep_running).await,
        Cmd::List {
            project,
            origin,
            backend,
            format,
        } => list::run(project, origin, backend, format).await,
    }
}
