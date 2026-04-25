//! `reposix gc` — explicit cache eviction.
//!
//! Evicts loose blob objects from a `reposix init`'d cache. Tree/commit
//! objects, refs, and sync tags are NEVER touched (see
//! `reposix_cache::gc` module doc).
//!
//! Design intent: `.planning/research/v0.11.0-vision-and-innovations.md` §3j.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use reposix_cache::db::open_cache_db;
use reposix_cache::path::resolve_cache_path;
use reposix_cache::{gc_at, GcReport, GcStrategy};
use reposix_core::parse_remote_url;

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

/// Resolve cache path from a working tree. Mirrors history.rs's resolver.
fn cache_path_from_worktree(work: &Path) -> Result<PathBuf> {
    let url = git_config_get(work, "remote.origin.url").ok_or_else(|| {
        anyhow!(
            "no remote.origin.url in {} (run `reposix init` first)",
            work.display()
        )
    })?;
    let spec = parse_remote_url(&url).with_context(|| format!("parse remote.origin.url={url}"))?;
    let backend = backend_slug_from_origin(&spec.origin);
    if !resolve_cache_path(&backend, spec.project.as_str()).is_ok_and(|p| p.exists()) {
        // Path may not exist yet — still resolve it so the report can
        // print a sensible "no cache" message.
    }
    let cache_path = resolve_cache_path(&backend, spec.project.as_str()).with_context(|| {
        format!(
            "resolve cache path for ({backend}, {project})",
            project = spec.project
        )
    })?;
    if !cache_path.exists() {
        bail!(
            "no cache at {} (nothing to gc; run a `git fetch` first)",
            cache_path.display()
        );
    }
    Ok(cache_path)
}

fn backend_slug_from_worktree(work: &Path) -> Option<String> {
    let url = git_config_get(work, "remote.origin.url")?;
    let spec = parse_remote_url(&url).ok()?;
    Some(backend_slug_from_origin(&spec.origin))
}

fn project_slug_from_worktree(work: &Path) -> Option<String> {
    let url = git_config_get(work, "remote.origin.url")?;
    let spec = parse_remote_url(&url).ok()?;
    Some(spec.project.as_str().to_owned())
}

fn backend_slug_from_origin(origin: &str) -> String {
    if origin.contains("api.github.com") {
        "github".to_string()
    } else if origin.contains("atlassian.net") {
        "confluence".to_string()
    } else {
        "sim".to_string()
    }
}

fn git_config_get(path: &Path, key: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["config", "--get", key])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
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
}
