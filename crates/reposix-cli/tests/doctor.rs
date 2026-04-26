//! Integration tests for `reposix doctor`.
//!
//! Each test sets `REPOSIX_CACHE_DIR` to a tempdir so the doctor's cache
//! checks operate against an isolated state — no global writes.

use std::process::Command;

use assert_cmd::Command as AssertCmd;
use tempfile::tempdir;

/// Helper: set up a barebones `reposix init`'d-looking dir manually so we
/// don't need to actually invoke `git init` + every config from a subprocess
/// in every test. We DO call real git for `git init` because doctor probes
/// the on-disk `.git` layout.
fn git_init(path: &std::path::Path) {
    let out = Command::new("git")
        .args(["init", "-q"])
        .arg(path)
        .output()
        .unwrap();
    assert!(out.status.success(), "git init failed: {out:?}");
}

fn git_config_set(path: &std::path::Path, key: &str, val: &str) {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["config", key, val])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git config {key} {val} failed: {out:?}"
    );
}

#[test]
fn doctor_help_renders() {
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .args(["doctor", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success(), "doctor --help failed: {out:?}");
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("--fix"), "doctor --help missing --fix: {s}");
}

#[test]
fn doctor_clean_repo_reports_findings() {
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    git_config_set(work, "extensions.partialClone", "origin");
    git_config_set(
        work,
        "remote.origin.url",
        "reposix::http://127.0.0.1:7878/projects/demo",
    );

    // Isolate the cache.
    let cache = tempdir().unwrap();

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .arg("doctor")
        .arg(work)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Expected findings present in the output.
    assert!(
        stdout.contains("git.repo"),
        "git.repo missing: stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.contains("git.extensions.partialClone"),
        "partialClone check missing: stdout={stdout}"
    );
    assert!(
        stdout.contains("git.remote.origin.url"),
        "remote.origin.url check missing: stdout={stdout}"
    );
    // Summary line shape.
    assert!(
        stdout.contains("checks ·"),
        "summary line missing: stdout={stdout}"
    );
}

#[test]
fn doctor_no_remote_errors_and_exits_one() {
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    let cache = tempdir().unwrap();

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .arg("doctor")
        .arg(work)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success(),
        "doctor should exit non-zero with no remote: stdout={stdout}"
    );
    assert!(
        stdout.contains("remote.origin.url is unset"),
        "missing-remote message absent: {stdout}"
    );
}

#[test]
fn doctor_missing_partial_clone_warns() {
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    git_config_set(
        work,
        "remote.origin.url",
        "reposix::http://127.0.0.1:7878/projects/demo",
    );
    // Note: deliberately NOT setting extensions.partialClone.

    let cache = tempdir().unwrap();
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .arg("doctor")
        .arg(work)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("extensions.partialClone is unset"),
        "missing partialClone warning absent: {stdout}"
    );
    assert!(
        stdout.contains("git config extensions.partialClone origin"),
        "fix command absent: {stdout}"
    );
}

#[test]
fn doctor_fix_sets_partial_clone() {
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    git_config_set(
        work,
        "remote.origin.url",
        "reposix::http://127.0.0.1:7878/projects/demo",
    );

    let cache = tempdir().unwrap();
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .args(["doctor", "--fix"])
        .arg(work)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("(set by --fix)") || stdout.contains("partialClone=origin"),
        "doctor --fix did not report applying the fix: {stdout}"
    );

    // Verify post-state: git config now has the value.
    let post = Command::new("git")
        .arg("-C")
        .arg(work)
        .args(["config", "--get", "extensions.partialClone"])
        .output()
        .unwrap();
    assert!(
        post.status.success(),
        "post-fix config get failed: {post:?}"
    );
    let val = String::from_utf8_lossy(&post.stdout);
    assert_eq!(val.trim(), "origin", "config not set: {val}");
}

#[test]
fn doctor_blob_limit_zero_warns_on_real_remote() {
    // Non-sim remote (api.github.com) + REPOSIX_BLOB_LIMIT=0 → WARN.
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    git_config_set(work, "extensions.partialClone", "origin");
    git_config_set(
        work,
        "remote.origin.url",
        "reposix::https://api.github.com/projects/owner/repo",
    );

    let cache = tempdir().unwrap();
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .env("REPOSIX_BLOB_LIMIT", "0")
        // Allow github so the URL parse succeeds and the allowlist check
        // doesn't itself blow up first.
        .env("REPOSIX_ALLOWED_ORIGINS", "https://api.github.com")
        .arg("doctor")
        .arg(work)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("REPOSIX_BLOB_LIMIT=0") || stdout.contains("unlimited"),
        "blob-limit warning absent: {stdout}"
    );
}

#[test]
fn doctor_outdated_cache_warns() {
    use rusqlite::params;

    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    git_config_set(work, "extensions.partialClone", "origin");
    git_config_set(
        work,
        "remote.origin.url",
        "reposix::http://127.0.0.1:7878/projects/demo",
    );

    // Build the cache directory layout that the doctor expects:
    // <cache_root>/reposix/sim-demo.git/cache.db
    let cache = tempdir().unwrap();
    let cache_dir = cache.path().join("reposix").join("sim-demo.git");
    std::fs::create_dir_all(&cache_dir).unwrap();
    // Open via the cache crate so the schema/triggers are correct.
    let conn = reposix_cache::db::open_cache_db(&cache_dir).unwrap();
    // Set last_fetched_at to 48 hours ago.
    let stale = (chrono::Utc::now() - chrono::Duration::hours(48)).to_rfc3339();
    conn.execute(
        "INSERT INTO meta (key, value, updated_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
        params![
            "last_fetched_at",
            stale.as_str(),
            chrono::Utc::now().to_rfc3339()
        ],
    )
    .unwrap();
    drop(conn);

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .arg("doctor")
        .arg(work)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("cache.freshness"),
        "freshness check missing: {stdout}"
    );
    assert!(
        stdout.contains("stale") || stdout.contains("WARN  cache.freshness"),
        "stale-cache warning absent: {stdout}"
    );
}

#[test]
fn doctor_prints_backend_capability_row_for_sim() {
    // POLISH2-08: capability row surfaces in `reposix doctor` output so an
    // agent doesn't have to grep docs to learn what the configured backend
    // supports.
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    git_config_set(work, "extensions.partialClone", "origin");
    git_config_set(
        work,
        "remote.origin.url",
        "reposix::http://127.0.0.1:7878/projects/demo",
    );
    let cache = tempdir().unwrap();

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .arg("doctor")
        .arg(work)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("backend.capabilities"),
        "backend.capabilities check missing: {stdout}"
    );
    assert!(
        stdout.contains("backend capabilities"),
        "capability header missing: {stdout}"
    );
    assert!(
        stdout.contains("sim"),
        "sim slug missing: {stdout}"
    );
    assert!(
        stdout.contains("yes"),
        "yes column missing: {stdout}"
    );
    assert!(
        stdout.contains("strong"),
        "strong versioning label missing: {stdout}"
    );
}

#[test]
fn doctor_non_git_dir_errors() {
    let tmp = tempdir().unwrap();
    let cache = tempdir().unwrap();
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .arg("doctor")
        .arg(tmp.path())
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "doctor on a non-git dir should exit 1, got success"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("not a git repo"),
        "expected 'not a git repo' message: {stdout}"
    );
}
