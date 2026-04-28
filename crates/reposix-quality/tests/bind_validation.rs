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

// W7b -- multi-test `--test` repetition (CLI surface follow-up).

#[test]
fn bind_with_two_tests_persists_both_with_parallel_hashes() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();
    // Two genuinely-different test fns -> distinct body hashes.
    let test_file = dir.path().join("t.rs");
    fs::write(
        &test_file,
        "fn alpha() { let _ = 1; assert_eq!(1, 1); }\n\
         fn beta() { let v = vec![1, 2, 3]; assert_eq!(v.len(), 3); }\n",
    )
    .unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/multi-two",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--test",
            &format!("{}::alpha", test_file.to_string_lossy()),
            "--test",
            &format!("{}::beta", test_file.to_string_lossy()),
            "--grade",
            "GREEN",
            "--rationale",
            "rat",
        ])
        .assert()
        .success();

    let raw = fs::read_to_string(&cat).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let row = &v["rows"][0];
    let tests = row["tests"].as_array().expect("tests array present");
    let hashes = row["test_body_hashes"]
        .as_array()
        .expect("test_body_hashes array present");
    assert_eq!(tests.len(), 2, "two --test args persist as two entries");
    assert_eq!(
        hashes.len(),
        2,
        "test_body_hashes is parallel to tests (len 2)"
    );
    let h0 = hashes[0].as_str().expect("hash 0 is a string");
    let h1 = hashes[1].as_str().expect("hash 1 is a string");
    assert!(!h0.is_empty(), "hash 0 non-empty");
    assert!(!h1.is_empty(), "hash 1 non-empty");
    assert_ne!(
        h0, h1,
        "alpha and beta have genuinely-different bodies -> distinct hashes"
    );
    assert_eq!(row["last_verdict"], "BOUND");
}

#[test]
fn bind_rejects_when_one_of_many_tests_is_invalid() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();
    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();
    let bogus = dir.path().join("nope.rs");
    // Note: bogus file is intentionally NOT created.

    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/multi-one-invalid",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--test",
            &format!("{}::alpha", test_file.to_string_lossy()),
            "--test",
            &format!("{}::ghost", bogus.to_string_lossy()),
            "--grade",
            "GREEN",
            "--rationale",
            "rat",
        ])
        .assert()
        .failure();

    // Error names which --test index failed.
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("--test #1"),
        "stderr names which test index failed -- got:\n{stderr}"
    );

    // Catalog row was NOT created (validation runs before any mutation).
    let raw = fs::read_to_string(&cat).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let rows = v["rows"].as_array().expect("rows array present");
    assert!(
        rows.is_empty(),
        "no row created when validation fails: {rows:?}"
    );
}

// W4 / v0.12.1 P68 -- next_action field tests.

#[test]
fn bind_sets_next_action_to_bind_green() {
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
            "test/next-action-bind",
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

    let raw = fs::read_to_string(&cat).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let row = &v["rows"][0];
    assert_eq!(
        row["next_action"], "BIND_GREEN",
        "bind sets next_action=BIND_GREEN"
    );
}

#[test]
fn mark_missing_test_with_impl_gap_rationale_sets_fix_impl_then_bind() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "mark-missing-test",
            "--row-id",
            "test/next-action-impl-gap",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--rationale",
            "IMPL_GAP: feature exists in lib.rs:280 but not bound to any test",
        ])
        .assert()
        .success();

    let raw = fs::read_to_string(&cat).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let row = &v["rows"][0];
    assert_eq!(
        row["next_action"], "FIX_IMPL_THEN_BIND",
        "IMPL_GAP: prefix -> next_action=FIX_IMPL_THEN_BIND"
    );

    // Counter-cases: DOC_DRIFT: prefix -> UPDATE_DOC; no prefix -> WRITE_TEST.
    let cat2 = seed(&dir);
    let doc2 = dir.path().join("doc2.md");
    fs::write(&doc2, "line one\nline two\n").unwrap();
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat2.to_str().unwrap(),
            "doc-alignment",
            "mark-missing-test",
            "--row-id",
            "test/next-action-doc-drift",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc2.to_string_lossy()),
            "--rationale",
            "DOC_DRIFT: prose names FUSE-mount transport that no longer exists",
        ])
        .assert()
        .success();
    let v2: serde_json::Value = serde_json::from_str(&fs::read_to_string(&cat2).unwrap()).unwrap();
    assert_eq!(
        v2["rows"][0]["next_action"], "UPDATE_DOC",
        "DOC_DRIFT: prefix -> next_action=UPDATE_DOC"
    );

    let cat3 = seed(&dir);
    let doc3 = dir.path().join("doc3.md");
    fs::write(&doc3, "line one\nline two\n").unwrap();
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat3.to_str().unwrap(),
            "doc-alignment",
            "mark-missing-test",
            "--row-id",
            "test/next-action-default",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc3.to_string_lossy()),
            "--rationale",
            "no prefix here, just plain rationale",
        ])
        .assert()
        .success();
    let v3: serde_json::Value = serde_json::from_str(&fs::read_to_string(&cat3).unwrap()).unwrap();
    assert_eq!(
        v3["rows"][0]["next_action"], "WRITE_TEST",
        "no rationale prefix -> next_action=WRITE_TEST (default)"
    );
}

#[test]
fn mark_missing_test_explicit_flag_overrides_heuristic() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();

    // Rationale would heuristically map to FIX_IMPL_THEN_BIND, but the
    // explicit --next-action UPDATE_DOC overrides.
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "mark-missing-test",
            "--row-id",
            "test/next-action-override",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--rationale",
            "IMPL_GAP: would normally pick FIX_IMPL_THEN_BIND",
            "--next-action",
            "UPDATE_DOC",
        ])
        .assert()
        .success();

    let raw = fs::read_to_string(&cat).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let row = &v["rows"][0];
    assert_eq!(
        row["next_action"], "UPDATE_DOC",
        "explicit --next-action overrides rationale-prefix heuristic"
    );

    // And invalid values are rejected at parse time.
    let cat2 = seed(&dir);
    let doc2 = dir.path().join("doc2.md");
    fs::write(&doc2, "line one\nline two\n").unwrap();
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat2.to_str().unwrap(),
            "doc-alignment",
            "mark-missing-test",
            "--row-id",
            "test/next-action-invalid",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc2.to_string_lossy()),
            "--rationale",
            "rat",
            "--next-action",
            "BOGUS_VALUE",
        ])
        .assert()
        .failure();
}

#[test]
fn propose_retire_sets_next_action_to_retire_feature() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "propose-retire",
            "--row-id",
            "test/next-action-retire",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--rationale",
            "Superseded by ADR-001",
        ])
        .assert()
        .success();

    let raw = fs::read_to_string(&cat).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let row = &v["rows"][0];
    assert_eq!(
        row["next_action"], "RETIRE_FEATURE",
        "propose-retire sets next_action=RETIRE_FEATURE"
    );
}

#[test]
fn bind_zero_tests_is_rejected_at_clap() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "line one\nline two\n").unwrap();

    // No --test argument at all: clap rejects with the standard
    // "required" message naming the long flag.
    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            "test/zero-tests",
            "--claim",
            "claim text",
            "--source",
            &format!("{}:1-2", doc.to_string_lossy()),
            "--grade",
            "GREEN",
            "--rationale",
            "rat",
        ])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("--test"),
        "clap error mentions --test -- got:\n{stderr}"
    );
}
