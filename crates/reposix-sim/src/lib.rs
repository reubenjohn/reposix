//! Reposix simulator — in-process REST API that mimics issue-tracker semantics.
//!
//! Exposes a handful of pure functions so integration tests can spin a real
//! HTTP server on a random port without forking a process. The standalone
//! `reposix-sim` binary is a thin `tokio::main` wrapper over [`run`].
//!
//! # Module layout
//!
//! - [`state`] — [`AppState`] shared across handlers.
//! - [`db`] — `SQLite` connection opener + issues-table DDL.
//! - [`seed`] — deterministic seed loader (reads `fixtures/seed.json`).
//! - [`error`] — [`error::ApiError`] enum + `IntoResponse` impl.
//! - (routes and middleware land in task 2 of plan 02-01 / plan 02-02.)

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;
use axum::Router;
use serde::{Deserialize, Serialize};

pub mod db;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod seed;
pub mod state;

pub use state::AppState;

/// Capability matrix row published by this backend for `reposix doctor`.
///
/// The simulator implements the full reference matrix: read, create, update,
/// delete, comments round-tripped in the body, and strong versioning via the
/// `version` field. Other backends adopt this shape with caveats; the sim is
/// the contract every other connector is benchmarked against.
pub const CAPABILITIES: reposix_core::BackendCapabilities = reposix_core::BackendCapabilities::new(
    true,
    true,
    true,
    true,
    reposix_core::CommentSupport::InBody,
    reposix_core::VersioningModel::Strong,
);

/// Runtime configuration for the simulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    /// Bind address. Use `127.0.0.1:0` for a random port (recommended in tests).
    pub bind: SocketAddr,
    /// Path to the `SQLite` DB file. Created if absent. Use `:memory:` or set
    /// `ephemeral=true` for a transient DB.
    pub db_path: PathBuf,
    /// Whether to install seed data on first run.
    pub seed: bool,
    /// Optional path to the seed JSON. If `None` and `seed=true`, nothing is
    /// seeded — callers pass the fixture path via `--seed-file`.
    #[serde(default)]
    pub seed_file: Option<PathBuf>,
    /// Open DB as `:memory:` regardless of `db_path`.
    #[serde(default)]
    pub ephemeral: bool,
    /// Per-agent rate limit in requests per second. Default 100.
    #[serde(default = "default_rate_limit_rps")]
    pub rate_limit_rps: u32,
}

fn default_rate_limit_rps() -> u32 {
    100
}

impl SimConfig {
    /// Default config for a one-off in-memory simulator.
    ///
    /// # Panics
    /// Never in practice; the bind address is a static, valid `SocketAddr` literal.
    #[must_use]
    pub fn ephemeral() -> Self {
        Self {
            bind: "127.0.0.1:0".parse().expect("static addr parses"),
            db_path: PathBuf::from(":memory:"),
            seed: true,
            seed_file: None,
            ephemeral: true,
            rate_limit_rps: default_rate_limit_rps(),
        }
    }
}

/// Build the axum router with both middleware layers attached.
///
/// Layer ordering (outermost first): **audit → rate-limit → handlers**. Axum
/// `.layer()` wraps inside-out, so the last `.layer()` call is the
/// outermost. That means audit sees every request (including 429s), and
/// rate-limit sees every request that survives the audit recording.
pub fn build_router(state: AppState, rate_limit_rps: u32) -> Router {
    let handlers = Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .merge(routes::router(state.clone()));
    // Attach INNER first (rate-limit), then OUTER (audit).
    let with_rate_limit = middleware::rate_limit::attach(handlers, rate_limit_rps);
    middleware::audit::attach(with_rate_limit, state)
}

#[allow(clippy::unused_async)]
async fn healthz() -> &'static str {
    "ok"
}

/// Open the DB, seed if configured, and return an [`AppState`].
///
/// # Errors
/// Propagates any error from [`db::open_db`] or [`seed::load_seed`].
pub fn prepare_state(cfg: &SimConfig) -> Result<AppState> {
    let conn =
        db::open_db(&cfg.db_path, cfg.ephemeral).map_err(|e| anyhow::anyhow!("open_db: {e}"))?;

    if cfg.seed {
        if let Some(ref path) = cfg.seed_file {
            let inserted =
                seed::load_seed(&conn, path).map_err(|e| anyhow::anyhow!("load_seed: {e}"))?;
            tracing::info!(inserted, path = %path.display(), "seed loaded");
        }
    }

    Ok(AppState::new(conn, cfg.clone()))
}

/// Run the sim on an already-bound listener. Integration tests use this to
/// bind `127.0.0.1:0`, read the ephemeral port, and drive the sim without
/// racing a separate binary.
///
/// # Errors
/// Returns any error from `axum::serve` or state preparation.
pub async fn run_with_listener(listener: tokio::net::TcpListener, cfg: SimConfig) -> Result<()> {
    let state = prepare_state(&cfg)?;
    tracing::info!(addr = %listener.local_addr()?, "reposix-sim listening");
    axum::serve(listener, build_router(state, cfg.rate_limit_rps)).await?;
    Ok(())
}

/// Bind the configured address and serve until the listener dies.
///
/// # Errors
/// Returns any I/O error from binding the listener, opening the DB, or
/// driving the server.
pub async fn run(cfg: SimConfig) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    run_with_listener(listener, cfg).await
}
