//! Litmus REOPEN regression: a SECOND `git push` that carries no new commit
//! must NOT mass-delete the backend.
//!
//! git re-invokes the `export` helper on every push (our `list for-push`
//! answers `?`, so git can never decide the ref is up-to-date). On a
//! no-new-commit push git's fast-export emits a stream with NO `commit`
//! directive — the literal bytes are `feature done` / `reset refs/heads/main`
//! / `from 000…000` / `done` (captured from a local sim reproduction). The
//! pre-fix helper parsed that empty stream into an empty tree and planned a
//! DELETE for EVERY prior record, mass-deleting a live Confluence space in
//! the P91 vision litmus (3 real DELETEs, audit 2026-07-04T21:44).
//!
//! This test drives the real `git-remote-reposix` binary against a wiremock
//! backend seeded with 3 records and asserts ZERO DELETE calls + a clean
//! `ok refs/heads/main` ack. RED against pre-fix code.

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

/// The EXACT stream git's remote-helper export pipes on a no-new-commit
/// push: a `reset` + null `from` and, critically, NO `commit` directive.
fn no_commit_export_stream() -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    writeln!(&mut out, "feature done").unwrap();
    writeln!(&mut out, "reset refs/heads/main").unwrap();
    writeln!(&mut out, "from 0000000000000000000000000000000000000000").unwrap();
    writeln!(&mut out, "done").unwrap();
    out
}

#[tokio::test]
async fn second_push_without_commit_deletes_nothing() {
    let server = MockServer::start().await;
    let issues: Vec<Value> = (1..=3).map(sample_issue).collect();
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    // The load-bearing assertion: not a single DELETE may be issued.
    Mock::given(method("DELETE"))
        .and(any())
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;
    // Nor any create/update — a no-commit push touches nothing.
    Mock::given(method("POST"))
        .and(any())
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&no_commit_export_stream());
        buf
    };
    let assert = tokio::task::spawn_blocking(move || {
        Command::cargo_bin("git-remote-reposix")
            .expect("binary built")
            .args(["origin", &url])
            .write_stdin(stdin_data)
            .timeout(std::time::Duration::from_secs(15))
            .assert()
    })
    .await
    .unwrap();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "no-commit push must ack cleanly (no-op); stderr: {stderr}"
    );
    assert!(
        stdout.contains("ok refs/heads/main"),
        "no-commit push must ack `ok refs/heads/main`; stdout: {stdout}"
    );
    // The wiremock `.expect(0)` mounts verify on drop that ZERO
    // DELETE/POST/PATCH calls were made — the mass-delete regression.
}
