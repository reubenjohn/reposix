//! `reposix history` and `reposix at` — time-travel via git tags.
//!
//! Both subcommands operate on a `reposix init`'d working tree. They parse
//! `remote.origin.url`, resolve the cache directory, and read sync tags
//! from the cache's bare repo (`refs/reposix/sync/<ISO8601-no-colons>`).
//!
//! Design intent: `.planning/research/v0.11.0-vision-and-innovations.md` §3b.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Utc};
use reposix_cache::path::resolve_cache_path;
use reposix_cache::{list_sync_tags_at, SyncTag};
use reposix_core::parse_remote_url;
use rusqlite::Connection;

/// Default count for `reposix history` pagination.
const DEFAULT_HISTORY_LIMIT: usize = 10;

/// Resolve the cache's bare-repo path from a working-tree dir. Mirrors the
/// inference doctor uses: parse `remote.origin.url`, map host → backend slug,
/// look up `<cache_root>/<backend>-<project>.git`.
fn cache_path_from_worktree(work: &Path) -> Result<PathBuf> {
    let url = git_config_get(work, "remote.origin.url").ok_or_else(|| {
        anyhow!(
            "no remote.origin.url in {} (run `reposix init` first)",
            work.display()
        )
    })?;
    let spec = parse_remote_url(&url)
        .with_context(|| format!("parse remote.origin.url={url}"))?;
    let backend = backend_slug_from_origin(&spec.origin);
    resolve_cache_path(&backend, spec.project.as_str())
        .with_context(|| format!("resolve cache path for ({backend}, {project})", project = spec.project))
}

/// Map a remote origin to the cache `backend` slug. Mirrors `doctor.rs`.
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

/// Open the cache's bare repo (read-only) and list sync tags. Wraps the
/// reposix-cache helper to surface a clean error when the cache directory
/// hasn't been created yet (fresh init pre-fetch).
fn read_sync_tags(cache_path: &Path) -> Result<Vec<SyncTag>> {
    if !cache_path.exists() {
        bail!(
            "no cache at {} (run a `git fetch` to seed it first)",
            cache_path.display()
        );
    }
    list_sync_tags_at(cache_path)
        .with_context(|| format!("list sync tags from {}", cache_path.display()))
}

/// Read the `audit_events_cache` row for the synthesis commit pointed at by
/// a sync tag. Returns `(op, reason, bytes)` if a row is found that matches
/// the tag's reason text (the ref name) — typically `(sync_tag_written, ref_name, NULL)`.
///
/// To find the synthesis-event op (`tree_sync` / `delta_sync`), we look up the
/// most recent matching row at-or-before the tag's timestamp. Best-effort:
/// returns `None` if the cache.db is missing or the query fails.
fn synthesis_op_for_tag(cache_path: &Path, tag: &SyncTag) -> Option<(String, i64)> {
    let db = cache_path.join("cache.db");
    if !db.exists() {
        return None;
    }
    let conn = Connection::open(&db).ok()?;
    // Tag timestamps are second-precision; the synthesis row's `ts` is a
    // millisecond-precision RFC3339 written immediately before the tag. We
    // look for the most recent tree_sync OR delta_sync row at-or-before the
    // tag's wall-clock time + 2 seconds (the wider window absorbs slight
    // ts skew between SQL row insert and gix ref write).
    let upper = (tag.timestamp + chrono::Duration::seconds(2)).to_rfc3339();
    let mut stmt = conn
        .prepare(
            "SELECT op, COALESCE(bytes, 0) FROM audit_events_cache \
             WHERE op IN ('tree_sync', 'delta_sync') AND ts <= ?1 \
             ORDER BY id DESC LIMIT 1",
        )
        .ok()?;
    let row: Option<(String, i64)> = stmt
        .query_row([&upper], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .ok();
    row
}

/// `reposix history` entry point.
///
/// Lists sync tags from most-recent to oldest, paginated to `limit` (default
/// 10). Each line shows the timestamp slug, the synthesis commit OID
/// (short), and the synthesis op + count where derivable from `cache.db`.
///
/// # Errors
/// Returns an error if the working tree has no `remote.origin.url`, the
/// cache directory cannot be resolved, or the cache's bare repo cannot
/// be opened. A cache with zero sync tags returns Ok(()) and prints a
/// single explanatory line.
pub fn run_history(path: PathBuf, limit: Option<usize>) -> Result<()> {
    let cache_path = cache_path_from_worktree(&path)?;
    let tags = read_sync_tags(&cache_path)?;
    if tags.is_empty() {
        println!(
            "no sync tags in {} — run `git fetch` to create one",
            cache_path.display()
        );
        return Ok(());
    }

    let n = limit.unwrap_or(DEFAULT_HISTORY_LIMIT);
    let total = tags.len();
    // Most-recent-first, take N.
    for tag in tags.iter().rev().take(n) {
        let slug = tag
            .name
            .strip_prefix(reposix_cache::SYNC_TAG_PREFIX)
            .unwrap_or(tag.name.as_str());
        let short = tag.commit.to_hex_with_len(7).to_string();
        let synth = synthesis_op_for_tag(&cache_path, tag);
        let detail = match synth {
            Some((op, bytes)) => format!("{op} ({bytes} record(s) in this sync)"),
            None => "synthesis op unknown (audit row missing)".to_string(),
        };
        println!("{slug}   commit {short}   {detail}");
    }

    // Trailer.
    let earliest = tags
        .first()
        .map(|t| {
            t.name
                .strip_prefix(reposix_cache::SYNC_TAG_PREFIX)
                .unwrap_or(t.name.as_str())
                .to_string()
        })
        .unwrap_or_default();
    println!();
    println!(
        "{total} sync tag(s). Earliest: {earliest}. \
         Use `git -C {cache} checkout <tag>` to inspect a historical state.",
        cache = cache_path.display()
    );
    Ok(())
}

/// `reposix at <ts>` entry point.
///
/// Prints the closest-not-after sync tag for `target` and the cache path,
/// or an explanatory line if `target` predates every tag.
///
/// `target` is parsed as RFC-3339 (e.g. `2026-04-25T01:00:00Z`).
///
/// # Errors
/// Returns an error if `target` is not RFC-3339, the working tree has no
/// `remote.origin.url`, or the cache cannot be opened.
pub fn run_at(target: String, path: PathBuf) -> Result<()> {
    let target_dt: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(&target)
        .with_context(|| format!("invalid timestamp {target} — expected RFC-3339, e.g. 2026-04-25T01:00:00Z"))?
        .with_timezone(&Utc);
    let cache_path = cache_path_from_worktree(&path)?;
    let tags = read_sync_tags(&cache_path)?;
    let chosen = tags
        .into_iter()
        .rev()
        .find(|t| t.timestamp <= target_dt);
    if let Some(tag) = chosen {
        let short = tag.commit.to_hex_with_len(7).to_string();
        println!("{name}   commit {short}", name = tag.name);
        println!(
            "(use: git -C {cache} checkout {name})",
            cache = cache_path.display(),
            name = tag.name
        );
    } else {
        println!(
            "no sync tag at-or-before {target} (target predates all sync history in {})",
            cache_path.display()
        );
        // Non-error exit — the caller can still distinguish "not found"
        // from a hard failure by stdout content.
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_slug_mapping() {
        assert_eq!(backend_slug_from_origin("http://127.0.0.1:7878"), "sim");
        assert_eq!(backend_slug_from_origin("https://api.github.com"), "github");
        assert_eq!(
            backend_slug_from_origin("https://reuben-john.atlassian.net"),
            "confluence"
        );
    }
}
