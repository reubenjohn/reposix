//! `reposix sync [--reconcile]` — cache reconciliation against the
//! `SoT`. The L1 conflict-detection escape hatch (DVCS-PERF-L1-02).
//!
//! Without `--reconcile`, prints a one-line hint pointing at
//! `--reconcile` (NOT an error — `reposix sync` is a v0.13.0+ surface
//! whose bare form is reserved for future flag combinations per D-02).
//!
//! With `--reconcile`, opens the cache for the working tree and forces
//! a full rebuild via [`Cache::build_from`] directly — NOT
//! [`Cache::sync`], which would take the delta path whenever a
//! `last_fetched_at` cursor is present (ADR-010 / RBF-LR-01). This is
//! what makes `--reconcile` an actual escape hatch: it heals a cache
//! that has already drifted from the tree↔`oid_map` coherence
//! invariant (e.g. one written by a pre-fix binary), which a delta
//! sync cannot do. After the call, the cursor is bumped to
//! `Utc::now()` and the cache's tree state matches the backend
//! exactly (a full `list_records` walk, not a delta).
//!
//! Scope caveat (what `--reconcile` does NOT fix): it heals tree↔`oid_map`
//! coherence (ghost / missing rows) and genuine eventual-consistency
//! races, but a `systematic` backend-side rendering mismatch — the same
//! id rendered to different bytes by `list_records` vs `get_record`
//! regardless of timing, the class `reposix_cache::Error::OidDrift` now
//! documents — reproduces the SAME oid on a re-list, so that class
//! requires the adapter/backend fix itself, not a reconcile
//! (proof: `crates/reposix-cache/tests/oid_drift_reconcile.rs`).
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
use reposix_core::errmsg::teach;
use reposix_remote::backend_dispatch::{self, BackendKind};

use crate::errors::cache_build_error;
use crate::worktree_helpers::{resolve_reposix_remote_url, strip_bus_query};

/// Entrypoint for `reposix sync [--reconcile] [path]`.
///
/// `reconcile=false`: prints a hint pointing at `--reconcile`; exits 0.
/// `reconcile=true`: opens the cache for the working tree at `path`
/// (or cwd) and calls [`Cache::build_from`] directly — a full
/// `list_records` walk and tree rebuild, bypassing `Cache::sync`'s
/// delta path entirely (ADR-010 / RBF-LR-01) — so `--reconcile` can
/// heal a cache that has already drifted from the tree↔`oid_map`
/// coherence invariant. The `meta.last_fetched_at` cursor is bumped as
/// a side effect.
///
/// Scope: this recovers tree↔`oid_map` coherence and genuine
/// eventual-consistency races. It does NOT recover a `systematic`
/// backend rendering mismatch (`list_records` vs `get_record` disagree on
/// the same id regardless of timing) — a re-list recomputes the SAME oid,
/// so that class needs the adapter/backend fix, not a reconcile.
///
/// # Errors
/// - The working tree has no reposix remote configured.
/// - The remote URL fails to parse.
/// - Backend construction fails (e.g. a real backend is missing a
///   required credential env var — the error names each unset var and
///   points at `docs/reference/testing-targets.md`).
/// - I/O when opening the cache.
/// - REST when calling `Cache::build_from` against the backend.
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
            "{}",
            teach(
                &format!(
                    "`reposix sync` found no reposix remote in {}.",
                    work.display()
                ),
                "sync reconciles a tree that is already bound to a backend; this directory has \
                 no reposix remote, so there is nothing to sync against. Bind it first with \
                 `reposix init` (fresh tree) or `reposix attach` (existing checkout).",
                "not sure which backend this tree belongs to? inspect `git remote -v` — a bound \
                 tree has a `reposix::<backend>::<project>` remote URL.",
                &[
                    "reposix init sim::demo /tmp/demo          # create a fresh bound tree",
                    "reposix attach <backend>::<project> .     # bind THIS existing checkout",
                ],
            )
        )
    })?;
    let sot = strip_bus_query(&url);
    let mut parsed = backend_dispatch::parse_remote_url(sot).map_err(|e| {
        anyhow!(
            "{}",
            teach(
                &format!(
                    "the reposix remote URL in this tree could not be parsed: `{url}`.\n\
                     (underlying: {e:#})"
                ),
                "the tree's `remote.*.url` (or its `?mirror=` bus form) is not a valid \
                 `reposix::<backend>::<project>` URL — it may have been hand-edited or written \
                 by an incompatible reposix version.",
                "re-create the binding from scratch: `reposix init` for a fresh tree, or \
                 `reposix attach` to rebind this existing checkout.",
                &[
                    "git remote -v                             # inspect the current remote URL",
                    "reposix attach <backend>::<project> .     # rebind THIS checkout",
                ],
            )
        )
    })?;

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
    let backend_slug = parsed.kind.slug();
    let backend = backend_dispatch::instantiate(&parsed)
        .map_err(|e| cache_build_error(backend_slug, &parsed.project, e))?;
    // S-260707-gh404: pass the RAW project slug — `Cache::open` sanitizes it to
    // the flat cache dir internally; the backend must see `owner/repo` verbatim.
    let cache = Cache::open(backend, backend_slug, &parsed.project)
        .map_err(|e| cache_build_error(backend_slug, &parsed.project, e))?;
    // ADR-010 / RBF-LR-01: call `build_from()` directly (NOT `cache.sync()`,
    // which takes the delta path whenever a `last_fetched_at` cursor is
    // present). `--reconcile` promises "a full list_records walk + cache
    // rebuild" — only a forced full rebuild honours that promise and can
    // heal a cache that already drifted from the tree↔`oid_map` coherence
    // invariant (e.g. one written by a pre-fix binary).
    let commit = cache
        .build_from()
        .await
        .map_err(|e| cache_build_error(backend_slug, &parsed.project, e))?;
    println!(
        "reposix sync: cache rebuilt from a full list_records walk \
         (synthesis-commit OID = {commit}); meta.last_fetched_at advanced."
    );
    Ok(())
}
