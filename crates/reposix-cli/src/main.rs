//! Top-level `reposix` CLI: orchestrates the simulator, partial-clone
//! init, and demo flows.
//!
//! Subcommands (v0.9.0):
//! - `reposix sim` — run the Phase-2 simulator as a child process.
//! - `reposix init <backend>::<project> <path>` — initialize a partial-clone
//!   working tree backed by reposix.
//! - `reposix mount` — REMOVED in v0.9.0 (stub emits migration error).
//! - `reposix demo` — end-to-end orchestration against the simulator.
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
mod init;
mod mount;
mod sim;

// Modules shared with the lib target — imported via the library crate path.
use reposix_cli::list;
use reposix_cli::refresh;
use reposix_cli::spaces;

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
    /// Initialize a partial-clone working tree backed by reposix.
    ///
    /// Replaces the deleted `reposix mount` subcommand. Runs `git init`,
    /// configures `extensions.partialClone=origin`, sets
    /// `remote.origin.url=reposix::<scheme>://<host>/projects/<project>`,
    /// and runs `git fetch --filter=blob:none origin` (best-effort).
    ///
    /// Spec form: `<backend>::<project>` where `<backend>` is one of
    /// `sim`, `github`, `confluence`, `jira`. Examples:
    ///   reposix init `sim::demo` /tmp/repo
    ///   reposix init `github::reubenjohn/reposix` /tmp/issues
    ///   reposix init `confluence::TokenWorld` /tmp/space
    ///   reposix init `jira::TEST` /tmp/jira
    Init {
        /// Backend + project spec, e.g. `sim::demo`.
        #[arg(value_name = "BACKEND::PROJECT")]
        spec: String,
        /// Local path to initialize as the partial-clone working tree.
        path: PathBuf,
    },
    /// REMOVED in v0.9.0 — kept as a stub that emits a migration error so
    /// stale scripts get a clear message rather than "unknown subcommand".
    /// The fuse spawn code remains in `mount.rs` until Phase 36 deletes
    /// the FUSE crate entirely.
    Mount {
        /// Path argument retained for backward compat with stale scripts;
        /// ignored — the command always errors out.
        #[arg(value_name = "PATH")]
        mount_point: Option<PathBuf>,
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
        /// Error instead of silently capping at 500 pages (Confluence only).
        /// No-op for --backend sim and --backend github.
        #[arg(long)]
        no_truncate: bool,
    },
    /// Re-fetch all issues/pages from the backend, write `.md` files into the
    /// mount directory, and create a git commit so `git diff HEAD~1` shows
    /// backend changes.
    ///
    /// The mount must NOT be actively FUSE-mounted (unmount first).
    Refresh {
        /// Mount point (a plain directory that is also a git working tree).
        mount_point: PathBuf,
        /// Backend origin (simulator URL).
        #[arg(long, default_value = "http://127.0.0.1:7878")]
        origin: String,
        /// Project slug (sim) or `owner/repo` (github) or space KEY (confluence).
        #[arg(long, default_value = "demo")]
        project: String,
        /// Which backend to speak.
        #[arg(long, value_enum, default_value_t = list::ListBackend::Sim)]
        backend: list::ListBackend,
        /// Serve from cached `.md` files; no network egress.
        /// NOTE: offline read path is Phase 21; this flag is accepted but
        /// currently returns an error.
        #[arg(long)]
        offline: bool,
    },
    /// List all readable Confluence spaces as a table of KEY / NAME / URL.
    ///
    /// Only `--backend confluence` is supported (sim + github have no space
    /// concept). Requires `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`,
    /// `REPOSIX_CONFLUENCE_TENANT` env vars plus `REPOSIX_ALLOWED_ORIGINS`
    /// including the tenant origin. Output is pipe-friendly fixed-width text.
    Spaces {
        /// Which backend to query. Only `confluence` is currently supported.
        #[arg(long, value_enum, default_value_t = list::ListBackend::Confluence)]
        backend: list::ListBackend,
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
        Cmd::Init { spec, path } => init::run(spec, path),
        Cmd::Mount { mount_point: _ } => {
            // v0.9.0 breaking: `reposix mount` is removed. The clap variant
            // is kept only so stale scripts get a clear migration message
            // rather than `error: unrecognized subcommand 'mount'`.
            anyhow::bail!(
                "reposix mount has been removed in v0.9.0. Use 'reposix init <backend>::<project> <path>' — see CHANGELOG and docs/reference/testing-targets.md."
            );
        }
        Cmd::Demo { keep_running } => demo::run(keep_running).await,
        Cmd::List {
            project,
            origin,
            backend,
            format,
            no_truncate,
        } => list::run(project, origin, backend, format, no_truncate).await,
        Cmd::Refresh {
            mount_point,
            origin,
            project,
            backend,
            offline,
        } => {
            refresh::run_refresh(refresh::RefreshConfig {
                mount_point,
                origin,
                project,
                backend,
                offline,
            })
            .await
        }
        Cmd::Spaces { backend } => spaces::run(backend).await,
    }
}
