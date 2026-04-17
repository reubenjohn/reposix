//! `sim-direct` workload: each client hits the simulator via HTTP
//! ([`SimBackend`]) concurrently.
//!
//! Workload per cycle:
//!   1 × `list_issues`  (warm up / discover ids)
//!   3 × `get_issue`    (random ids from the discovered set)
//!   1 × `update_issue` (random id, patch status to `in_progress` with
//!                       wildcard etag — stays idempotent even under races)
//!
//! This is deliberately the "cheapest realistic agent" — it matches what an
//! LLM agent would do in one turn (browse list, open a few, touch one).

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use parking_lot::Mutex;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use reposix_core::backend::sim::SimBackend;
use reposix_core::{
    sanitize, BackendConnector, Issue, IssueId, IssueStatus, ServerMetadata, Tainted,
};

// chrono is a transitive dep via reposix-core (Issue.created_at uses it).
use chrono::Utc;

use crate::metrics::{ErrorKind, MetricsAccumulator, OpKind};
use crate::workload::Workload;

/// A sim-direct workload instance. Holds a shared [`SimBackend`] clone and
/// a per-client RNG.
pub struct SimDirectWorkload {
    backend: SimBackend,
    project: String,
    rng: Mutex<StdRng>,
    /// Cached ids from the first `list_issues` call; workload reads/patches
    /// from this set to keep requests warm. Refreshed on every step's
    /// list call.
    ids: Mutex<Vec<IssueId>>,
}

impl SimDirectWorkload {
    /// Build a new instance. `seed` is used for per-client determinism.
    ///
    /// # Errors
    /// Propagates [`SimBackend::new`] failures (e.g. allowlist build
    /// errors from [`reposix_core::http::client`]).
    pub fn new(origin: String, project: String, seed: u64) -> anyhow::Result<Self> {
        // Per-client agent suffix so the simulator's per-agent rate-limit
        // buckets don't all collide into one. This is the "realistic
        // swarm" property: each simulated agent has its own identity.
        let suffix = format!("swarm-{seed}");
        let backend = SimBackend::with_agent_suffix(origin, Some(&suffix))?;
        Ok(Self {
            backend,
            project,
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
            ids: Mutex::new(Vec::new()),
        })
    }

    /// Pick a random id from the cached set. Returns `None` if the set is
    /// empty (e.g. first iteration before any list succeeded).
    fn random_id(&self) -> Option<IssueId> {
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
impl Workload for SimDirectWorkload {
    async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()> {
        // 1. list
        let start = Instant::now();
        match self.backend.list_issues(&self.project).await {
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

        // 2. 3 × get
        for _ in 0..3 {
            let Some(id) = self.random_id() else {
                break;
            };
            let start = Instant::now();
            match self.backend.get_issue(&self.project, id).await {
                Ok(_issue) => {
                    metrics.record(OpKind::Get, elapsed_us(start));
                }
                Err(err) => {
                    metrics.record(OpKind::Get, elapsed_us(start));
                    metrics.record_error(ErrorKind::classify(&err));
                }
            }
        }

        // 3. 1 × patch
        if let Some(id) = self.random_id() {
            // Build a trivial patch: flip status to `in_progress` with a
            // wildcard etag (expected_version = None) so we don't storm
            // 409s when two clients race on the same id.
            let now = Utc::now();
            let issue = Issue {
                id,
                title: "swarm-patched".to_string(),
                status: IssueStatus::InProgress,
                assignee: None,
                labels: vec!["swarm".to_string()],
                created_at: now,
                updated_at: now,
                version: 0,
                body: "patched by swarm".to_string(),
                parent_id: None,
                extensions: std::collections::BTreeMap::new(),
            };
            let untainted = sanitize(
                Tainted::new(issue),
                ServerMetadata {
                    id,
                    created_at: now,
                    updated_at: now,
                    version: 0,
                },
            );
            let start = Instant::now();
            match self
                .backend
                .update_issue(&self.project, id, untainted, None)
                .await
            {
                Ok(_) => {
                    metrics.record(OpKind::Patch, elapsed_us(start));
                }
                Err(err) => {
                    metrics.record(OpKind::Patch, elapsed_us(start));
                    metrics.record_error(ErrorKind::classify(&err));
                }
            }
        }
        Ok(())
    }
}

/// Elapsed-microseconds helper. Saturates at `u64::MAX` for any duration
/// that somehow overflows (practically impossible for a swarm run).
fn elapsed_us(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}
