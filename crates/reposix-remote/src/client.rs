//! Thin async HTTP client for the simulator API.
//!
//! Mirrors the `EgressPayload` shape used by `reposix-fuse::fetch` so the
//! sim's `deny_unknown_fields` enforcement covers both clients identically.
//! Server-controlled fields physically cannot leak into PATCH/POST bodies.

#![forbid(unsafe_code)]

use std::time::Duration;

use reposix_core::http::HttpClient;
use reposix_core::{Issue, IssueId, Untainted};
use reqwest::Method;
use serde::Serialize;
use thiserror::Error;

const REQ_TIMEOUT: Duration = Duration::from_secs(5);

/// Errors reachable from this client.
#[derive(Debug, Error)]
pub enum ClientError {
    /// Wall-clock 5s elapsed before the backend responded.
    #[error("backend did not respond within {:?}", REQ_TIMEOUT)]
    Timeout,
    /// Backend returned a non-success status (with body for diagnostic).
    #[error("backend status {0}: {1}")]
    Status(reqwest::StatusCode, String),
    /// Backend returned 409 with a `current` version in the body.
    #[error("version mismatch: current={current}")]
    Conflict {
        /// Server's current version, parsed from the 409 body.
        current: u64,
    },
    /// Transport-level failure.
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    /// Allowlist or other core error.
    #[error("core: {0}")]
    Core(String),
    /// JSON parse failure.
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
}

fn from_core(e: reposix_core::Error) -> ClientError {
    match e {
        reposix_core::Error::InvalidOrigin(o) => ClientError::Core(format!("origin: {o}")),
        reposix_core::Error::Http(t) => ClientError::Transport(t),
        other => ClientError::Core(other.to_string()),
    }
}

/// Restricted JSON shape for outbound POST/PATCH bodies. Mirrors the same
/// shape used in `reposix-fuse::fetch::EgressPayload` to keep both clients
/// producing identical wire bytes.
#[derive(Debug, Serialize)]
struct EgressPayload<'a> {
    title: &'a str,
    status: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee: Option<&'a str>,
    labels: &'a [String],
    body: &'a str,
}

impl<'a> EgressPayload<'a> {
    fn from_untainted(u: &'a Untainted<Issue>) -> Self {
        let issue = u.inner_ref();
        Self {
            title: issue.title.as_str(),
            status: issue.status.as_str(),
            assignee: issue.assignee.as_deref(),
            labels: &issue.labels,
            body: issue.body.as_str(),
        }
    }
}

/// 409 conflict body shape. Matches the sim's `version_mismatch` response.
#[derive(serde::Deserialize)]
struct ConflictBody {
    current: u64,
}

fn issue_url(origin: &str, project: &str, id: IssueId) -> String {
    format!(
        "{}/projects/{}/issues/{}",
        origin.trim_end_matches('/'),
        project,
        id.0
    )
}

fn list_url(origin: &str, project: &str) -> String {
    format!(
        "{}/projects/{}/issues",
        origin.trim_end_matches('/'),
        project
    )
}

/// `GET /projects/{slug}/issues` — returns the full list.
///
/// # Errors
/// See [`ClientError`].
pub async fn list_issues(
    http: &HttpClient,
    origin: &str,
    project: &str,
    agent: &str,
) -> Result<Vec<Issue>, ClientError> {
    let url = list_url(origin, project);
    let resp = tokio::time::timeout(
        REQ_TIMEOUT,
        http.request_with_headers(Method::GET, &url, &[("X-Reposix-Agent", agent)]),
    )
    .await
    .map_err(|_| ClientError::Timeout)?
    .map_err(from_core)?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ClientError::Status(status, body));
    }
    let bytes = resp.bytes().await?;
    let issues: Vec<Issue> = serde_json::from_slice(&bytes)?;
    Ok(issues)
}

/// `PATCH /projects/{slug}/issues/{id}` with `If-Match: <version>`.
///
/// # Errors
/// - [`ClientError::Conflict`] on 409.
/// - See [`ClientError`].
pub async fn patch_issue(
    http: &HttpClient,
    origin: &str,
    project: &str,
    id: IssueId,
    version: u64,
    sanitized: Untainted<Issue>,
    agent: &str,
) -> Result<Issue, ClientError> {
    let url = issue_url(origin, project, id);
    let payload = EgressPayload::from_untainted(&sanitized);
    let body = serde_json::to_vec(&payload)?;
    let version_str = version.to_string();
    let headers = [
        ("If-Match", version_str.as_str()),
        ("Content-Type", "application/json"),
        ("X-Reposix-Agent", agent),
    ];
    let resp = tokio::time::timeout(
        REQ_TIMEOUT,
        http.request_with_headers_and_body(Method::PATCH, &url, &headers, Some(body)),
    )
    .await
    .map_err(|_| ClientError::Timeout)?
    .map_err(from_core)?;
    let status = resp.status();
    if status == reqwest::StatusCode::CONFLICT {
        let bytes = resp.bytes().await?;
        let parsed: ConflictBody =
            serde_json::from_slice(&bytes).unwrap_or(ConflictBody { current: 0 });
        return Err(ClientError::Conflict {
            current: parsed.current,
        });
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ClientError::Status(status, body));
    }
    let bytes = resp.bytes().await?;
    Ok(serde_json::from_slice(&bytes)?)
}

/// `POST /projects/{slug}/issues` — returns 201 + Issue.
///
/// # Errors
/// See [`ClientError`].
pub async fn post_issue(
    http: &HttpClient,
    origin: &str,
    project: &str,
    sanitized: Untainted<Issue>,
    agent: &str,
) -> Result<Issue, ClientError> {
    let url = list_url(origin, project);
    let payload = EgressPayload::from_untainted(&sanitized);
    let body = serde_json::to_vec(&payload)?;
    let headers = [
        ("Content-Type", "application/json"),
        ("X-Reposix-Agent", agent),
    ];
    let resp = tokio::time::timeout(
        REQ_TIMEOUT,
        http.request_with_headers_and_body(Method::POST, &url, &headers, Some(body)),
    )
    .await
    .map_err(|_| ClientError::Timeout)?
    .map_err(from_core)?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ClientError::Status(status, body));
    }
    let bytes = resp.bytes().await?;
    Ok(serde_json::from_slice(&bytes)?)
}

/// `DELETE /projects/{slug}/issues/{id}` — returns 204.
///
/// # Errors
/// See [`ClientError`].
pub async fn delete_issue(
    http: &HttpClient,
    origin: &str,
    project: &str,
    id: IssueId,
    agent: &str,
) -> Result<(), ClientError> {
    let url = issue_url(origin, project, id);
    let resp = tokio::time::timeout(
        REQ_TIMEOUT,
        http.request_with_headers(Method::DELETE, &url, &[("X-Reposix-Agent", agent)]),
    )
    .await
    .map_err(|_| ClientError::Timeout)?
    .map_err(from_core)?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ClientError::Status(status, body));
    }
    Ok(())
}
