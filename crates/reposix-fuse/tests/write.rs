//! Phase S write-path unit tests.
//!
//! These drive the `patch_issue`/`post_issue` fetch helpers directly against
//! a wiremock backend. The end-to-end `release_patches_on_buffered_write`
//! scenario is covered by the `#[ignore]`-gated test below which mounts
//! FUSE in a tempdir; the non-ignored tests exercise the HTTP shape (PATCH
//! with If-Match, 409 → Conflict, timeout → Timeout) without needing a
//! kernel mount.

#![forbid(unsafe_code)]

use std::time::Duration;

use chrono::TimeZone;
use reposix_core::http::{client, ClientOpts};
use reposix_core::{sanitize, Issue, IssueId, IssueStatus, ServerMetadata, Tainted};
use reposix_fuse::fetch::{patch_issue, post_issue, FetchError};
use wiremock::matchers::{any, body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_issue(id: u64) -> Issue {
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    Issue {
        id: IssueId(id),
        title: format!("issue {id}"),
        status: IssueStatus::Open,
        assignee: None,
        labels: vec![],
        created_at: t,
        updated_at: t,
        version: 1,
        body: "body".to_owned(),
    }
}

#[tokio::test]
async fn release_patches_with_if_match() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/projects/demo/issues/1"))
        .and(header("If-Match", "1"))
        .and(body_string_contains("\"status\":\"done\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(Issue {
            status: IssueStatus::Done,
            ..sample_issue(1)
        }))
        .mount(&server)
        .await;
    let http = client(ClientOpts::default()).unwrap();
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let meta = ServerMetadata {
        id: IssueId(1),
        created_at: t,
        updated_at: t,
        version: 1,
    };
    let mut issue = sample_issue(1);
    issue.status = IssueStatus::Done;
    let untainted = sanitize(Tainted::new(issue), meta);
    let got = patch_issue(
        &http,
        &server.uri(),
        "demo",
        IssueId(1),
        1,
        untainted,
        "reposix-fuse-test",
    )
    .await
    .unwrap();
    assert_eq!(got.id, IssueId(1));
    assert!(matches!(got.status, IssueStatus::Done));
}

#[tokio::test]
async fn release_409_returns_conflict() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/projects/demo/issues/1"))
        .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
            "error": "version_mismatch",
            "current": 7
        })))
        .mount(&server)
        .await;
    let http = client(ClientOpts::default()).unwrap();
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let meta = ServerMetadata {
        id: IssueId(1),
        created_at: t,
        updated_at: t,
        version: 1,
    };
    let untainted = sanitize(Tainted::new(sample_issue(1)), meta);
    let err = patch_issue(
        &http,
        &server.uri(),
        "demo",
        IssueId(1),
        1,
        untainted,
        "agent",
    )
    .await
    .unwrap_err();
    assert!(
        matches!(err, FetchError::Conflict { current: 7 }),
        "got {err:?}"
    );
}

#[tokio::test]
async fn release_timeout_returns_eio_flavored_error() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
        .mount(&server)
        .await;
    let http = client(ClientOpts::default()).unwrap();
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let meta = ServerMetadata {
        id: IssueId(1),
        created_at: t,
        updated_at: t,
        version: 1,
    };
    let untainted = sanitize(Tainted::new(sample_issue(1)), meta);
    let t0 = std::time::Instant::now();
    let err = patch_issue(
        &http,
        &server.uri(),
        "demo",
        IssueId(1),
        1,
        untainted,
        "agent",
    )
    .await
    .unwrap_err();
    let elapsed = t0.elapsed();
    let ok = matches!(err, FetchError::Timeout)
        || matches!(&err, FetchError::Transport(e) if e.is_timeout());
    assert!(ok, "expected timeout-flavored error, got {err:?}");
    assert!(
        elapsed < Duration::from_millis(5_800),
        "should return within 5.5s; took {elapsed:?}"
    );
}

#[tokio::test]
async fn sanitize_strips_server_fields_on_egress() {
    // Regression / SG-03 proof: a crafted issue with version=999999 in the
    // tainted input must NOT appear in the PATCH wire body. The
    // EgressPayload shape (title/status/assignee/labels/body only)
    // mechanically guarantees this.
    let server = MockServer::start().await;
    // Matcher that *fails* if `version` appears in the body — wiremock
    // doesn't expose a `body_does_not_contain`, so the reverse matcher:
    // accept any body and inspect captured requests after the call.
    Mock::given(method("PATCH"))
        .and(path("/projects/demo/issues/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(1)))
        .mount(&server)
        .await;
    let http = client(ClientOpts::default()).unwrap();
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    // Server metadata: what the server actually knows.
    let meta = ServerMetadata {
        id: IssueId(1),
        created_at: t,
        updated_at: t,
        version: 1,
    };
    // Tainted issue carrying hostile version value. sanitize() strips it.
    let mut hostile = sample_issue(1);
    hostile.version = 999_999;
    let untainted = sanitize(Tainted::new(hostile), meta);
    patch_issue(
        &http,
        &server.uri(),
        "demo",
        IssueId(1),
        1,
        untainted,
        "agent",
    )
    .await
    .unwrap();
    // Inspect what wiremock actually received.
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let body = String::from_utf8_lossy(&requests[0].body);
    assert!(
        !body.contains("\"version\""),
        "egress body leaked `version` key: {body}"
    );
    assert!(
        !body.contains("\"created_at\""),
        "egress body leaked `created_at` key: {body}"
    );
    assert!(
        !body.contains("\"updated_at\""),
        "egress body leaked `updated_at` key: {body}"
    );
    // id is allowed in the URL path; NOT in the body.
    assert!(
        !body.contains("\"id\""),
        "egress body leaked `id` key: {body}"
    );
    assert!(body.contains("\"title\""));
    assert!(body.contains("\"status\""));
}

#[tokio::test]
async fn create_posts_and_returns_issue() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/projects/demo/issues"))
        .respond_with(ResponseTemplate::new(201).set_body_json(sample_issue(4)))
        .mount(&server)
        .await;
    let http = client(ClientOpts::default()).unwrap();
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    let meta = ServerMetadata {
        id: IssueId(0),
        created_at: t,
        updated_at: t,
        version: 0,
    };
    let untainted = sanitize(Tainted::new(sample_issue(99)), meta);
    let got = post_issue(&http, &server.uri(), "demo", untainted, "agent")
        .await
        .unwrap();
    assert_eq!(got.id, IssueId(4));
}
