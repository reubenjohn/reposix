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

// -----------------------------------------------------------------------------
// P75 / BIND-VERB-FIX-01 regression tests
// -----------------------------------------------------------------------------
//
// Three tests exercise the bind-verb hash-overwrite invariant:
//   - A: Single -> Multi promotion preserves source_hash == hash(first source).
//   - B: Multi row first-source drift fires STALE_DOCS_DRIFT correctly.
//   - C: Single row STALE_DOCS_DRIFT heals to BOUND on re-bind with same source.
//
// See `.planning/phases/75-bind-verb-hash-fix/PLAN.md` § Task 2.
// -----------------------------------------------------------------------------

/// Local helper -- bind with explicit source range so tests can target
/// specific files without mutating the shared `bind_row` helper above.
fn bind_row_at(catalog: &std::path::Path, row_id: &str, source: &str, test_file: &std::path::Path) {
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
            source,
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

fn read_row(cat: &std::path::Path, row_id: &str) -> Value {
    let v: Value = serde_json::from_str(&fs::read_to_string(cat).unwrap()).unwrap();
    v["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == row_id)
        .unwrap()
        .clone()
}

/// P75 Test A: Single->Multi promotion preserves first-source hash.
///
/// Pre-fix this test FAILS at the source_hash assertion -- the bind verb
/// unconditionally overwrites `source_hash` with the newly-cited source's
/// hash on promotion. Post-fix the row.source_hash stays at hash(doc_a:1-2)
/// and the walker (which compares against source.as_slice()[0] == doc_a)
/// stays BOUND.
#[test]
fn walk_multi_source_stable_no_false_drift() {
    let dir = TempDir::new().unwrap();
    let cat = seed_catalog(&dir, json!([]));

    let doc_a = dir.path().join("doc_a.md");
    let doc_b = dir.path().join("doc_b.md");
    fs::write(&doc_a, "alpha line\nstable A\n").unwrap();
    fs::write(&doc_b, "beta line\nstable B\n").unwrap();

    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();

    // Step 1: bind to doc_a -- row is Source::Single(doc_a).
    bind_row_at(
        &cat,
        "row/multi",
        &format!("{}:1-2", doc_a.to_string_lossy()),
        &test_file,
    );
    let row_v1 = read_row(&cat, "row/multi");
    let hash_a = row_v1["source_hash"].as_str().unwrap().to_string();

    // Step 2: re-bind to doc_b -- row promotes to Source::Multi([doc_a, doc_b]).
    bind_row_at(
        &cat,
        "row/multi",
        &format!("{}:1-2", doc_b.to_string_lossy()),
        &test_file,
    );
    let row_v2 = read_row(&cat, "row/multi");

    // Invariant: source_hash MUST still be hash(doc_a:1-2). The walker
    // compares against source.as_slice()[0] which is doc_a.
    assert_eq!(
        row_v2["source_hash"].as_str().unwrap(),
        hash_a,
        "BIND-VERB-FIX-01: source_hash must be preserved on Single->Multi promotion (== hash of first source)"
    );

    // Sanity: source is now Multi shape with doc_a first.
    let sources = row_v2["source"].as_array().expect("Multi source array");
    assert_eq!(sources.len(), 2, "row should be Multi with 2 sources");
    assert_eq!(
        sources[0]["file"].as_str().unwrap(),
        doc_a.to_string_lossy().as_ref(),
        "first source must be doc_a"
    );

    // Step 3: walk should NOT fire STALE_DOCS_DRIFT (both files byte-stable).
    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        !stderr.contains("STALE_DOCS_DRIFT"),
        "false STALE_DOCS_DRIFT on stable Multi row: {stderr}"
    );

    let row_post_walk = read_row(&cat, "row/multi");
    assert_eq!(row_post_walk["last_verdict"], "BOUND");
}

/// P75 Test B: Multi row's FIRST source drift fires STALE_DOCS_DRIFT.
///
/// This is the path-(a) compare site: the walker hashes the first
/// source citation only. Drift in the first source MUST fire. Drift
/// in non-first sources is the documented path-(a) limitation
/// (MULTI-SOURCE-WATCH-01, deferred to v0.13.0); we do NOT assert
/// the limitation here -- that would lock it in. This test only
/// asserts the positive case.
#[test]
fn walk_multi_source_first_drift_fires_stale() {
    let dir = TempDir::new().unwrap();
    let cat = seed_catalog(&dir, json!([]));

    let doc_a = dir.path().join("doc_a.md");
    let doc_b = dir.path().join("doc_b.md");
    fs::write(&doc_a, "alpha line\nstable A\n").unwrap();
    fs::write(&doc_b, "beta line\nstable B\n").unwrap();

    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();

    // Bind doc_a then promote to Multi via doc_b.
    bind_row_at(
        &cat,
        "row/multi-drift-a",
        &format!("{}:1-2", doc_a.to_string_lossy()),
        &test_file,
    );
    bind_row_at(
        &cat,
        "row/multi-drift-a",
        &format!("{}:1-2", doc_b.to_string_lossy()),
        &test_file,
    );

    // Drift the FIRST source (doc_a).
    fs::write(&doc_a, "TOTALLY DIFFERENT\nNEW BYTES\n").unwrap();

    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("STALE_DOCS_DRIFT"),
        "stderr missing STALE_DOCS_DRIFT after first-source drift: {stderr}"
    );
    let r = read_row(&cat, "row/multi-drift-a");
    assert_eq!(r["last_verdict"], "STALE_DOCS_DRIFT");
}

/// P75 Test C: Single row that drifted to STALE_DOCS_DRIFT heals to BOUND
/// on re-bind with the same source citation (P74 SURPRISES-INTAKE finding).
///
/// Per CLAUDE.md docs-alignment dimension: walks NEVER refresh stored
/// hashes; binds DO. Re-binding with the same citation (sources stays
/// len==1) under the fix MUST refresh `source_hash` so the next walk
/// sees the row as BOUND.
#[test]
fn walk_single_source_rebind_heals_after_drift() {
    let dir = TempDir::new().unwrap();
    let cat = seed_catalog(&dir, json!([]));

    let doc_c = dir.path().join("doc_c.md");
    fs::write(&doc_c, "gamma line\noriginal C\n").unwrap();

    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();

    // Step 1: initial bind, capture source_hash_v1.
    bind_row_at(
        &cat,
        "row/heal",
        &format!("{}:1-2", doc_c.to_string_lossy()),
        &test_file,
    );
    let row_v1 = read_row(&cat, "row/heal");
    let hash_v1 = row_v1["source_hash"].as_str().unwrap().to_string();

    // Step 2: drift the source bytes.
    fs::write(&doc_c, "REWRITTEN GAMMA\nNEW C BYTES\n").unwrap();

    // Step 3: walk -> STALE_DOCS_DRIFT, source_hash unchanged.
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .failure();
    let row_post_drift = read_row(&cat, "row/heal");
    assert_eq!(row_post_drift["last_verdict"], "STALE_DOCS_DRIFT");
    assert_eq!(
        row_post_drift["source_hash"].as_str().unwrap(),
        hash_v1,
        "walker must NOT refresh stored source_hash"
    );

    // Step 4: re-bind with the same citation. source_hash refreshes.
    bind_row_at(
        &cat,
        "row/heal",
        &format!("{}:1-2", doc_c.to_string_lossy()),
        &test_file,
    );
    let row_post_rebind = read_row(&cat, "row/heal");
    let hash_v2 = row_post_rebind["source_hash"].as_str().unwrap().to_string();
    assert_ne!(
        hash_v2, hash_v1,
        "BIND-VERB-FIX-01: Single re-bind with current bytes must refresh source_hash (heal path)"
    );
    assert_eq!(row_post_rebind["last_verdict"], "BOUND");

    // Step 5: walk is clean.
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .success();
    let row_post_walk = read_row(&cat, "row/heal");
    assert_eq!(row_post_walk["last_verdict"], "BOUND");
}

// -----------------------------------------------------------------------------
// P78 / MULTI-SOURCE-WATCH-01 regression tests
// -----------------------------------------------------------------------------
//
// Three tests close the v0.12.1 P75 carry-forward false-negative window:
//   - walk_multi_source_non_first_drift_fires_stale (LOAD-BEARING NEW):
//     a Multi row whose SECOND source drifts must fire STALE_DOCS_DRIFT.
//     Pre-P78 (path-(a)) the walker only watched source.as_slice()[0],
//     so this case was a false-negative.
//   - walk_legacy_catalog_backfills_source_hash_to_source_hashes:
//     loading a pre-P78 catalog (only source_hash, no source_hashes)
//     transparently backfills source_hashes[0] from source_hash.
//   - bind_multi_same_source_rebind_refreshes_just_that_index:
//     re-binding one source of a Multi row updates JUST that index;
//     the sibling index keeps its prior hash.
//
// See `.planning/phases/78-pre-dvcs-hygiene/78-03-PLAN.md`.
// -----------------------------------------------------------------------------

/// P78 LOAD-BEARING: prove the path-(a) false-negative window is closed.
///
/// Build a Multi row with 2 source citations where the SECOND source's
/// bytes change post-bind. Walker MUST fire STALE_DOCS_DRIFT and the
/// diagnostic MUST name the second-source index/file.
#[test]
fn walk_multi_source_non_first_drift_fires_stale() {
    let dir = TempDir::new().unwrap();
    let cat = seed_catalog(&dir, json!([]));

    let doc_a = dir.path().join("doc_a.md");
    let doc_b = dir.path().join("doc_b.md");
    fs::write(&doc_a, "alpha line\nstable A\n").unwrap();
    fs::write(&doc_b, "beta line\nstable B\n").unwrap();

    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();

    // Bind doc_a, then promote to Multi via doc_b. After this, the row
    // has source_hashes = [hash(doc_a), hash(doc_b)] (P78 invariant).
    bind_row_at(
        &cat,
        "row/multi-non-first-drift",
        &format!("{}:1-2", doc_a.to_string_lossy()),
        &test_file,
    );
    bind_row_at(
        &cat,
        "row/multi-non-first-drift",
        &format!("{}:1-2", doc_b.to_string_lossy()),
        &test_file,
    );

    // Sanity: source_hashes is parallel to sources (2 elements).
    let row_pre = read_row(&cat, "row/multi-non-first-drift");
    let hashes_pre = row_pre["source_hashes"]
        .as_array()
        .expect("P78: source_hashes parallel-array must be persisted");
    assert_eq!(
        hashes_pre.len(),
        2,
        "P78 invariant: source_hashes.len == sources.len for Multi row"
    );
    let hash_b_pre = hashes_pre[1].as_str().unwrap().to_string();

    // Drift the SECOND source (doc_b). doc_a stays byte-stable.
    fs::write(&doc_b, "TOTALLY DIFFERENT\nNEW B BYTES\n").unwrap();

    // Walker MUST fire STALE_DOCS_DRIFT. Pre-P78 (path-(a)) this would
    // have stayed BOUND -- the false-negative window we close.
    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("STALE_DOCS_DRIFT"),
        "stderr missing STALE_DOCS_DRIFT after non-first source drift (path-(b) regression): {stderr}"
    );
    // Diagnostic must name the drifted source index for forensic clarity.
    assert!(
        stderr.contains("sources_drifted=[1]"),
        "stderr should surface drifted source index 1 (doc_b): {stderr}"
    );
    // Diagnostic should also name doc_b (the actual drifted file), not doc_a.
    assert!(
        stderr.contains("doc_b.md"),
        "stderr should surface doc_b.md as the drifted source: {stderr}"
    );

    // Verdict on disk must be STALE_DOCS_DRIFT.
    let r = read_row(&cat, "row/multi-non-first-drift");
    assert_eq!(r["last_verdict"], "STALE_DOCS_DRIFT");

    // Walker MUST NOT refresh stored hashes (read-only on hashes invariant).
    let hashes_post = r["source_hashes"]
        .as_array()
        .expect("source_hashes must persist post-walk");
    assert_eq!(
        hashes_post[1].as_str().unwrap(),
        hash_b_pre,
        "walker must NOT refresh stored source_hashes[1]"
    );
}

/// P78 BACK-COMPAT: legacy catalogs (only `source_hash`, no `source_hashes`)
/// must transparently backfill `source_hashes[0]` from `source_hash` on load.
///
/// The backfill is idempotent: subsequent loads of the same catalog (now
/// containing both fields) keep `source_hashes` populated.
#[test]
fn walk_legacy_catalog_backfills_source_hash_to_source_hashes() {
    use reposix_quality::hash;

    let dir = TempDir::new().unwrap();
    let doc = dir.path().join("doc.md");
    fs::write(&doc, "alpha line\nbeta line\n").unwrap();
    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();

    // Pre-compute the legitimate source_hash (so the walker does NOT
    // surface drift -- we want to see backfill mechanics, not drift).
    let src_hash = hash::source_hash(&doc, 1, 2).unwrap();
    let test_hash = hash::test_body_hash(&test_file, "alpha").unwrap();

    // Hand-rolled legacy-shape catalog: source_hash present, source_hashes ABSENT.
    let test_str = test_file.to_string_lossy().to_string();
    let legacy_row = json!({
        "id": "row/legacy",
        "claim": "legacy claim",
        "source": {
            "file": doc.to_string_lossy(),
            "line_start": 1,
            "line_end": 2,
        },
        "source_hash": src_hash,
        // NOTE: no `source_hashes` field -- pre-P78 catalog shape.
        "tests": [format!("{test_str}::alpha")],
        "test_body_hashes": [test_hash],
        "rationale": "legacy fixture",
        "last_verdict": "BOUND",
        "last_run": "2026-04-28T08:00:00Z",
        "last_extracted": "2026-04-28T08:00:00Z",
        "last_extracted_by": "fixture"
    });
    let cat = seed_catalog(&dir, json!([legacy_row]));

    // Sanity: pre-walk catalog on disk has NO source_hashes field on the row.
    let pre: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let row_pre = &pre["rows"].as_array().unwrap()[0];
    assert!(
        row_pre.get("source_hashes").is_none(),
        "legacy fixture must lack source_hashes field"
    );

    // Walk the catalog. This load-saves it; backfill runs on load. The
    // walk should exit clean (no drift -- src_hash matches doc bytes).
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .success();

    // Post-walk catalog has source_hashes = [src_hash] (1-element vec).
    let post: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let row_post = &post["rows"].as_array().unwrap()[0];
    let hashes = row_post["source_hashes"]
        .as_array()
        .expect("post-load backfill must populate source_hashes");
    assert_eq!(hashes.len(), 1, "Single source -> 1-element source_hashes");
    assert_eq!(
        hashes[0].as_str().unwrap(),
        src_hash,
        "backfill copies source_hash into source_hashes[0]"
    );
    // Legacy field still present (back-compat for one release cycle).
    assert_eq!(
        row_post["source_hash"].as_str().unwrap(),
        src_hash,
        "legacy source_hash field preserved post-backfill"
    );
    assert_eq!(row_post["last_verdict"], "BOUND");
}

/// P78 BIND HEAL: re-binding one source of a Multi row refreshes JUST that
/// index in `source_hashes`; the sibling index keeps its prior hash.
///
/// This is the path-(b) heal path for individual Multi entries. Pre-P78
/// the bind verb's `already_present` short-circuit treated rebinds as
/// no-ops (no hash refresh on the rebound index); the path-(a) walker
/// ignored non-first-source drift anyway, so the gap was invisible.
/// Post-P78 the walker AND-compares per index, so the bind verb MUST
/// refresh the rebound index.
#[test]
fn bind_multi_same_source_rebind_refreshes_just_that_index() {
    let dir = TempDir::new().unwrap();
    let cat = seed_catalog(&dir, json!([]));

    let doc_a = dir.path().join("doc_a.md");
    let doc_b = dir.path().join("doc_b.md");
    fs::write(&doc_a, "alpha line\nstable A\n").unwrap();
    fs::write(&doc_b, "beta line\noriginal B\n").unwrap();

    let test_file = dir.path().join("t.rs");
    fs::write(&test_file, "fn alpha() { let _ = 1; }\n").unwrap();

    // Step 1: bind doc_a then promote to Multi via doc_b. Capture both hashes.
    bind_row_at(
        &cat,
        "row/multi-rebind",
        &format!("{}:1-2", doc_a.to_string_lossy()),
        &test_file,
    );
    bind_row_at(
        &cat,
        "row/multi-rebind",
        &format!("{}:1-2", doc_b.to_string_lossy()),
        &test_file,
    );
    let row_v1 = read_row(&cat, "row/multi-rebind");
    let hashes_v1 = row_v1["source_hashes"].as_array().unwrap().clone();
    assert_eq!(hashes_v1.len(), 2);
    let hash_a_v1 = hashes_v1[0].as_str().unwrap().to_string();
    let hash_b_v1 = hashes_v1[1].as_str().unwrap().to_string();

    // Step 2: drift doc_b's bytes WITHOUT walking; then re-bind to doc_b
    // (same citation). This is the bind-driven heal path: source_hashes[1]
    // MUST refresh; source_hashes[0] (doc_a) MUST stay equal to hash_a_v1.
    fs::write(&doc_b, "REWRITTEN B\nFRESH B BYTES\n").unwrap();
    bind_row_at(
        &cat,
        "row/multi-rebind",
        &format!("{}:1-2", doc_b.to_string_lossy()),
        &test_file,
    );
    let row_v2 = read_row(&cat, "row/multi-rebind");
    let hashes_v2 = row_v2["source_hashes"].as_array().unwrap();
    assert_eq!(
        hashes_v2.len(),
        2,
        "still 2 sources after same-source rebind"
    );
    let hash_a_v2 = hashes_v2[0].as_str().unwrap().to_string();
    let hash_b_v2 = hashes_v2[1].as_str().unwrap().to_string();

    assert_eq!(
        hash_a_v2, hash_a_v1,
        "P78: doc_a's hash (index 0) must be UNCHANGED on doc_b rebind"
    );
    assert_ne!(
        hash_b_v2, hash_b_v1,
        "P78: doc_b's hash (index 1) MUST refresh on same-source rebind to current bytes"
    );

    // Step 3: walk -> BOUND (both indices match current bytes after rebind heal).
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args(["--catalog", cat.to_str().unwrap(), "walk"])
        .assert()
        .success();
    let row_post_walk = read_row(&cat, "row/multi-rebind");
    assert_eq!(row_post_walk["last_verdict"], "BOUND");
}
