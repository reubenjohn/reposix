//! `doc-alignment bind` source-relocation semantics (P89, BIND-RELOCATION-FIX).
//!
//! A line-shift refresh (`bind --source <same-file>:<new-lines>`) must REPLACE
//! the row's existing cite for that file in place -- NOT append a phantom stale
//! cite that keeps the row STALE_DOCS_DRIFT forever. Different-file sources must
//! still append (legitimate multi-source rows). A single rebind must also HEAL a
//! row already carrying phantom duplicate same-file cites from the pre-fix bug.

use std::fs;

use assert_cmd::Command;
use serde_json::Value;
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

/// Run `bind` against `cat` for `row_id`, citing `source` (a `<file>:<a>-<b>`
/// string) and `test`. Asserts success.
fn bind(cat: &std::path::Path, row_id: &str, claim: &str, source: &str, test: &str) {
    Command::cargo_bin("reposix-quality")
        .unwrap()
        .args([
            "--catalog",
            cat.to_str().unwrap(),
            "doc-alignment",
            "bind",
            "--row-id",
            row_id,
            "--claim",
            claim,
            "--source",
            source,
            "--test",
            test,
            "--grade",
            "GREEN",
            "--rationale",
            "rat",
        ])
        .assert()
        .success();
}

fn cites(cat: &std::path::Path, row_id: &str) -> Vec<(String, u64, u64)> {
    let v: Value = serde_json::from_str(&fs::read_to_string(cat).unwrap()).unwrap();
    let row = v["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == row_id)
        .unwrap_or_else(|| panic!("row {row_id} not found"));
    // `source` may be a single object (Single) or an array (Multi).
    let arr = match &row["source"] {
        Value::Array(a) => a.clone(),
        obj @ Value::Object(_) => vec![obj.clone()],
        other => panic!("unexpected source shape: {other:?}"),
    };
    arr.into_iter()
        .map(|c| {
            (
                c["file"].as_str().unwrap().to_string(),
                c["line_start"].as_u64().unwrap(),
                c["line_end"].as_u64().unwrap(),
            )
        })
        .collect()
}

/// Same-file rebind with a shifted line range REPLACES the cite (no phantom).
#[test]
fn same_file_rebind_replaces_cite_in_place() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);

    let doc = dir.path().join("doc.md");
    // 6 lines so we can cite line 1 then line 5.
    fs::write(&doc, "a\nb\nc\nd\ne\nf\n").unwrap();
    let test = dir.path().join("t.sh");
    fs::write(&test, "#!/usr/bin/env bash\necho ok\n").unwrap();

    let doc_s = doc.to_string_lossy().to_string();

    // Bind at line 1.
    bind(
        &cat,
        "row/shift",
        "claim",
        &format!("{doc_s}:1-1"),
        test.to_str().unwrap(),
    );
    assert_eq!(cites(&cat, "row/shift"), vec![(doc_s.clone(), 1, 1)]);

    // Rebind at line 5 (same file, shifted lines) -> REPLACE, not append.
    bind(
        &cat,
        "row/shift",
        "claim",
        &format!("{doc_s}:5-5"),
        test.to_str().unwrap(),
    );
    let after = cites(&cat, "row/shift");
    assert_eq!(
        after,
        vec![(doc_s.clone(), 5, 5)],
        "same-file line-shift must replace the cite, not strand a phantom at 1-1"
    );

    // source_hashes must stay parallel (length 1, matches the single cite).
    let v: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    let row = &v["rows"][0];
    assert_eq!(row["source_hashes"].as_array().unwrap().len(), 1);
    assert_eq!(row["last_verdict"], "BOUND");
}

/// Different-file rebind APPENDS (multi-source rows are legitimate).
#[test]
fn different_file_rebind_appends_second_cite() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir);

    let doc_a = dir.path().join("a.md");
    let doc_b = dir.path().join("b.md");
    fs::write(&doc_a, "alpha\n").unwrap();
    fs::write(&doc_b, "beta\n").unwrap();
    let test = dir.path().join("t.sh");
    fs::write(&test, "#!/usr/bin/env bash\necho ok\n").unwrap();

    let a_s = doc_a.to_string_lossy().to_string();
    let b_s = doc_b.to_string_lossy().to_string();

    bind(
        &cat,
        "row/multi",
        "claim",
        &format!("{a_s}:1-1"),
        test.to_str().unwrap(),
    );
    bind(
        &cat,
        "row/multi",
        "claim",
        &format!("{b_s}:1-1"),
        test.to_str().unwrap(),
    );

    let after = cites(&cat, "row/multi");
    assert_eq!(
        after,
        vec![(a_s.clone(), 1, 1), (b_s.clone(), 1, 1)],
        "different-file source must append -> Multi(a, b)"
    );
    let v: Value = serde_json::from_str(&fs::read_to_string(&cat).unwrap()).unwrap();
    assert_eq!(v["rows"][0]["source_hashes"].as_array().unwrap().len(), 2);

    // A subsequent same-file rebind of a.md replaces ONLY the a.md cite; the
    // b.md cite is preserved in position.
    bind(
        &cat,
        "row/multi",
        "claim",
        &format!("{a_s}:1-1"),
        test.to_str().unwrap(),
    );
    assert_eq!(
        cites(&cat, "row/multi"),
        vec![(a_s, 1, 1), (b_s, 1, 1)],
        "same-file rebind must not disturb the sibling different-file cite"
    );
}
