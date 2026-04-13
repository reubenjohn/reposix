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
pub mod routes;
pub mod seed;
pub mod state;

pub use state::AppState;

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

/// Build the axum router.
///
/// Task 2 of plan 02-01 wires the issue/transitions routes through the shared
/// [`AppState`]. Middleware (audit + rate-limit) is attached on top by plan
/// 02-02.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .merge(routes::router(state))
}

#[allow(clippy::unused_async)]
async fn healthz() -> &'static str {
    "ok"
}

/// Run the simulator until its listener is closed (currently: forever).
///
/// Task 1 of plan 02-01 only wires up DB open + seed; HTTP serve still binds
/// `build_router` without real handlers. Task 2 replaces this body.
///
/// # Errors
/// Returns any I/O error from binding the listener, opening the DB, or
/// driving the server.
pub async fn run(cfg: SimConfig) -> Result<()> {
    let conn =
        db::open_db(&cfg.db_path, cfg.ephemeral).map_err(|e| anyhow::anyhow!("open_db: {e}"))?;

    if cfg.seed {
        if let Some(ref path) = cfg.seed_file {
            let inserted =
                seed::load_seed(&conn, path).map_err(|e| anyhow::anyhow!("load_seed: {e}"))?;
            tracing::info!(inserted, path = %path.display(), "seed loaded");
        }
    }

    let state = AppState::new(conn, cfg.clone());

    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    tracing::info!(addr = %listener.local_addr()?, "reposix-sim listening");
    axum::serve(listener, build_router(state)).await?;
    Ok(())
}
