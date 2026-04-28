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
            // Synthetic walker fixtures cite docs in the temp dir, which
            // sit outside the eligible set keyed off `docs/`. Setting both
            // the alignment floor AND the coverage floor to 0.0 keeps the
            // walker focused on drift verdicts -- not on floor-trips on
            // unrelated synthetic data.
            "floor": 0.0,
            "trend_30d": "+0.00",
            "last_walked": null,
            "coverage_floor": 0.0
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
    // W7: test_body_hashes is now an array parallel to tests. Capture the
    // first (and, for these single-test fixture rows, only) element.
    let stored_test_body_hash = drift_row_pre["test_body_hashes"]
        .as_array()
        .expect("test_body_hashes array")[0]
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
        drift_row_post["test_body_hashes"]
            .as_array()
            .expect("test_body_hashes array")[0]
            .as_str()
            .unwrap(),
        stored_test_body_hash,
        "walker MUST NOT refresh stored test_body_hashes[0]"
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

/// W7 / v0.12.1: multi-test parallel-array drift detection.
///
/// Seeds a row with `tests.len() == 2` (`tests[0]` clean, `tests[1]` drifted)
/// by writing the JSON catalog directly (the bind CLI does not yet support
/// repeated `--test`; that is W7b). Asserts that the walker:
///   1. Sets the row's verdict to `STALE_TEST_DRIFT` (per-element compare).
///   2. Mutating ONLY `tests[1]` does NOT promote `tests[0]` to drifted.
///   3. Does NOT refresh either stored hash (read-only on hashes).
///
/// Note: `STALE_TEST_DRIFT` is currently non-blocking (`blocks_pre_push() ==
/// false`), so this test does not assert on exit code or stderr -- the
/// diagnostic line is only emitted on the blocking path. The verdict on
/// disk is the unambiguous proof that per-element comparison ran. (The
/// non-blocking design predates W7 and is unrelated to schema rollout.)
#[test]
fn walk_multi_test_per_element_drift_detection() {
    use reposix_quality::hash;

    let dir = TempDir::new().unwrap();

    // Source: clean prose. Doc lives in temp dir so it's outside the
    // per-file coverage eligible set (which is keyed off docs/).
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "shared claim line\nsecond line\n").unwrap();

    // Two test fns in the same file. We hash them with the real hasher
    // so the seeded `test_body_hashes` start out matching reality, then
    // mutate `bravo`'s body to force per-element drift on index 1.
    let test_file = dir.path().join("t.rs");
    fs::write(
        &test_file,
        "fn alpha() { let _ = 1; }\nfn bravo() { let _ = 2; }\n",
    )
    .unwrap();
    let h_alpha = hash::test_body_hash(&test_file, "alpha").unwrap();
    let h_bravo = hash::test_body_hash(&test_file, "bravo").unwrap();
    let src_hash = hash::source_hash(&doc, 1, 2).unwrap();

    let test_str = test_file.to_string_lossy().to_string();
    let row = json!({
        "id": "row/multi",
        "claim": "shared claim",
        "source": {
            "file": doc.to_string_lossy(),
            "line_start": 1,
            "line_end": 2,
        },
        "source_hash": src_hash,
        "tests": [format!("{test_str}::alpha"), format!("{test_str}::bravo")],
        "test_body_hashes": [h_alpha.clone(), h_bravo.clone()],
        "rationale": "multi-test fixture",
        "last_verdict": "BOUND",
        "last_run": "2026-04-28T08:00:00Z",
        "last_extracted": "2026-04-28T08:00:00Z",
        "last_extracted_by": "fixture"
    });
    let cat = seed_catalog(&dir, json!([row]));

    // Drift bravo only -- alpha stays clean.
    fs::write(
        &test_file,
        "fn alpha() { let _ = 1; }\nfn bravo() { let _ = 99; let _ = 100; }\n",
    )
    .unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .success();

    // Per-element verdict: index 1 drifted -> row is STALE_TEST_DRIFT.
    // Index 0 (alpha) stayed clean, so the row is NOT STALE_TEST_GONE.
    let post: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let r = &post["rows"].as_array().unwrap()[0];
    assert_eq!(
        r["last_verdict"], "STALE_TEST_DRIFT",
        "row should land in STALE_TEST_DRIFT (got {})",
        r["last_verdict"]
    );

    // Stored hashes must be untouched (walker is read-only on hashes).
    let stored_hashes = r["test_body_hashes"].as_array().unwrap();
    assert_eq!(stored_hashes[0].as_str().unwrap(), h_alpha);
    assert_eq!(stored_hashes[1].as_str().unwrap(), h_bravo);
}
