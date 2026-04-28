//! Walker golden test:
//!   - 2-row synthetic catalog (one BOUND clean, one BOUND with drifted source)
//!   - run `walk`, assert drifted -> STALE_DOCS_DRIFT, exit non-zero
//!   - assert stderr names the slash command
//!   - POST-CONDITION: stored hashes are unchanged (walker NEVER refreshes hashes)

use std::fs;

use assert_cmd::Command;
use serde_json::{json, Value};
use tempfile::TempDir;

fn seed_catalog(dir: &TempDir, rows: Value) -> std::path::PathBuf {
    let cat = json!({
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
        "rows": rows,
    });
    let p = dir.path().join("doc-alignment.json");
    fs::write(&p, serde_json::to_string_pretty(&cat).unwrap()).unwrap();
    p
}

fn bind_row(
    catalog: &std::path::Path,
    row_id: &str,
    doc: &std::path::Path,
    test_file: &std::path::Path,
) {
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            catalog.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            row_id,
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--test",
            &format!("{}::alpha", test_file.to_string_lossy()),
            "--grade",
            "GREEN",
            "--rationale",
            "rat",
        ])
        .assert()
        .success();
}

#[test]
fn walk_detects_source_drift_and_preserves_stored_hashes() {
    let dir = TempDir::new().unwrap();
    let cat = seed_catalog(&dir, json!([]));

    // Row 1: clean. Row 2: will drift.
    let doc_clean = dir.path().join("doc-clean.md");
    let doc_drift = dir.path().join("doc-drift.md");
    fs::write(&doc_clean, "alpha line\nbeta line\n").unwrap();
    fs::write(&doc_drift, "old content\nstable line\n").unwrap();

    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; assert_eq!(1, 1); }\n").unwrap();

    bind_row(&cat, "row/clean", &doc_clean, &test_file);
    bind_row(&cat, "row/drift", &doc_drift, &test_file);

    // Capture the stored hashes BEFORE the drift.
    let snapshot: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let drift_row_pre = snapshot["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "row/drift")
        .unwrap()
        .clone();
    let stored_source_hash = drift_row_pre["source_hash"].as_str().unwrap().to_string();
    let stored_test_body_hash = drift_row_pre["test_body_hash"]
        .as_str()
        .unwrap()
        .to_string();

    // Drift the source file.
    fs::write(&doc_drift, "BRAND NEW\nVERY DIFFERENT\n").unwrap();

    // Run walk. Expect non-zero exit and stderr naming the slash command.
    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("STALE_DOCS_DRIFT"),
        "stderr missing STALE_DOCS_DRIFT: {stderr}"
    );
    assert!(
        stderr.contains("/reposix-quality-refresh"),
        "stderr missing slash command: {stderr}"
    );

    // POST-CONDITION: stored hashes UNCHANGED on the drifted row.
    let post: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let drift_row_post = post["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "row/drift")
        .unwrap()
        .clone();
    assert_eq!(drift_row_post["last_verdict"], "STALE_DOCS_DRIFT");
    assert_eq!(
        drift_row_post["source_hash"].as_str().unwrap(),
        stored_source_hash,
        "walker MUST NOT refresh stored source_hash"
    );
    assert_eq!(
        drift_row_post["test_body_hash"].as_str().unwrap(),
        stored_test_body_hash,
        "walker MUST NOT refresh stored test_body_hash"
    );

    // The clean row stays BOUND.
    let clean_row_post = post["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "row/clean")
        .unwrap()
        .clone();
    assert_eq!(clean_row_post["last_verdict"], "BOUND");
}

#[test]
fn walk_clean_catalog_exits_zero() {
    let dir = TempDir::new().unwrap();
    let cat = seed_catalog(&dir, json!([]));

    let doc = dir.path().join("doc.md");
    fs::write(&doc, "alpha\nbeta\n").unwrap();
    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();
    bind_row(&cat, "row/clean", &doc, &test_file);

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .success();
}
