//! Integration tests for `reposix-quality bind --dimension agent-ux`.
//!
//! GOOD-TO-HAVES-01 Path A. These cover the validation envelope (verifier
//! existence, source existence, row-id prefix, dimension routing) plus the
//! happy path and idempotent re-bind.

use std::fs;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

const EMPTY_AGENT_UX_CATALOG: &str = r#"{
  "$schema": "https://json-schema.org/draft-07/schema#",
  "comment": "test fixture -- agent-ux dim",
  "dimension": "agent-ux",
  "rows": []
}
"#;

fn seed(dir: &TempDir) -> std::path::PathBuf {
    let p = dir.path().join("agent-ux.json");
    fs::write(&p, EMPTY_AGENT_UX_CATALOG).unwrap();
    p
}

/// Build a verifier shell script + a source file in `dir`. Returns
/// (`verifier_path`, `source_path`) as tempdir-rooted paths.
fn seed_files(dir: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let verifier = dir.path().join("verifier.sh");
    fs::write(&verifier, "#!/usr/bin/env bash\necho ok\n").unwrap();
    let source = dir.path().join("src.rs");
    fs::write(&source, "fn alpha() {}\n").unwrap();
    (verifier, source)
}

#[test]
fn agent_ux_happy_path_mints_row() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let (verifier, source) = seed_files(&dir);

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "bind",
            "--dimension",
            "agent-ux",
            "--row-id",
            "agent-ux/test-fixture-row",
            "--verifier",
            verifier.to_str().unwrap(),
            "--cadence",
            "pre-pr",
            "--kind",
            "mechanical",
            "--source",
            source.to_str().unwrap(),
            "--blast-radius",
            "P1",
            "--asserts",
            "exits 0 against the local cargo workspace",
        ])
        .assert()
        .success();

    let raw = fs::read_to_string(&cat).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    let rows = v["rows"].as_array().expect("rows array");
    assert_eq!(rows.len(), 1, "exactly one row was minted");
    let row = &rows[0];
    assert_eq!(row["id"], "agent-ux/test-fixture-row");
    assert_eq!(row["dimension"], "agent-ux");
    assert_eq!(row["cadence"], "pre-pr");
    assert_eq!(row["kind"], "mechanical");
    assert_eq!(row["blast_radius"], "P1");
    assert_eq!(
        row["sources"],
        Value::Array(vec![Value::String(source.to_string_lossy().into_owned())]),
    );
    assert_eq!(
        row["status"], "FAIL",
        "mint state is FAIL until runner grades"
    );
    assert_eq!(row["freshness_ttl"], Value::Null);
    assert_eq!(row["waiver"], Value::Null);
    assert_eq!(
        row["expected"]["asserts"],
        Value::Array(vec![Value::String(
            "exits 0 against the local cargo workspace".to_string()
        )]),
    );
    assert_eq!(
        row["verifier"]["script"],
        verifier.to_string_lossy().as_ref()
    );
    assert_eq!(row["verifier"]["timeout_s"], 180);
    assert_eq!(row["verifier"]["container"], Value::Null);
    assert_eq!(
        row["artifact"],
        "quality/reports/verifications/agent-ux/test-fixture-row".to_string() + ".json",
    );
    let last_verified = row["last_verified"]
        .as_str()
        .expect("last_verified populated");
    assert!(
        last_verified.ends_with('Z') && last_verified.contains('T'),
        "last_verified is RFC3339-shaped: {last_verified}"
    );
}

#[test]
fn agent_ux_rejects_missing_verifier() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let (_verifier, source) = seed_files(&dir);
    let bogus = dir.path().join("does-not-exist.sh");

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "bind",
            "--dimension",
            "agent-ux",
            "--row-id",
            "agent-ux/missing-verifier",
            "--verifier",
            bogus.to_str().unwrap(),
            "--cadence",
            "pre-pr",
            "--kind",
            "mechanical",
            "--source",
            source.to_str().unwrap(),
            "--blast-radius",
            "P1",
        ])
        .assert()
        .failure();

    // No row was created.
    let v: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    assert!(
        v["rows"].as_array().unwrap().is_empty(),
        "no row created when verifier is missing"
    );
}

#[test]
fn agent_ux_rejects_missing_source() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let (verifier, _source) = seed_files(&dir);

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "bind",
            "--dimension",
            "agent-ux",
            "--row-id",
            "agent-ux/missing-source",
            "--verifier",
            verifier.to_str().unwrap(),
            "--cadence",
            "pre-pr",
            "--kind",
            "mechanical",
            "--source",
            "does/not/exist.rs",
            "--blast-radius",
            "P1",
        ])
        .assert()
        .failure();
}

#[test]
fn agent_ux_rejects_wrong_row_id_prefix() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let (verifier, source) = seed_files(&dir);

    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "bind",
            "--dimension",
            "agent-ux",
            // docs-alignment prefix on an agent-ux dimension -- rejected.
            "--row-id",
            "docs-alignment/wrong-prefix",
            "--verifier",
            verifier.to_str().unwrap(),
            "--cadence",
            "pre-pr",
            "--kind",
            "mechanical",
            "--source",
            source.to_str().unwrap(),
            "--blast-radius",
            "P1",
        ])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("agent-ux/"),
        "stderr names the required prefix -- got:\n{stderr}"
    );
}

#[test]
fn agent_ux_rejects_invalid_blast_radius() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let (verifier, source) = seed_files(&dir);

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "bind",
            "--dimension",
            "agent-ux",
            "--row-id",
            "agent-ux/bad-blast",
            "--verifier",
            verifier.to_str().unwrap(),
            "--cadence",
            "pre-pr",
            "--kind",
            "mechanical",
            "--source",
            source.to_str().unwrap(),
            "--blast-radius",
            "P9",
        ])
        .assert()
        .failure();
}

#[test]
fn agent_ux_idempotent_rebind_updates_last_verified_only() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let (verifier, source) = seed_files(&dir);

    let common_args = [
        "bind".to_string(),
        "--dimension".to_string(),
        "agent-ux".to_string(),
        "--row-id".to_string(),
        "agent-ux/idempotent".to_string(),
        "--verifier".to_string(),
        verifier.to_string_lossy().to_string(),
        "--cadence".to_string(),
        "pre-pr".to_string(),
        "--kind".to_string(),
        "mechanical".to_string(),
        "--source".to_string(),
        source.to_string_lossy().to_string(),
        "--blast-radius".to_string(),
        "P1".to_string(),
    ];

    // First mint.
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .arg("--catalog")
        .arg(&cat)
        .args(&common_args)
        .assert()
        .success();

    let v1: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let row1 = &v1["rows"][0].clone();
    let lv1 = row1["last_verified"].as_str().unwrap().to_string();

    // Second mint, same args -- sleep briefly to ensure RFC3339 clock tick
    // OR rely on the timestamp being equal (we accept either; what matters
    // is exactly one row remains and the static fields are unchanged).
    std::thread::sleep(std::time::Duration::from_secs(1));

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .arg("--catalog")
        .arg(&cat)
        .args(&common_args)
        .assert()
        .success();

    let v2: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let rows = v2["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "idempotent re-bind keeps row count at 1");
    let row2 = &rows[0];
    assert_eq!(row2["id"], row1["id"]);
    assert_eq!(row2["dimension"], row1["dimension"]);
    assert_eq!(row2["cadence"], row1["cadence"]);
    assert_eq!(row2["kind"], row1["kind"]);
    assert_eq!(row2["blast_radius"], row1["blast_radius"]);
    assert_eq!(row2["sources"], row1["sources"]);
    assert_eq!(row2["expected"], row1["expected"]);
    assert_eq!(row2["verifier"], row1["verifier"]);
    let lv2 = row2["last_verified"].as_str().unwrap();
    assert!(
        lv2 >= lv1.as_str(),
        "last_verified non-decreasing on re-bind ({lv1} -> {lv2})"
    );
}

#[test]
fn agent_ux_preserves_provenance_note_on_existing_row() {
    let dir = TempDir::new().unwrap();
    let cat = dir.path().join("agent-ux.json");
    // Seed a catalog that already contains a row with `_provenance_note`,
    // matching the v0.13.0 P79-P88 hand-edit annotations. The bind verb
    // must not strip this opaque field.
    let seed_contents = r#"{
      "$schema": "https://json-schema.org/draft-07/schema#",
      "dimension": "agent-ux",
      "rows": [
        {
          "id": "agent-ux/historical-row",
          "_provenance_note": "Hand-edit per documented gap (NOT Principle A): kept verbatim.",
          "dimension": "agent-ux",
          "cadence": "pre-pr",
          "kind": "mechanical",
          "sources": [],
          "command": "old",
          "expected": {"asserts": []},
          "verifier": {"script": "old.sh", "args": [], "timeout_s": 60, "container": null},
          "artifact": "quality/reports/verifications/agent-ux/historical-row.json",
          "status": "PASS",
          "last_verified": "2026-04-01T00:00:00Z",
          "freshness_ttl": null,
          "blast_radius": "P1",
          "owner_hint": null,
          "waiver": null
        }
      ]
    }
    "#;
    fs::write(&cat, seed_contents).unwrap();
    let (verifier, source) = seed_files(&dir);

    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "bind",
            "--dimension",
            "agent-ux",
            "--row-id",
            "agent-ux/historical-row",
            "--verifier",
            verifier.to_str().unwrap(),
            "--cadence",
            "pre-pr",
            "--kind",
            "mechanical",
            "--source",
            source.to_str().unwrap(),
            "--blast-radius",
            "P1",
        ])
        .assert()
        .success();

    let v: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let row = &v["rows"][0];
    assert_eq!(
        row["_provenance_note"]
            .as_str()
            .expect("provenance note preserved verbatim"),
        "Hand-edit per documented gap (NOT Principle A): kept verbatim.",
    );
}

#[test]
fn unknown_dimension_errors_with_v014_message() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let (verifier, source) = seed_files(&dir);

    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "bind",
            "--dimension",
            "release",
            "--row-id",
            "release/foo",
            "--verifier",
            verifier.to_str().unwrap(),
            "--cadence",
            "pre-release",
            "--kind",
            "mechanical",
            "--source",
            source.to_str().unwrap(),
            "--blast-radius",
            "P1",
        ])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("v0.14.0") && stderr.contains("GOOD-TO-HAVES-01"),
        "stderr names the v0.14.0 carry-forward issue -- got:\n{stderr}"
    );
}

#[test]
fn docs_alignment_via_top_level_bind_redirects_to_subcommand() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);
    let (verifier, source) = seed_files(&dir);

    let assert = Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "bind",
            // explicit default
            "--dimension",
            "docs-alignment",
            "--row-id",
            "docs-alignment/whatever",
            "--verifier",
            verifier.to_str().unwrap(),
            "--cadence",
            "pre-pr",
            "--kind",
            "mechanical",
            "--source",
            source.to_str().unwrap(),
            "--blast-radius",
            "P1",
        ])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains("doc-alignment bind"),
        "stderr names the existing subcommand -- got:\n{stderr}"
    );
}
