//! P73 CONNECTOR-GAP-01: byte-exact Bearer-auth header assertion.
//!
//! Drives [`GithubReadOnlyBackend`] through the public [`BackendConnector`]
//! trait seam and asserts the `Authorization` header on every request
//! matches `Bearer <token>` exactly. Per
//! `docs/guides/write-your-own-connector.md:158`, byte-exact is the
//! contract — uses [`wiremock::matchers::header`] (returns
//! `HeaderExactMatcher`, NOT regex) per D-02. Auth header construction
//! confirmed at `crates/reposix-github/src/lib.rs:236`:
//! `format!("Bearer {tok}")`.

use reposix_core::backend::BackendConnector;
use reposix_github::GithubReadOnlyBackend;
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn auth_header_bearer_byte_exact() {
    // Distinctive synthetic token (does not authenticate against real GH).
    let token = "ghp_test_personal_access_token_xyz";
    let expected_header = format!("Bearer {token}");

    let server = MockServer::start().await;

    // GitHub's `list_records` issues GET /repos/<owner>/<repo>/issues with
    // query params state=all & per_page=100. Mount with byte-exact
    // Authorization matcher so wiremock panics on drop if the header
    // drifts even by a single byte.
    Mock::given(method("GET"))
        .and(path("/repos/acme/demo/issues"))
        .and(query_param("state", "all"))
        .and(query_param("per_page", "100"))
        .and(header("Authorization", expected_header.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let backend = GithubReadOnlyBackend::new_with_base_url(Some(token.to_string()), server.uri())
        .expect("GithubReadOnlyBackend::new_with_base_url");

    let records = backend
        .list_records("acme/demo")
        .await
        .expect("list_records should succeed");
    assert!(records.is_empty(), "expected empty issue list from wiremock");
}
