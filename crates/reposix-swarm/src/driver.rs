//! Swarm driver — spawns N `Workload` tasks, runs them for `duration`,
//! collects metrics, and renders the final summary.
//!
//! The driver is a thin loop:
//!   - spawn N tokio tasks, each calling `workload.step()` in a while-loop
//!     until `Instant::now() >= deadline`;
//!   - at `deadline`, every task drops out of its while-loop and awaits
//!     join;
//!   - the driver renders markdown from the shared
//!     [`MetricsAccumulator`](crate::metrics::MetricsAccumulator).

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::task::JoinSet;

use crate::metrics::{MetricsAccumulator, SummaryHeader};
use crate::workload::Workload;

/// Configuration knobs the driver needs.
pub struct SwarmConfig<'a> {
    /// Number of concurrent clients.
    pub clients: usize,
    /// Wall-clock duration to run.
    pub duration: Duration,
    /// Mode label (for the summary header).
    pub mode: &'a str,
    /// Target label (for the summary header).
    pub target: &'a str,
}

/// Run the swarm. `factory` is called once per client, returning the
/// `Workload` that client will step.
///
/// Returns the rendered markdown summary as a `String`.
///
/// # Errors
/// Propagates factory errors (e.g. the first `SimBackend::new` fails).
/// Individual step errors are counted in the metrics, not propagated.
pub async fn run_swarm<F, W>(cfg: SwarmConfig<'_>, factory: F) -> anyhow::Result<String>
where
    F: Fn(usize) -> anyhow::Result<W>,
    W: Workload + 'static,
{
    let metrics = Arc::new(MetricsAccumulator::new());
    let deadline = Instant::now() + cfg.duration;
    let mut set: JoinSet<()> = JoinSet::new();

    for i in 0..cfg.clients {
        let workload = factory(i)?;
        let workload = Arc::new(workload);
        let metrics_c = Arc::clone(&metrics);
        set.spawn(async move {
            while Instant::now() < deadline {
                if let Err(err) = workload.step(&metrics_c).await {
                    // Fatal-per-task: log and stop this client. The other
                    // clients keep running.
                    tracing::warn!(error = %err, "swarm client stopped (fatal step error)");
                    break;
                }
            }
        });
    }

    while set.join_next().await.is_some() {}

    let secs = cfg.duration.as_secs();
    let header = SummaryHeader {
        clients: cfg.clients,
        duration_sec: secs,
        mode: cfg.mode,
        target: cfg.target,
    };
    Ok(metrics.render_markdown(&header))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workload::Workload;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A workload that just sleeps 1ms and records a fake `list` call.
    struct Fake {
        count: Arc<AtomicU64>,
    }

    #[async_trait]
    impl Workload for Fake {
        async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()> {
            tokio::time::sleep(Duration::from_millis(1)).await;
            metrics.record(crate::metrics::OpKind::List, 1_000);
            self.count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }

    #[tokio::test]
    async fn driver_runs_clients_for_duration_and_renders() {
        let count = Arc::new(AtomicU64::new(0));
        let c1 = Arc::clone(&count);
        let md = run_swarm(
            SwarmConfig {
                clients: 4,
                duration: Duration::from_millis(150),
                mode: "test",
                target: "fake",
            },
            |_i| {
                Ok(Fake {
                    count: Arc::clone(&c1),
                })
            },
        )
        .await
        .expect("run");
        assert!(md.contains("Clients: 4"), "{md}");
        // At 1ms per step × 4 clients × 150ms, we should see at least a
        // handful of completed steps. Tolerant threshold to avoid flake.
        let observed = count.load(Ordering::Relaxed);
        assert!(observed >= 4, "expected >=4 steps, got {observed}");
    }
}
