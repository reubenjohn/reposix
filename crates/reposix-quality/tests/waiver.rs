//! `doc-alignment waive` / `unwaive` + walker waiver semantics.
//!
//! The docs-alignment dimension's per-row schema gains a time-boxed
//! `waiver` block that mirrors the unified-catalog waiver contract
//! (`quality/runners/run.py` `waiver_active` + the `structure/file-size-limits`
//! row). A row in a blocking state (e.g. `MISSING_TEST`) carrying an
//! UNEXPIRED waiver does NOT count toward the walker's blocking exit, but the
//! walker prints a loud `WAIVED-<STATE>` line on EVERY push. An EXPIRED
//! waiver blocks again (`WAIVER-EXPIRED`). `waive` refuses non-existent rows,
//! non-blocking rows, past dates, and dates more than 90 days out.

use std::fs;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

/// A catalog with one row already parked in `MISSING_TEST` (a blocking state)
/// and no waiver. `floor`/`coverage_floor` are 0.0 so the synthetic row's
/// out-of-`docs/` citation can't trip a floor block and mask the per-row
/// signal under test.
const BLOCKED_CATALOG: &str = r#"{
  "schema_version": "1.0",
  "summary": {
    "claims_total": 1,
    "claims_bound": 0,
    "claims_missing_test": 1,
    "claims_retire_proposed": 0,
    "claims_retired": 0,
    "alignment_ratio": 0.0,
    "floor": 0.0,
    "trend_30d": "+0.00",
    "last_walked": null,
    "coverage_floor": 0.0
  },
  "rows": [
    {
      "id": "docs/x/blocked-claim",
      "claim": "the widget frobnicates",
      "source": { "file": "docs/x.md", "line_start": 1, "line_end": 1 },
      "last_verdict": "MISSING_TEST",
      "next_action": "WRITE_TEST"
    }
  ]
}
"#;

fn seed(dir: &TempDir, body: &str) -> std::path::PathBuf {
    let p = dir.path().join("doc-alignment.json");
    fs::write(&p, body).unwrap();
    p
}

fn qcmd(cat: &std::path::Path) -> Command {
    let mut c = Command::cargo_bin("reposix-quality").unwrap();
    c.args(["--catalog", cat.to_str().unwrap(), "doc-alignment"]);
    c
}

/// RFC3339 `now + days`. Uses chrono so the test stays deterministic
/// regardless of the wall-clock date it runs on.
fn iso_in_days(days: i64) -> String {
    let dt = chrono::Utc::now() + chrono::Duration::days(days);
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn read_cat(cat: &std::path::Path) -> Value {
    serde_json::from_str(&fs::read_to_string(cat).unwrap()).unwrap()
}

fn row_waiver(cat: &std::path::Path, row_id: &str) -> Value {
    let v = read_cat(cat);
    v["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == row_id)
        .unwrap_or_else(|| panic!("row {row_id} not found"))["waiver"]
        .clone()
}

/// Run `walk`, return (success_bool, stderr_string).
fn walk(cat: &std::path::Path) -> (bool, String) {
    let out = qcmd(cat).arg("walk").assert();
    let output = out.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stderr)
}

/// waive -> walk exits 0 with a loud WAIVED line; the row's honest verdict is
/// untouched; summary counts the waived row distinctly.
#[test]
fn waive_then_walk_passes_with_loud_line() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir, BLOCKED_CATALOG);

    // Baseline: an un-waived MISSING_TEST row BLOCKS.
    let (ok, stderr) = walk(&cat);
    assert!(
        !ok,
        "un-waived MISSING_TEST row must block; stderr: {stderr}"
    );
    assert!(stderr.contains("MISSING_TEST"), "stderr: {stderr}");

    // Waive it 30 days out.
    let until = iso_in_days(30);
    qcmd(&cat)
        .args([
            "waive",
            "--row-id",
            "docs/x/blocked-claim",
            "--until",
            &until,
            "--reason",
            "claim unverifiable until the round-trip lands (tracked)",
            "--tracked-in",
            "SURPRISES-INTAKE QL-001",
        ])
        .assert()
        .success();

    // Waiver persisted; last_verdict stays honest (MISSING_TEST).
    let w = row_waiver(&cat, "docs/x/blocked-claim");
    assert_eq!(w["until"], until);
    assert_eq!(w["tracked_in"], "SURPRISES-INTAKE QL-001");
    assert_eq!(read_cat(&cat)["rows"][0]["last_verdict"], "MISSING_TEST");

    // walk now PASSES but prints the loud WAIVED-MISSING_TEST line.
    let (ok, stderr) = walk(&cat);
    assert!(ok, "waived row must not block; stderr: {stderr}");
    assert!(
        stderr.contains("WAIVED-MISSING_TEST"),
        "expected loud WAIVED line; stderr: {stderr}"
    );
    assert!(stderr.contains("docs/x/blocked-claim"), "stderr: {stderr}");
    assert!(
        stderr.contains("tracked_in=SURPRISES-INTAKE QL-001"),
        "stderr: {stderr}"
    );

    // Summary counts the waived row distinctly.
    assert_eq!(read_cat(&cat)["summary"]["claims_waived"], 1);
}

/// An EXPIRED waiver blocks again with WAIVER-EXPIRED.
#[test]
fn expired_waiver_blocks() {
    let dir = TempDir::new().unwrap();
    // Seed the waiver directly with a past `until` (the waive verb refuses to
    // MINT a past date, so we construct the expired state on disk).
    let body = r#"{
  "schema_version": "1.0",
  "summary": {
    "claims_total": 1, "claims_bound": 0, "claims_missing_test": 1,
    "claims_retire_proposed": 0, "claims_retired": 0,
    "alignment_ratio": 0.0, "floor": 0.0, "trend_30d": "+0.00",
    "last_walked": null, "coverage_floor": 0.0
  },
  "rows": [
    {
      "id": "docs/x/blocked-claim",
      "claim": "the widget frobnicates",
      "source": { "file": "docs/x.md", "line_start": 1, "line_end": 1 },
      "last_verdict": "MISSING_TEST",
      "next_action": "WRITE_TEST",
      "waiver": {
        "until": "2020-01-01T00:00:00Z",
        "reason": "lapsed waiver",
        "tracked_in": "OLD-TICKET"
      }
    }
  ]
}
"#;
    let cat = seed(&dir, body);
    let (ok, stderr) = walk(&cat);
    assert!(!ok, "expired waiver must block; stderr: {stderr}");
    assert!(stderr.contains("WAIVER-EXPIRED"), "stderr: {stderr}");
    assert!(stderr.contains("docs/x/blocked-claim"), "stderr: {stderr}");
    // Not double-counted as a live waiver.
    assert_eq!(read_cat(&cat)["summary"]["claims_waived"], 0);
}

fn assert_waive_fails(cat: &std::path::Path, args: &[&str], needle: &str) {
    let mut full = vec!["waive"];
    full.extend_from_slice(args);
    let assert = qcmd(cat).args(&full).assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(
        stderr.contains(needle),
        "expected stderr to contain {needle:?}; got: {stderr}"
    );
}

/// waive refuses a date more than 90 days out (anti self-licensing).
#[test]
fn waive_refuses_beyond_90_days() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir, BLOCKED_CATALOG);
    assert_waive_fails(
        &cat,
        &[
            "--row-id",
            "docs/x/blocked-claim",
            "--until",
            "2099-01-01T00:00:00Z",
            "--reason",
            "too far",
            "--tracked-in",
            "TICKET",
        ],
        "more than 90 days out",
    );
    // No waiver was written.
    assert!(row_waiver(&cat, "docs/x/blocked-claim").is_null());
}

/// waive refuses a past date.
#[test]
fn waive_refuses_past_date() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir, BLOCKED_CATALOG);
    assert_waive_fails(
        &cat,
        &[
            "--row-id",
            "docs/x/blocked-claim",
            "--until",
            "2020-01-01T00:00:00Z",
            "--reason",
            "past",
            "--tracked-in",
            "TICKET",
        ],
        "not in the future",
    );
}

/// waive refuses a non-existent row.
#[test]
fn waive_refuses_missing_row() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir, BLOCKED_CATALOG);
    let until = iso_in_days(10);
    assert_waive_fails(
        &cat,
        &[
            "--row-id",
            "docs/x/does-not-exist",
            "--until",
            &until,
            "--reason",
            "r",
            "--tracked-in",
            "TICKET",
        ],
        "not found",
    );
}

/// waive refuses a non-blocking (BOUND) row -- nothing to defer.
#[test]
fn waive_refuses_non_blocking_row() {
    let dir = TempDir::new().unwrap();
    let body = r#"{
  "schema_version": "1.0",
  "summary": {
    "claims_total": 1, "claims_bound": 1, "claims_missing_test": 0,
    "claims_retire_proposed": 0, "claims_retired": 0,
    "alignment_ratio": 1.0, "floor": 0.0, "trend_30d": "+0.00",
    "last_walked": null, "coverage_floor": 0.0
  },
  "rows": [
    {
      "id": "docs/x/bound-claim",
      "claim": "already bound",
      "source": { "file": "docs/x.md", "line_start": 1, "line_end": 1 },
      "last_verdict": "BOUND",
      "next_action": "BIND_GREEN"
    }
  ]
}
"#;
    let cat = seed(&dir, body);
    let until = iso_in_days(10);
    assert_waive_fails(
        &cat,
        &[
            "--row-id",
            "docs/x/bound-claim",
            "--until",
            &until,
            "--reason",
            "r",
            "--tracked-in",
            "TICKET",
        ],
        "non-blocking state",
    );
}

/// unwaive removes a waiver; the row then blocks again on walk.
#[test]
fn unwaive_restores_block() {
    let dir = TempDir::new().unwrap();
    let cat = seed(&dir, BLOCKED_CATALOG);
    let until = iso_in_days(20);
    qcmd(&cat)
        .args([
            "waive",
            "--row-id",
            "docs/x/blocked-claim",
            "--until",
            &until,
            "--reason",
            "temporary",
            "--tracked-in",
            "TICKET",
        ])
        .assert()
        .success();
    // Waived -> walk passes.
    assert!(walk(&cat).0, "waived row should pass");

    qcmd(&cat)
        .args(["unwaive", "--row-id", "docs/x/blocked-claim"])
        .assert()
        .success();
    assert!(row_waiver(&cat, "docs/x/blocked-claim").is_null());
    // Un-waived -> walk blocks again.
    let (ok, stderr) = walk(&cat);
    assert!(!ok, "un-waived row must block again");
    assert!(stderr.contains("MISSING_TEST"), "stderr: {stderr}");
}
