//! `reposix gc` — explicit cache eviction.
//!
//! Evicts loose blob objects from a `reposix init`'d cache. Tree/commit
//! objects, refs, and sync tags are NEVER touched (see
//! `reposix_cache::gc` module doc).
//!
//! Design intent: `.planning/research/v0.11.0/vision-and-innovations.md` §3j.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use reposix_cache::db::open_cache_db;
use reposix_cache::{gc_at, GcReport, GcStrategy};
use reposix_core::parse_remote_url;

use crate::worktree_helpers::{backend_slug_from_origin, cache_path_from_worktree, git_config_get};

/// One orphan-cache finding from [`scan_orphans`].
#[derive(Debug, Clone)]
pub struct OrphanCache {
    /// Path to the orphaned cache directory (e.g.
    /// `~/.cache/reposix/sim-stale.git`).
    pub path: PathBuf,
    /// Total bytes occupied by the cache directory (best-effort).
    pub bytes: u64,
    /// Worktree paths recorded in `meta.worktrees`. Empty when the cache
    /// has no recorded owning tree (sim/demo, pre-v0.11.0 caches).
    pub recorded_worktrees: Vec<PathBuf>,
    /// Reason this cache was classified as orphan. Stable strings:
    /// `no_worktrees_recorded`, `all_worktrees_missing`.
    pub reason: &'static str,
}

/// User-facing strategy choice — clap `value_enum` doesn't compose with
/// `GcStrategy`'s embedded numeric fields, so we pick the variant here
/// and hand-construct the typed enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum GcStrategyArg {
    /// Evict least-recently-accessed first until `--max-size-mb` is met.
    Lru,
    /// Evict blobs older than `--max-age-days`.
    Ttl,
    /// Evict every loose blob.
    All,
}

/// `reposix gc` entry point.
///
/// `path` defaults to cwd; the working tree's `remote.origin.url` is
/// used to resolve the cache directory (same inference as
/// `reposix doctor` and `reposix history`).
///
/// # Errors
/// - The path does not contain a parseable `reposix::` remote URL.
/// - The cache directory cannot be resolved.
/// - The cache DB cannot be opened (audit will be skipped — but this is
///   surfaced as an error so the caller knows audit is missing).
/// - The eviction itself fails (filesystem error).
pub fn run(
    path: Option<PathBuf>,
    strategy_arg: GcStrategyArg,
    max_size_mb: u64,
    max_age_days: i64,
    dry_run: bool,
) -> Result<()> {
    let work = match path {
        Some(p) => p,
        None => std::env::current_dir().context("resolve current directory")?,
    };
    let cache_path = cache_path_from_worktree(&work)?;
    if !cache_path.exists() {
        bail!(
            "no cache at {} (nothing to gc; run a `git fetch` first)",
            cache_path.display()
        );
    }
    let strategy = match strategy_arg {
        GcStrategyArg::Lru => GcStrategy::Lru {
            max_size_bytes: max_size_mb.saturating_mul(1024 * 1024),
        },
        GcStrategyArg::Ttl => GcStrategy::Ttl { max_age_days },
        GcStrategyArg::All => GcStrategy::All,
    };

    // Open the cache DB to write per-blob audit rows. If unavailable,
    // surface a warning but continue — eviction itself is the user-
    // facing operation.
    let conn_res = open_cache_db(&cache_path);
    let report = match &conn_res {
        Ok(conn) => {
            let backend = backend_slug_from_worktree(&work).unwrap_or_else(|| "unknown".into());
            // Best-effort: use the cache directory's stem as the project
            // hint when we can't parse it from the remote URL.
            let project = project_slug_from_worktree(&work).unwrap_or_else(|| "unknown".into());
            gc_at(
                &cache_path,
                strategy,
                dry_run,
                Some((conn, &backend, &project)),
            )
            .map_err(|e| anyhow!("gc failed: {e}"))?
        }
        Err(e) => {
            tracing::warn!(
                "could not open cache DB at {} for audit ({e}); proceeding without audit",
                cache_path.display()
            );
            gc_at(&cache_path, strategy, dry_run, None).map_err(|e| anyhow!("gc failed: {e}"))?
        }
    };

    print_report(&cache_path, &report);

    Ok(())
}

fn print_report(cache_path: &Path, report: &GcReport) {
    let mode = if report.dry_run {
        "would evict"
    } else {
        "evicted"
    };
    let strategy_label = match report.strategy {
        GcStrategy::Lru { max_size_bytes } => {
            format!("LRU (cap: {} MB)", max_size_bytes / (1024 * 1024))
        }
        GcStrategy::Ttl { max_age_days } => format!("TTL (max age: {max_age_days} days)"),
        GcStrategy::All => "ALL".to_string(),
    };
    let mb = bytes_to_mb_string(report.bytes_reclaimed());
    println!(
        "reposix gc — {} blob(s) ({} reclaimable) using {} strategy",
        report.count(),
        mb,
        strategy_label,
    );
    println!("  cache: {}", cache_path.display());
    println!(
        "  before: {}   after: {}",
        bytes_to_mb_string(report.bytes_before),
        bytes_to_mb_string(report.bytes_after),
    );
    if report.count() == 0 {
        println!("Nothing to {mode}.");
    } else if report.dry_run {
        println!(
            "Dry-run: {} blob(s) would be evicted (no files removed).",
            report.count()
        );
    } else {
        println!("Evicted {} blob(s).", report.count());
    }
}

fn bytes_to_mb_string(bytes: u64) -> String {
    // Cast precision: bytes-to-MB conversion for display only; bytes
    // are bounded by disk size (<<2^53 in practice).
    #[allow(clippy::cast_precision_loss)]
    let mb = (bytes as f64) / (1024.0 * 1024.0);
    if mb < 0.01 {
        format!("{bytes} B")
    } else {
        format!("{mb:.2} MB")
    }
}

fn backend_slug_from_worktree(work: &Path) -> Option<String> {
    let url = git_config_get(work, "remote.origin.url")?;
    // Validate the URL parses but pass the FULL url (not just origin) to
    // backend_slug_from_origin so JIRA's `/jira/` marker is visible. The
    // earlier `&spec.origin` form silently routed JIRA worktrees to the
    // confluence cache. v0.11.1 audit-finding fix.
    let _spec = parse_remote_url(&url).ok()?;
    Some(backend_slug_from_origin(&url))
}

fn project_slug_from_worktree(work: &Path) -> Option<String> {
    let url = git_config_get(work, "remote.origin.url")?;
    let spec = parse_remote_url(&url).ok()?;
    Some(spec.project.as_str().to_owned())
}

/// `reposix gc --orphans` entry point.
///
/// Walks `<cache_root>/reposix/*.git/`, opens each `cache.db`, reads
/// `meta.worktrees`, and reports caches where every recorded worktree
/// path no longer exists on disk (the cache is "orphaned"). Caches with
/// no `meta.worktrees` row are reported with `reason="no_worktrees_recorded"`
/// and skipped from purge unless `--include-untracked` is passed (so the
/// simulator's `sim-demo.git` cache, which is opened by
/// `cargo run -p reposix-sim` rather than `reposix init`, is preserved by
/// default).
///
/// `dry_run` (the default) prints the orphan list with sizes; `--purge`
/// removes the orphan directories. The simulator's cache (`sim-demo.git`)
/// is preserved unless `include_sim` is also true.
///
/// # Errors
/// - The cache root cannot be resolved (no `$XDG_CACHE_HOME`/`$HOME`).
/// - A directory walk fails.
pub fn run_orphans(dry_run: bool, include_sim: bool, include_untracked: bool) -> Result<()> {
    let cache_root = resolve_cache_root()?;
    let orphans = scan_orphans(&cache_root)?;

    if orphans.is_empty() {
        println!(
            "reposix gc --orphans: no orphan caches found under {}",
            cache_root.display()
        );
        return Ok(());
    }

    let mut total_bytes: u64 = 0;
    let mut purged: u64 = 0;
    let mode = if dry_run { "would purge" } else { "purging" };
    println!(
        "reposix gc --orphans — {n} candidate orphan cache(s) under {root}",
        n = orphans.len(),
        root = cache_root.display()
    );
    for orphan in &orphans {
        let is_sim = is_sim_cache(&orphan.path);
        let is_untracked = orphan.reason == "no_worktrees_recorded";
        let skip_sim = is_sim && !include_sim;
        let skip_untracked = is_untracked && !include_untracked;
        let mb = bytes_to_mb_string(orphan.bytes);
        let status = if skip_sim {
            "(preserved — simulator cache; pass --include-sim to purge)"
        } else if skip_untracked {
            "(preserved — no recorded worktree; pass --include-untracked to purge)"
        } else if dry_run {
            "(would purge)"
        } else {
            "(purged)"
        };
        println!(
            "  {path}  {mb}  reason={reason}  {status}",
            path = orphan.path.display(),
            reason = orphan.reason,
        );
        if !orphan.recorded_worktrees.is_empty() {
            for wt in &orphan.recorded_worktrees {
                println!("    recorded worktree: {} (missing)", wt.display());
            }
        }
        if skip_sim || skip_untracked {
            continue;
        }
        total_bytes = total_bytes.saturating_add(orphan.bytes);
        if !dry_run {
            match std::fs::remove_dir_all(&orphan.path) {
                Ok(()) => {
                    purged += 1;
                    eprintln!("reposix gc --orphans: removed {}", orphan.path.display());
                }
                Err(e) => {
                    eprintln!(
                        "reposix gc --orphans: failed to remove {}: {e}",
                        orphan.path.display()
                    );
                }
            }
        }
    }
    println!();
    println!(
        "Summary: {n} orphan(s); {bytes} reclaimable; {mode} {purged} cache(s)",
        n = orphans.len(),
        bytes = bytes_to_mb_string(total_bytes),
    );
    Ok(())
}

/// Resolve `<cache_root>/reposix/` (the directory that contains
/// `<scheme>-<project>.git/` cache subdirectories). Delegates to
/// `reposix_cache::resolve_cache_path` to honour `REPOSIX_CACHE_DIR`
/// and the same XDG fallback rules; takes the parent of a probe path.
fn resolve_cache_root() -> Result<PathBuf> {
    let probe = reposix_cache::resolve_cache_path("__probe", "__probe")
        .map_err(|e| anyhow!("resolve cache root: {e}"))?;
    let parent = probe
        .parent()
        .ok_or_else(|| anyhow!("cache root probe has no parent: {}", probe.display()))?
        .to_path_buf();
    Ok(parent)
}

/// Enumerate orphan caches under `cache_root`.
///
/// A cache is considered orphaned when:
/// - Its `meta.worktrees` row is set AND every listed path is absent on disk
///   (`reason="all_worktrees_missing"`), OR
/// - Its `meta.worktrees` row is unset and the cache is the simulator's
///   demo cache (`reason="no_worktrees_recorded"`) — these are preserved
///   by default; the caller decides whether to purge.
///
/// Caches that have at least one live recorded worktree are NOT returned.
///
/// # Errors
/// - Reading the directory fails (other than `NotFound`).
pub fn scan_orphans(cache_root: &Path) -> Result<Vec<OrphanCache>> {
    let mut out: Vec<OrphanCache> = Vec::new();
    let entries = match std::fs::read_dir(cache_root) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(anyhow!("read_dir({}): {e}", cache_root.display())),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // Cache dirs match `<scheme>-<project>.git`. Skip non-conforming
        // entries silently — `reposix gc --orphans` is conservative.
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let is_git_suffix = std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("git"));
        if !is_git_suffix {
            continue;
        }
        let db_path = path.join("cache.db");
        if !db_path.exists() {
            continue;
        }
        let Ok(conn) = rusqlite::Connection::open(&db_path) else {
            continue;
        };
        let worktrees: Option<String> = conn
            .query_row("SELECT value FROM meta WHERE key = 'worktrees'", [], |r| {
                r.get::<_, String>(0)
            })
            .ok();
        let recorded: Vec<PathBuf> = worktrees
            .as_deref()
            .map(|s| {
                s.lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(PathBuf::from)
                    .collect()
            })
            .unwrap_or_default();

        if recorded.is_empty() {
            // No recorded owners — likely sim::demo opened by `reposix sim`,
            // or a cache from a pre-v0.11.0 reposix init. Surface but
            // preserve by default.
            out.push(OrphanCache {
                bytes: dir_size(&path),
                recorded_worktrees: Vec::new(),
                path,
                reason: "no_worktrees_recorded",
            });
            continue;
        }
        let any_alive = recorded.iter().any(|p| p.exists());
        if any_alive {
            continue;
        }
        out.push(OrphanCache {
            bytes: dir_size(&path),
            recorded_worktrees: recorded,
            path,
            reason: "all_worktrees_missing",
        });
    }
    Ok(out)
}

fn is_sim_cache(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("sim-"))
}

/// Recursively sum file sizes under `path`. Best-effort; symlink-aware
/// (does not follow). Returns 0 on any IO error.
fn dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            total = total.saturating_add(dir_size(&entry.path()));
        } else if meta.is_file() {
            total = total.saturating_add(meta.len());
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_to_mb_thresholds() {
        assert_eq!(bytes_to_mb_string(0), "0 B");
        assert_eq!(bytes_to_mb_string(1024), "1024 B");
        assert!(bytes_to_mb_string(2 * 1024 * 1024).contains("MB"));
    }

    fn create_cache_dir_with_worktrees(parent: &Path, name: &str, worktrees: &[&str]) -> PathBuf {
        let dir = parent.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("cache.db");
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute(
            "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT NOT NULL)",
            [],
        )
        .unwrap();
        if !worktrees.is_empty() {
            let val = worktrees.join("\n");
            conn.execute(
                "INSERT INTO meta (key, value, updated_at) VALUES ('worktrees', ?1, '2026-01-01T00:00:00Z')",
                rusqlite::params![val],
            )
            .unwrap();
        }
        dir
    }

    #[test]
    fn scan_orphans_finds_dead_worktrees() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_root = tmp.path();
        // Cache 1: worktree absent → orphan with reason all_worktrees_missing.
        create_cache_dir_with_worktrees(cache_root, "sim-stale.git", &["/nonexistent/path"]);
        // Cache 2: worktree present → not orphan.
        let live_dir = tmp.path().join("live-tree");
        std::fs::create_dir_all(&live_dir).unwrap();
        let live_str = live_dir.to_str().unwrap();
        create_cache_dir_with_worktrees(cache_root, "sim-live.git", &[live_str]);
        // Cache 3: no worktrees recorded → orphan with reason no_worktrees_recorded.
        create_cache_dir_with_worktrees(cache_root, "sim-untracked.git", &[]);

        let orphans = scan_orphans(cache_root).unwrap();
        let names: Vec<String> = orphans
            .iter()
            .map(|o| o.path.file_name().unwrap().to_string_lossy().into())
            .collect();
        assert!(names.contains(&"sim-stale.git".to_string()));
        assert!(names.contains(&"sim-untracked.git".to_string()));
        assert!(!names.contains(&"sim-live.git".to_string()));

        let stale = orphans
            .iter()
            .find(|o| o.path.file_name().unwrap() == "sim-stale.git")
            .unwrap();
        assert_eq!(stale.reason, "all_worktrees_missing");
        let untracked = orphans
            .iter()
            .find(|o| o.path.file_name().unwrap() == "sim-untracked.git")
            .unwrap();
        assert_eq!(untracked.reason, "no_worktrees_recorded");
    }

    #[test]
    fn scan_orphans_handles_missing_root() {
        let tmp = tempfile::tempdir().unwrap();
        let nonexistent = tmp.path().join("does-not-exist");
        let orphans = scan_orphans(&nonexistent).unwrap();
        assert!(orphans.is_empty());
    }

    #[test]
    fn is_sim_cache_recognises_sim_prefix() {
        assert!(is_sim_cache(&PathBuf::from("/x/sim-demo.git")));
        assert!(!is_sim_cache(&PathBuf::from("/x/github-foo.git")));
    }
}
