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
//! Design intent: the bus remote (P82–P83) names this command in its
//! reject-path stderr hints. Renaming or making the bare form error
//! would force a doc-rev cascade.
//!
//! See `.planning/research/v0.13.0-dvcs/architecture-sketch.md`
//! § Performance subtlety.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use reposix_cache::Cache;
use reposix_core::backend::sim::SimBackend;
use reposix_core::{parse_remote_url, BackendConnector};

use crate::worktree_helpers::{backend_slug_from_origin, git_config_get};

/// Entrypoint for `reposix sync [--reconcile] [path]`.
///
/// `reconcile=false`: prints a hint pointing at `--reconcile`; exits 0.
/// `reconcile=true`: opens the cache for the working tree at `path`
/// (or cwd) and calls [`Cache::sync`] to perform a `list_records` walk
/// (or `list_changed_since` delta if cursor present) and rebuild the
/// cache. The `meta.last_fetched_at` cursor is bumped as a side effect.
///
/// # Errors
/// - The working tree has no `remote.origin.url` configured.
/// - The remote URL fails to parse.
/// - The backend slug isn't `sim` (real-backend wiring lands in P82+).
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

    // Resolve (backend, project) from the working tree's remote URL.
    // Pattern matches what attach.rs / refresh.rs / history.rs use; we
    // reuse the existing `cache_path_from_worktree` parser via the
    // worktree_helpers module instead of inventing a new accessor.
    let url = git_config_get(&work, "remote.origin.url").ok_or_else(|| {
        anyhow!(
            "no remote.origin.url in {} (run `reposix init` or `reposix attach` first)",
            work.display()
        )
    })?;
    let spec = parse_remote_url(&url).with_context(|| format!("parse remote.origin.url={url}"))?;
    let backend_slug = backend_slug_from_origin(&url);

    // Construct the backend connector. v0.13.0 scaffolds sim only —
    // matches `attach.rs`'s scope; real backends arrive when the
    // credential paths land in later milestones.
    let backend: Arc<dyn BackendConnector> = match backend_slug.as_str() {
        "sim" => {
            let origin = std::env::var("REPOSIX_SIM_ORIGIN")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "http://127.0.0.1:7878".to_string());
            let sim = SimBackend::new(origin).context("build SimBackend")?;
            Arc::new(sim)
        }
        other => bail!(
            "sync --reconcile: backend `{other}` not yet wired in v0.13.0 (sim only); \
             github/confluence/jira land alongside the bus-remote work in P82+"
        ),
    };

    // Filesystem-safe project (only github needs the slash rewrite).
    let cache_project = if backend_slug == "github" {
        spec.project.as_str().replace('/', "-")
    } else {
        spec.project.as_str().to_string()
    };

    let cache = Cache::open(backend, &backend_slug, &cache_project).context("open cache")?;
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
