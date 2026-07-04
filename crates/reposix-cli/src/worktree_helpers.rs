//! Worktree-context helpers shared by the `reposix {doctor, history, gc,
//! tokens, cost, sync}` subcommands.
//!
//! Each subcommand needs to (1) resolve the working tree's reposix `SoT`
//! remote URL, (2) parse it into a `RemoteSpec`, (3) map the origin to a
//! backend slug, and (4) resolve the corresponding cache directory. Before
//! this module they each defined verbatim copies of the trio
//! (`cache_path_from_worktree`, `backend_slug_from_origin`,
//! `git_config_get`); this module is the shared home that consolidates
//! them.
//!
//! Remote resolution is partialClone-aware (QL-004): it reads the remote
//! named by `extensions.partialClone` (`origin` for `reposix init`,
//! `<remote-name>` for `reposix attach`), falling back to a reposix-URL
//! scan and finally `remote.origin.url`. See [`resolve_reposix_remote_url`].

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

/// Strip a bus URL's `?mirror=<mirror-url>` query so only the `SoT` spec
/// remains. The first unescaped `?` is the query boundary
/// (`bus_url::parse` in `reposix-remote` uses the same rule). For a
/// single-backend URL (no `?`) this is a no-op.
///
/// The `SoT` spec is what [`parse_remote_url`] and [`backend_slug_from_origin`]
/// expect — the mirror half is a plain-git URL that is meaningless to the
/// cache-path resolver.
#[must_use]
pub fn strip_bus_query(url: &str) -> &str {
    url.split('?').next().unwrap_or(url)
}

/// Whether `url` is (or wraps) a reposix `SoT` remote URL — i.e. carries
/// the `reposix::` scheme or otherwise parses to a `/projects/<slug>`
/// spec. Bus URLs (`reposix::<sot>?mirror=<url>`) are recognised via their
/// `SoT` half. Plain-git mirror URLs (`git@github.com:org/repo.git`,
/// `https://…/repo.git`) do NOT match — they have no `/projects/` segment.
#[must_use]
pub fn looks_like_reposix_url(url: &str) -> bool {
    let sot = strip_bus_query(url);
    sot.starts_with("reposix::") || parse_remote_url(sot).is_ok()
}

/// Resolve the reposix `SoT` remote URL for a working tree, handling both
/// bootstrap shapes:
///
/// 1. **`extensions.partialClone`** names the reposix remote (init sets it
///    to `origin`; attach sets it to `--remote-name`, default `reposix`).
///    Read `remote.<that>.url` — this is the authoritative binding.
/// 2. **Scan** all remotes for the first (alphabetically) whose URL looks
///    like a reposix URL. Covers hand-edited trees where `partialClone`
///    was never set.
/// 3. **Fallback** to `remote.origin.url` (the pre-attach behaviour), even
///    if it doesn't look reposix — so the caller's parse step produces the
///    same diagnostic it used to.
///
/// Returns the RAW URL (bus `?mirror=` query intact); callers strip it via
/// [`strip_bus_query`] before parsing.
#[must_use]
pub fn resolve_reposix_remote_url(work: &Path) -> Option<String> {
    // (1) extensions.partialClone → remote.<name>.url
    if let Some(remote_name) = git_config_get(work, "extensions.partialClone") {
        if let Some(url) = git_config_get(work, &format!("remote.{remote_name}.url")) {
            return Some(url);
        }
    }
    // (2) scan every remote for a reposix-looking URL (deterministic order).
    if let Some(url) = scan_remotes_for_reposix(work) {
        return Some(url);
    }
    // (3) fallback: origin (pre-attach behaviour).
    git_config_get(work, "remote.origin.url")
}

/// Scan `remote.<name>.url` config entries for the first (alphabetical)
/// value that [`looks_like_reposix_url`]. Returns `None` if the tree has
/// no reposix remote.
fn scan_remotes_for_reposix(work: &Path) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(work)
        .args(["config", "--get-regexp", r"^remote\..+\.url$"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut matches: Vec<(String, String)> = Vec::new();
    for line in stdout.lines() {
        // `remote.<name>.url <value>` — split on first whitespace (URL
        // values may contain whitespace, rare but legal).
        let mut parts = line.splitn(2, char::is_whitespace);
        let (Some(key), Some(value)) = (parts.next(), parts.next()) else {
            continue;
        };
        let Some(name) = key
            .strip_prefix("remote.")
            .and_then(|s| s.strip_suffix(".url"))
        else {
            continue;
        };
        if looks_like_reposix_url(value) {
            matches.push((name.to_owned(), value.to_owned()));
        }
    }
    matches.sort_by(|a, b| a.0.cmp(&b.0));
    matches.into_iter().next().map(|(_, url)| url)
}

/// Resolve the cache's bare-repo path from a working-tree directory.
///
/// Resolves the reposix `SoT` remote URL via [`resolve_reposix_remote_url`]
/// (handles both `reposix init` trees — `partialClone=origin` — and
/// `reposix attach` trees — `partialClone=<remote-name>`, plus the bus
/// `?mirror=` URL form), strips any bus query, parses it via
/// [`parse_remote_url`], maps the host to a backend slug via
/// [`backend_slug_from_origin`], and resolves the
/// `<cache_root>/<backend>-<project>.git` path via
/// [`reposix_cache::path::resolve_cache_path`]. Does NOT verify the cache
/// directory exists — callers that need that check should add it themselves.
///
/// # Errors
///
/// - The working tree has no reposix remote (`init`/`attach` never run).
/// - The URL fails to parse (no `/projects/<slug>` segment, etc.).
/// - The cache path cannot be resolved (no `$XDG_CACHE_HOME`/`$HOME`).
pub fn cache_path_from_worktree(work: &Path) -> Result<PathBuf> {
    let url = resolve_reposix_remote_url(work).ok_or_else(|| {
        anyhow!(
            "no reposix remote in {} — run `reposix init <backend>::<project> <path>` \
             to bootstrap a new tree, or `reposix attach <backend>::<project>` to adopt \
             an existing checkout",
            work.display()
        )
    })?;
    // Strip the bus `?mirror=` half; only the SoT spec resolves a cache.
    let sot = strip_bus_query(&url);
    let spec = parse_remote_url(sot).with_context(|| format!("parse reposix remote url={url}"))?;
    // Pass the full SoT URL so JIRA's `/jira/` marker is visible — origin
    // alone (just the host) loses the disambiguator.
    let backend = backend_slug_from_origin(sot);
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
        let msg = err.to_string();
        // Error must teach BOTH recovery paths (QL-004): init and attach.
        assert!(msg.contains("no reposix remote"), "got: {msg}");
        assert!(msg.contains("reposix init"), "init hint missing: {msg}");
        assert!(msg.contains("reposix attach"), "attach hint missing: {msg}");
    }

    // ── resolve_reposix_remote_url: bootstrap-shape matrix (QL-004) ──────

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn init_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        git(tmp.path(), &["init", "."]);
        tmp
    }

    #[test]
    fn resolve_init_shape_reads_origin_via_partial_clone() {
        // `reposix init`: partialClone=origin, remote.origin.url=reposix::...
        let tmp = init_repo();
        let url = "reposix::http://127.0.0.1:7878/projects/demo";
        git(tmp.path(), &["remote", "add", "origin", url]);
        git(tmp.path(), &["config", "extensions.partialClone", "origin"]);
        assert_eq!(resolve_reposix_remote_url(tmp.path()).as_deref(), Some(url));
        let cache = cache_path_from_worktree(tmp.path()).unwrap();
        assert!(
            cache.to_string_lossy().contains("sim-demo.git"),
            "{cache:?}"
        );
    }

    #[test]
    fn resolve_attach_shape_reads_named_remote_via_partial_clone() {
        // `reposix attach`: partialClone=reposix, remote.reposix.url=reposix::...
        // (origin is a plain-git mirror clone — must NOT be used).
        let tmp = init_repo();
        git(
            tmp.path(),
            &["remote", "add", "origin", "git@github.com:org/repo.git"],
        );
        let url = "reposix::https://reuben-john.atlassian.net/jira/projects/TEST";
        git(tmp.path(), &["remote", "add", "reposix", url]);
        git(
            tmp.path(),
            &["config", "extensions.partialClone", "reposix"],
        );
        assert_eq!(resolve_reposix_remote_url(tmp.path()).as_deref(), Some(url));
        let cache = cache_path_from_worktree(tmp.path()).unwrap();
        assert!(
            cache.to_string_lossy().contains("jira-TEST.git"),
            "{cache:?}"
        );
    }

    #[test]
    fn resolve_attach_bus_url_form_strips_mirror_query() {
        // Attach with a bus URL: partialClone=reposix, remote.reposix.url
        // carries `?mirror=<plain-git>`. The SoT half drives the cache.
        let tmp = init_repo();
        let bus = "reposix::https://reuben-john.atlassian.net/confluence/projects/TokenWorld?mirror=git@github.com:org/repo.git";
        git(tmp.path(), &["remote", "add", "reposix", bus]);
        git(
            tmp.path(),
            &["config", "extensions.partialClone", "reposix"],
        );
        assert_eq!(resolve_reposix_remote_url(tmp.path()).as_deref(), Some(bus));
        let cache = cache_path_from_worktree(tmp.path()).unwrap();
        assert!(
            cache
                .to_string_lossy()
                .contains("confluence-TokenWorld.git"),
            "{cache:?}"
        );
    }

    #[test]
    fn resolve_scans_when_partial_clone_unset() {
        // Hand-edited tree: no partialClone, but a reposix remote exists
        // alongside a plain-git one. Scan finds the reposix one.
        let tmp = init_repo();
        git(
            tmp.path(),
            &["remote", "add", "origin", "git@github.com:org/repo.git"],
        );
        let url = "reposix::http://127.0.0.1:7878/projects/demo";
        git(tmp.path(), &["remote", "add", "backend", url]);
        assert_eq!(resolve_reposix_remote_url(tmp.path()).as_deref(), Some(url));
    }

    #[test]
    fn looks_like_reposix_url_rejects_plain_git_mirror() {
        assert!(!looks_like_reposix_url("git@github.com:org/repo.git"));
        assert!(!looks_like_reposix_url("https://github.com/org/repo.git"));
        assert!(looks_like_reposix_url(
            "reposix::http://127.0.0.1:7878/projects/demo"
        ));
        assert!(looks_like_reposix_url(
            "reposix::https://reuben-john.atlassian.net/jira/projects/TEST?mirror=git@github.com:o/r.git"
        ));
    }
}
