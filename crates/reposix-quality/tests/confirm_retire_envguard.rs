//! `doc-alignment confirm-retire` env-guard test.
//!
//! Refuses with exit 1 + stderr "human-only" if:
//!   - CLAUDE_AGENT_CONTEXT is set, OR
//!   - stdin is non-tty (the test harness drives non-tty by default,
//!     so this also asserts the belt-and-suspenders TTY check).

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

const EMPTY_CATALOG: &str = r#"{
  "schema_version": "1.0",
  "summary": {
    "claims_total": 0,
    "claims_bound": 0,
    "claims_missing_test": 0,
    "claims_retire_proposed": 0,
    "claims_retired": 0,
    "alignment_ratio": 1.0,
    "floor": 0.5,
    "trend_30d": "+0.00",
    "last_walked": null
  },
  "rows": []
}
"#;

fn seed(dir: &TempDir) -> std::path::PathBuf {
    let p = dir.path().join("doc-alignment.json");
    fs::write(&p, EMPTY_CATALOG).unwrap();
    p
}

#[test]
fn confirm_retire_refuses_when_agent_context_env_set() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);

    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .env("CLAUDE_AGENT_CONTEXT", "set-by-test")
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "confirm-retire",
            "--row-id",
            "test/whatever",
        ])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("human-only"),
        "stderr should contain `human-only`; got: {stderr}"
    );
}

#[test]
fn confirm_retire_refuses_when_stdin_not_tty() {
    // assert_cmd by default does NOT attach a tty. Even without
    // CLAUDE_AGENT_CONTEXT, the TTY belt-and-suspenders should fire.
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);

    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .env_remove("CLAUDE_AGENT_CONTEXT")
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "confirm-retire",
            "--row-id",
            "test/whatever",
        ])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("human-only"),
        "stderr should contain `human-only`; got: {stderr}"
    );
}
