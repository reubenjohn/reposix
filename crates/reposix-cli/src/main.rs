//! Top-level `reposix` CLI: orchestrates the simulator, partial-clone
//! init, and refresh flows.
//!
//! Subcommands (v0.9.0):
//! - `reposix sim` — run the Phase-2 simulator as a child process.
//! - `reposix init <backend>::<project> <path>` — initialize a partial-clone
//!   working tree backed by reposix.
//! - `reposix list` — query the backend's `list_records` and dump JSON/table.
//! - `reposix refresh` — re-fetch all issues into a working tree + commit.
//! - `reposix spaces` — list readable Confluence spaces.
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

// All subcommand modules live in `lib.rs` so integration tests can call
// them directly. `main.rs` is intentionally thin: clap-derive dispatch only.
use reposix_cli::{cost, doctor, gc, history, init, list, refresh, sim, spaces, tokens};

/// reposix — git-native partial clone for autonomous agents.
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
        /// Optional RFC-3339 timestamp. After init, rewind the working
        /// tree to the closest cache sync tag at-or-before this timestamp
        /// (time-travel init; v0.11.0 §3b). Errors when no such tag exists.
        ///
        /// Example: `--since=2026-04-25T01:00:00Z`.
        #[arg(long)]
        since: Option<String>,
    },
    /// List issues for a project by calling the backend's `list_records`
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
    /// working-tree directory, and create a git commit so `git diff HEAD~1`
    /// shows backend changes.
    Refresh {
        /// Working-tree directory (a plain directory that is also a git working tree).
        working_tree: PathBuf,
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
    /// Diagnose a reposix-init'd working tree and print copy-pastable fix
    /// commands for each issue found. Exit code is 1 if any ERROR-severity
    /// findings are reported, 0 otherwise.
    ///
    /// With `--fix`, applies the small allowlist of safe fixes inline (today:
    /// `git config extensions.partialClone origin`). Never mutates cache,
    /// audit log, or backend state.
    ///
    /// Examples:
    ///   reposix doctor                    # diagnose current dir
    ///   reposix doctor /tmp/repo
    ///   reposix doctor --fix /tmp/repo    # also apply safe fixes
    Doctor {
        /// Apply deterministic, non-destructive fixes inline.
        #[arg(long)]
        fix: bool,
        /// Working tree to audit. Defaults to the current directory.
        path: Option<PathBuf>,
    },
    /// Print sync-tag history for a `reposix init`'d working tree.
    ///
    /// Every `Cache::sync` writes a private tag under `refs/reposix/sync/`
    /// in the cache's bare repo, pointing at the synthesis commit for that
    /// sync. Listing them gives a fully replayable view of what reposix
    /// observed from the backend over time.
    ///
    /// Defaults to the most recent 10 entries (override with `--limit`).
    /// See `.planning/research/v0.11.0-vision-and-innovations.md` §3b.
    History {
        /// Working-tree directory (a `reposix init`'d repo). Defaults to cwd.
        path: Option<PathBuf>,
        /// Cap on the number of entries printed (most-recent first).
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Time-travel log of cache sync points (alias for `reposix history`
    /// with the `--time-travel` framing from v0.11.0 §3b).
    ///
    /// `reposix log --time-travel` enumerates the `refs/reposix/sync/<ts>`
    /// tags in the cache's bare repo, most-recent first. Default 10 entries
    /// (cap with `--limit N`). Without `--time-travel` the subcommand is
    /// reserved for future commit-graph-style log views.
    Log {
        /// List sync tags in reverse chronological order. Required today —
        /// the bare `reposix log` form is reserved for future commit log
        /// views.
        #[arg(long)]
        time_travel: bool,
        /// Working-tree directory (a `reposix init`'d repo). Defaults to cwd.
        path: Option<PathBuf>,
        /// Cap on the number of entries printed (most-recent first).
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Print the closest sync tag at-or-before a given RFC-3339 timestamp.
    ///
    /// Useful for "what did reposix think the world looked like at <ts>?".
    /// Prints the matching `refs/reposix/sync/<slug>` ref and a copy-pastable
    /// `git -C <cache> checkout` invocation.
    At {
        /// Target timestamp (RFC-3339, e.g. `2026-04-25T01:00:00Z`).
        timestamp: String,
        /// Working-tree directory (a `reposix init`'d repo). Defaults to cwd.
        path: Option<PathBuf>,
    },
    /// Evict materialized blobs from a reposix cache.
    ///
    /// Tree/commit objects, refs, and sync tags are NEVER touched —
    /// only loose blob objects under `.git/objects/<2>/<38>` are
    /// eligible. Blobs re-fetch transparently on next read.
    ///
    /// See `.planning/research/v0.11.0-vision-and-innovations.md` §3j.
    ///
    /// Examples:
    ///   reposix gc                                     # LRU evict to 500 MB cap, current dir
    ///   reposix gc --strategy ttl --max-age-days 7     # evict blobs not touched in a week
    ///   reposix gc --strategy all --dry-run /tmp/repo  # plan, don't execute
    Gc {
        /// Eviction strategy.
        #[arg(long, value_enum, default_value_t = gc::GcStrategyArg::Lru)]
        strategy: gc::GcStrategyArg,
        /// Cap for `--strategy lru` (default 500 MB).
        #[arg(long, default_value_t = 500)]
        max_size_mb: u64,
        /// Cutoff for `--strategy ttl` (default 30 days).
        #[arg(long, default_value_t = 30)]
        max_age_days: i64,
        /// Plan-only: print what would be evicted, don't touch disk.
        #[arg(long)]
        dry_run: bool,
        /// Working-tree directory (a `reposix init`'d repo). Defaults to cwd.
        path: Option<PathBuf>,
    },
    /// Print a token-economy ledger from the cache audit log.
    ///
    /// Aggregates `op='token_cost'` rows (one per helper RPC) and
    /// estimates total token spend. Prints an honest comparison against
    /// a back-of-envelope MCP-equivalent estimate.
    ///
    /// See `.planning/research/v0.11.0-vision-and-innovations.md` §3c.
    Tokens {
        /// Working-tree directory (a `reposix init`'d repo). Defaults to cwd.
        path: Option<PathBuf>,
    },
    /// Print a per-op cost table over the token-cost audit log.
    ///
    /// Aggregates `op='token_cost'` rows since the (optional) `--since`
    /// cutoff, grouped by op kind (`fetch` / `push`). Output is a
    /// pipe-friendly Markdown table with bytes-in, bytes-out, estimated
    /// input tokens, and estimated output tokens, plus a TOTAL row.
    ///
    /// `--since` accepts duration shortcuts (`7d`, `30d`, `1m`, `12h`)
    /// or full RFC-3339 timestamps. Token estimate is `bytes /
    /// chars-per-token`; configurable via `--chars-per-token` (default
    /// 3.5).
    ///
    /// See `.planning/research/v0.11.0-vision-and-innovations.md` §3c.
    Cost {
        /// Working-tree directory (a `reposix init`'d repo). Defaults to cwd.
        path: Option<PathBuf>,
        /// Filter to rows newer than this. Duration (`7d`/`1m`/`12h`) or
        /// RFC-3339 timestamp. Default: include all rows.
        #[arg(long)]
        since: Option<String>,
        /// Heuristic divisor for the token estimate (default 3.5).
        #[arg(long)]
        chars_per_token: Option<f64>,
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
        Cmd::Init { spec, path, since } => init::run_with_since(spec, path, since),
        Cmd::List {
            project,
            origin,
            backend,
            format,
            no_truncate,
        } => list::run(project, origin, backend, format, no_truncate).await,
        Cmd::Refresh {
            working_tree,
            origin,
            project,
            backend,
            offline,
        } => {
            refresh::run_refresh(refresh::RefreshConfig {
                working_tree,
                origin,
                project,
                backend,
                offline,
            })
            .await
        }
        Cmd::Spaces { backend } => spaces::run(backend).await,
        Cmd::History { path, limit } => {
            let p = match path {
                Some(p) => p,
                None => std::env::current_dir()?,
            };
            history::run_history(p, Some(limit))
        }
        Cmd::Log {
            time_travel,
            path,
            limit,
        } => {
            if !time_travel {
                anyhow::bail!(
                    "`reposix log` currently requires `--time-travel`; use `reposix history` for the same listing or wait for a future commit-graph view"
                );
            }
            let p = match path {
                Some(p) => p,
                None => std::env::current_dir()?,
            };
            history::run_history(p, Some(limit))
        }
        Cmd::At { timestamp, path } => {
            let p = match path {
                Some(p) => p,
                None => std::env::current_dir()?,
            };
            history::run_at(timestamp, p)
        }
        Cmd::Doctor { fix, path } => {
            let report = doctor::run(path.as_deref(), fix)?;
            // Colour iff stdout is a TTY. Plain ASCII otherwise so logs and
            // CI artifacts stay clean.
            let colour = std::io::IsTerminal::is_terminal(&std::io::stdout());
            report.print(colour);
            let code = report.exit_code();
            if code != 0 {
                std::process::exit(code);
            }
            Ok(())
        }
        Cmd::Gc {
            strategy,
            max_size_mb,
            max_age_days,
            dry_run,
            path,
        } => gc::run(path, strategy, max_size_mb, max_age_days, dry_run),
        Cmd::Tokens { path } => tokens::run(path),
        Cmd::Cost {
            path,
            since,
            chars_per_token,
        } => cost::run(path, since, chars_per_token),
    }
}
