//! Hash-binary golden tests (≥3 fixtures):
//! - comment-edit invariance: adding a comment inside the fn body MUST
//!   leave the hash unchanged (token-stream normalization).
//! - rename detection: renaming the fn MUST change the hash.
//! - whitespace invariance: reformatting (extra spaces, line breaks)
//!   MUST NOT change the hash.

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

fn write(dir: &TempDir, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.path().join(name);
    fs::write(&p, body).unwrap();
    p
}

fn hash(file: &std::path::Path, fn_name: &str) -> String {
    let out = Command::cargo_bin("hash_test_fn")
        .unwrap()
        .args(["--file", file.to_str().unwrap(), "--fn", fn_name])
        .assert()
        .success();
    String::from_utf8_lossy(&out.get_output().stdout)
        .trim()
        .to_string()
}

#[test]
fn comment_edit_does_not_change_hash() {
    let dir = TempDir::new().unwrap();
    let a = write(
        &dir,
        "a.rs",
        "fn alpha() {\n    let x = 1;\n    assert_eq!(x, 1);\n}\n",
    );
    let b = write(
        &dir,
        "b.rs",
        "fn alpha() {\n    // a comment that should be ignored\n    let x = 1;\n    /* block comment too */\n    assert_eq!(x, 1);\n}\n",
    );
    assert_eq!(hash(&a, "alpha"), hash(&b, "alpha"));
}

#[test]
fn rename_changes_hash() {
    let dir = TempDir::new().unwrap();
    let a = write(
        &dir,
        "a.rs",
        "fn alpha() { let x = 1; assert_eq!(x, 1); }\n",
    );
    let b = write(&dir, "b.rs", "fn beta() { let x = 1; assert_eq!(x, 1); }\n");
    let h_a = hash(&a, "alpha");
    let h_b = hash(&b, "beta");
    assert_ne!(
        h_a, h_b,
        "renaming the fn must change the token-stream hash"
    );
}

#[test]
fn whitespace_only_reformat_does_not_change_hash() {
    let dir = TempDir::new().unwrap();
    let a = write(
        &dir,
        "a.rs",
        "fn alpha() { let x = 1; assert_eq!(x, 1); }\n",
    );
    // Same token sequence, different whitespace + line breaks (no extra
    // trailing commas, no extra punctuation -- those would be real token
    // changes).
    let b = write(
        &dir,
        "b.rs",
        "fn alpha()\n{\n    let     x =      1;\n    assert_eq!(\n        x,\n        1\n    );\n}\n",
    );
    assert_eq!(hash(&a, "alpha"), hash(&b, "alpha"));
}

#[test]
fn missing_fn_exits_non_zero() {
    let dir = TempDir::new().unwrap();
    let p = write(&dir, "t.rs", "fn other() {}\n");
    Command::cargo_bin("hash_test_fn")
        .unwrap()
        .args(["--file", p.to_str().unwrap(), "--fn", "alpha"])
        .assert()
        .failure();
}
