//! Axum middleware layers applied to the simulator router.
//!
//! Layer ordering matters (axum wraps: last `.layer()` call is outermost).
//! `build_router` attaches them in the order:
//!
//! ```text
//! handlers
//!     .layer(rate_limit::layer(rps))   // inner — rate limit before work
//!     .layer(audit::layer(state))      // outer — audit even rate-limited reqs
//! ```

pub mod audit;
// pub mod rate_limit; — added in plan 02-02 task 2
