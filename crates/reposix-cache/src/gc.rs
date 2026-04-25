//! Cache eviction — `Cache::gc` and supporting types.
//!
//! Evicts loose blob objects from the cache's bare repo to bound disk
//! usage. v0.11.0 §3j — "the cache keeps growing" answer.
//!
//! ## Safety constraint (load-bearing)
//!
//! GC NEVER touches tree/commit objects, refs, refs-packs, or sync tags.
//! Only loose blob objects under `objects/<2-char>/<remaining-OID>`
//! whose `git cat-file -t <oid>` is `blob` are eligible. Tree and commit
//! objects share the same on-disk layout but are forensically valuable
//! (commits anchor sync-tag history, trees enumerate the issue manifest)
//! and can be re-derived only by re-syncing from the backend — far more
//! costly than re-fetching a single blob's contents.
//!
//! Blobs, by contrast, can be re-materialized on demand by
//! [`Cache::read_blob`] from the backend, so eviction is a true cache
//! property: the next read transparently re-fetches.
//!
//! Sync tags live under `refs/reposix/sync/` (a ref namespace, not loose
//! objects on disk), so the loose-objects-only rule trivially excludes
//! them.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Duration, Utc};
use gix::ObjectId;

use crate::audit;
use crate::cache::Cache;
use crate::error::{Error, Result};

/// Default size cap for `--strategy=lru` (mirrors the CLI default).
pub const DEFAULT_MAX_SIZE_MB: u64 = 500;

/// Default age cap for `--strategy=ttl` (mirrors the CLI default).
pub const DEFAULT_MAX_AGE_DAYS: i64 = 30;

/// Eviction strategy for [`Cache::gc`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcStrategy {
    /// Evict least-recently-accessed (by file mtime) blobs first until
    /// total cache size <= `max_size_bytes`.
    Lru {
        /// Stop when total loose-blob size on disk drops below this.
        max_size_bytes: u64,
    },
    /// Evict blobs whose mtime is older than the cutoff. Cutoff is
    /// computed at call time as `now - max_age_days`.
    Ttl {
        /// Maximum age in days. Blobs with mtime older than this are evicted.
        max_age_days: i64,
    },
    /// Evict every loose blob. Used by tests + the rare "rebuild from
    /// scratch" workflow.
    All,
}

impl GcStrategy {
    /// Short slug for audit/log payloads (`lru`, `ttl`, `all`).
    #[must_use]
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Lru { .. } => "lru",
            Self::Ttl { .. } => "ttl",
            Self::All => "all",
        }
    }
}

/// One evicted blob — captured before deletion so the audit-row writer
/// can persist `(oid, bytes_reclaimed)` even on a `--dry-run`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvictedBlob {
    /// Hex OID (40 chars for SHA1).
    pub oid_hex: String,
    /// File size in bytes (loose-object on-disk representation, i.e.
    /// post-zlib-deflate).
    pub bytes: u64,
}

/// Outcome of one [`Cache::gc`] call.
#[derive(Debug, Clone)]
pub struct GcReport {
    /// Strategy that was applied.
    pub strategy: GcStrategy,
    /// Whether `--dry-run` mode was set (no actual eviction).
    pub dry_run: bool,
    /// Per-blob eviction record (in selection order, not file order).
    pub evicted: Vec<EvictedBlob>,
    /// Cache total size on disk BEFORE eviction (loose blobs only).
    pub bytes_before: u64,
    /// Cache total size on disk AFTER eviction (loose blobs only). On
    /// a dry-run this equals `bytes_before`.
    pub bytes_after: u64,
}

impl GcReport {
    /// Number of blobs evicted (or scheduled for eviction in dry-run mode).
    #[must_use]
    pub fn count(&self) -> usize {
        self.evicted.len()
    }

    /// Total bytes reclaimed. Sum of [`EvictedBlob::bytes`] across `evicted`.
    #[must_use]
    pub fn bytes_reclaimed(&self) -> u64 {
        self.evicted.iter().map(|b| b.bytes).sum()
    }
}

/// One discovered loose object — read once, sorted by the strategy.
struct LooseObject {
    oid_hex: String,
    path: PathBuf,
    bytes: u64,
    mtime: SystemTime,
}

impl Cache {
    /// Run garbage collection on this cache's loose blob objects.
    ///
    /// Always read-only on tree/commit objects, refs, sync tags, and
    /// `cache.db` rows other than the audit table. Only loose blob
    /// objects under `<repo>/objects/<2>/<38>` are eligible for
    /// eviction; the type check uses `git cat-file -t <oid>` which is
    /// type-safe (will refuse to mis-classify a tree as a blob).
    ///
    /// On `dry_run = true`, the on-disk blobs are NOT removed; only the
    /// returned [`GcReport`] is populated and a per-blob `cache_gc`
    /// audit row is written with `reason=dry_run:strategy=<slug>`.
    ///
    /// On a real run, each blob is unlinked from disk and a per-blob
    /// `cache_gc` audit row is written with `reason=evicted:strategy=<slug>`
    /// (plus `;age_days=N` for the TTL variant).
    ///
    /// # Errors
    /// - [`Error::Io`] for filesystem traversal failures or unlink errors.
    /// - [`Error::Git`] if `git cat-file` invocation fails.
    ///
    /// # Panics
    /// Panics if the internal `cache.db` mutex is poisoned (matches the
    /// rest of the cache's panic policy).
    pub fn gc(&self, strategy: GcStrategy, dry_run: bool) -> Result<GcReport> {
        let conn = self.db.lock().expect("cache.db mutex poisoned");
        gc_at(
            self.repo_path(),
            strategy,
            dry_run,
            Some((&conn, self.backend_name(), self.project())),
        )
    }
}

/// Same as [`Cache::gc`] but operates on a path + optional already-locked
/// connection. Useful for the CLI path which doesn't construct a full
/// [`Cache`] (no `BackendConnector` handle) but still wants to enumerate
/// + evict loose blobs.
///
/// `audit_target` is `Some((conn, backend, project))` to write per-blob
/// `cache_gc` audit rows; pass `None` to skip the audit (used by
/// integration tests that own their own audit assertions). The caller
/// owns the lock — pass an already-locked `&Connection`.
///
/// # Errors
/// Mirrors [`Cache::gc`].
pub fn gc_at(
    cache_path: &Path,
    strategy: GcStrategy,
    dry_run: bool,
    audit_target: Option<(&rusqlite::Connection, &str, &str)>,
) -> Result<GcReport> {
    let objects_dir = cache_path.join("objects");
    if !objects_dir.exists() {
        return Ok(GcReport {
            strategy,
            dry_run,
            evicted: Vec::new(),
            bytes_before: 0,
            bytes_after: 0,
        });
    }

    // Step 1: enumerate every candidate loose object.
    let candidates = enumerate_loose_objects(&objects_dir)?;

    // Step 2: filter to BLOBS only via `git cat-file -t <oid>`.
    // Tree + commit objects are forensically valuable and are NEVER
    // evicted — see module doc.
    let mut blobs: Vec<LooseObject> = Vec::with_capacity(candidates.len());
    for obj in candidates {
        if is_blob_object(cache_path, &obj.oid_hex)? {
            blobs.push(obj);
        }
    }

    let bytes_before: u64 = blobs.iter().map(|b| b.bytes).sum();

    // Step 3: select per strategy.
    let to_evict = select_evictees(blobs, strategy);

    // Step 4: evict (or dry-run-record).
    let mut evicted: Vec<EvictedBlob> = Vec::with_capacity(to_evict.len());
    let strategy_slug = strategy.slug();
    let reason_prefix = if dry_run { "dry_run" } else { "evicted" };
    for obj in &to_evict {
        if !dry_run {
            // Remove the loose-object file. NEVER touches `refs/`,
            // `packed-refs`, or any non-loose-object path.
            std::fs::remove_file(&obj.path).map_err(|e| {
                Error::Io(std::io::Error::new(
                    e.kind(),
                    format!("remove loose object {}: {e}", obj.path.display()),
                ))
            })?;
        }
        evicted.push(EvictedBlob {
            oid_hex: obj.oid_hex.clone(),
            bytes: obj.bytes,
        });

        if let Some((conn, backend, project)) = audit_target {
            let reason = match strategy {
                GcStrategy::Ttl { max_age_days } => {
                    format!("{reason_prefix}:strategy={strategy_slug};age_days={max_age_days}")
                }
                GcStrategy::Lru { .. } | GcStrategy::All => {
                    format!("{reason_prefix}:strategy={strategy_slug}")
                }
            };
            audit::log_cache_gc(conn, backend, project, &obj.oid_hex, obj.bytes, &reason);
        }
    }

    let bytes_after = if dry_run {
        bytes_before
    } else {
        bytes_before.saturating_sub(evicted.iter().map(|b| b.bytes).sum::<u64>())
    };

    Ok(GcReport {
        strategy,
        dry_run,
        evicted,
        bytes_before,
        bytes_after,
    })
}

/// Walk `<repo>/objects/<2-char>/<38-char>` and collect every loose
/// object's `(oid, path, bytes, mtime)`. Skips `info/` and `pack/`
/// subdirectories.
fn enumerate_loose_objects(objects_dir: &Path) -> Result<Vec<LooseObject>> {
    let mut out = Vec::new();
    let prefix_iter = std::fs::read_dir(objects_dir)?;
    for prefix_entry in prefix_iter {
        let prefix_entry = prefix_entry?;
        let prefix_path = prefix_entry.path();
        if !prefix_path.is_dir() {
            continue;
        }
        let prefix_name = match prefix_entry.file_name().to_str() {
            Some(s) => s.to_owned(),
            None => continue,
        };
        // Loose-object dirs are 2-char hex. Skip `info`, `pack`, etc.
        if prefix_name.len() != 2 || !prefix_name.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        for obj_entry in std::fs::read_dir(&prefix_path)? {
            let obj_entry = obj_entry?;
            let obj_path = obj_entry.path();
            if !obj_path.is_file() {
                continue;
            }
            let obj_name = match obj_entry.file_name().to_str() {
                Some(s) => s.to_owned(),
                None => continue,
            };
            // Loose-object filenames are the remaining 38 hex chars.
            if obj_name.len() != 38 || !obj_name.bytes().all(|b| b.is_ascii_hexdigit()) {
                continue;
            }
            let metadata = obj_entry.metadata()?;
            let bytes = metadata.len();
            let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let oid_hex = format!("{prefix_name}{obj_name}");
            out.push(LooseObject {
                oid_hex,
                path: obj_path,
                bytes,
                mtime,
            });
        }
    }
    Ok(out)
}

/// Return `true` iff `git cat-file -t <oid>` reports `blob`.
fn is_blob_object(cache_path: &Path, oid_hex: &str) -> Result<bool> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(cache_path)
        .args(["cat-file", "-t", oid_hex])
        .output()
        .map_err(|e| Error::Git(format!("spawn git cat-file: {e}")))?;
    if !out.status.success() {
        // Object can't be classified — be conservative and refuse to
        // evict (treat as non-blob).
        return Ok(false);
    }
    let kind = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    Ok(kind == "blob")
}

/// Apply the strategy to pick which blobs are evicted.
fn select_evictees(mut blobs: Vec<LooseObject>, strategy: GcStrategy) -> Vec<LooseObject> {
    match strategy {
        GcStrategy::All => blobs,
        GcStrategy::Lru { max_size_bytes } => {
            // Sort by mtime ASCENDING (oldest first) so we evict
            // least-recently-touched.
            blobs.sort_by_key(|b| b.mtime);
            let total: u64 = blobs.iter().map(|b| b.bytes).sum();
            if total <= max_size_bytes {
                return Vec::new();
            }
            let mut to_remove = total - max_size_bytes;
            let mut out = Vec::new();
            for b in blobs {
                if to_remove == 0 {
                    break;
                }
                let bytes = b.bytes;
                out.push(b);
                to_remove = to_remove.saturating_sub(bytes);
            }
            out
        }
        GcStrategy::Ttl { max_age_days } => {
            let cutoff = SystemTime::now()
                .checked_sub(std::time::Duration::from_secs(
                    u64::try_from(max_age_days.max(0)).unwrap_or(0) * 86_400,
                ))
                .unwrap_or(SystemTime::UNIX_EPOCH);
            blobs.into_iter().filter(|b| b.mtime < cutoff).collect()
        }
    }
}

/// Compute a human-readable cutoff [`DateTime<Utc>`] for the TTL
/// strategy. Used by the CLI for `--dry-run` reporting; not used inside
/// the eviction logic itself.
#[must_use]
pub fn ttl_cutoff(max_age_days: i64) -> DateTime<Utc> {
    Utc::now() - Duration::days(max_age_days)
}

/// Helper for tests: parse an OID hex into `gix::ObjectId`.
#[doc(hidden)]
#[must_use]
pub fn parse_oid(hex: &str) -> Option<ObjectId> {
    gix::ObjectId::from_hex(hex.as_bytes()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategy_slug_round_trip() {
        assert_eq!(
            GcStrategy::Lru {
                max_size_bytes: 100
            }
            .slug(),
            "lru"
        );
        assert_eq!(GcStrategy::Ttl { max_age_days: 30 }.slug(), "ttl");
        assert_eq!(GcStrategy::All.slug(), "all");
    }

    #[test]
    fn report_count_and_bytes() {
        let r = GcReport {
            strategy: GcStrategy::All,
            dry_run: false,
            evicted: vec![
                EvictedBlob {
                    oid_hex: "a".into(),
                    bytes: 10,
                },
                EvictedBlob {
                    oid_hex: "b".into(),
                    bytes: 20,
                },
            ],
            bytes_before: 30,
            bytes_after: 0,
        };
        assert_eq!(r.count(), 2);
        assert_eq!(r.bytes_reclaimed(), 30);
    }

    #[test]
    fn empty_objects_dir_is_no_op() {
        let tmp = tempfile::tempdir().unwrap();
        let report = gc_at(tmp.path(), GcStrategy::All, false, None).unwrap();
        assert_eq!(report.count(), 0);
        assert_eq!(report.bytes_before, 0);
    }
}
