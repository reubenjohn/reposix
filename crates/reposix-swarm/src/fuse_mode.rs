//! `fuse` mode workload: each client performs real `std::fs` operations
//! against a mounted FUSE tree.
//!
//! Per cycle:
//!   1 × list  (`std::fs::read_dir` on the mount root)
//!   3 × read  (`std::fs::read_to_string` on a random `<id>.md` file)
//!   1 × patch (`std::fs::write` that flips the `status:` line)
//!
//! The patch path is best-effort: if another client races us, the `write`
//! still succeeds (last-writer-wins via FUSE) and the simulator's optimistic
//! concurrency handles conflicts. We rewrite the whole file so we never
//! emit a malformed frontmatter.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use parking_lot::Mutex;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::metrics::{ErrorKind, MetricsAccumulator, OpKind};
use crate::workload::Workload;

/// Workload impl backed by real syscalls against a FUSE mount.
pub struct FuseWorkload {
    mount: PathBuf,
    rng: Mutex<StdRng>,
    files: Mutex<Vec<PathBuf>>,
}

impl FuseWorkload {
    /// Construct a new FUSE workload pointed at `mount`. Does not probe the
    /// mount — the first step's `read_dir` surfaces mount-not-ready as a
    /// normal error.
    #[must_use]
    pub fn new(mount: PathBuf, seed: u64) -> Self {
        Self {
            mount,
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
            files: Mutex::new(Vec::new()),
        }
    }

    fn random_path(&self) -> Option<PathBuf> {
        let files = self.files.lock();
        if files.is_empty() {
            return None;
        }
        let mut r = self.rng.lock();
        let idx = r.gen_range(0..files.len());
        Some(files[idx].clone())
    }
}

/// Classify a `std::io::Error` into one of the swarm error buckets.
fn classify_io(err: &std::io::Error) -> ErrorKind {
    use std::io::ErrorKind as IoKind;
    match err.kind() {
        IoKind::NotFound => ErrorKind::NotFound,
        IoKind::TimedOut => ErrorKind::Timeout,
        _ => ErrorKind::Other,
    }
}

#[async_trait]
impl Workload for FuseWorkload {
    async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()> {
        let mount = self.mount.clone();
        // 1. list via read_dir
        let start = Instant::now();
        let list_res = tokio::task::spawn_blocking(move || {
            std::fs::read_dir(&mount).map(Iterator::collect::<Result<Vec<_>, _>>)
        })
        .await??;
        match list_res {
            Ok(entries) => {
                metrics.record(OpKind::List, elapsed_us(start));
                let mut g = self.files.lock();
                g.clear();
                for e in entries {
                    let p = e.path();
                    if p.extension().and_then(|s| s.to_str()) == Some("md") {
                        g.push(p);
                    }
                }
            }
            Err(err) => {
                metrics.record(OpKind::List, elapsed_us(start));
                metrics.record_error(classify_io(&err));
            }
        }

        // 2. 3 × read
        for _ in 0..3 {
            let Some(path) = self.random_path() else {
                break;
            };
            let start = Instant::now();
            let path_c = path.clone();
            let res = tokio::task::spawn_blocking(move || std::fs::read_to_string(&path_c)).await?;
            match res {
                Ok(_) => metrics.record(OpKind::Get, elapsed_us(start)),
                Err(err) => {
                    metrics.record(OpKind::Get, elapsed_us(start));
                    metrics.record_error(classify_io(&err));
                }
            }
        }

        // 3. 1 × patch — rewrite the file flipping `status:` to
        //    `in_progress`. Last-writer-wins; no coordination.
        if let Some(path) = self.random_path() {
            let start = Instant::now();
            let path_c = path.clone();
            let res = tokio::task::spawn_blocking(move || -> std::io::Result<()> {
                let content = std::fs::read_to_string(&path_c)?;
                let patched: String = content
                    .lines()
                    .map(|l| {
                        if l.starts_with("status:") {
                            "status: in_progress".to_string()
                        } else {
                            l.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(&path_c, patched)
            })
            .await?;
            match res {
                Ok(()) => metrics.record(OpKind::Patch, elapsed_us(start)),
                Err(err) => {
                    metrics.record(OpKind::Patch, elapsed_us(start));
                    metrics.record_error(classify_io(&err));
                }
            }
        }
        Ok(())
    }
}

/// Elapsed-microseconds helper.
fn elapsed_us(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}
