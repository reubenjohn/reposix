//! `doc-alignment confirm-retire` env-guard test.
//!
//! Refuses with exit 1 + stderr "human-only" if:
//!   - CLAUDE_AGENT_CONTEXT is set, OR
//!   - stdin is non-tty (the test harness drives non-tty by default,
//!     so this also asserts the belt-and-suspenders TTY check).
//!
//! Also covers the W5/P69 `--i-am-human` bypass: explicit human authorization
//! flips a row from `RETIRE_PROPOSED` -> `RETIRE_CONFIRMED` even with the
//! env-guard active or stdin non-tty, with an audit-trail string in
//! `last_extracted_by` and a stderr WARNING line.

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

/// Catalog seeded with one `RETIRE_PROPOSED` row so `confirm-retire` has
/// something to flip on the success paths. Row id is `test/iah-row` for
/// readability in stderr capture.
const SEEDED_RETIRE_PROPOSED: &str = r#"{
  "schema_version": "1.0",
  "summary": {
    "claims_total": 1,
    "claims_bound": 0,
    "claims_missing_test": 0,
    "claims_retire_proposed": 1,
    "claims_retired": 0,
    "alignment_ratio": 1.0,
    "floor": 0.5,
    "trend_30d": "+0.00",
    "last_walked": null
  },
  "rows": [
    {
      "id": "test/iah-row",
      "claim": "stale claim slated for retirement",
      "source": { "file": "docs/x.md", "line_start": 1, "line_end": 2 },
      "rationale": "v0.9.0 dropped this surface",
      "last_verdict": "RETIRE_PROPOSED",
      "next_action": "RETIRE_FEATURE",
      "last_run": "2026-04-28T00:00:00Z",
      "last_extracted": "2026-04-28T00:00:00Z",
      "last_extracted_by": "propose-retire-call"
    }
  ]
}
"#;

fn seed(dir: &TempDir) -> std::path::PathBuf {
    let p = dir.path().join("doc-alignment.json");
    fs::write(&p, EMPTY_CATALOG).unwrap();
    p
}

fn seed_retire_proposed(dir: &TempDir) -> std::path::PathBuf {
    let p = dir.path().join("doc-alignment.json");
    fs::write(&p, SEEDED_RETIRE_PROPOSED).unwrap();
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

/// W5/P69: --i-am-human MUST NOT be honored implicitly. The strict-path
/// rejection (no flag) MUST stand under the env-guard so a subagent's
/// path of least resistance cannot become "delete the claim to make CI green."
#[test]
fn confirm_retire_without_i_am_human_still_rejects_under_env_guard() {
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

/// W5/P69: --i-am-human bypasses the env-guard AND the stdin-tty check,
/// flips the row's verdict, writes the audit-trail string, and emits a
/// stderr WARNING. Combined coverage of (a) + (b) + (d) from the phase
/// brief: env-guard-set, non-tty (assert_cmd never attaches a tty), and
/// the WARNING string land in one assertion.
#[test]
fn confirm_retire_with_i_am_human_succeeds_and_audit_trails() {
    let dir = TempDir::new().unwrap();
    let cat = seed_retire_proposed(&dir);

    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .env("CLAUDE_AGENT_CONTEXT", "set-by-test")
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "confirm-retire",
            "--row-id",
            "test/iah-row",
            "--i-am-human",
        ])
        .assert()
        .success();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("WARNING: --i-am-human bypassing"),
        "stderr should contain the WARNING line; got: {stderr}"
    );
    assert!(
        stderr.contains("test/iah-row"),
        "stderr WARNING should name the row id; got: {stderr}"
    );
    assert!(
        stderr.contains("confirm-retire-i-am-human"),
        "stderr WARNING should name the audit-trail string; got: {stderr}"
    );

    // Re-read catalog and assert the verdict flipped + audit trail string
    // landed in last_extracted_by.
    let catalog_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let row = catalog_json["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "test/iah-row")
        .expect("seeded row should still exist");
    assert_eq!(
        row["last_verdict"], "RETIRE_CONFIRMED",
        "row should have flipped to RETIRE_CONFIRMED; got: {row:#?}"
    );
    assert_eq!(
        row["last_extracted_by"], "confirm-retire-i-am-human",
        "audit trail string mismatch; got: {row:#?}"
    );
}
