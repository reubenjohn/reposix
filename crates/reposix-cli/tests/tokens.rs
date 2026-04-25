//! Integration tests for `reposix tokens`.

use std::process::Command;

use assert_cmd::Command as AssertCmd;
use tempfile::tempdir;

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
fn tokens_help_renders() {
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .args(["tokens", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success(), "tokens --help failed: {out:?}");
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.to_lowercase().contains("token"),
        "tokens --help should mention tokens: {s}"
    );
}

#[test]
fn tokens_no_remote_url_reports_error() {
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    let cache = tempdir().unwrap();

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .args(["tokens"])
        .arg(work)
        .output()
        .unwrap();
    assert!(!out.status.success(), "tokens with no remote should fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("remote.origin.url"),
        "expected remote.origin.url in error: {stderr}"
    );
}

/// Seed a cache.db with N token_cost rows and run `reposix tokens` against
/// the working tree pointing at it. The summary should print the running
/// totals + the MCP-equivalent comparison line.
#[test]
fn tokens_with_seeded_data_prints_summary() {
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    git_config_set(
        work,
        "remote.origin.url",
        "reposix::http://127.0.0.1:7878/projects/demo",
    );

    let cache_root = tempdir().unwrap();
    let cache_dir = cache_root.path().join("reposix").join("sim-demo.git");
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Open a cache.db via reposix-cache and write 5 rows directly.
    let conn = reposix_cache::db::open_cache_db(&cache_dir).unwrap();
    for i in 0..5 {
        reposix_cache::audit::log_token_cost(
            &conn,
            "sim",
            "demo",
            1000 * (i + 1),
            2000 * (i + 1),
            "fetch",
        );
    }
    drop(conn);

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache_root.path())
        .args(["tokens"])
        .arg(work)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "tokens with seeded data should succeed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Sessions"),
        "expected Sessions line: {stdout}"
    );
    assert!(
        stdout.contains("MCP-equivalent"),
        "expected MCP comparison: {stdout}"
    );
    assert!(stdout.contains("5"), "5 sessions should appear: {stdout}");
}

#[test]
fn tokens_empty_cache_says_no_sessions() {
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    git_config_set(
        work,
        "remote.origin.url",
        "reposix::http://127.0.0.1:7878/projects/demo",
    );

    let cache_root = tempdir().unwrap();
    let cache_dir = cache_root.path().join("reposix").join("sim-demo.git");
    std::fs::create_dir_all(&cache_dir).unwrap();
    // Open db so the schema is laid down; do NOT add token_cost rows.
    let _conn = reposix_cache::db::open_cache_db(&cache_dir).unwrap();

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache_root.path())
        .args(["tokens"])
        .arg(work)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No sessions"),
        "expected 'No sessions' message: {stdout}"
    );
}
