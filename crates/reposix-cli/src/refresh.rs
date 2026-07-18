//! `reposix refresh` — re-fetch backend issues, write `.md` files, git commit.
//!
//! After this command the working-tree directory is a git working tree whose
//! `git log` is a history of backend snapshots.  `git diff HEAD~1` shows what
//! changed at the backend since the last refresh.
//!
//! # Errors
//! Every public function documents its error conditions.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context as _, Result};
use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
use reposix_core::backend::sim::SimBackend;
use reposix_core::codes::ids;
use reposix_core::errmsg::teach_coded;
use reposix_core::BackendConnector as _;
use reposix_github::GithubReadOnlyBackend;
use reposix_jira::{JiraBackend, JiraCreds};

use crate::cache_db;
use crate::list::ListBackend;

/// Configuration for a single `reposix refresh` run.
pub struct RefreshConfig {
    /// Working-tree directory (a plain directory that is also, or will become,
    /// a git working tree).
    pub working_tree: PathBuf,
    /// Backend origin URL (simulator URL; ignored for github/confluence).
    pub origin: String,
    /// Project slug — sim project name, `owner/repo` for GitHub, or space KEY
    /// for Confluence.
    pub project: String,
    /// Which backend to speak.
    pub backend: ListBackend,
    /// When `true`, skip network egress and serve from cached `.md` files.
    /// Currently returns an error — the offline read path is not yet
    /// implemented; consumers should `cat` existing `.md` files in the
    /// working tree directly.
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
/// 1. Guard against `--offline` (not yet implemented).
/// 2. Open (or create) `.reposix/cache.db`.
/// 3. Fetch all issues from the configured backend.
/// 4. Delegate the rest to [`run_refresh_inner`].
///
/// # Errors
///
/// - `--offline` is set: returns a not-yet-implemented error.
/// - Backend network call fails: propagated from the backend.
/// - Propagates any error from [`run_refresh_inner`].
pub async fn run_refresh(cfg: RefreshConfig) -> Result<()> {
    if cfg.offline {
        bail!(
            "{}",
            teach_coded(
                ids::REFRESH_OFFLINE_UNIMPL,
                "`reposix refresh --offline` is not implemented yet.",
                "refresh always fetches a fresh snapshot from the backend today — there is no \
                 offline read path. The working tree already holds the last-fetched `.md` \
                 files, so read them directly instead of refreshing.",
                "to inspect issues without any network egress, `cat` / `grep` the record files \
                 already in the working tree.",
                &[
                    "ls issues/                     # already-fetched records (pages/ for confluence)",
                    "grep -rl TODO issues/          # search the last snapshot offline",
                    "reposix refresh <path>         # when you DO want a fresh backend snapshot",
                ],
            )
        );
    }

    // Open (or create) the metadata DB — this also acquires the advisory lock.
    let db = cache_db::open_cache_db(&cfg.working_tree)?;

    // Fetch issues from the configured backend.
    let issues = fetch_issues(&cfg).await?;

    run_refresh_inner(&cfg, issues, Some(&db))
}

/// Inner refresh logic: write `.md` files, update timestamps, commit.
///
/// Separated from [`run_refresh`] so integration tests can supply a
/// pre-built `Vec<Record>` without needing a live network backend.
///
/// # Errors
///
/// - `frontmatter::render` fails: propagated.
/// - Any git subprocess exits non-zero: propagated.
/// - `cache.db` update fails: propagated.
pub fn run_refresh_inner(
    cfg: &RefreshConfig,
    issues: Vec<reposix_core::Record>,
    db: Option<&crate::cache_db::CacheDb>,
) -> Result<()> {
    let n = issues.len();

    // Determine the bucket directory name via the shared canonical mapping
    // (Wave-5.5): confluence → pages/, everything else → issues/. This is
    // the same helper the cache tree builder and fast-import emit use, so
    // refresh output round-trips through the push planner per-backend.
    let bucket = match cfg.backend {
        ListBackend::Confluence => reposix_core::path::bucket_for_backend("confluence"),
        ListBackend::Sim | ListBackend::Github | ListBackend::Jira => {
            reposix_core::path::bucket_for_backend("sim")
        }
    };

    // Ensure the .reposix and bucket directories exist.
    let reposix_dir = cfg.working_tree.join(".reposix");
    std::fs::create_dir_all(&reposix_dir)
        .with_context(|| format!("create .reposix dir {}", reposix_dir.display()))?;

    let bucket_dir = cfg.working_tree.join(bucket);
    std::fs::create_dir_all(&bucket_dir)
        .with_context(|| format!("create bucket dir {}", bucket_dir.display()))?;

    // D91-10: remove stale, differently-spelled record files before writing.
    // Prior `refresh` runs wrote 11-zero-padded names (`00000000042.md`); the
    // canonical spelling is now unpadded (`42.md`, QL-001 / D91-01). Without
    // this sweep a schema change would leave BOTH files on disk for the same
    // id — two divergent git blobs. Only files whose stem parses to a record
    // id AND whose spelling differs from canonical are removed; non-record
    // files (READMEs, hand-authored notes, `.gitignore`) are never touched.
    if bucket_dir.exists() {
        for entry in std::fs::read_dir(&bucket_dir)
            .with_context(|| format!("scan bucket dir {}", bucket_dir.display()))?
        {
            let entry = entry.with_context(|| "read bucket dir entry")?;
            let os_name = entry.file_name();
            let name = os_name.to_string_lossy();
            if let Ok(rid) = reposix_core::path::validate_record_filename(&name) {
                if *name != reposix_core::path::record_filename(rid.0) {
                    std::fs::remove_file(entry.path()).with_context(|| {
                        format!("remove stale-padded record {}", entry.path().display())
                    })?;
                }
            }
        }
    }

    // Write one .md file per issue under the canonical unpadded filename.
    for issue in &issues {
        let rendered =
            reposix_core::frontmatter::render(issue).context("render issue frontmatter")?;
        let filename = reposix_core::path::record_filename(issue.id.0);
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
    std::fs::write(&reposix_gitignore, "cache.db\ncache.db-wal\ncache.db-shm\n")
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

    git_refresh_commit(&cfg.working_tree, bucket, &author, &message)?;

    // Update the metadata DB with the refresh result if one is provided.
    if let Some(db) = db {
        // commit_sha intentionally left None: `git_refresh_commit` does not
        // return the SHA today, and querying `git rev-parse HEAD` afterwards
        // would race with concurrent commits. The metadata row is keyed on
        // (backend, project, ts) which is sufficient for the cache freshness
        // check; SHA capture would be a separate signature change.
        cache_db::update_metadata(db, label, &cfg.project, &ts, None)?;
    }

    println!("refreshed {n} issues into {}", cfg.working_tree.display());
    Ok(())
}

/// Fetch issues from whichever backend `cfg` selects.
///
/// # Errors
///
/// Propagates backend construction or network errors.
async fn fetch_issues(cfg: &RefreshConfig) -> Result<Vec<reposix_core::Record>> {
    match cfg.backend {
        ListBackend::Sim => {
            let b = SimBackend::new(cfg.origin.clone()).context("build SimBackend")?;
            // Same teach-the-fix wrap as `reposix list` (DOCS-03) — a reader who
            // saw one recognizes the other; only the retry command differs.
            let retry = format!(
                "reposix refresh --project {} --origin {} {}",
                cfg.project,
                cfg.origin,
                cfg.working_tree.display()
            );
            b.list_records(&cfg.project)
                .await
                .map_err(|e| crate::list::wrap_sim_fetch_error(e, &cfg.origin, &retry))
        }
        ListBackend::Github => {
            let token = std::env::var("GITHUB_TOKEN").ok();
            let b = GithubReadOnlyBackend::new(token).context("build GithubReadOnlyBackend")?;
            b.list_records(&cfg.project).await.with_context(|| {
                format!(
                    "github list_records repo={} \
                     (REPOSIX_ALLOWED_ORIGINS must include https://api.github.com)",
                    cfg.project
                )
            })
        }
        ListBackend::Confluence => {
            // NB: these three `std::env::var("…")` literals are asserted by the
            // `env_vars_are_consumed_by_binary` docs-alignment grep (cli.rs) — keep
            // them here rather than delegating to `list::read_confluence_env`.
            let email = std::env::var("ATLASSIAN_EMAIL").unwrap_or_default();
            let token = std::env::var("ATLASSIAN_API_KEY").unwrap_or_default();
            let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").unwrap_or_default();
            if email.is_empty() || token.is_empty() || tenant.is_empty() {
                return Err(missing_confluence_env_error());
            }
            let creds = ConfluenceCreds {
                email,
                api_token: token,
            };
            let b = ConfluenceBackend::new(creds, &tenant).context("build ConfluenceBackend")?;
            b.list_records(&cfg.project).await.with_context(|| {
                format!(
                    "confluence list_records space_key={} \
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
                return Err(missing_jira_env_error());
            }
            let creds = JiraCreds {
                email,
                api_token: token,
            };
            let b = JiraBackend::new(creds, &instance).context("build JiraBackend")?;
            b.list_records(&cfg.project).await.with_context(|| {
                format!(
                    "jira list_records project_key={} \
                     (REPOSIX_ALLOWED_ORIGINS must include https://{instance}.atlassian.net)",
                    cfg.project
                )
            })
        }
    }
}

/// The teaching error for `reposix refresh --backend confluence` when one or
/// more of the three Atlassian env vars is unset. Extracted from `fetch_issues`
/// so the hot path stays under the line limit; the `std::env::var("…")` reads
/// themselves remain inline in `fetch_issues` (the docs-alignment grep).
fn missing_confluence_env_error() -> anyhow::Error {
    anyhow::anyhow!(
        "{}",
        teach_coded(
            ids::MISSING_ENV_CLI,
            "confluence backend requires ATLASSIAN_EMAIL, ATLASSIAN_API_KEY, and \
             REPOSIX_CONFLUENCE_TENANT env vars, but at least one is unset.",
            "set all three Atlassian Cloud vars — ATLASSIAN_EMAIL (your account email), \
             ATLASSIAN_API_KEY (a token from \
             id.atlassian.com/manage-profile/security/api-tokens), and \
             REPOSIX_CONFLUENCE_TENANT (your `<tenant>.atlassian.net` subdomain).",
            "no Atlassian account handy? the simulator needs no credentials — target \
             `sim::demo` instead.",
            &[
                "export ATLASSIAN_EMAIL=you@example.com",
                "export ATLASSIAN_API_KEY=<api-token>",
                "export REPOSIX_CONFLUENCE_TENANT=<subdomain>",
                "# then re-run: reposix refresh <path> --backend confluence --project <SPACE-KEY>",
            ],
        )
    )
}

/// The teaching error for `reposix refresh --backend jira` when one or more of
/// the three JIRA env vars is unset. Extracted for the same line-limit reason
/// as [`missing_confluence_env_error`].
fn missing_jira_env_error() -> anyhow::Error {
    anyhow::anyhow!(
        "{}",
        teach_coded(
            ids::MISSING_ENV_CLI,
            "jira backend requires JIRA_EMAIL, JIRA_API_TOKEN, and REPOSIX_JIRA_INSTANCE \
             env vars, but at least one is unset.",
            "set all three Atlassian Cloud vars — JIRA_EMAIL (your account email), \
             JIRA_API_TOKEN (a token from \
             id.atlassian.com/manage-profile/security/api-tokens), and REPOSIX_JIRA_INSTANCE \
             (your `<tenant>.atlassian.net` subdomain, e.g. `mycompany`).",
            "no Atlassian account handy? the simulator needs no credentials — target \
             `sim::demo` instead.",
            &[
                "export JIRA_EMAIL=you@example.com",
                "export JIRA_API_TOKEN=<api-token>",
                "export REPOSIX_JIRA_INSTANCE=<subdomain>",
                "# then re-run: reposix refresh <path> --backend jira --project <PROJECT-KEY>",
            ],
        )
    )
}

/// Run git operations to stage the refresh output and create a commit.
///
/// `git init` is idempotent.  Uses `--allow-empty` so a second refresh with
/// no backend changes still produces a commit (each refresh is a snapshot).
///
/// # Errors
///
/// Returns an error if any git subprocess exits with a non-zero status.
pub fn git_refresh_commit(
    working_tree: &Path,
    bucket: &str,
    author: &str,
    message: &str,
) -> Result<()> {
    // Helper: run a git command in the working-tree directory.
    let g = |args: &[&str]| -> Result<()> {
        let status = Command::new("git")
            .arg("-C")
            .arg(working_tree)
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
            // Internal git-subprocess wrapper: git inherits this process's stderr, so
            // its own diagnostic already reached the user; the user-facing refresh entry
            // errors (offline, missing creds, unreachable sim) teach at the call sites.
            // teach-exempt: ok — surfaces git's own stderr; not a user-facing teaching site
            bail!("git {args:?} failed: {status}")
        }
    };

    // `git init` is idempotent; -b main sets the default branch on first init.
    {
        let status = Command::new("git")
            .arg("-C")
            .arg(working_tree)
            .args(["-c", "init.defaultBranch=main", "init"])
            .status()
            .context("spawn git init")?;
        if !status.success() {
            // teach-exempt: ok — internal git-subprocess wrapper; git's own stderr
            // already reached the user (inherited fds). See the `g` closure marker.
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
        .arg(working_tree)
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
        // teach-exempt: ok — internal git-subprocess wrapper; git's own stderr
        // already reached the user (inherited fds). See the `g` closure marker.
        bail!("git commit failed: {status}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;
    use tempfile::tempdir;

    fn sample_record(id: u64) -> reposix_core::Record {
        let t = chrono::Utc::now();
        reposix_core::Record {
            id: reposix_core::RecordId(id),
            title: format!("issue {id}"),
            status: reposix_core::RecordStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: t,
            updated_at: t,
            version: 1,
            body: "body".to_owned(),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        }
    }

    /// D91-10: a working tree left with a stale 11-zero-padded record file
    /// from a prior `refresh` must be regenerated to the canonical unpadded
    /// spelling, and the stale duplicate removed — never leaving two divergent
    /// blobs for the same id. Non-record files are left untouched.
    #[test]
    fn refresh_removes_stale_padded_duplicate_and_regenerates_canonical() {
        let dir = tempdir().unwrap();
        let working_tree = dir.path().to_path_buf();
        let bucket_dir = working_tree.join("issues");
        std::fs::create_dir_all(&bucket_dir).unwrap();

        // Pre-seed the stale-padded file (old 11-pad producer output) + a
        // hand-authored non-record file that must survive the sweep.
        let stale = bucket_dir.join("00000000042.md");
        std::fs::write(&stale, b"stale padded blob").unwrap();
        let keep = bucket_dir.join("NOTES.md");
        std::fs::write(&keep, b"human notes, not a record").unwrap();

        let cfg = RefreshConfig {
            working_tree: working_tree.clone(),
            origin: "http://127.0.0.1:0".to_owned(),
            project: "demo".to_owned(),
            backend: ListBackend::Sim,
            offline: false,
        };
        run_refresh_inner(&cfg, vec![sample_record(42)], None).expect("refresh");

        assert!(
            !stale.exists(),
            "stale 11-pad `00000000042.md` must be removed (D91-10)"
        );
        assert!(
            bucket_dir.join("42.md").exists(),
            "canonical `42.md` must be regenerated"
        );
        assert!(
            keep.exists(),
            "non-record file `NOTES.md` must never be touched"
        );
    }

    /// DOCS-03: `reposix refresh` against a closed port must teach the sim
    /// recovery identically to `reposix list` — name the cause, suggest
    /// `reposix sim`, and give the copy-paste retry (adapted to refresh args).
    #[tokio::test]
    async fn refresh_against_closed_port_teaches_sim_recovery() {
        // Reserve then release a port so the connect is refused.
        let port = {
            use std::net::TcpListener;
            TcpListener::bind("127.0.0.1:0")
                .expect("bind 127.0.0.1:0")
                .local_addr()
                .expect("local_addr")
                .port()
        };
        let origin = format!("http://127.0.0.1:{port}");
        let dir = tempdir().unwrap();
        let cfg = RefreshConfig {
            working_tree: dir.path().to_path_buf(),
            origin: origin.clone(),
            project: "demo".to_owned(),
            backend: ListBackend::Sim,
            offline: false,
        };
        let err = fetch_issues(&cfg).await.expect_err("closed port must fail");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("reposix sim"),
            "closed-port refresh error must teach the sim recovery: {msg}"
        );
        assert!(
            msg.contains(&format!("reposix refresh --project demo --origin {origin}")),
            "closed-port refresh error must give the copy-paste retry: {msg}"
        );
    }

    /// `git_refresh_commit` creates a git commit in a fresh temp directory.
    #[test]
    fn git_refresh_commit_creates_commit() {
        let dir = tempdir().unwrap();
        let working_tree = dir.path();

        // Create the bucket directory and a test file.
        let issues_dir = working_tree.join("issues");
        std::fs::create_dir_all(&issues_dir).unwrap();
        std::fs::write(issues_dir.join("00000000001.md"), b"# test").unwrap();

        // Create the .reposix dir with fetched_at.txt and .gitignore.
        let reposix_dir = working_tree.join(".reposix");
        std::fs::create_dir_all(&reposix_dir).unwrap();
        std::fs::write(reposix_dir.join("fetched_at.txt"), b"2026-04-15T00:00:00Z").unwrap();
        std::fs::write(
            reposix_dir.join(".gitignore"),
            b"cache.db\ncache.db-wal\ncache.db-shm\n",
        )
        .unwrap();

        git_refresh_commit(
            working_tree,
            "issues",
            "reposix <simulator@demo>",
            "reposix refresh: simulator/demo — 1 issues at 2026-04-15T00:00:00Z",
        )
        .expect("git_refresh_commit");

        // Verify that a commit was created.
        let log = Command::new("git")
            .args([
                "-C",
                &working_tree.display().to_string(),
                "log",
                "--oneline",
            ])
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
