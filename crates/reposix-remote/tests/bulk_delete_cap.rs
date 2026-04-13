//! SG-02 cap end-to-end: feed an export stream that deletes 6 issues and
//! assert the helper refuses without making any DELETE call. With the
//! `[allow-bulk-delete]` tag in the commit message, deletes go through.
//!
//! These tests drive the binary against a wiremock backend.

#![forbid(unsafe_code)]

use std::io::Write;

use assert_cmd::Command;
use chrono::TimeZone;
use reposix_core::{Issue, IssueId, IssueStatus};
use serde_json::Value;
use wiremock::matchers::{any, method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_issue(id: u64) -> Value {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let i = Issue {
        id: IssueId(id),
        title: format!("issue {id}"),
        status: IssueStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: t,
        updated_at: t,
        version: 1,
        body: "body".to_owned(),
    };
    serde_json::to_value(i).unwrap()
}

/// Build a fast-export-style stream representing a commit that removes
/// every prior issue (empty new tree). The optional `msg` is embedded as
/// the commit's `data` payload so the SG-02 override tag can be tested.
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

#[tokio::test]
async fn six_deletes_refuses_and_calls_no_delete() {
    let server = MockServer::start().await;
    let issues: Vec<Value> = (1..=6).map(sample_issue).collect();
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(any())
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;
    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "capabilities").unwrap();
        writeln!(&mut buf).unwrap();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&empty_tree_export("cleanup\n"));
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
    let exit_ok = out.status.success();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!exit_ok, "expected non-zero exit; stderr: {stderr}");
    assert!(
        stderr.contains("refusing to push") && stderr.contains("cap is 5"),
        "stderr missing SG-02 message: {stderr}"
    );
    assert!(
        stdout.contains("error refs/heads/main bulk-delete"),
        "stdout missing protocol error: {stdout}"
    );
}

#[tokio::test]
async fn five_deletes_passes_cap() {
    let server = MockServer::start().await;
    let issues: Vec<Value> = (1..=5).map(sample_issue).collect();
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(any())
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&empty_tree_export("cleanup\n"));
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
        "5 deletes must succeed; stderr: {stderr}"
    );
    assert!(
        stdout.contains("ok refs/heads/main"),
        "stdout missing ok: {stdout}"
    );
}

#[tokio::test]
async fn six_deletes_with_allow_tag_actually_deletes() {
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
        .expect(6)
        .mount(&server)
        .await;
    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&empty_tree_export("[allow-bulk-delete] cleanup\n"));
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
        "[allow-bulk-delete] should bypass cap; stderr: {stderr}"
    );
    assert!(
        stdout.contains("ok refs/heads/main"),
        "stdout missing ok: {stdout}"
    );
}
