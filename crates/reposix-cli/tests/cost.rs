//! Integration tests for `reposix cost`.

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
fn cost_help_renders() {
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .args(["cost", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success(), "cost --help failed: {out:?}");
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("--since"),
        "cost --help missing --since flag: {s}"
    );
    assert!(
        s.contains("--chars-per-token"),
        "cost --help missing --chars-per-token flag: {s}"
    );
}

#[test]
fn cost_no_remote_url_reports_error() {
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    let cache = tempdir().unwrap();

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .arg("cost")
        .arg(work)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "expected non-zero exit: {out:?}");
    assert!(
        stderr.contains("no remote.origin.url") || stderr.contains("remote.origin.url"),
        "expected remote-url error: stderr={stderr}"
    );
}

#[test]
fn cost_with_seeded_data_prints_markdown_table() {
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

    let cache = tempdir().unwrap();
    let cache_dir = cache.path().join("reposix").join("sim-demo.git");
    std::fs::create_dir_all(&cache_dir).unwrap();

    // Use the cache crate to get the schema right.
    let conn = reposix_cache::db::open_cache_db(&cache_dir).unwrap();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, reason, bytes) \
         VALUES (?1, 'token_cost', 'sim', 'demo', ?2, 300)",
        params![now, r#"{"in":100,"out":200,"kind":"fetch"}"#],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, reason, bytes) \
         VALUES (?1, 'token_cost', 'sim', 'demo', ?2, 300)",
        params![now, r#"{"in":50,"out":10,"kind":"push"}"#],
    )
    .unwrap();
    drop(conn);

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .arg("cost")
        .arg(work)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "cost failed: {out:?}");
    assert!(stdout.contains("| op"), "header missing: {stdout}");
    assert!(stdout.contains("| fetch"), "fetch row missing: {stdout}");
    assert!(stdout.contains("| push"), "push row missing: {stdout}");
    assert!(stdout.contains("| TOTAL"), "TOTAL row missing: {stdout}");
}
