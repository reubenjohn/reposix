//! `doc-alignment bind` validation: 4 cases per the plan must_haves.truths.

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
fn bind_rejects_nonexistent_source_file() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/missing-source",
            "--claim",
            "claim text",
            "--source",
            "does/not/exist.md:1-2",
            "--test",
            &format!("{}::alpha", test_file.to_string_lossy()),
            "--grade",
            "GREEN",
            "--rationale",
            "rat",
        ])
        .assert()
        .failure();
}

#[test]
fn bind_rejects_out_of_bounds_line_range() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();
    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/oob-range",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-99", doc.to_string_lossy()),
            "--test",
            &format!("{}::alpha", test_file.to_string_lossy()),
            "--grade",
            "GREEN",
            "--rationale",
            "rat",
        ])
        .assert()
        .failure();
}

#[test]
fn bind_rejects_missing_fn_symbol() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();
    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn other() { let _ = 1; }\n").unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/missing-fn",
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
        .failure();
}

#[test]
fn bind_valid_round_trip() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();
    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; assert_eq!(1, 1); }\n").unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/valid",
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

    // Catalog now contains the row with hashes populated.
    let raw = fs::read_to_string(&cat).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let rows = v["rows"].as_array().expect("rows array present");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row["id"], "test/valid");
    assert_eq!(row["last_verdict"], "BOUND");
    assert!(
        row["source_hash"].as_str().is_some(),
        "source_hash populated"
    );
    // W7: tests + test_body_hashes are parallel arrays.
    let tests = row["tests"].as_array().expect("tests array present");
    let hashes = row["test_body_hashes"]
        .as_array()
        .expect("test_body_hashes array present");
    assert_eq!(tests.len(), 1, "single-test bind populates one entry");
    assert_eq!(hashes.len(), 1, "test_body_hashes parallel to tests");
    assert!(hashes[0].as_str().is_some(), "first hash is populated");

    // Summary is recomputed.
    assert_eq!(v["summary"]["claims_total"], 1);
    assert_eq!(v["summary"]["claims_bound"], 1);
}

#[test]
fn bind_rejects_non_green_grade() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();
    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/non-green",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--test",
            &format!("{}::alpha", test_file.to_string_lossy()),
            "--grade",
            "RED",
            "--rationale",
            "rat",
        ])
        .assert()
        .failure();
}
