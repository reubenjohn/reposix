//! Workload loop abstraction.
//!
//! A [`Workload`] represents "one agent" — it is looped by the swarm driver
//! until the shared deadline elapses. Each iteration performs a realistic
//! mix of `list + 3 reads + 1 patch` on the chosen target, recording
//! per-operation latencies and error classes in the shared
//! [`MetricsAccumulator`](crate::metrics::MetricsAccumulator).
//!
//! Concrete implementations live in [`crate::sim_direct`] and
//! [`crate::fuse_mode`] (the latter behind a cargo-time compile guard if
//! FUSE isn't available — v0.1 of the swarm only ships `sim-direct`).

use std::sync::Arc;

use async_trait::async_trait;

use crate::metrics::MetricsAccumulator;

/// A "simulated agent" workload. Implementors define a single
/// [`Workload::step`] that performs one workload cycle and records metrics
/// into `metrics`.
#[async_trait]
pub trait Workload: Send + Sync {
    /// Perform one workload cycle (list + reads + patch) and record the
    /// per-operation latencies. Returns `Ok(())` on a fully-completed cycle
    /// (even if individual ops returned errors — those are counted in
    /// metrics). Returns `Err` only for fatal conditions that should stop
    /// the client task entirely.
    ///
    /// # Errors
    /// Fatal errors (e.g. the simulator crashed permanently) surface as
    /// `anyhow::Error` so the driver can decide whether to stop the whole
    /// swarm.
    async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()>;
}
