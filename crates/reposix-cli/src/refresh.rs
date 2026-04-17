//! `reposix refresh` — re-fetch backend issues, write `.md` files, git commit.
//!
//! After this command the mount directory is a git working tree whose `git
//! log` is a history of backend snapshots.  `git diff HEAD~1` shows what
//! changed at the backend since the last refresh.
//!
//! # Errors
//! Every public function documents its error conditions.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context as _, Result};
use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
use reposix_core::backend::sim::SimBackend;
use reposix_core::BackendConnector as _;
use reposix_github::GithubReadOnlyBackend;
use reposix_jira::{JiraBackend, JiraCreds};

use crate::cache_db;
use crate::list::ListBackend;

/// Configuration for a single `reposix refresh` run.
pub struct RefreshConfig {
    /// Mount point (a plain directory that is also, or will become, a git
    /// working tree).
    pub mount_point: PathBuf,
    /// Backend origin URL (simulator URL; ignored for github/confluence).
    pub origin: String,
    /// Project slug — sim project name, `owner/repo` for GitHub, or space KEY
    /// for Confluence.
    pub project: String,
    /// Which backend to speak.
    pub backend: ListBackend,
    /// When `true`, skip network egress and serve from cached `.md` files.
    /// Currently returns an error (offline read path is Phase 21).
    pub offline: bool,
}

impl RefreshConfig {
    /// Return a short, static label for the active backend.
    #[must_use]
    pub fn backend_label(&self) -> &'static str {
        match self.backend {
            ListBackend::Sim => "simulator",
            ListBackend::Github => "github",
            ListBackend::Confluence => "confluence",
            ListBackend::Jira => "jira",
        }
    }
}

/// Execute `reposix refresh`:
///
/// 1. Guard against `--offline` (not yet implemented) and active FUSE mounts.
/// 2. Open (or create) `.reposix/cache.db`.
/// 3. Fetch all issues from the configured backend.
/// 4. Delegate the rest to [`run_refresh_inner`].
///
/// # Errors
///
/// - `--offline` is set: returns a not-yet-implemented error.
/// - FUSE is active (live `.reposix/fuse.pid`): returns an error telling the
///   user to unmount first.
/// - Backend network call fails: propagated from the backend.
/// - Propagates any error from [`run_refresh_inner`].
pub async fn run_refresh(cfg: RefreshConfig) -> Result<()> {
    if cfg.offline {
        bail!(
            "--offline mode is not yet implemented for refresh; \
             serve existing .md files from the mount directly (Phase 21)"
        );
    }

    if is_fuse_active(&cfg.mount_point)? {
        bail!(
            "FUSE mount is active at {}; run `reposix unmount` first, then refresh",
            cfg.mount_point.display()
        );
    }

    // Open (or create) the metadata DB — this also acquires the advisory lock.
    let db = cache_db::open_cache_db(&cfg.mount_point)?;

    // Fetch issues from the configured backend.
    let issues = fetch_issues(&cfg).await?;

    run_refresh_inner(&cfg, issues, Some(&db))
}

/// Inner refresh logic: write `.md` files, update timestamps, commit.
///
/// Separated from [`run_refresh`] so integration tests can supply a
/// pre-built `Vec<Issue>` without needing a live network backend.
///
/// # Errors
///
/// - `frontmatter::render` fails: propagated.
/// - Any git subprocess exits non-zero: propagated.
/// - `cache.db` update fails: propagated.
pub fn run_refresh_inner(
    cfg: &RefreshConfig,
    issues: Vec<reposix_core::Issue>,
    db: Option<&crate::cache_db::CacheDb>,
) -> Result<()> {
    let n = issues.len();

    // Determine the bucket directory name.
    let bucket = match cfg.backend {
        ListBackend::Confluence => "pages",
        ListBackend::Sim | ListBackend::Github | ListBackend::Jira => "issues",
    };

    // Ensure the .reposix and bucket directories exist.
    let reposix_dir = cfg.mount_point.join(".reposix");
    std::fs::create_dir_all(&reposix_dir)
        .with_context(|| format!("create .reposix dir {}", reposix_dir.display()))?;

    let bucket_dir = cfg.mount_point.join(bucket);
    std::fs::create_dir_all(&bucket_dir)
        .with_context(|| format!("create bucket dir {}", bucket_dir.display()))?;

    // Write one .md file per issue.
    for issue in &issues {
        let rendered =
            reposix_core::frontmatter::render(issue).context("render issue frontmatter")?;
        let filename = format!("{:011}.md", issue.id.0);
        let dest = bucket_dir.join(&filename);
        std::fs::write(&dest, rendered.as_bytes())
            .with_context(|| format!("write {}", dest.display()))?;
    }

    // Write the fetched_at sentinel.
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let fetched_at_path = reposix_dir.join("fetched_at.txt");
    std::fs::write(&fetched_at_path, &ts)
        .with_context(|| format!("write {}", fetched_at_path.display()))?;

    // Write .reposix/.gitignore (commit alongside fetched_at.txt).
    let reposix_gitignore = reposix_dir.join(".gitignore");
    std::fs::write(
        &reposix_gitignore,
        "cache.db\ncache.db-wal\ncache.db-shm\nfuse.pid\n",
    )
    .with_context(|| format!("write {}", reposix_gitignore.display()))?;

    // Build the commit message and author.
    let label = cfg.backend_label();
    let message = format!(
        "reposix refresh: {label}/{project} — {n} issues at {ts}",
        project = cfg.project
    );
    // Sanitize project so that only alphanumeric, `-`, `_`, and `/` appear in
    // the git author email field.  Any other character (including newlines or
    // `<`/`>` which are structurally significant in git) is replaced with `-`
    // to prevent a malformed `--author=` argument.
    let safe_project = cfg.project.replace(
        |c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '/',
        "-",
    );
    let author = format!("reposix <{label}@{safe_project}>");

    git_refresh_commit(&cfg.mount_point, bucket, &author, &message)?;

    // Update the metadata DB with the refresh result if one is provided.
    if let Some(db) = db {
        // TODO(Phase-21): populate commit_sha after git_refresh_commit returns
        cache_db::update_metadata(db, label, &cfg.project, &ts, None)?;
    }

    println!("refreshed {n} issues into {}", cfg.mount_point.display());
    Ok(())
}

/// Fetch issues from whichever backend `cfg` selects.
///
/// # Errors
///
/// Propagates backend construction or network errors.
async fn fetch_issues(cfg: &RefreshConfig) -> Result<Vec<reposix_core::Issue>> {
    match cfg.backend {
        ListBackend::Sim => {
            let b = SimBackend::new(cfg.origin.clone()).context("build SimBackend")?;
            b.list_issues(&cfg.project)
                .await
                .with_context(|| format!("sim list_issues project={}", cfg.project))
        }
        ListBackend::Github => {
            let token = std::env::var("GITHUB_TOKEN").ok();
            let b = GithubReadOnlyBackend::new(token).context("build GithubReadOnlyBackend")?;
            b.list_issues(&cfg.project).await.with_context(|| {
                format!(
                    "github list_issues repo={} \
                     (REPOSIX_ALLOWED_ORIGINS must include https://api.github.com)",
                    cfg.project
                )
            })
        }
        ListBackend::Confluence => {
            let email = std::env::var("ATLASSIAN_EMAIL").unwrap_or_default();
            let token = std::env::var("ATLASSIAN_API_KEY").unwrap_or_default();
            let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").unwrap_or_default();
            if email.is_empty() || token.is_empty() || tenant.is_empty() {
                bail!(
                    "confluence backend requires ATLASSIAN_EMAIL, ATLASSIAN_API_KEY, \
                     and REPOSIX_CONFLUENCE_TENANT env vars"
                );
            }
            let creds = ConfluenceCreds {
                email,
                api_token: token,
            };
            let b = ConfluenceBackend::new(creds, &tenant).context("build ConfluenceBackend")?;
            b.list_issues(&cfg.project).await.with_context(|| {
                format!(
                    "confluence list_issues space_key={} \
                     (REPOSIX_ALLOWED_ORIGINS must include https://{tenant}.atlassian.net)",
                    cfg.project
                )
            })
        }
        ListBackend::Jira => {
            let email = std::env::var("JIRA_EMAIL").unwrap_or_default();
            let token = std::env::var("JIRA_API_TOKEN").unwrap_or_default();
            let instance = std::env::var("REPOSIX_JIRA_INSTANCE").unwrap_or_default();
            if email.is_empty() || token.is_empty() || instance.is_empty() {
                bail!(
                    "jira backend requires JIRA_EMAIL, JIRA_API_TOKEN, \
                     and REPOSIX_JIRA_INSTANCE env vars"
                );
            }
            let creds = JiraCreds {
                email,
                api_token: token,
            };
            let b = JiraBackend::new(creds, &instance).context("build JiraBackend")?;
            b.list_issues(&cfg.project).await.with_context(|| {
                format!(
                    "jira list_issues project_key={} \
                     (REPOSIX_ALLOWED_ORIGINS must include https://{instance}.atlassian.net)",
                    cfg.project
                )
            })
        }
    }
}

/// Check whether a live FUSE daemon is currently mounted at `mount`.
///
/// Uses `.reposix/fuse.pid` as the sentinel.  If the file does not exist,
/// the mount is considered inactive.  If the file exists and the PID is alive
/// (`test_kill_process` returns `Ok`), the mount is active.  If the PID is
/// dead (ESRCH), the pid file is stale and the mount is considered inactive.
///
/// # Errors
///
/// - IO errors reading the pid file (other than `NotFound`).
/// - Malformed PID (not a valid `u32`).
/// - Unexpected `kill(pid, 0)` errors (not ESRCH).
pub fn is_fuse_active(mount: &Path) -> Result<bool> {
    let pid_path = mount.join(".reposix").join("fuse.pid");
    let content = match std::fs::read_to_string(&pid_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e).with_context(|| format!("read {}", pid_path.display())),
    };

    // Parse as i32 directly: Linux PIDs fit in i32 (max 4_194_304).
    let raw: i32 = content
        .trim()
        .parse()
        .with_context(|| format!("parse PID from {}", pid_path.display()))?;

    let pid = rustix::process::Pid::from_raw(raw).ok_or_else(|| {
        anyhow::anyhow!("fuse.pid contains PID 0, which is not a valid process id")
    })?;

    match rustix::process::test_kill_process(pid) {
        Ok(()) => Ok(true),
        Err(e) if e == rustix::io::Errno::SRCH => Ok(false),
        // EPERM: process exists but is owned by a different user — treat as alive.
        Err(e) if e == rustix::io::Errno::PERM => Ok(true),
        Err(e) => Err(anyhow::anyhow!(e)).context("test_kill_process"),
    }
}

/// Run git operations to stage the refresh output and create a commit.
///
/// `git init` is idempotent.  Uses `--allow-empty` so a second refresh with
/// no backend changes still produces a commit (each refresh is a snapshot).
///
/// # Errors
///
/// Returns an error if any git subprocess exits with a non-zero status.
pub fn git_refresh_commit(mount: &Path, bucket: &str, author: &str, message: &str) -> Result<()> {
    // Helper: run a git command in the mount directory.
    let g = |args: &[&str]| -> Result<()> {
        let status = Command::new("git")
            .arg("-C")
            .arg(mount)
            .args(args)
            .env("GIT_AUTHOR_NAME", "reposix")
            .env("GIT_COMMITTER_NAME", "reposix")
            // GIT_AUTHOR_EMAIL / GIT_COMMITTER_EMAIL are set on commit only
            // to avoid overriding the user's global config needlessly.
            .status()
            .with_context(|| format!("spawn git {args:?}"))?;
        if status.success() {
            Ok(())
        } else {
            bail!("git {args:?} failed: {status}")
        }
    };

    // `git init` is idempotent; -b main sets the default branch on first init.
    {
        let status = Command::new("git")
            .arg("-C")
            .arg(mount)
            .args(["-c", "init.defaultBranch=main", "init"])
            .status()
            .context("spawn git init")?;
        if !status.success() {
            bail!("git init failed: {status}");
        }
    }

    g(&[
        "add",
        "--",
        bucket,
        ".reposix/fetched_at.txt",
        ".reposix/.gitignore",
    ])?;

    // Commit with explicit author env vars so the commit works in bare CI
    // environments without a global git user.email configured.
    let status = Command::new("git")
        .arg("-C")
        .arg(mount)
        .args([
            "commit",
            "--allow-empty",
            &format!("--author={author}"),
            "-m",
            message,
        ])
        .env("GIT_AUTHOR_NAME", "reposix")
        .env("GIT_AUTHOR_EMAIL", "reposix@localhost")
        .env("GIT_COMMITTER_NAME", "reposix")
        .env("GIT_COMMITTER_EMAIL", "reposix@localhost")
        .status()
        .context("spawn git commit")?;
    if !status.success() {
        bail!("git commit failed: {status}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;
    use tempfile::tempdir;

    /// `is_fuse_active` returns `true` when fuse.pid contains the current
    /// process's PID (which is definitely alive).
    #[test]
    fn fuse_active_with_live_pid() {
        let dir = tempdir().unwrap();
        let reposix_dir = dir.path().join(".reposix");
        std::fs::create_dir_all(&reposix_dir).unwrap();
        let pid = std::process::id(); // current process — definitely alive
        std::fs::write(reposix_dir.join("fuse.pid"), pid.to_string()).unwrap();
        assert!(
            is_fuse_active(dir.path()).expect("is_fuse_active"),
            "current process PID must be alive"
        );
    }

    /// `is_fuse_active` returns `false` when `.reposix/fuse.pid` does not exist.
    #[test]
    fn fuse_inactive_no_pid_file() {
        let dir = tempdir().unwrap();
        assert!(
            !is_fuse_active(dir.path()).expect("is_fuse_active"),
            "no fuse.pid → inactive"
        );
    }

    /// `is_fuse_active` returns `false` when fuse.pid contains a PID that
    /// does not exist (e.g. 99999999).
    #[test]
    fn fuse_inactive_dead_pid() {
        let dir = tempdir().unwrap();
        let reposix_dir = dir.path().join(".reposix");
        std::fs::create_dir_all(&reposix_dir).unwrap();
        // PID 99999999 is well above Linux's PID_MAX (default 32768 / max
        // 4194304) and will never be a real process.
        std::fs::write(reposix_dir.join("fuse.pid"), "99999999").unwrap();
        let result = is_fuse_active(dir.path());
        // Accept either Ok(false) (ESRCH — process not found) or an error if
        // the kernel rejects the PID. The important thing is it's NOT Ok(true).
        assert!(
            !matches!(result, Ok(true)),
            "PID 99999999 must not be alive"
        );
    }

    /// `git_refresh_commit` creates a git commit in a fresh temp directory.
    #[test]
    fn git_refresh_commit_creates_commit() {
        let dir = tempdir().unwrap();
        let mount = dir.path();

        // Create the bucket directory and a test file.
        let issues_dir = mount.join("issues");
        std::fs::create_dir_all(&issues_dir).unwrap();
        std::fs::write(issues_dir.join("00000000001.md"), b"# test").unwrap();

        // Create the .reposix dir with fetched_at.txt and .gitignore.
        let reposix_dir = mount.join(".reposix");
        std::fs::create_dir_all(&reposix_dir).unwrap();
        std::fs::write(reposix_dir.join("fetched_at.txt"), b"2026-04-15T00:00:00Z").unwrap();
        std::fs::write(
            reposix_dir.join(".gitignore"),
            b"cache.db\ncache.db-wal\ncache.db-shm\nfuse.pid\n",
        )
        .unwrap();

        git_refresh_commit(
            mount,
            "issues",
            "reposix <simulator@demo>",
            "reposix refresh: simulator/demo — 1 issues at 2026-04-15T00:00:00Z",
        )
        .expect("git_refresh_commit");

        // Verify that a commit was created.
        let log = Command::new("git")
            .args(["-C", &mount.display().to_string(), "log", "--oneline"])
            .output()
            .expect("git log");
        let log_str = String::from_utf8_lossy(&log.stdout);
        assert!(
            !log_str.trim().is_empty(),
            "git log must show at least one commit"
        );
        assert!(
            log.status.success(),
            "git log must exit 0 inside a valid repo"
        );
    }
}
