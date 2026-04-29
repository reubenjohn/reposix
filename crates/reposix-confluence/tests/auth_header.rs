//! P73 CONNECTOR-GAP-01: byte-exact Basic-auth header assertion.
//!
//! Drives [`ConfluenceBackend`] through the public [`BackendConnector`] trait
//! and asserts the `Authorization` header on every request matches
//! `Basic <base64(email:token)>` exactly. Per
//! `docs/guides/write-your-own-connector.md:158`, byte-exact prefix is the
//! contract — uses [`wiremock::matchers::header`] (returns `HeaderExactMatcher`,
//! NOT regex) per D-02. The plan-time prose cited `header_exact` as the
//! function name; the actual public API is `header(K, V) -> HeaderExactMatcher`
//! per `wiremock-0.6.5/src/matchers.rs:355`. Same byte-exact semantics.
//!
//! Drives the public `BackendConnector` trait seam (the same surface the
//! FUSE daemon and CLI consume), so private helpers cannot drift the
//! header without this test catching it.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
use reposix_core::backend::BackendConnector;
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn auth_header_basic_byte_exact() {
    // Test creds chosen to make the base64 distinctive (not all-zero, not
    // valid against any real Atlassian tenant).
    let email = "test-user@example.com";
    let token = "atlassian-api-token-xyz123";
    let expected_header = format!(
        "Basic {}",
        STANDARD.encode(format!("{email}:{token}").as_bytes())
    );

    let server = MockServer::start().await;

    // 1. Space resolver — `BackendConnector::list_records` issues this first.
    //    Mount with `header` so wiremock panics on drop if the header
    //    drifted from `Basic <base64>` even by a single byte.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "DEMO"))
        .and(header("Authorization", expected_header.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"id": "1001", "key": "DEMO"}]
        })))
        .mount(&server)
        .await;

    // 2. Page list endpoint — empty results so we don't have to mount
    //    individual page bodies. The header matcher on this mount
    //    catches any per-call header divergence (e.g. if a future change
    //    introduced a request-specific Authorization variant).
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces/1001/pages"))
        .and(header("Authorization", expected_header.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [],
            "_links": {}
        })))
        .mount(&server)
        .await;

    let creds = ConfluenceCreds {
        email: email.to_string(),
        api_token: token.to_string(),
    };
    let backend =
        ConfluenceBackend::new_with_base_url(creds, server.uri()).expect("ConfluenceBackend");

    // Drive list_records through the public BackendConnector seam.
    // Wiremock panics on drop if any mounted Mock receives no matching
    // request — so a wrong header produces a clear failure pointing at
    // the offending matcher.
    let records = backend
        .list_records("DEMO")
        .await
        .expect("list_records should succeed");
    assert!(records.is_empty(), "expected empty page list from wiremock");
}
