//! docs/decisions/009-stability-commitment/exit-codes-locked — helper
//! arm. Pins `git-remote-reposix`'s exact locked exit-code set:
//! {0, 1, 2}, per `crates/reposix-remote/src/main.rs:96-111`. The
//! `reposix` CLI arm of the same claim lives in
//! `crates/reposix-cli/tests/cli.rs::exit_codes_locked_reposix_and_helper`
//! (same fn name, sibling crate — the two arms of one claim).

#![forbid(unsafe_code)]

use std::io::Write;

use assert_cmd::Command;
use chrono::TimeZone;
use reposix_core::{Record, RecordId, RecordStatus};
use serde_json::Value;
use wiremock::matchers::{any, method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_issue(id: u64) -> Value {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let i = Record {
        id: RecordId(id),
        title: format!("issue {id}"),
        status: RecordStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: t,
        updated_at: t,
        version: 1,
        body: "body".to_owned(),
        parent_id: None,
        extensions: std::collections::BTreeMap::new(),
    };
    serde_json::to_value(i).unwrap()
}

/// A fast-export-style stream representing a commit that removes every
/// prior issue (empty new tree) — deliberately over the SG-02 bulk-delete
/// cap (5) so the helper refuses the push. Copied from
/// `bulk_delete_cap.rs`'s `empty_tree_export` (each integration-test file
/// compiles as its own binary, so small fixture helpers are duplicated
/// per-file by existing convention rather than shared via a common module).
fn empty_tree_export(msg: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    writeln!(&mut out, "feature done").unwrap();
    writeln!(&mut out, "commit refs/heads/main").unwrap();
    writeln!(&mut out, "mark :1").unwrap();
    writeln!(&mut out, "committer test <t@t> 0 +0000").unwrap();
    let bytes = msg.as_bytes();
    writeln!(&mut out, "data {}", bytes.len()).unwrap();
    out.extend_from_slice(bytes);
    out.push(b'\n');
    writeln!(&mut out, "done").unwrap();
    out
}

/// Drives `git-remote-reposix` to all three documented exit codes:
///
/// - `0` — protocol session completes cleanly, no push attempted/rejected
///   (`capabilities` only, per `bus_capabilities.rs`'s established
///   pattern — the helper never touches the network for this verb).
/// - `1` — push refused at the protocol layer. Reuses the SG-02
///   bulk-delete-cap refusal (`bulk_delete_cap.rs`'s scenario: 6 deletes
///   against a cap of 5) rather than duplicating `push_conflict.rs`'s
///   heavier stale-base-version fixture — either refusal path exercises
///   the same `Ok(false)` → `ExitCode::from(1)` branch in `main.rs`.
/// - `2` — helper crashes before completing the protocol session.
///   Omitting the required `<url>` argv entry trips `real_main`'s own
///   `argv.len() < 3` bail (`main.rs:117-119`), independent of any
///   backend/network state.
#[tokio::test]
async fn exit_codes_locked_reposix_and_helper() {
    // --- 0: capabilities-only session, no network needed. ---
    let cap_out = tokio::task::spawn_blocking(|| {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", "reposix::http://127.0.0.1:9/projects/demo"])
            .write_stdin("capabilities\n\n")
            .timeout(std::time::Duration::from_secs(15))
            .output()
            .expect("run helper")
    })
    .await
    .unwrap();
    assert_eq!(
        cap_out.status.code(),
        Some(0),
        "capabilities-only session should exit 0: {cap_out:?}"
    );

    // --- 1: push refused (SG-02 bulk-delete cap: 6 deletes > cap of 5). ---
    let server = MockServer::start().await;
    let issues: Vec<Value> = (1..=6).map(sample_issue).collect();
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(any())
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;
    let url = format!("reposix::{}/projects/demo", server.uri());
    // Hermetic cache (P94 blocker). The SG-02 cap only fires when the L1
    // precheck takes the first-push fallback: no `last_fetched_at` cursor
    // ⇒ full `list_records` ⇒ 6-record prior ⇒ 6 deletes > cap of 5. With
    // no isolated `REPOSIX_CACHE_DIR` the helper shares
    // `~/.cache/reposix/sim-demo.git`, where a cursor leaked from ANY prior
    // push-success test (every sibling pushes to `demo`) flips the precheck
    // to the hot path and materializes an EMPTY prior from the unpopulated
    // oid_map — 0 deletes, cap never trips, exit 0 instead of the locked
    // exit 1. Isolating the cache per the crate's `CacheDirGuard` convention
    // (see `common.rs` + the 18 sibling push tests) makes this
    // stability-LOCKED exit-code assertion deterministic.
    // `cache_dir` (TempDir) is bound for the rest of the test so the isolated
    // dir outlives the spawned child; `cache_path` is moved into the closure.
    let cache_dir = tempfile::tempdir().expect("cache tempdir");
    let cache_path = cache_dir.path().to_path_buf();
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&empty_tree_export("cleanup\n"));
        buf
    };
    let push_out = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .env("REPOSIX_CACHE_DIR", &cache_path)
            .args(["origin", &url])
            .write_stdin(stdin_data)
            .timeout(std::time::Duration::from_secs(15))
            .output()
            .expect("run helper")
    })
    .await
    .unwrap();
    // Keep the isolated cache alive until the child has been reaped above.
    drop(cache_dir);
    assert_eq!(
        push_out.status.code(),
        Some(1),
        "SG-02 bulk-delete refusal should exit 1: {push_out:?}"
    );

    // --- 2: helper crash — missing required <url> argv entry. ---
    let crash_out = tokio::task::spawn_blocking(|| {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin"])
            .timeout(std::time::Duration::from_secs(15))
            .output()
            .expect("run helper")
    })
    .await
    .unwrap();
    assert_eq!(
        crash_out.status.code(),
        Some(2),
        "missing <url> argv should exit 2: {crash_out:?}"
    );
}
