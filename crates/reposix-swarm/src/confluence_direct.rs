//! `confluence-direct` workload: each client drives [`ConfluenceBackend`]
//! directly over HTTP. Mirror of `sim_direct.rs` minus the patch step
//! (Phase 17 is read-only by design — writes ship in Phase 21 / OP-7).
//!
//! Rate-limit handling is transparent: [`ConfluenceBackend`]'s internal
//! `rate_limit_gate` sleeps on 429 Retry-After; the workload records a
//! `RateLimited` error only if the backend surfaces one.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use parking_lot::Mutex;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
use reposix_core::{BackendConnector, RecordId};

use crate::metrics::{ErrorKind, MetricsAccumulator, OpKind};
use crate::workload::Workload;

/// A confluence-direct workload instance. Holds a per-client
/// [`ConfluenceBackend`] (so each swarm client has its own rate-limit
/// gate) and a per-client RNG.
pub struct ConfluenceDirectWorkload {
    backend: ConfluenceBackend,
    space: String,
    rng: Mutex<StdRng>,
    /// Cached ids from the most recent `list_records` call.
    ids: Mutex<Vec<RecordId>>,
}

impl ConfluenceDirectWorkload {
    /// Build a new instance.
    ///
    /// `base_url` is the Confluence tenant base (e.g.
    /// `https://tenant.atlassian.net`) or a wiremock URI for tests.
    /// `space` is the Confluence space key used as the `project`
    /// argument to [`BackendConnector::list_records`].
    ///
    /// # Errors
    /// Propagates [`ConfluenceBackend::new_with_base_url`] failures
    /// (allowlist build errors, invalid base URL).
    pub fn new(
        base_url: String,
        creds: ConfluenceCreds,
        space: String,
        seed: u64,
    ) -> anyhow::Result<Self> {
        let backend = ConfluenceBackend::new_with_base_url(creds, base_url)
            .map_err(|e| anyhow::anyhow!("ConfluenceBackend init: {e}"))?;
        Ok(Self {
            backend,
            space,
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
            ids: Mutex::new(Vec::new()),
        })
    }

    fn random_id(&self) -> Option<RecordId> {
        let ids = self.ids.lock();
        if ids.is_empty() {
            return None;
        }
        let mut r = self.rng.lock();
        let idx = r.gen_range(0..ids.len());
        Some(ids[idx])
    }
}

#[async_trait]
impl Workload for ConfluenceDirectWorkload {
    async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()> {
        // 1. list
        let start = Instant::now();
        match self.backend.list_records(&self.space).await {
            Ok(issues) => {
                metrics.record(OpKind::List, elapsed_us(start));
                let mut g = self.ids.lock();
                g.clear();
                g.extend(issues.iter().map(|i| i.id));
            }
            Err(err) => {
                metrics.record(OpKind::List, elapsed_us(start));
                metrics.record_error(ErrorKind::classify(&err));
            }
        }

        // 2. 3 × get (random ids; break early if list never populated
        //    the cache)
        for _ in 0..3 {
            let Some(id) = self.random_id() else {
                break;
            };
            let start = Instant::now();
            match self.backend.get_record(&self.space, id).await {
                Ok(_issue) => {
                    metrics.record(OpKind::Get, elapsed_us(start));
                }
                Err(err) => {
                    metrics.record(OpKind::Get, elapsed_us(start));
                    metrics.record_error(ErrorKind::classify(&err));
                }
            }
        }
        // NOTE: no patch step in Phase 17 (read-only; writes in Phase 21).
        Ok(())
    }
}

/// Elapsed-microseconds helper. Saturates at `u64::MAX` for any duration
/// that somehow overflows (practically impossible for a swarm run).
fn elapsed_us(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}
