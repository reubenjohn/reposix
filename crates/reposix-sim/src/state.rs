//! Shared application state for the simulator's axum handlers.
//!
//! Holds the single `SQLite` [`rusqlite::Connection`] behind a
//! [`parking_lot::Mutex`] (the whole connection is the critical section — we
//! never hold the lock across `.await`), plus an [`Arc<SimConfig>`] so every
//! request can see the bind address / rate-limit settings without a global.

use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::Connection;

use crate::SimConfig;

/// Application state shared with every axum handler and middleware.
///
/// Cloning an [`AppState`] is cheap: it is two `Arc` bumps. Handlers receive
/// it via `axum::extract::State(AppState)` thanks to `Router::with_state`.
#[derive(Clone)]
pub struct AppState {
    /// Single global `SQLite` connection. Lock scope is always synchronous
    /// (never across `.await`) so the mutex never stalls the tokio runtime.
    pub db: Arc<Mutex<Connection>>,
    /// Runtime configuration, shared read-only.
    pub config: Arc<SimConfig>,
}

impl AppState {
    /// Construct a new state from a ready-to-use connection and config.
    #[must_use]
    pub fn new(conn: Connection, config: SimConfig) -> Self {
        Self {
            db: Arc::new(Mutex::new(conn)),
            config: Arc::new(config),
        }
    }
}
