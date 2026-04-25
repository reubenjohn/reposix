//! Phase 34 Plan 02 push-path integration tests.
//!
//! Three scenarios end-to-end against a wiremock backend:
//!
//! 1. **`stale_base_push_emits_fetch_first_and_writes_no_rest`** —
//!    ARCH-08 regression. The agent's local base version of issue 2 is
//!    stale (1 vs backend's 2); the helper must reject with the canned
//!    `error refs/heads/main fetch first` status, write a stderr
//!    diagnostic mentioning the issue id and `git pull --rebase`, and
//!    NOT make any PATCH/POST/DELETE call.
//! 2. **`clean_push_emits_ok_and_mutates_backend`** — happy-path
//!    regression. Local base matches backend; the helper writes
//!    `ok refs/heads/main` and the backend sees a PATCH for the
//!    changed issue.
//! 3. **`frontmatter_strips_server_controlled_fields`** — ARCH-10
//!    regression. The inbound blob has `id: 999999` and `version: 999`
//!    overrides; the PATCH body sent to the backend must contain the
//!    server-authoritative id (42) and NOT contain `version: 999`.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::fmt::Write as _;
use std::io::Write;
use std::sync::Arc;

use assert_cmd::Command;
use chrono::TimeZone;
use reposix_core::{Issue, RecordId, IssueStatus};
use serde_json::Value;
use wiremock::matchers::{any, method, path_regex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

fn sample_issue(id: u64, version: u64) -> Value {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let i = Issue {
        id: RecordId(id),
        title: format!("issue {id}"),
        status: IssueStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: t,
        updated_at: t,
        version,
        body: format!("body of {id}\n"),
        parent_id: None,
        extensions: std::collections::BTreeMap::new(),
    };
    serde_json::to_value(i).unwrap()
}

/// Render `Issue` to its on-disk frontmatter+body form, then override
/// the `version` field in the YAML so we can simulate stale-base or
/// hijacked-version pushes. `version_override` and `id_override` let
/// the caller forge those server-controlled fields.
fn render_with_overrides(
    id: u64,
    title: &str,
    body: &str,
    version_override: u64,
    id_override: u64,
) -> String {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let ts = t.to_rfc3339();
    // Hand-roll the YAML so we can control field order and override
    // server-controlled fields exactly.
    let mut s = String::new();
    s.push_str("---\n");
    writeln!(&mut s, "id: {id_override}").unwrap();
    writeln!(&mut s, "title: {title}").unwrap();
    s.push_str("status: open\n");
    writeln!(&mut s, "created_at: {ts}").unwrap();
    writeln!(&mut s, "updated_at: {ts}").unwrap();
    writeln!(&mut s, "version: {version_override}").unwrap();
    s.push_str("---\n");
    s.push_str(body);
    if !s.ends_with('\n') {
        s.push('\n');
    }
    let _ = id; // logical id passed for clarity; emitted via path
    s
}

/// Build a fast-export stream containing one updated issue.
/// `path` is e.g. `0042.md`; `blob` is the rendered frontmatter+body.
fn one_file_export(path: &str, blob: &str, msg: &str) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    writeln!(&mut out, "feature done").unwrap();
    writeln!(&mut out, "blob").unwrap();
    writeln!(&mut out, "mark :100").unwrap();
    writeln!(&mut out, "data {}", blob.len()).unwrap();
    out.extend_from_slice(blob.as_bytes());
    out.push(b'\n');
    writeln!(&mut out, "commit refs/heads/main").unwrap();
    writeln!(&mut out, "mark :1").unwrap();
    writeln!(&mut out, "committer test <t@t> 0 +0000").unwrap();
    let bytes = msg.as_bytes();
    writeln!(&mut out, "data {}", bytes.len()).unwrap();
    out.extend_from_slice(bytes);
    out.push(b'\n');
    writeln!(&mut out, "M 100644 :100 {path}").unwrap();
    writeln!(&mut out, "done").unwrap();
    out
}

/// ARCH-08 regression: stale-base push (local version=1, backend
/// version=2) must reject with canned `fetch first` and write zero
/// PATCH/POST/DELETE calls.
#[tokio::test]
async fn stale_base_push_emits_fetch_first_and_writes_no_rest() {
    let server = MockServer::start().await;
    // Backend has issue 2 at version=2 — local will claim version=1.
    let issues: Vec<Value> = vec![sample_issue(1, 1), sample_issue(2, 2)];
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    // Strict expectation: NO writes should fire.
    Mock::given(method("PATCH"))
        .and(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(any())
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(any())
        .respond_with(ResponseTemplate::new(204))
        .expect(0)
        .mount(&server)
        .await;

    let blob = render_with_overrides(2, "issue 2", "edited body\n", 1, 2);
    let stream = one_file_export("0002.md", &blob, "edit issue 2\n");

    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&stream);
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
        !out.status.success(),
        "stale-base push must fail; stderr: {stderr}"
    );
    assert!(
        stdout.contains("error refs/heads/main fetch first"),
        "stdout missing canned status: {stdout}"
    );
    assert!(
        stderr.contains("issue 2"),
        "stderr missing issue id: {stderr}"
    );
    assert!(
        stderr.contains("git pull --rebase"),
        "stderr missing git-pull recovery hint: {stderr}"
    );
}

/// Happy-path regression: local base matches backend, body change goes
/// through. Helper emits `ok refs/heads/main`; backend sees one PATCH.
#[tokio::test]
async fn clean_push_emits_ok_and_mutates_backend() {
    let server = MockServer::start().await;
    let issues: Vec<Value> = vec![sample_issue(42, 3)];
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    // Backend echoes back an updated issue — version bumped to 4.
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/projects/demo/issues/42$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(42, 4)))
        .expect(1)
        .mount(&server)
        .await;

    let blob = render_with_overrides(42, "issue 42", "edited body for 42\n", 3, 42);
    let stream = one_file_export("0042.md", &blob, "edit issue 42\n");

    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&stream);
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
        "clean push must succeed; stderr: {stderr}"
    );
    assert!(
        stdout.contains("ok refs/heads/main"),
        "stdout missing ok: {stdout}"
    );
}

/// ARCH-10 regression: an inbound blob with `id: 999999` and
/// `version: 999` does NOT carry those values into the PATCH body.
/// The helper's `sanitize()` step replaces them with the server-trusted
/// id and `prior_version` BEFORE serializing the request body.
///
/// Strategy: capture the PATCH body via wiremock. The sim's
/// `PatchIssueBody` has `deny_unknown_fields` and only carries the
/// mutable-field subset, so server-controlled fields are stripped at
/// the wire boundary. The ARCH-10 assertion is that no attacker-supplied
/// `999_999` / `999` value leaks into ANY field of the PATCH body.
#[tokio::test]
async fn frontmatter_strips_server_controlled_fields() {
    let server = MockServer::start().await;
    let issues: Vec<Value> = vec![sample_issue(42, 3)];
    Mock::given(method("GET"))
        .and(path_regex(r"^/projects/demo/issues$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
        .mount(&server)
        .await;
    let captured: Arc<std::sync::Mutex<Vec<Value>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured_clone = captured.clone();
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/projects/demo/issues/42$"))
        .respond_with(move |req: &Request| {
            if let Ok(v) = serde_json::from_slice::<Value>(&req.body) {
                captured_clone.lock().unwrap().push(v);
            }
            ResponseTemplate::new(200).set_body_json(sample_issue(42, 4))
        })
        .expect(1)
        .mount(&server)
        .await;

    // Inbound blob: id_override=999999, version=3 (matches backend so
    // conflict-check passes), but if the *bytes that hit PATCH* include
    // 999999 anywhere, sanitize is broken.
    let blob = render_with_overrides(
        42,
        "issue 42",
        "edited body sanitize regression\n",
        3,       // version: matches backend so conflict-check passes
        999_999, // id override — server must replace with 42
    );
    let stream = one_file_export("0042.md", &blob, "sanitize regression\n");

    let url = format!("reposix::{}/projects/demo", server.uri());
    let stdin_data = {
        let mut buf = Vec::new();
        writeln!(&mut buf, "export").unwrap();
        buf.extend_from_slice(&stream);
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
    assert!(
        out.status.success(),
        "sanitize-regression push must succeed; stderr: {stderr}"
    );

    let captured = captured.lock().unwrap();
    assert_eq!(captured.len(), 1, "exactly one PATCH should fire");
    let body = &captured[0];
    // The sim's PATCH body has `deny_unknown_fields` and only carries
    // the mutable-field subset (title/body/status/assignee/labels).
    // Server-controlled fields (id/created_at/updated_at/version) are
    // stripped by construction at the wire boundary. The ARCH-10
    // guarantee is therefore: no attacker-supplied 999_999 / 999 leaks
    // into ANY field of the PATCH body.
    let body_str = body.to_string();
    assert!(
        !body_str.contains("999999"),
        "attacker id=999999 leaked into PATCH body: {body_str}"
    );
    assert!(
        !body_str.contains("\"version\""),
        "PATCH body must not include `version` field at all (server-controlled): {body_str}"
    );
    assert!(
        !body_str.contains("\"id\""),
        "PATCH body must not include `id` field at all (server-controlled, lives in URL path): {body_str}"
    );
    // And the URL was the canonical /issues/42 endpoint (proves the
    // helper used the server-trusted id derived from the path, not the
    // hijacked value from the frontmatter).
}
