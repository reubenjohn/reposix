//! End-to-end binary tests for the git remote helper protocol surface.
//!
//! Drives the compiled `git-remote-reposix` binary via `assert_cmd`,
//! feeding stdin and inspecting stdout / stderr. Verifies the
//! `capabilities`, `option`, and basic dispatch behavior.

#![forbid(unsafe_code)]

use std::io::Write;

use assert_cmd::Command;
use serde_json::{json, Value};
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn capabilities_advertises_import_export_refspec() {
    let mut cmd = Command::cargo_bin("git-remote-reposix").expect("binary built");
    let assert = cmd
        .args(["origin", "reposix::http://127.0.0.1:7878/projects/demo"])
        .write_stdin("capabilities\n")
        .timeout(std::time::Duration::from_secs(10))
        .assert();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.starts_with("import\nexport\nrefspec refs/heads/*:refs/reposix/*\n"),
        "stdout did not start with capability advertisement; got:\n{stdout}"
    );
}

#[test]
fn option_replies_unsupported() {
    let mut cmd = Command::cargo_bin("git-remote-reposix").expect("binary built");
    let assert = cmd
        .args(["origin", "reposix::http://127.0.0.1:7878/projects/demo"])
        .write_stdin("option dry-run true\n")
        .timeout(std::time::Duration::from_secs(10))
        .assert();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "unsupported", "stdout: {stdout:?}");
}

#[test]
fn unknown_command_writes_to_stderr_not_stdout() {
    let mut cmd = Command::cargo_bin("git-remote-reposix").expect("binary built");
    let assert = cmd
        .args(["origin", "reposix::http://127.0.0.1:7878/projects/demo"])
        .write_stdin("floofle\n")
        .timeout(std::time::Duration::from_secs(10))
        .assert();
    let out = assert.get_output();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.is_empty() || stdout.trim().is_empty(),
        "stdout polluted on unknown command: {stdout:?}"
    );
    assert!(
        stderr.contains("unknown command"),
        "stderr missing diagnostic: {stderr:?}"
    );
}

/// Build a synthetic fast-export stream with a single new blob at mark 1
/// mapped to path `0001.md`. The blob bytes are `blob_bytes` verbatim —
/// no UTF-8 encoding, no line-ending normalization — so tests can exercise
/// the raw-bytes path with CRLF or non-UTF-8 payloads.
fn single_blob_export(blob_bytes: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    writeln!(&mut out, "feature done").unwrap();
    writeln!(&mut out, "blob").unwrap();
    writeln!(&mut out, "mark :1").unwrap();
    writeln!(&mut out, "data {}", blob_bytes.len()).unwrap();
    out.extend_from_slice(blob_bytes);
    out.push(b'\n');
    writeln!(&mut out, "commit refs/heads/main").unwrap();
    writeln!(&mut out, "mark :2").unwrap();
    writeln!(&mut out, "committer test <t@t> 0 +0000").unwrap();
    let msg = b"create\n";
    writeln!(&mut out, "data {}", msg.len()).unwrap();
    out.extend_from_slice(msg);
    out.push(b'\n');
    writeln!(&mut out, "M 100644 :1 0001.md").unwrap();
    writeln!(&mut out, "done").unwrap();
    out
}

fn issue_response(id: u64, body: &str) -> Value {
    json!({
        "id": id,
        "title": "crlf test",
        "status": "open",
        "labels": [],
        "created_at": "2026-04-13T00:00:00Z",
        "updated_at": "2026-04-13T00:00:00Z",
        "version": 0,
        "body": body,
    })
}

/// H-01: A blob body containing `\r\n` line endings must round-trip
/// through the helper's raw-bytes path — no silent `\r` stripping.
#[tokio::test]
async fn crlf_blob_body_round_trips_byte_for_byte() {
    let server = MockServer::start().await;
    // Empty prior tree → the new blob triggers a Create (POST).
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json::<Vec<Value>>(vec![]))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(201).set_body_json(issue_response(1, "ok")))
        .mount(&server)
        .await;

    // Frontmatter fences are LF-only; the body contains CRLF that must
    // survive intact all the way to the outgoing POST body.
    let blob = b"---\n\
id: 1\n\
title: crlf\n\
status: open\n\
created_at: 2026-04-13T00:00:00Z\n\
updated_at: 2026-04-13T00:00:00Z\n\
version: 0\n\
---\n\
line-one\r\nline-two\r\n";
    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&single_blob_export(blob));
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
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "push must succeed; stderr: {stderr}; stdout: {stdout}"
    );
    assert!(
        stdout.contains("ok refs/heads/main"),
        "stdout missing ok: {stdout}"
    );

    // Inspect what wiremock actually received on the POST.
    let requests = server.received_requests().await.unwrap();
    let post = requests
        .iter()
        .find(|r| r.method == wiremock::http::Method::POST)
        .expect("a POST was issued");
    // JSON-encoded `\r` appears as the 2-char escape `\r` (backslash-r).
    let body_str = std::str::from_utf8(&post.body).expect("POST body is UTF-8 JSON");
    assert!(
        body_str.contains("line-one\\r\\nline-two\\r\\n"),
        "POST body did not preserve CRLF — raw-bytes path stripped \\r; body={body_str}"
    );
}

/// H-02: Non-UTF-8 bytes in a blob must NOT cause the helper to fail
/// with a "stream did not contain valid UTF-8" torn-pipe error. The
/// raw-bytes `ProtoReader` path accepts any bytes; downstream processing
/// (frontmatter::parse via String::from_utf8_lossy) may substitute
/// U+FFFD replacement chars but must not abort mid-protocol.
#[tokio::test]
async fn non_utf8_blob_body_does_not_tear_pipe() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json::<Vec<Value>>(vec![]))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(201).set_body_json(issue_response(1, "ok")))
        .mount(&server)
        .await;

    // Frontmatter YAML stays valid UTF-8; the body contains raw 0xFF 0xFE
    // 0xFD — the bytes that used to trip `read_line`'s UTF-8 check and
    // torn the git pipe (H-02). Raw-bytes path must carry them through.
    let mut blob: Vec<u8> = Vec::new();
    blob.extend_from_slice(
        b"---\n\
id: 1\n\
title: non-utf8\n\
status: open\n\
created_at: 2026-04-13T00:00:00Z\n\
updated_at: 2026-04-13T00:00:00Z\n\
version: 0\n\
---\n\
prefix-",
    );
    blob.extend_from_slice(&[0xFF, 0xFE, 0xFD]);
    blob.push(b'\n');

    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&single_blob_export(&blob));
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
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The key H-02 invariant: the helper must NOT surface a UTF-8
    // decoding error from the protocol reader. It must either push
    // (with U+FFFD replacement) OR emit a clean protocol error line.
    assert!(
        !stderr.contains("stream did not contain valid UTF-8"),
        "H-02 regression: raw bytes tripped UTF-8 check; stderr: {stderr}"
    );
    assert!(
        !stderr.contains("invalid UTF-8"),
        "H-02 regression: raw bytes tripped UTF-8 check; stderr: {stderr}"
    );
    // The happy outcome is a successful push (from_utf8_lossy substitutes
    // U+FFFD, planner accepts, POST fires). Either way stdout must end in
    // a clean protocol response — NOT a torn pipe.
    assert!(
        stdout.contains("ok refs/heads/main") || stdout.contains("error refs/heads/main"),
        "stdout missing protocol response line (torn pipe?); stdout: {stdout}"
    );
}
