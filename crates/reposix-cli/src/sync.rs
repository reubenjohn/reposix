//! `reposix sync [--reconcile]` — cache reconciliation against the
//! `SoT`. The L1 conflict-detection escape hatch (DVCS-PERF-L1-02).
//!
//! Without `--reconcile`, prints a one-line hint pointing at
//! `--reconcile` (NOT an error — `reposix sync` is a v0.13.0+ surface
//! whose bare form is reserved for future flag combinations per D-02).
//!
//! With `--reconcile`, opens the cache for the working tree and runs
//! [`Cache::sync`] (which delegates to `Cache::build_from` on a fresh
//! cache, or runs the delta path with `last_fetched_at` if a cursor
//! exists). After the call, the cursor is bumped to `Utc::now()` and
//! the cache's tree state matches the backend.
//!
//! Design intent: the bus remote names this command in its reject-path
//! stderr hints. Renaming or making the bare form error would force a
//! doc-rev cascade.
//!
//! See `.planning/research/v0.13.0-dvcs/architecture-sketch.md`
//! § Performance subtlety.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use reposix_cache::Cache;
use reposix_remote::backend_dispatch::{self, BackendKind};

use crate::worktree_helpers::{resolve_reposix_remote_url, strip_bus_query};

/// Entrypoint for `reposix sync [--reconcile] [path]`.
///
/// `reconcile=false`: prints a hint pointing at `--reconcile`; exits 0.
/// `reconcile=true`: opens the cache for the working tree at `path`
/// (or cwd) and calls [`Cache::sync`] to perform a `list_records` walk
/// (or `list_changed_since` delta if cursor present) and rebuild the
/// cache. The `meta.last_fetched_at` cursor is bumped as a side effect.
///
/// # Errors
/// - The working tree has no reposix remote configured.
/// - The remote URL fails to parse.
/// - Backend construction fails (e.g. a real backend is missing a
///   required credential env var — the error names each unset var and
///   points at `docs/reference/testing-targets.md`).
/// - I/O when opening the cache.
/// - REST when calling `Cache::sync` against the backend.
pub async fn run(reconcile: bool, path: Option<PathBuf>) -> Result<()> {
    if !reconcile {
        println!(
            "reposix sync: pass --reconcile to perform a full \
             list_records walk + cache rebuild (the L1 escape hatch)."
        );
        println!(
            "see: architecture-sketch.md § Performance subtlety + \
             docs/concepts/dvcs-topology.md (P85 forthcoming)."
        );
        return Ok(());
    }

    let work = match path {
        Some(p) => p,
        None => std::env::current_dir().context("get cwd")?,
    };

    // Resolve (backend, project) from the working tree's reposix remote.
    // QL-004: partialClone-aware so this works on both `reposix init`
    // (partialClone=origin) and `reposix attach` (partialClone=<name>)
    // trees, and strips any bus `?mirror=` query to the SoT spec.
    let url = resolve_reposix_remote_url(&work).ok_or_else(|| {
        anyhow!(
            "no reposix remote in {} — run `reposix init <backend>::<project> <path>` \
             or `reposix attach <backend>::<project>` first",
            work.display()
        )
    })?;
    let sot = strip_bus_query(&url);
    let mut parsed = backend_dispatch::parse_remote_url(sot)
        .with_context(|| format!("parse reposix remote url={url}"))?;

    // Construct the backend connector through the git remote helper's
    // shared dispatch factory (D91-03) — the same path `attach` and the
    // helper binary use, so sim/github/confluence/jira all instantiate
    // identically and Confluence/JIRA inherit the OP-3 `.with_audit(…)`
    // wiring. For sim, honour REPOSIX_SIM_ORIGIN so tests can target a
    // random port while the remote URL keeps its canonical origin.
    if parsed.kind == BackendKind::Sim {
        if let Some(origin) = std::env::var("REPOSIX_SIM_ORIGIN")
            .ok()
            .filter(|s| !s.is_empty())
        {
            parsed.origin = origin;
        }
    }
    let backend = backend_dispatch::instantiate(&parsed)
        .with_context(|| format!("instantiate backend for {sot}"))?;
    let backend_slug = parsed.kind.slug();
    let cache_project = backend_dispatch::sanitize_project_for_cache(&parsed.project);

    let cache = Cache::open(backend, backend_slug, &cache_project).context("open cache")?;
    let report = cache.sync().await.context("Cache::sync for --reconcile")?;
    if let Some(commit) = report.new_commit {
        println!(
            "reposix sync: cache rebuilt (synthesis-commit OID = {commit}, \
             changed records = {n}); meta.last_fetched_at advanced.",
            n = report.changed_ids.len(),
        );
    } else {
        println!(
            "reposix sync: cache already up-to-date (no changed records); \
             meta.last_fetched_at advanced."
        );
    }
    Ok(())
}
