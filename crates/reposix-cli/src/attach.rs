//! `reposix attach <backend>::<project>` — attach an existing checkout
//! to a `SoT` backend (DVCS-ATTACH-01..04, v0.13.0).
//!
//! Pre-conditions: CWD (or `<path>` if passed) is a git working tree
//! (i.e. `.git/` exists). The working tree may have been created any
//! way — vanilla `git clone`, prior `reposix init`, hand-edited.
//!
//! Steps (architecture-sketch.md § "1. `reposix attach <backend>::<project>`"):
//!   1. Resolve cache path from `SoT` spec (NOT from `remote.origin.url`) — Q1.1.
//!   2. REST-list the backend; populate cache OIDs (lazy blobs).
//!   3. Walk current HEAD tree; reconcile against backend records by
//!      `id` in frontmatter (see `crates/reposix-cache/src/reconciliation.rs`).
//!   4. Add remote `<remote-name>` (default `reposix`) with URL
//!      `reposix::<sot-spec>?mirror=<existing-mirror-url>` (or no
//!      `?mirror=` if `--no-bus`).
//!   5. Set `extensions.partialClone=<remote-name>` (NOT `origin`).
//!
//! Re-attach idempotency (Q1.3): same `SoT` → refresh cache + reconciliation
//! table; different `SoT` → reject (Q1.2).
//!
//! After the reconciliation walk completes, calls `Cache::log_attach_walk`
//! to write a row to `audit_events_cache` per OP-3 (UNCONDITIONAL —
//! no deferral).
//!
//! Per POC-FINDINGS F02: help text says "records" not "issues" for
//! portability. The architecture-sketch refers to records abstractly;
//! the simulator's wire path is /issues but the abstraction layer is
//! Record (`reposix_core::Record`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use reposix_core::BackendConnector;
use reposix_remote::backend_dispatch::{self, BackendKind};

use crate::init::translate_spec_to_url;

/// Default `--ignore` glob list (POC-FINDINGS F01) — directories pruned by
/// the reconciliation walker so vendored docs don't pollute the
/// `cache_reconciliation` table on real checkouts.
const DEFAULT_IGNORE: &str = ".git,.github";

/// Clap-derived arguments for `reposix attach`.
#[derive(clap::Args, Debug)]
pub struct AttachArgs {
    /// Backend + project spec, e.g. `sim::demo`.
    #[arg(value_name = "BACKEND::PROJECT")]
    pub spec: String,
    /// Working-tree path; defaults to CWD.
    pub path: Option<PathBuf>,
    /// Skip the `?mirror=` query param (single-SoT remote URL).
    #[arg(long)]
    pub no_bus: bool,
    /// Existing plain-git remote name to fold into the bus URL.
    #[arg(long, default_value = "origin")]
    pub mirror_name: String,
    /// Name of the new reposix remote.
    #[arg(long, default_value = "reposix")]
    pub remote_name: String,
    /// Orphan policy when local records reference IDs the backend lacks.
    #[arg(long, value_enum, default_value_t = OrphanPolicy::Abort)]
    pub orphan_policy: OrphanPolicy,
    /// Comma-separated directory names to skip during the reconciliation
    /// walk. Default `.git,.github` (POC-FINDINGS F01).
    #[arg(long, default_value = DEFAULT_IGNORE)]
    pub ignore: String,
}

/// CLI-side mirror of `reposix_cache::reconciliation::OrphanPolicy`.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OrphanPolicy {
    /// Delete the local file (destructive).
    DeleteLocal,
    /// Treat the local file as a new record to be created on next push.
    ForkAsNew,
    /// Report the orphan and leave the file in place for manual triage
    /// (default). Does NOT abort the attach — attach still succeeds;
    /// duplicate-id is the only hard stop.
    Abort,
}

impl From<OrphanPolicy> for reposix_cache::reconciliation::OrphanPolicy {
    fn from(v: OrphanPolicy) -> Self {
        match v {
            OrphanPolicy::Abort => Self::Abort,
            OrphanPolicy::DeleteLocal => Self::DeleteLocal,
            OrphanPolicy::ForkAsNew => Self::ForkAsNew,
        }
    }
}

/// Run `reposix attach`. See module docs.
///
/// # Errors
/// Returns an error on any of:
/// - Spec parse failure (delegated to `translate_spec_to_url`).
/// - Working tree is not a git repo.
/// - Re-attach with a different `SoT` (Q1.2 reject).
/// - Duplicate `id` across two local records (architecture-sketch row 4).
/// - Cache materialization or REST list failure.
//
// The body inlines architecture-sketch steps 1-5 in order (cache-path
// derive, build_from, reconcile, remote add/set-url, partialClone
// config). Splitting across helpers would obscure the step ordering
// the architecture-sketch fixes verbatim.
#[allow(clippy::too_many_lines)]
pub async fn run(args: AttachArgs) -> Result<()> {
    let spec = &args.spec;
    let work = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    if !work.join(".git").exists() {
        bail!("not a git working tree: {} (.git/ missing)", work.display());
    }

    // Step 1: derive cache identity + backend connector FROM the SoT spec
    // (Q1.1 — NOT from remote.origin.url). `translate_spec_to_url` produces
    // the canonical `reposix::…` URL, which the git remote helper's mature
    // dispatch factory (`reposix_remote::backend_dispatch`, D91-03) parses
    // into a `(kind, origin, project)` tuple. Reusing that factory means
    // sim/github/confluence/jira all construct through ONE code path — and
    // Confluence/JIRA inherit the OP-3 `.with_audit(…)` wiring for free
    // instead of a hand-rolled arm that could silently drop it.
    let translated = translate_spec_to_url(spec)?;
    let mut parsed = backend_dispatch::parse_remote_url(&translated)
        .with_context(|| format!("parse SoT url for spec `{spec}`"))?;

    // Q1.2 / Q1.3 — re-attach handling. Compare only the core SoT spec
    // (strip any `?mirror=` suffix on the stored URL).
    let existing_url = git_config_get(&work, &format!("remote.{}.url", args.remote_name))?;
    if let Some(existing) = &existing_url {
        let existing_sot = existing.split('?').next().unwrap_or(existing);
        let new_sot = translated.split('?').next().unwrap_or(&translated);
        if existing_sot != new_sot {
            bail!(
                "working tree already attached to {existing_sot}; multi-SoT not supported in v0.13.0 (Q1.2). \
                 Run `reposix detach` first or pick the existing SoT."
            );
        }
        // Same SoT → idempotent re-attach; fall through and refresh.
    }

    // Step 2: build the backend connector through the shared factory.
    // For sim, honour REPOSIX_SIM_ORIGIN so tests + alternate-port
    // deployments can point the REST round-trips (build_from + reconcile)
    // at a non-default port while the remote URL keeps the canonical
    // origin. Real backends read their credentials from the environment
    // inside the factory and surface a doc-linked error naming any unset
    // var (docs/reference/testing-targets.md).
    if parsed.kind == BackendKind::Sim {
        if let Some(origin) = std::env::var("REPOSIX_SIM_ORIGIN")
            .ok()
            .filter(|s| !s.is_empty())
        {
            parsed.origin = origin;
        }
    }
    let connector: Arc<dyn BackendConnector> = backend_dispatch::instantiate(&parsed)
        .with_context(|| format!("instantiate backend connector for `{spec}`"))?;

    let backend_slug = parsed.kind.slug();
    let cache_project = backend_dispatch::sanitize_project_for_cache(&parsed.project);

    let mut cache = reposix_cache::Cache::open(connector, backend_slug, &cache_project)
        .context("open cache")?;

    // Build the cache tree from REST. `build_from` populates `oid_map`
    // (filenames + tree OIDs) without materializing blobs — the lazy
    // contract that makes the partial-clone topology work.
    let _tree_oid = cache
        .build_from()
        .await
        .context("build cache from backend")?;

    // F07: initialize `last_fetched_at` to NOW on attach so the first push
    // after attach doesn't trigger a full `list_records` walk.
    // `Cache::build_from` already writes `last_fetched_at` to NOW as part of
    // its happy path (see crates/reposix-cache/src/builder.rs:119). Document
    // the dependency here so the contract stays visible.

    // Step 3: reconcile working tree against backend (DVCS-ATTACH-02).
    let ignore_list: Vec<String> = args
        .ignore
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let report = reposix_cache::reconciliation::walk_and_reconcile(
        &work,
        &mut cache,
        args.orphan_policy.into(),
        &ignore_list,
    )
    .context("reconcile working tree against backend")?;

    if !report.duplicate_id_files.is_empty() {
        let dup = &report.duplicate_id_files;
        bail!(
            "duplicate id across local records: {dup:?}; reconciliation aborted (no rows committed)"
        );
    }

    eprintln!(
        "attach: matched={} no_id={} backend_deleted={} mirror_lag={}",
        report.matched_count,
        report.no_id_count,
        report.backend_deleted_count,
        report.mirror_lag_count,
    );

    // OP-3 audit: write a single audit_events_cache row noting attach walk
    // completed. Regular (event_type, payload_json) signature per F04 so
    // P83's sibling `mirror_lag_partial_failure` audit hook can use the
    // same surface.
    let payload = serde_json::json!({
        "matched": report.matched_count,
        "no_id": report.no_id_count,
        "backend_deleted": report.backend_deleted_count,
        "mirror_lag": report.mirror_lag_count,
        "duplicate_ids_aborted": report.duplicate_id_files.len(),
    });
    cache
        .log_attach_walk("attach_walk", &payload)
        .context("write attach_walk audit row")?;

    // Step 4: compose remote URL + add (or update-if-idempotent) remote.
    let mirror_url = git_config_get(&work, &format!("remote.{}.url", args.mirror_name))?;
    let remote_url = match (args.no_bus, &mirror_url) {
        (false, Some(m)) => {
            // Security (Wave-5.5, credential-leak MEDIUM intake): never fold
            // embedded credentials (`user:token@`) into `remote.<name>.url`.
            // The bus URL is persisted in `.git/config` AND printed by git on
            // every push (`To reposix::…?mirror=…`), so a token-in-URL origin
            // would leak into plaintext config + stderr/logs — a lethal-
            // trifecta exfiltration leg. Auth is unaffected: the actual
            // mirror push uses the LOCAL remote name (whose own URL keeps
            // its credentials); the folded URL is only used for matching +
            // drift prechecks.
            let (clean, had_creds) = reposix_core::http::strip_url_userinfo(m);
            if had_creds {
                eprintln!(
                    "attach: warning: remote `{}`'s URL embeds credentials; they were NOT \
                     copied into remote.{}.url. Prefer an SSH origin or a git credential \
                     helper for the mirror remote.",
                    args.mirror_name, args.remote_name
                );
            }
            format!("{translated}?mirror={clean}")
        }
        // --no-bus, or no mirror remote configured (user can add one later).
        (true, _) | (false, None) => translated.clone(),
    };
    if existing_url.is_some() {
        run_git_in(
            &work,
            &["remote", "set-url", &args.remote_name, &remote_url],
        )?;
    } else {
        run_git_in(&work, &["remote", "add", &args.remote_name, &remote_url])?;
    }

    // Step 5: set extensions.partialClone=<remote-name> (NOT origin).
    run_git_in(
        &work,
        &["config", "extensions.partialClone", &args.remote_name],
    )?;

    println!(
        "reposix attach: configured `{}` against {spec}\n  remote `{}`.url = {remote_url}\n  extensions.partialClone = {}",
        work.display(),
        args.remote_name,
        args.remote_name,
    );

    Ok(())
}

/// Read `git -C <work> config --get <key>`. Returns `Ok(None)` when the
/// key is absent (git config exits 1 in that case — not a real error).
fn git_config_get(work: &Path, key: &str) -> Result<Option<String>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(work)
        .args(["config", "--get", key])
        .output()
        .with_context(|| format!("invoke `git config --get {key}` in {}", work.display()))?;
    if out.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&out.stdout).trim().to_string(),
        ))
    } else {
        Ok(None)
    }
}

/// Run `git -C <work> <args...>`; bail on non-zero exit with stderr.
fn run_git_in(work: &Path, args: &[&str]) -> Result<()> {
    let out = Command::new("git")
        .arg("-C")
        .arg(work)
        .args(args)
        .output()
        .with_context(|| format!("invoke `git {}` in {}", args.join(" "), work.display()))?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}
