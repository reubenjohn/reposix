//! `reposix init <backend>::<project> <path>` — git-native partial-clone bootstrap.
//!
//! Replaces `reposix mount` (deleted in v0.9.0). Runs the six-step git
//! sequence locked in `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` §5:
//!
//! 1. `git init <path>`
//! 2. `git -C <path> config extensions.partialClone origin`
//! 3. `git -C <path> config remote.origin.url <url>`
//! 4. `git -C <path> config remote.origin.promisor true`
//! 5. `git -C <path> config remote.origin.partialclonefilter blob:none`
//! 6. `git -C <path> fetch --filter=blob:none origin` *(best-effort)*
//!
//! The translation from the friendly `<backend>::<project>` form to the
//! helper-compatible `reposix::<scheme>://<host>/projects/<project>` URL is
//! [`translate_spec_to_url`].

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

/// Default sim REST origin used when the user runs `reposix init sim::<slug>`.
/// Matches the default bind in `crates/reposix-sim` (`127.0.0.1:7878`).
const DEFAULT_SIM_ORIGIN: &str = "http://127.0.0.1:7878";

/// Default GitHub API origin for `github::<owner>/<repo>` specs.
const DEFAULT_GITHUB_ORIGIN: &str = "https://api.github.com";

/// Translate a friendly `<backend>::<project>` spec into a
/// `reposix::<scheme>://<host>/projects/<project>` URL the helper accepts.
///
/// Backends:
/// - `sim::<slug>` → uses [`DEFAULT_SIM_ORIGIN`].
/// - `github::<owner>/<repo>` → uses [`DEFAULT_GITHUB_ORIGIN`]; the project
///   slug is the full `<owner>/<repo>` pair.
/// - `confluence::<space>` → requires `REPOSIX_CONFLUENCE_TENANT`;
///   constructs `https://<tenant>.atlassian.net`.
/// - `jira::<key>` → requires `REPOSIX_JIRA_INSTANCE`;
///   constructs `https://<instance>.atlassian.net`.
///
/// # Errors
/// Returns an error if the spec is missing the `::` separator, the backend
/// is unknown, or a required env var (`REPOSIX_CONFLUENCE_TENANT` /
/// `REPOSIX_JIRA_INSTANCE`) is unset for confluence/jira.
pub fn translate_spec_to_url(spec: &str) -> Result<String> {
    let (backend, project) = spec
        .split_once("::")
        .ok_or_else(|| anyhow!("invalid spec `{spec}`: expected `<backend>::<project>` form"))?;

    if project.is_empty() {
        bail!("invalid spec `{spec}`: empty project");
    }

    match backend {
        "sim" => Ok(format!("reposix::{DEFAULT_SIM_ORIGIN}/projects/{project}")),
        "github" => Ok(format!(
            "reposix::{DEFAULT_GITHUB_ORIGIN}/projects/{project}"
        )),
        "confluence" => {
            let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT")
                .ok()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    anyhow!(
                        "REPOSIX_CONFLUENCE_TENANT must be set for `confluence::<space>` (subdomain of your Atlassian Cloud tenant)"
                    )
                })?;
            // Phase 36-followup: the `/confluence/` path marker
            // disambiguates the URL from JIRA at the helper's
            // backend-dispatch layer (both share the same
            // *.atlassian.net origin).
            Ok(format!(
                "reposix::https://{tenant}.atlassian.net/confluence/projects/{project}"
            ))
        }
        "jira" => {
            let instance = std::env::var("REPOSIX_JIRA_INSTANCE")
                .ok()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    anyhow!(
                        "REPOSIX_JIRA_INSTANCE must be set for `jira::<key>` (subdomain of your Atlassian Cloud tenant)"
                    )
                })?;
            // Phase 36-followup: the `/jira/` path marker
            // disambiguates the URL from Confluence at the helper's
            // backend-dispatch layer.
            Ok(format!(
                "reposix::https://{instance}.atlassian.net/jira/projects/{project}"
            ))
        }
        other => bail!(
            "unknown backend `{other}`: expected one of `sim`, `github`, `confluence`, `jira`"
        ),
    }
}

/// Run `git <args...>` and return a useful error on non-zero exit.
fn run_git(args: &[&str]) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    let out = cmd.output().with_context(|| {
        format!(
            "failed to spawn `git {}` (is git installed and on PATH?)",
            args.join(" ")
        )
    })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        bail!(
            "`git {}` failed with status {}: {}",
            args.join(" "),
            out.status,
            stderr.trim()
        );
    }
    Ok(())
}

/// Run `git -C <path> <args...>` (best-effort variant).
///
/// Intended for the trailing `git fetch` step where a credential failure
/// (real backend without env vars) should not fail the whole `init`. The
/// caller controls whether to bail on error.
fn run_git_in(path: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path).args(args);
    cmd.output()
}

/// `reposix init` entry point.
///
/// `since` is an optional RFC-3339 timestamp. When set, after the normal
/// `git fetch` completes, the working tree's HEAD is rewound to the
/// closest cache sync tag at-or-before the timestamp. Errors clearly
/// when no sync tag exists at-or-before the target.
///
/// # Errors
/// Returns an error if `spec` cannot be translated, if any of `git init`
/// or the four `git config` invocations fail, or if `git` is not on PATH.
/// The trailing `git fetch` is best-effort: a failure logs a warning but
/// does not prevent `init` from succeeding (the user may bring credentials
/// later). When `since` is set and no matching sync tag exists, `init`
/// errors with a non-zero exit (after configuring the working tree).
pub fn run(spec: String, path: PathBuf) -> Result<()> {
    run_with_since(spec, path, None)
}

/// `reposix init --since=<RFC3339>` entry point.
///
/// Same as [`run`] except that, after the normal `git fetch` completes,
/// `since` (if `Some`) selects the closest cache sync tag at-or-before
/// the target and rewinds the working tree's HEAD + `refs/remotes/origin/main`
/// to that historical commit.
///
/// # Errors
/// Same as [`run`], plus:
/// - `since` is not a valid RFC-3339 timestamp.
/// - No sync tag exists at-or-before `since` in the cache.
/// - The local `git fetch <cache-path> <oid>` to bring the historical
///   commit into the working tree fails.
pub fn run_with_since(spec: String, path: PathBuf, since: Option<String>) -> Result<()> {
    let url = translate_spec_to_url(&spec)?;

    // Ensure parent dir exists for `git init`. `git init` creates the leaf
    // dir but not intermediate parents.
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create parent dir for {path}", path = path.display()))?;
        }
    }

    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))?;

    // 1. git init <path>
    run_git(&["init", path_str])?;
    // 2-5. configure partial clone + remote.
    run_git(&[
        "-C",
        path_str,
        "config",
        "extensions.partialClone",
        "origin",
    ])?;
    run_git(&["-C", path_str, "config", "remote.origin.url", &url])?;
    run_git(&["-C", path_str, "config", "remote.origin.promisor", "true"])?;
    run_git(&[
        "-C",
        path_str,
        "config",
        "remote.origin.partialclonefilter",
        "blob:none",
    ])?;

    // 6. git fetch --filter=blob:none origin (best-effort).
    let out = run_git_in(&path, &["fetch", "--filter=blob:none", "origin"]);
    match out {
        Ok(o) if o.status.success() => {
            tracing::info!("git fetch --filter=blob:none succeeded");
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            tracing::warn!(
                "git fetch --filter=blob:none failed with status {} — local repo is configured but not yet synced. Stderr: {}",
                o.status,
                stderr.trim()
            );
        }
        Err(e) => {
            tracing::warn!("could not invoke git fetch: {e}");
        }
    }

    println!(
        "reposix init: configured `{path_str}` with remote.origin.url = {url}\nNext: cd {path_str} && git checkout origin/main (or git sparse-checkout set <pathspec> first)"
    );

    // --since=<RFC3339> handling — rewind the working tree to a historical
    // sync tag from the cache. Runs AFTER the normal fetch so the cache is
    // populated and contains the tag refs.
    if let Some(ts) = since {
        rewind_to_since(&spec, &path, &ts)?;
    }

    // Record the working-tree path in the cache's meta table so
    // `reposix gc --orphans` can detect caches whose owning working tree
    // has since been deleted. Best-effort: if the cache doesn't exist yet
    // (network-less init, fetch failed), this silently no-ops — the next
    // successful fetch will trigger Cache::open and the same recording
    // call from a subsequent `reposix init` will fix it up.
    record_worktree_in_cache(&spec, &path);

    Ok(())
}

/// Append the absolute working-tree path to the cache's `meta.worktrees`
/// row. Best-effort: a missing cache, an unparseable path, or a SQL error
/// downgrades to a tracing warning. Used by `reposix gc --orphans` to
/// detect caches whose owning working trees have been deleted.
///
/// `meta.worktrees` is stored as a newline-separated list of absolute
/// paths. Duplicates are deduped on insert; ordering is insertion-order
/// for forensics.
fn record_worktree_in_cache(spec: &str, path: &Path) {
    let Some((backend, project)) = spec.split_once("::") else {
        return;
    };
    let cache_project = if backend == "github" {
        project.replace('/', "-")
    } else {
        project.to_string()
    };
    let Ok(cache_path) = reposix_cache::resolve_cache_path(backend, &cache_project) else {
        return;
    };
    let db = cache_path.join("cache.db");
    if !db.exists() {
        return;
    }
    let abs_path = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    };
    let abs_str = match abs_path.to_str() {
        Some(s) => s.to_string(),
        None => return,
    };

    let conn = match rusqlite::Connection::open(&db) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "could not open {db} to record worktree: {e}",
                db = db.display()
            );
            return;
        }
    };
    let existing: Option<String> = conn
        .query_row("SELECT value FROM meta WHERE key = 'worktrees'", [], |r| {
            r.get::<_, String>(0)
        })
        .ok();
    let mut entries: Vec<String> = existing
        .as_deref()
        .map(|s| s.lines().map(str::to_string).collect())
        .unwrap_or_default();
    if !entries.iter().any(|e| e == &abs_str) {
        entries.push(abs_str);
    }
    let value = entries.join("\n");
    let now = chrono::Utc::now().to_rfc3339();
    if let Err(e) = conn.execute(
        "INSERT INTO meta (key, value, updated_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
        rusqlite::params!["worktrees", value, now],
    ) {
        tracing::warn!("could not write meta.worktrees in {}: {e}", db.display());
    }
}

/// Resolve the cache path for `spec`, look up the closest sync tag at-or-before
/// `target_rfc3339`, and rewind the working tree's `refs/heads/main` +
/// `refs/remotes/origin/main` to that commit. Errors clearly if no
/// matching tag is found.
fn rewind_to_since(spec: &str, path: &Path, target_rfc3339: &str) -> Result<()> {
    use chrono::{DateTime, Utc};

    let target: DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(target_rfc3339)
        .with_context(|| {
            format!(
                "invalid --since timestamp `{target_rfc3339}` — expected RFC-3339 (e.g. 2026-04-25T01:00:00Z)"
            )
        })?
        .with_timezone(&Utc);

    // Map spec → (backend, project) for the cache resolver. We re-derive
    // here rather than calling translate_spec_to_url + parse_remote_url
    // because the cache path keying uses the friendly slug directly.
    let (backend, project) = spec
        .split_once("::")
        .ok_or_else(|| anyhow!("invalid spec `{spec}`: expected `<backend>::<project>` form"))?;
    // GitHub uses `owner/repo` in the spec but `owner-repo` as the cache
    // dir name (sanitize_project_for_cache); mirror that here.
    let cache_project = if backend == "github" {
        project.replace('/', "-")
    } else {
        project.to_string()
    };
    let cache_path = reposix_cache::resolve_cache_path(backend, &cache_project)
        .with_context(|| format!("resolve cache path for {backend}::{cache_project}"))?;
    if !cache_path.exists() {
        bail!(
            "no cache at {} — run `reposix init` without --since first to populate it",
            cache_path.display()
        );
    }

    let tags = reposix_cache::list_sync_tags_at(&cache_path)
        .with_context(|| format!("list sync tags from {}", cache_path.display()))?;
    let chosen = tags.into_iter().rev().find(|t| t.timestamp <= target);
    let tag = chosen.ok_or_else(|| {
        anyhow!(
            "no sync tag at-or-before `{target_rfc3339}` in {} — try a later timestamp or omit --since",
            cache_path.display()
        )
    })?;

    let oid_hex = tag.commit.to_hex().to_string();

    // Bring the historical commit into the working tree's object store.
    // Local-path fetch by SHA works against the cache's bare repo regardless
    // of `transfer.hideRefs` because we name the OID, not the hidden ref.
    let cache_str = cache_path
        .to_str()
        .ok_or_else(|| anyhow!("cache path is not valid UTF-8: {}", cache_path.display()))?;
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow!("working-tree path is not valid UTF-8: {}", path.display()))?;
    let fetch_out = Command::new("git")
        .arg("-C")
        .arg(path_str)
        .args(["fetch", "--filter=blob:none", cache_str, &oid_hex])
        .output()
        .with_context(|| {
            format!("invoke `git fetch --filter=blob:none {cache_str} {oid_hex}` from {path_str}")
        })?;
    if !fetch_out.status.success() {
        bail!(
            "git fetch of historical commit {oid} from cache {cache} failed: {stderr}",
            oid = oid_hex,
            cache = cache_path.display(),
            stderr = String::from_utf8_lossy(&fetch_out.stderr).trim()
        );
    }

    // Update the working tree's main + origin/main refs to the historical
    // commit so `git checkout main` puts the agent at the snapshot.
    for refname in ["refs/heads/main", "refs/remotes/origin/main"] {
        let out = Command::new("git")
            .arg("-C")
            .arg(path_str)
            .args(["update-ref", refname, &oid_hex])
            .output()
            .with_context(|| format!("update-ref {refname} -> {oid_hex}"))?;
        if !out.status.success() {
            bail!(
                "git update-ref {refname} {oid_hex} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
    }

    println!(
        "reposix init --since={target_rfc3339}: rewound to sync tag {tag} (commit {oid_short})\n      cache: {cache}",
        tag = tag.name,
        oid_short = oid_hex.chars().take(12).collect::<String>(),
        cache = cache_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests that mutate process-wide env vars must run serially; cargo test
    // spawns one thread per test, so concurrent set_var/remove_var races.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn translate_sim_spec() {
        let url = translate_spec_to_url("sim::demo").unwrap();
        assert_eq!(url, "reposix::http://127.0.0.1:7878/projects/demo");
    }

    #[test]
    fn translate_github_spec() {
        let url = translate_spec_to_url("github::reubenjohn/reposix").unwrap();
        assert_eq!(
            url,
            "reposix::https://api.github.com/projects/reubenjohn/reposix"
        );
    }

    #[test]
    fn translate_confluence_emits_path_marker() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Phase 36-followup: the `/confluence/` path marker is what
        // the helper's URL-scheme dispatcher uses to disambiguate
        // between Confluence and JIRA on the shared *.atlassian.net
        // origin. Pin it here so init/helper stay in sync.
        let saved = std::env::var("REPOSIX_CONFLUENCE_TENANT").ok();
        std::env::set_var("REPOSIX_CONFLUENCE_TENANT", "reuben-john");
        let url = translate_spec_to_url("confluence::TokenWorld").unwrap();
        assert_eq!(
            url,
            "reposix::https://reuben-john.atlassian.net/confluence/projects/TokenWorld"
        );
        match saved {
            Some(v) => std::env::set_var("REPOSIX_CONFLUENCE_TENANT", v),
            None => std::env::remove_var("REPOSIX_CONFLUENCE_TENANT"),
        }
    }

    #[test]
    fn translate_jira_emits_path_marker() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = std::env::var("REPOSIX_JIRA_INSTANCE").ok();
        std::env::set_var("REPOSIX_JIRA_INSTANCE", "reuben-john");
        let url = translate_spec_to_url("jira::TEST").unwrap();
        assert_eq!(
            url,
            "reposix::https://reuben-john.atlassian.net/jira/projects/TEST"
        );
        match saved {
            Some(v) => std::env::set_var("REPOSIX_JIRA_INSTANCE", v),
            None => std::env::remove_var("REPOSIX_JIRA_INSTANCE"),
        }
    }

    #[test]
    fn translate_confluence_requires_tenant() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Save and clear the env var to ensure this test is deterministic.
        let saved = std::env::var("REPOSIX_CONFLUENCE_TENANT").ok();
        std::env::remove_var("REPOSIX_CONFLUENCE_TENANT");
        let err = translate_spec_to_url("confluence::TokenWorld").unwrap_err();
        assert!(
            err.to_string().contains("REPOSIX_CONFLUENCE_TENANT"),
            "expected error to name env var, got: {err}"
        );
        if let Some(v) = saved {
            std::env::set_var("REPOSIX_CONFLUENCE_TENANT", v);
        }
    }

    #[test]
    fn translate_jira_requires_instance() {
        let _guard = ENV_LOCK.lock().unwrap();
        let saved = std::env::var("REPOSIX_JIRA_INSTANCE").ok();
        std::env::remove_var("REPOSIX_JIRA_INSTANCE");
        let err = translate_spec_to_url("jira::TEST").unwrap_err();
        assert!(
            err.to_string().contains("REPOSIX_JIRA_INSTANCE"),
            "expected error to name env var, got: {err}"
        );
        if let Some(v) = saved {
            std::env::set_var("REPOSIX_JIRA_INSTANCE", v);
        }
    }

    #[test]
    fn translate_rejects_missing_separator() {
        let err = translate_spec_to_url("sim").unwrap_err();
        assert!(
            err.to_string().contains("expected `<backend>::<project>`"),
            "got: {err}"
        );
    }

    #[test]
    fn translate_rejects_unknown_backend() {
        let err = translate_spec_to_url("foo::bar").unwrap_err();
        assert!(
            err.to_string().contains("unknown backend `foo`"),
            "got: {err}"
        );
    }

    #[test]
    fn translate_rejects_empty_project() {
        let err = translate_spec_to_url("sim::").unwrap_err();
        assert!(err.to_string().contains("empty project"), "got: {err}");
    }
}
