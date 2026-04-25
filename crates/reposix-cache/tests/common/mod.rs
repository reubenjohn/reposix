//! Shared test harness: spin up a wiremock server that satisfies the
//! sim's `GET /projects/<p>/issues` + `GET /projects/<p>/issues/<id>`
//! routes so `reposix_core::backend::sim::SimBackend` can be pointed at
//! it. Used by every integration test in this crate.

#![allow(dead_code)]

use std::sync::Arc;

use reposix_core::backend::sim::SimBackend;
use reposix_core::BackendConnector;
use reposix_core::{Issue, IssueId, IssueStatus};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Build `n` deterministic test issues with ids 1..=n.
#[must_use]
pub fn sample_issues(project: &str, n: usize) -> Vec<Issue> {
    use chrono::TimeZone;
    let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
    (1..=n)
        .map(|i| Issue {
            id: IssueId(i as u64),
            title: format!("issue {i} in {project}"),
            status: IssueStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: t,
            updated_at: t,
            version: 1,
            body: format!("body of issue {i}"),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        })
        .collect()
}

/// Seed a wiremock server so that
/// `GET /projects/<project>/issues` returns the provided list and
/// `GET /projects/<project>/issues/<id>` returns the matching single
/// issue (or 404).
pub async fn seed_mock(server: &MockServer, project: &str, issues: &[Issue]) {
    // List route.
    let list_body: Vec<serde_json::Value> = issues.iter().map(issue_to_json).collect();
    Mock::given(method("GET"))
        .and(path(format!("/projects/{project}/issues")))
        .respond_with(ResponseTemplate::new(200).set_body_json(list_body))
        .mount(server)
        .await;

    // Per-issue routes.
    for issue in issues {
        let id = issue.id.0;
        Mock::given(method("GET"))
            .and(path(format!("/projects/{project}/issues/{id}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue_to_json(issue)))
            .mount(server)
            .await;
    }

    // Catch-all 404 for unknown ids (so tests fail fast if the cache
    // requests an OID it hasn't populated).
    Mock::given(method("GET"))
        .and(path_regex(format!(r"^/projects/{project}/issues/\d+$")))
        .respond_with(ResponseTemplate::new(404))
        .mount(server)
        .await;
}

fn issue_to_json(issue: &Issue) -> serde_json::Value {
    serde_json::json!({
        "id": issue.id.0,
        "title": issue.title,
        "status": issue.status.as_str(),
        "assignee": issue.assignee,
        "labels": issue.labels,
        "created_at": issue.created_at.to_rfc3339(),
        "updated_at": issue.updated_at.to_rfc3339(),
        "version": issue.version,
        "body": issue.body,
    })
}

/// Build an Arc-wrapped `SimBackend` pointed at `server`.
#[must_use]
pub fn sim_backend(server: &MockServer) -> Arc<dyn BackendConnector> {
    Arc::new(SimBackend::new(server.uri()).expect("SimBackend::new"))
}

/// Replace `$REPOSIX_CACHE_DIR` with a path inside the tempdir so the
/// test never writes into the real user cache. Returns the previous
/// value (if any) for restoration.
#[must_use]
pub fn set_cache_dir(path: &std::path::Path) -> Option<String> {
    let prev = std::env::var(reposix_cache::CACHE_DIR_ENV).ok();
    std::env::set_var(reposix_cache::CACHE_DIR_ENV, path);
    prev
}

pub fn restore_cache_dir(prev: Option<String>) {
    match prev {
        Some(v) => std::env::set_var(reposix_cache::CACHE_DIR_ENV, v),
        None => std::env::remove_var(reposix_cache::CACHE_DIR_ENV),
    }
}
