//! `reposix-fuse` binary entry point.
//!
//! Usage:
//!   reposix-fuse <mount_point> [--backend-kind sim|github|confluence] \
//!                [--backend <origin>] [--project <slug>]
//!
//! The mount is foreground-blocking — Ctrl-C unmounts.
//!
//! `--backend-kind` selects the read-path backend:
//! - `sim` (default): speaks the simulator REST API at `--backend`.
//! - `github`: speaks `https://api.github.com`; `--project` is `owner/repo`.
//!   Requires `REPOSIX_ALLOWED_ORIGINS` to include `https://api.github.com`
//!   and optionally `GITHUB_TOKEN` for the 5000 req/hr ceiling.
//! - `confluence`: speaks `https://<tenant>.atlassian.net`; `--project` is
//!   the Confluence space KEY (e.g. `REPOSIX`). Requires
//!   `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`
//!   env vars and `REPOSIX_ALLOWED_ORIGINS` containing the tenant origin.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Result};
use clap::{Parser, ValueEnum};
use reposix_confluence::{ConfluenceCreds, ConfluenceBackend};
use reposix_core::backend::sim::SimBackend;
use reposix_core::IssueBackend;
use reposix_fuse::{Mount, MountConfig};
use reposix_github::GithubReadOnlyBackend;

/// Read-path backend choice. `sim` preserves the v0.1 default so existing
/// demos and tests are untouched; `github` is the Phase 10 rewire entry point;
/// `confluence` is the Phase 11 read-only Atlassian adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum BackendKind {
    /// In-process reposix simulator at `--backend` (`http://127.0.0.1:7878`).
    Sim,
    /// Real GitHub Issues at `https://api.github.com`. `--project` is
    /// `owner/repo`.
    Github,
    /// Real Atlassian Confluence Cloud REST v2 at
    /// `https://{tenant}.atlassian.net`. `--project` is the Confluence
    /// space KEY (e.g. `REPOSIX`). Requires `ATLASSIAN_API_KEY`,
    /// `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` env vars and an
    /// allowlist that contains the tenant origin.
    Confluence,
}

/// FUSE daemon mounting a reposix backend at a local path.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Mount point (must be an empty directory or non-existent).
    mount_point: PathBuf,

    /// Which backend to speak. `sim` (default) = simulator REST at
    /// `--backend`; `github` = real `api.github.com`.
    #[arg(long = "backend-kind", value_enum, default_value_t = BackendKind::Sim)]
    backend_kind: BackendKind,

    /// Backend origin (e.g. `http://127.0.0.1:7878`). Used by `--backend-kind
    /// sim`. Ignored (but recorded in the mount config) for `--backend-kind
    /// github`. `--origin` kept as an alias for backward compatibility.
    #[arg(
        long,
        alias = "origin",
        env = "REPOSIX_BACKEND",
        default_value = "http://127.0.0.1:7878"
    )]
    backend: String,

    /// Project slug (sim) or `owner/repo` (github). Every issue is
    /// presented as a file.
    #[arg(long, default_value = "demo")]
    project: String,

    /// Read-only mount (forward-compat flag; default stays RW for sim
    /// write-path parity with v0.1).
    #[arg(long)]
    read_only: bool,
}

fn build_backend(kind: BackendKind, origin: &str) -> Result<Arc<dyn IssueBackend>> {
    match kind {
        BackendKind::Sim => {
            // Agent-header suffix "fuse" so the simulator's audit log can
            // distinguish FUSE-driven writes from other SimBackend callers
            // (swarm harness, CLI, remote helper). The emitted header is
            // `reposix-core-simbackend-<pid>-fuse` — see Phase 14 Wave B1.
            let b = SimBackend::with_agent_suffix(origin.to_owned(), Some("fuse"))?;
            Ok(Arc::new(b))
        }
        BackendKind::Github => {
            // Fail fast if the allowlist clearly excludes api.github.com —
            // the actual enforcement happens at request time inside the
            // sealed HttpClient, but a loud CLI error is kinder than an
            // opaque EIO on first readdir.
            let raw = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
            if !raw.contains("api.github.com") {
                bail!(
                    "REPOSIX_ALLOWED_ORIGINS must include https://api.github.com for --backend-kind github (got {:?})",
                    raw
                );
            }
            let token = std::env::var("GITHUB_TOKEN").ok();
            let b = GithubReadOnlyBackend::new(token)?;
            Ok(Arc::new(b))
        }
        BackendKind::Confluence => {
            // Same pattern as github: fail fast with a loud CLI error
            // rather than an opaque EIO once the FUSE mount starts
            // serving reads. The sealed HttpClient re-checks the
            // allowlist before any socket I/O so the substring match here
            // is a courtesy, not a trust boundary.
            let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").unwrap_or_default();
            let email = std::env::var("ATLASSIAN_EMAIL").unwrap_or_default();
            let token = std::env::var("ATLASSIAN_API_KEY").unwrap_or_default();
            if tenant.is_empty() || email.is_empty() || token.is_empty() {
                // Names-only, values never echoed (T-11B-01).
                bail!(
                    "confluence backend requires ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, REPOSIX_CONFLUENCE_TENANT env vars"
                );
            }
            let raw = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
            let expected = format!("{tenant}.atlassian.net");
            if !raw.contains(&expected) {
                bail!(
                    "REPOSIX_ALLOWED_ORIGINS must include https://{expected} for --backend-kind confluence (got {:?})",
                    raw
                );
            }
            let creds = ConfluenceCreds {
                email,
                api_token: token,
            };
            let b = ConfluenceBackend::new(creds, &tenant)?;
            Ok(Arc::new(b))
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let args = Args::parse();
    let backend = build_backend(args.backend_kind, &args.backend)?;
    tracing::info!(
        backend = backend.name(),
        project = %args.project,
        "opening reposix mount"
    );
    let _mount = Mount::open(
        &MountConfig {
            mount_point: args.mount_point,
            origin: args.backend,
            project: args.project,
            read_only: args.read_only,
        },
        backend,
    )?;
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
