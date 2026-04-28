//! `doc-alignment bind --test <shell-or-script-path>` tests (P71).
//!
//! Verifies the overloaded `--test` argument: non-Rust verifier paths bind
//! via the new `hash::file_hash` (full-file sha256) instead of the syn-based
//! `hash::test_body_hash`.

use std::fs;

use assert_cmd::Command;
use serde_json::Value;
use sha2::{Digest, Sha256};
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
    "floor": 0.0,
    "trend_30d": "+0.00",
    "last_walked": null,
    "coverage_floor": 0.0
  },
  "rows": []
}
"#;

fn seed(dir: &TempDir) -> std::path::PathBuf {
    let p = dir.path().join("doc-alignment.json");
    fs::write(&p, EMPTY_CATALOG).unwrap();
    p
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

#[test]
fn bind_with_shell_verifier_test_succeeds() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();

    // Real-shape shell script (matches the catalog use case: shebang, set,
    // commentary, body).
    let script = dir.path().join("check-foo.sh");
    let script_body = "#!/usr/bin/env bash\nset -euo pipefail\necho \"verifier OK\"\n";
    fs::write(&script, script_body).unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/shell-verifier",
            "--claim",
            "verifier exists",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--test",
            script.to_str().unwrap(),
            "--grade",
            "GREEN",
            "--rationale",
            "rat",
        ])
        .assert()
        .success();

    let raw = fs::read_to_string(&cat).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    let row = &v["rows"][0];
    assert_eq!(row["id"], "test/shell-verifier");
    assert_eq!(row["last_verdict"], "BOUND");

    // tests[0] is the bare path (no `::`).
    let tests = row["tests"].as_array().unwrap();
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].as_str().unwrap(), script.to_str().unwrap());

    // test_body_hashes[0] is sha256 of the script's bytes.
    let hashes = row["test_body_hashes"].as_array().unwrap();
    assert_eq!(hashes.len(), 1);
    let stored = hashes[0].as_str().unwrap();
    let expected = sha256_hex(script_body.as_bytes());
    assert_eq!(
        stored, expected,
        "stored hash matches sha256 of full script bytes"
    );
}

#[test]
fn walk_detects_drift_when_shell_verifier_changes() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);

    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();
    let script = dir.path().join("check-bar.sh");
    fs::write(&script, "#!/usr/bin/env bash\necho v1\n").unwrap();

    // Bind.
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/shell-drift",
            "--claim",
            "verifier exists",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--test",
            script.to_str().unwrap(),
            "--grade",
            "GREEN",
            "--rationale",
            "rat",
        ])
        .assert()
        .success();

    // Capture stored hash.
    let pre: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let stored = pre["rows"][0]["test_body_hashes"][0]
        .as_str()
        .unwrap()
        .to_string();

    // Mutate the script (file_hash sha256s the full content; any byte change
    // moves the hash).
    fs::write(&script, "#!/usr/bin/env bash\necho v2-drifted\n").unwrap();

    // Walk -- the row should land in STALE_TEST_DRIFT (non-blocking, exit 0
    // per current state-machine wiring; the verdict on disk is the proof).
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .success();

    let post: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let row = &post["rows"][0];
    assert_eq!(
        row["last_verdict"], "STALE_TEST_DRIFT",
        "shell verifier mutation -> STALE_TEST_DRIFT"
    );

    // Walker MUST NOT refresh the stored hash.
    assert_eq!(
        row["test_body_hashes"][0].as_str().unwrap(),
        stored,
        "walker is read-only on stored hashes"
    );
}

#[test]
fn bind_with_missing_shell_verifier_path_errors() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();

    let ghost = dir.path().join("does-not-exist.sh");

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/shell-missing",
            "--claim",
            "claim",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--test",
            ghost.to_str().unwrap(),
            "--grade",
            "GREEN",
            "--rationale",
            "rat",
        ])
        .assert()
        .failure();
}
