//! Integration tests for `reposix gc`.
//!
//! Each test sets `REPOSIX_CACHE_DIR` to a tempdir for cache isolation.

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
fn gc_help_renders() {
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .args(["gc", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success(), "gc --help failed: {out:?}");
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("--strategy"),
        "gc --help missing --strategy: {s}"
    );
    assert!(s.contains("--dry-run"), "gc --help missing --dry-run: {s}");
}

#[test]
fn gc_invalid_strategy_rejected() {
    // No working tree needed — clap rejects the value before we touch
    // any path resolution.
    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .args(["gc", "--strategy", "bogus"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "gc --strategy bogus must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid value") || stderr.contains("not a valid"),
        "expected clap error on invalid strategy: {stderr}"
    );
}

#[test]
fn gc_no_remote_url_reports_error() {
    // A bare git repo without remote.origin.url has no cache to gc.
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    let cache = tempdir().unwrap();

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache.path())
        .args(["gc"])
        .arg(work)
        .output()
        .unwrap();
    assert!(!out.status.success(), "gc with no remote should fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("remote.origin.url"),
        "expected remote.origin.url in error: {stderr}"
    );
}

#[test]
fn gc_dry_run_with_empty_cache_succeeds() {
    // Wire up a working tree pointing at a (manually pre-created) empty
    // cache dir. The gc subcommand should run, find nothing to evict,
    // and exit 0.
    let tmp = tempdir().unwrap();
    let work = tmp.path();
    git_init(work);
    git_config_set(
        work,
        "remote.origin.url",
        "reposix::http://127.0.0.1:7878/projects/demo",
    );

    let cache_root = tempdir().unwrap();
    // Pre-create the empty cache dir at the deterministic location
    // (`<root>/reposix/<backend>-<project>.git` per resolve_cache_path).
    let cache_dir = cache_root.path().join("reposix").join("sim-demo.git");
    std::fs::create_dir_all(&cache_dir).unwrap();
    // Also create the objects dir so the gc helper has something to walk.
    std::fs::create_dir_all(cache_dir.join("objects")).unwrap();

    let out = AssertCmd::cargo_bin("reposix")
        .unwrap()
        .env("REPOSIX_CACHE_DIR", cache_root.path())
        .args(["gc", "--dry-run"])
        .arg(work)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "gc --dry-run on empty cache should succeed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("0 blob") || stdout.contains("Nothing to"),
        "dry-run on empty cache should report 0 blobs: {stdout}"
    );
}
