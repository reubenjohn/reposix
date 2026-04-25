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
            Ok(format!(
                "reposix::https://{tenant}.atlassian.net/projects/{project}"
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
            Ok(format!(
                "reposix::https://{instance}.atlassian.net/projects/{project}"
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
/// # Errors
/// Returns an error if `spec` cannot be translated, if any of `git init`
/// or the four `git config` invocations fail, or if `git` is not on PATH.
/// The trailing `git fetch` is best-effort: a failure logs a warning but
/// does not prevent `init` from succeeding (the user may bring credentials
/// later).
pub fn run(spec: String, path: PathBuf) -> Result<()> {
    let url = translate_spec_to_url(&spec)?;

    // Ensure parent dir exists for `git init`. `git init` creates the leaf
    // dir but not intermediate parents.
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("create parent dir for {path}", path = path.display())
            })?;
        }
    }

    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))?;

    // 1. git init <path>
    run_git(&["init", path_str])?;
    // 2-5. configure partial clone + remote.
    run_git(&["-C", path_str, "config", "extensions.partialClone", "origin"])?;
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn translate_confluence_requires_tenant() {
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
