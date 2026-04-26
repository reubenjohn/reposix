//! Worktree-context helpers shared by the `reposix doctor`, `reposix history`,
//! `reposix gc`, and `reposix tokens` subcommands.
//!
//! All four subcommands need to (1) read the working tree's `remote.origin.url`,
//! (2) parse it into a `RemoteSpec`, (3) map the origin to a backend slug, and
//! (4) resolve the corresponding cache directory. Before this module they each
//! defined verbatim copies of the trio (`cache_path_from_worktree`,
//! `backend_slug_from_origin`, `git_config_get`); this module is the shared
//! home that consolidates them.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use reposix_cache::path::resolve_cache_path;
use reposix_core::parse_remote_url;

/// Read a single git config value via `git -C <path> config --get <key>`.
///
/// Returns `None` if the key is unset, the value is empty, or the git
/// invocation itself fails (e.g. not a git repo). This function is
/// deliberately tolerant — callers escalate to errors at the level
/// where the missing value matters.
#[must_use]
pub fn git_config_get(path: &Path, key: &str) -> Option<String> {
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

/// Map a remote `url` to the cache `backend` slug.
///
/// Mirrors the runtime mapping in `git-remote-reposix`. The order matters:
/// Atlassian URLs carry a `/jira/` or `/confluence/` path marker (see
/// `init.rs` lines 73 / 89) — check the markers FIRST so JIRA worktrees
/// route to the right cache. GitHub is identified by `api.github.com`.
/// Everything else falls through to `sim`.
///
/// Audit-finding: pre-v0.11.1 this took only the origin host and silently
/// returned "confluence" for every JIRA worktree, sending `reposix
/// {gc,tokens,cost,history,doctor}` at the wrong cache directory.
#[must_use]
pub fn backend_slug_from_origin(url: &str) -> String {
    if url.contains("/jira/") {
        "jira".to_string()
    } else if url.contains("/confluence/") {
        "confluence".to_string()
    } else if url.contains("api.github.com") {
        "github".to_string()
    } else if url.contains("atlassian.net") {
        // Bare atlassian.net with no marker — unusual but possible if a
        // user hand-wrote the URL. Best-effort default.
        "confluence".to_string()
    } else {
        "sim".to_string()
    }
}

/// Resolve the cache's bare-repo path from a working-tree directory.
///
/// Reads `remote.origin.url`, parses it via [`parse_remote_url`], maps the
/// host to a backend slug via [`backend_slug_from_origin`], and resolves the
/// `<cache_root>/<backend>-<project>.git` path via
/// [`reposix_cache::path::resolve_cache_path`]. Does NOT verify the cache
/// directory exists — callers that need that check should add it themselves.
///
/// # Errors
///
/// - The working tree has no `remote.origin.url` configured.
/// - The URL fails to parse (no `/projects/<slug>` segment, etc.).
/// - The cache path cannot be resolved (no `$XDG_CACHE_HOME`/`$HOME`).
pub fn cache_path_from_worktree(work: &Path) -> Result<PathBuf> {
    let url = git_config_get(work, "remote.origin.url").ok_or_else(|| {
        anyhow!(
            "no remote.origin.url in {} (run `reposix init` first)",
            work.display()
        )
    })?;
    let spec = parse_remote_url(&url).with_context(|| format!("parse remote.origin.url={url}"))?;
    // Pass the full URL so JIRA's `/jira/` marker is visible — origin
    // alone (just the host) loses the disambiguator.
    let backend = backend_slug_from_origin(&url);
    resolve_cache_path(&backend, spec.project.as_str()).with_context(|| {
        format!(
            "resolve cache path for ({backend}, {project})",
            project = spec.project
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_slug_mapping() {
        // Sim — local origin.
        assert_eq!(backend_slug_from_origin("http://127.0.0.1:7878"), "sim");
        // GitHub — api.github.com.
        assert_eq!(
            backend_slug_from_origin("reposix::https://api.github.com/projects/foo/bar"),
            "github"
        );
        // Confluence — `/confluence/` marker.
        assert_eq!(
            backend_slug_from_origin(
                "reposix::https://reuben-john.atlassian.net/confluence/projects/TokenWorld"
            ),
            "confluence"
        );
        // JIRA — `/jira/` marker (the v0.11.1 fix; previously silently
        // returned "confluence" and routed the wrong cache).
        assert_eq!(
            backend_slug_from_origin(
                "reposix::https://reuben-john.atlassian.net/jira/projects/TEST"
            ),
            "jira"
        );
        // Bare atlassian.net (no marker) — best-effort default.
        assert_eq!(
            backend_slug_from_origin("https://reuben-john.atlassian.net"),
            "confluence"
        );
    }

    #[test]
    fn git_config_get_returns_none_for_non_repo() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(git_config_get(tmp.path(), "remote.origin.url").is_none());
    }

    #[test]
    fn cache_path_from_worktree_errors_without_remote() {
        let tmp = tempfile::tempdir().unwrap();
        let err = cache_path_from_worktree(tmp.path()).unwrap_err();
        assert!(
            err.to_string().contains("no remote.origin.url"),
            "got: {err}"
        );
    }
}
