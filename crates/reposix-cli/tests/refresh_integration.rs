//! Integration tests for `reposix refresh`.
//!
//! These tests call `run_refresh_inner` directly with a pre-built
//! `Vec<Issue>` so no network backend is needed. The `refresh_fuse_active_guard`
//! test calls `run_refresh` (the public fn) because the FUSE-active guard
//! fires before backend construction.

use chrono::DurationRound as _;
use chrono::TimeZone as _;
use reposix_cli::list::ListBackend;
use reposix_cli::refresh::{run_refresh, RefreshConfig};
use reposix_core::{Issue, IssueId, IssueStatus};
use tempfile::tempdir;

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_test_issue(id: u64, title: &str) -> Issue {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 15, 0, 0, 0).unwrap();
    Issue {
        id: IssueId(id),
        title: title.to_owned(),
        status: IssueStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: t,
        updated_at: t,
        version: 1,
        body: String::new(),
        parent_id: None,
    }
}

fn make_cfg(dir: &std::path::Path) -> RefreshConfig {
    RefreshConfig {
        mount_point: dir.to_path_buf(),
        origin: "http://127.0.0.1:17777".to_owned(),
        project: "test-project".to_owned(),
        backend: ListBackend::Sim,
        offline: false,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// `refresh_writes_md_files`:
/// - calling `run_refresh_inner` with one issue creates `issues/00000000001.md`
/// - the file contains YAML frontmatter (starts with `---`)
/// - `git log --oneline` shows exactly one commit
/// - the commit author line contains "reposix"
#[tokio::test]
async fn refresh_writes_md_files() {
    let dir = tempdir().unwrap();
    let cfg = make_cfg(dir.path());
    let issues = vec![make_test_issue(1, "Test issue")];

    reposix_cli::refresh::run_refresh_inner(&cfg, issues, None).expect("run_refresh_inner");

    // .md file exists and has YAML frontmatter.
    let md_path = dir.path().join("issues").join("00000000001.md");
    assert!(md_path.exists(), "md file must exist at {md_path:?}");
    let contents = std::fs::read_to_string(&md_path).unwrap();
    assert!(
        contents.starts_with("---\n"),
        "md file must start with YAML frontmatter, got: {contents}"
    );

    // git log shows exactly 1 commit.
    let log = std::process::Command::new("git")
        .args(["-C", &dir.path().display().to_string(), "log", "--oneline"])
        .output()
        .expect("git log");
    let log_str = String::from_utf8_lossy(&log.stdout);
    let commit_count = log_str.trim().lines().count();
    assert_eq!(commit_count, 1, "expected 1 commit, got: {log_str}");

    // Commit author name contains "reposix".
    let show = std::process::Command::new("git")
        .args([
            "-C",
            &dir.path().display().to_string(),
            "log",
            "--format=%an",
        ])
        .output()
        .expect("git log --format");
    let author_name = String::from_utf8_lossy(&show.stdout);
    assert!(
        author_name.contains("reposix"),
        "commit author name must contain 'reposix', got: {author_name}"
    );
}

/// `refresh_idempotent_no_diff`:
/// - two calls to `run_refresh_inner` with identical issues produce 2 commits
/// - `git diff HEAD~1 -- issues/` is empty (no issue file content changed)
#[tokio::test]
async fn refresh_idempotent_no_diff() {
    let dir = tempdir().unwrap();
    let cfg = make_cfg(dir.path());
    let issues = vec![make_test_issue(1, "Stable issue")];

    // First refresh.
    reposix_cli::refresh::run_refresh_inner(&cfg, issues.clone(), None)
        .expect("first run_refresh_inner");

    // Second refresh — same issues, no backend changes.
    reposix_cli::refresh::run_refresh_inner(&cfg, issues, None).expect("second run_refresh_inner");

    // git log must show exactly 2 commits.
    let log = std::process::Command::new("git")
        .args(["-C", &dir.path().display().to_string(), "log", "--oneline"])
        .output()
        .expect("git log");
    let log_str = String::from_utf8_lossy(&log.stdout);
    let commit_count = log_str.trim().lines().count();
    assert_eq!(commit_count, 2, "expected 2 commits, got: {log_str}");

    // git diff HEAD~1 -- issues/ must be empty (no issue content changed).
    let diff = std::process::Command::new("git")
        .args([
            "-C",
            &dir.path().display().to_string(),
            "diff",
            "HEAD~1",
            "--",
            "issues/",
        ])
        .output()
        .expect("git diff");
    let diff_str = String::from_utf8_lossy(&diff.stdout);
    assert!(
        diff_str.trim().is_empty(),
        "git diff HEAD~1 -- issues/ must be empty after identical re-refresh, got: {diff_str}"
    );
}

/// `refresh_fuse_active_guard`:
/// - writing current process PID to `.reposix/fuse.pid` makes `run_refresh`
///   return an error whose message contains "FUSE mount is active"
#[tokio::test]
async fn refresh_fuse_active_guard() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".reposix")).unwrap();

    // Current process is definitely alive.
    let pid = std::process::id();
    std::fs::write(
        dir.path().join(".reposix").join("fuse.pid"),
        pid.to_string(),
    )
    .unwrap();

    let cfg = RefreshConfig {
        mount_point: dir.path().to_path_buf(),
        origin: "http://127.0.0.1:17777".to_owned(),
        project: "test-project".to_owned(),
        backend: ListBackend::Sim,
        offline: false,
    };

    let err = run_refresh(cfg)
        .await
        .expect_err("run_refresh must fail when FUSE is active");

    assert!(
        err.to_string().contains("FUSE mount is active"),
        "unexpected error message: {err}"
    );
}

/// `fetched_at_is_current_timestamp`:
/// - after a refresh, `.reposix/fetched_at.txt` exists
/// - its content parses as a valid RFC3339 UTC timestamp
/// - the timestamp is within 30 seconds of wall clock time
#[tokio::test]
async fn fetched_at_is_current_timestamp() {
    let dir = tempdir().unwrap();
    let cfg = make_cfg(dir.path());
    let issues = vec![make_test_issue(2, "Clock test")];

    // Truncate to whole seconds — fetched_at.txt stores second-precision RFC3339.
    let before_secs = chrono::Utc::now()
        .duration_trunc(chrono::Duration::seconds(1))
        .unwrap();
    reposix_cli::refresh::run_refresh_inner(&cfg, issues, None).expect("run_refresh_inner");
    let after = chrono::Utc::now();

    let txt = std::fs::read_to_string(dir.path().join(".reposix").join("fetched_at.txt"))
        .expect("fetched_at.txt must exist");
    let ts = txt
        .trim()
        .parse::<chrono::DateTime<chrono::Utc>>()
        .expect("fetched_at.txt must be a valid RFC3339 timestamp");

    assert!(
        ts >= before_secs && ts <= after,
        "fetched_at {ts} not in [{before_secs}, {after}]"
    );
}
