//! `doc-alignment merge-shards <run-dir>` golden tests.
//!
//! Case A: same claim cited in 2 different sources, same test, both BOUND
//!   -> merge produces ONE row with sources=[a,b].
//! Case B: same claim, 2 different test bindings, both BOUND
//!   -> merge writes CONFLICTS.md, exits non-zero, catalog NOT mutated.

use std::fs;

use assert_cmd::Command;
use serde_json::{json, Value};
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

fn write_shard(run_dir: &std::path::Path, name: &str, rows: &Value) {
    let shards_dir = run_dir.join("shards");
    fs::create_dir_all(&shards_dir).unwrap();
    let p = shards_dir.join(name);
    fs::write(&p, serde_json::to_string_pretty(rows).unwrap()).unwrap();
}

fn make_row(id: &str, claim: &str, file: &str, test: &str) -> Value {
    // W7 / v0.12.1: `tests` + `test_body_hashes` are parallel arrays.
    // Single-test fixture rows degrade to one-element vectors.
    json!({
        "id": id,
        "claim": claim,
        "source": {
            "file": file,
            "line_start": 1,
            "line_end": 2,
        },
        "source_hash": "deadbeef",
        "tests": [test],
        "test_body_hashes": ["cafef00d"],
        "rationale": "fixture",
        "last_verdict": "BOUND",
        "last_run": "2026-04-28T08:00:00Z",
        "last_extracted": "2026-04-28T08:00:00Z",
        "last_extracted_by": "fixture"
    })
}

#[test]
fn merge_shards_auto_resolves_multi_source() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let run_dir = dir.path().join("run-multi");
    fs::create_dir_all(&run_dir).unwrap();

    // Same claim, same test, different sources.
    let s1 = json!([make_row(
        "shared/claim",
        "the shared claim",
        "docs/a.md",
        "tests/foo.rs::bar"
    )]);
    let s2 = json!([make_row(
        "shared/claim",
        "the shared claim",
        "docs/b.md",
        "tests/foo.rs::bar"
    )]);
    write_shard(&run_dir, "001.json", &s1);
    write_shard(&run_dir, "002.json", &s2);

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "merge-shards",
            run_dir.to_str().unwrap(),
        ])
        .assert()
        .success();

    let v: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "exactly one merged row expected");

    let row = &rows[0];
    assert_eq!(row["id"], "shared/claim");
    // Source is the multi-source array shape.
    let sources = row["source"]
        .as_array()
        .expect("multi-source row should serialize as a JSON array");
    assert_eq!(sources.len(), 2, "two source citations expected");
    let files: Vec<&str> = sources
        .iter()
        .map(|c| c["file"].as_str().unwrap())
        .collect();
    assert!(files.contains(&"docs/a.md"));
    assert!(files.contains(&"docs/b.md"));

    // No CONFLICTS.md emitted.
    assert!(!run_dir.join("CONFLICTS.md").exists());
    // MERGE.md summary written.
    assert!(run_dir.join("MERGE.md").exists());
}

#[test]
fn merge_shards_conflict_produces_conflicts_md_and_does_not_mutate_catalog() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let run_dir = dir.path().join("run-conflict");
    fs::create_dir_all(&run_dir).unwrap();

    // Same claim, different tests.
    let s1 = json!([make_row(
        "shared/claim",
        "the shared claim",
        "docs/a.md",
        "tests/foo.rs::bar"
    )]);
    let s2 = json!([make_row(
        "shared/claim",
        "the shared claim",
        "docs/a.md",
        "tests/foo.rs::baz"
    )]);
    write_shard(&run_dir, "001.json", &s1);
    write_shard(&run_dir, "002.json", &s2);

    let pre = fs::read_to_string(&cat).unwrap();

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "merge-shards",
            run_dir.to_str().unwrap(),
        ])
        .assert()
        .failure();

    // CONFLICTS.md written, catalog UNCHANGED.
    assert!(run_dir.join("CONFLICTS.md").exists());
    let conflicts_body = fs::read_to_string(run_dir.join("CONFLICTS.md")).unwrap();
    assert!(
        conflicts_body.contains("the shared claim"),
        "CONFLICTS.md should name the conflicting claim; got: {conflicts_body}"
    );
    let post = fs::read_to_string(&cat).unwrap();
    assert_eq!(pre, post, "catalog must NOT be mutated on conflict");
}
