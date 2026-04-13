//! Reposix simulator — in-process REST API that mimics issue-tracker semantics.
//!
//! Exposes `start(SimConfig) -> JoinHandle` so integration tests can spin a real HTTP server on
//! a random port without forking a process. The standalone `reposix-sim` binary is a thin
//! `tokio::main` wrapper over [`run`].
//!
//! # Status
//! Skeleton. Routes filled in by phase 2.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::net::SocketAddr;

use anyhow::Result;
use axum::Router;
use serde::{Deserialize, Serialize};

/// Runtime configuration for the simulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    /// Bind address. Use `127.0.0.1:0` for a random port (recommended in tests).
    pub bind: SocketAddr,
    /// Path to the `SQLite` audit log file. Created if absent.
    pub db_path: std::path::PathBuf,
    /// Whether to install seed data on first run.
    pub seed: bool,
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
            db_path: std::path::PathBuf::from(":memory:"),
            seed: true,
        }
    }
}

/// Build the axum router. Exposed so integration tests can call it directly without networking.
pub fn build_router() -> Router {
    // TODO(phase-2): mount real routes (projects, issues, transitions, permissions, audit).
    Router::new().route("/healthz", axum::routing::get(healthz))
}

#[allow(clippy::unused_async)]
async fn healthz() -> &'static str {
    "ok"
}

/// Run the simulator until its listener is closed (currently: forever).
///
/// # Errors
/// Returns any I/O error from binding the listener or driving the server.
pub async fn run(cfg: SimConfig) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    tracing::info!(addr = %listener.local_addr()?, "reposix-sim listening");
    axum::serve(listener, build_router()).await?;
    Ok(())
}
