//! Route modules for the simulator.
//!
//! Each submodule exports a `router(state)` function returning a `Router`
//! pre-bound to the shared [`crate::AppState`]. `routes::router` merges them
//! into a single router that the top-level [`crate::build_router`] nests
//! onto the app.

use axum::Router;

use crate::AppState;

pub mod issues;
pub mod transitions;

/// Merge the issue + transitions routers into one.
pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(issues::router(state.clone()))
        .merge(transitions::router(state))
}
