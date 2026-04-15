//! `contention` workload: N clients hammer the same issue via explicit
//! `If-Match` versions, proving the simulator's 409 path is deterministic.
//!
//! Workload per cycle:
//!   1 × `get_issue`    (read current version)
//!   1 × `update_issue` (PATCH with `Some(version)` — races other clients)
//!
//! Every client races to write to the same issue with an explicit expected
//! version. One client wins per version slot; the rest receive 409 Conflict.
//! This proves:
//! - The sim's If-Match guard actually fires under concurrent load.
//! - No torn writes: every winning PATCH increments the version by exactly 1.
//! - `win_count == final_version - starting_version` (no silent drops).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use chrono::Utc;
use parking_lot::Mutex;
use rand::SeedableRng;
use rand::rngs::StdRng;
use reposix_core::backend::sim::SimBackend;
use reposix_core::{sanitize, IssueBackend, IssueId, ServerMetadata, Tainted};

use crate::metrics::{ErrorKind, MetricsAccumulator, OpKind};
use crate::workload::Workload;

/// Workload where N clients hammer the same issue with explicit `If-Match`
/// versions. One winner per version; the rest must 409. Proves the
/// simulator's version-gating path is deterministic under load.
pub struct ContentionWorkload {
    backend: SimBackend,
    project: String,
    target_id: IssueId,
    /// Reserved for future jitter between GET and PATCH; not used in step() today.
    #[allow(dead_code)]
    rng: Mutex<StdRng>,
}

impl ContentionWorkload {
    /// Build a new instance. Each client gets its own `SimBackend` with a
    /// unique agent suffix so the simulator's per-agent rate-limit buckets
    /// don't collide. `seed` provides per-client determinism.
    ///
    /// # Errors
    /// Returns an error if `SimBackend::with_agent_suffix` fails (bad origin
    /// URL, allowlist build error, etc.).
    pub fn new(
        origin: String,
        project: String,
        target_id: IssueId,
        seed: u64,
    ) -> anyhow::Result<Self> {
        let suffix = format!("contention-{seed}");
        let backend = SimBackend::with_agent_suffix(origin, Some(&suffix))?;
        Ok(Self {
            backend,
            project,
            target_id,
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
        })
    }
}

#[async_trait]
impl Workload for ContentionWorkload {
    async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()> {
        // 1. GET live version (unsynchronised across clients → intentionally racy).
        let get_start = Instant::now();
        let issue = match self.backend.get_issue(&self.project, self.target_id).await {
            Ok(i) => {
                metrics.record(OpKind::Get, elapsed_us(get_start));
                i
            }
            Err(e) => {
                metrics.record(OpKind::Get, elapsed_us(get_start));
                metrics.record_error(ErrorKind::classify(&e));
                return Ok(()); // non-fatal; try next step
            }
        };
        let current_version = issue.version;

        // 2. Build mutation (append version stamp to title → small,
        // deterministic, bytes-in-bytes-out).
        let now = Utc::now();
        let mut patched = issue.clone();
        patched.title = format!("{} [c{}]", patched.title, current_version);
        patched.updated_at = now;

        let untainted = sanitize(
            Tainted::new(patched),
            ServerMetadata {
                id: self.target_id,
                created_at: issue.created_at,
                updated_at: now,
                version: current_version,
            },
        );

        // 3. PATCH with If-Match: <current_version> — explicit version
        //    triggers 409 when another client already incremented the version.
        let patch_start = Instant::now();
        match self
            .backend
            .update_issue(
                &self.project,
                self.target_id,
                untainted,
                Some(current_version),
            )
            .await
        {
            Ok(_) => {
                // Win: we incremented the version.
                metrics.record(OpKind::Patch, elapsed_us(patch_start));
            }
            Err(err) => {
                // 409 Conflict is the expected outcome for losing clients.
                metrics.record(OpKind::Patch, elapsed_us(patch_start));
                metrics.record_error(ErrorKind::classify(&err));
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
