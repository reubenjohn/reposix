//! [`SimBackend`] — concrete [`IssueBackend`] implementation that speaks to
//! the in-process `reposix-sim` HTTP routes.
//!
//! Every method is a thin shim:
//! 1. Build `{origin}/projects/{project}/issues[/<id>]`.
//! 2. Send via the shared [`HttpClient`] (re-routed through the egress
//!    allowlist gate, SG-01).
//! 3. Deserialize the response into [`Issue`] via `serde_json::from_slice`.
//!
//! The existing sim route handlers in `crates/reposix-sim/src/routes/issues.rs`
//! define the wire shape; this module is the client-side dual.
//!
//! # Agent identification
//!
//! Every request carries `X-Reposix-Agent: reposix-core-simbackend-<pid>` so
//! the simulator's audit middleware can attribute rows to the adapter
//! instance. PID is captured once at construction time.

use std::sync::Arc;

use async_trait::async_trait;
use reqwest::{Method, StatusCode};

use crate::backend::{BackendFeature, DeleteReason, IssueBackend};
use crate::http::{client, ClientOpts, HttpClient};
use crate::issue::{Issue, IssueId, IssueStatus};
use crate::taint::Untainted;
use crate::{Error, Result};

/// `IssueBackend` implementation for the in-process simulator.
///
/// Construct via [`SimBackend::new`] with the sim's origin (typically
/// `"http://127.0.0.1:7878"`). All methods reuse a single shared
/// [`HttpClient`] wrapped in [`Arc`] so clones are cheap.
#[derive(Debug, Clone)]
pub struct SimBackend {
    http: Arc<HttpClient>,
    origin: String,
    agent_header: String,
}

impl SimBackend {
    /// Build a new [`SimBackend`] pointed at `origin`.
    ///
    /// `origin` must be an allowlisted URL (SG-01) — typically
    /// `http://127.0.0.1:<port>`. A non-allowlisted origin will surface as
    /// [`Error::InvalidOrigin`] on the first request, not at construction.
    ///
    /// # Errors
    /// Propagates any error from [`client`] (e.g. bad
    /// `REPOSIX_ALLOWED_ORIGINS` spec).
    pub fn new(origin: String) -> Result<Self> {
        Self::with_agent_suffix(origin, None)
    }

    /// Build a [`SimBackend`] with a custom agent-header suffix so the
    /// simulator's rate-limit/audit layers can attribute requests to a
    /// specific simulated client.
    ///
    /// The emitted `X-Reposix-Agent` header is
    /// `reposix-core-simbackend-<pid>[-<suffix>]`. Passing `None` yields
    /// identical behaviour to [`SimBackend::new`].
    ///
    /// Use case: the swarm harness (`reposix-swarm`) calls this once per
    /// simulated agent to get per-client rate-limit buckets.
    ///
    /// # Errors
    /// Propagates any error from [`client`] (e.g. bad
    /// `REPOSIX_ALLOWED_ORIGINS` spec).
    pub fn with_agent_suffix(origin: String, suffix: Option<&str>) -> Result<Self> {
        let http = client(ClientOpts::default())?;
        let agent_header = match suffix {
            Some(s) => format!("reposix-core-simbackend-{}-{s}", std::process::id()),
            None => format!("reposix-core-simbackend-{}", std::process::id()),
        };
        Ok(Self {
            http: Arc::new(http),
            origin,
            agent_header,
        })
    }

    /// Strip a trailing slash from `origin` so URL assembly never produces
    /// `//projects/...`.
    fn base(&self) -> &str {
        self.origin.trim_end_matches('/')
    }

    /// Common headers used on every request. GET/DELETE pass only the agent
    /// header; POST/PATCH add `Content-Type: application/json`.
    fn agent_only(&self) -> Vec<(&str, &str)> {
        vec![("X-Reposix-Agent", self.agent_header.as_str())]
    }

    fn json_headers(&self) -> Vec<(&str, &str)> {
        vec![
            ("Content-Type", "application/json"),
            ("X-Reposix-Agent", self.agent_header.as_str()),
        ]
    }
}

/// Shared helper: consume a `reqwest::Response` and deserialize its body as
/// `Issue`, mapping HTTP errors to `Error::*`. 404 becomes
/// `Error::Other("not found: ...")`.
async fn decode_issue(resp: reqwest::Response, context: &str) -> Result<Issue> {
    let status = resp.status();
    let bytes = resp.bytes().await?;
    if status == StatusCode::NOT_FOUND {
        return Err(Error::Other(format!("not found: {context}")));
    }
    if !status.is_success() {
        return Err(Error::Other(format!(
            "sim returned {status} for {context}: {}",
            String::from_utf8_lossy(&bytes)
        )));
    }
    let issue: Issue = serde_json::from_slice(&bytes)?;
    Ok(issue)
}

async fn decode_issues(resp: reqwest::Response, context: &str) -> Result<Vec<Issue>> {
    let status = resp.status();
    let bytes = resp.bytes().await?;
    if !status.is_success() {
        return Err(Error::Other(format!(
            "sim returned {status} for {context}: {}",
            String::from_utf8_lossy(&bytes)
        )));
    }
    let list: Vec<Issue> = serde_json::from_slice(&bytes)?;
    Ok(list)
}

/// Render a `PatchIssueBody`-compatible JSON payload from an [`Issue`].
///
/// The sim's `PatchIssueBody` has `deny_unknown_fields`, so this only emits
/// the mutable-field subset: `title`, `body`, `status`, `assignee`, `labels`.
/// `assignee = None` is emitted as JSON `null` (clear); `Some(_)` as set.
fn render_patch_body(issue: &Issue) -> Result<Vec<u8>> {
    let mut map = serde_json::Map::new();
    map.insert(
        "title".into(),
        serde_json::Value::String(issue.title.clone()),
    );
    map.insert("body".into(), serde_json::Value::String(issue.body.clone()));
    map.insert(
        "status".into(),
        serde_json::Value::String(status_to_str(issue.status).to_owned()),
    );
    match &issue.assignee {
        None => {
            map.insert("assignee".into(), serde_json::Value::Null);
        }
        Some(a) => {
            map.insert("assignee".into(), serde_json::Value::String(a.clone()));
        }
    }
    map.insert(
        "labels".into(),
        serde_json::Value::Array(
            issue
                .labels
                .iter()
                .map(|l| serde_json::Value::String(l.clone()))
                .collect(),
        ),
    );
    Ok(serde_json::to_vec(&serde_json::Value::Object(map))?)
}

/// Render a `CreateIssueBody`-compatible JSON payload.
fn render_create_body(issue: &Issue) -> Result<Vec<u8>> {
    let mut map = serde_json::Map::new();
    map.insert(
        "title".into(),
        serde_json::Value::String(issue.title.clone()),
    );
    map.insert("body".into(), serde_json::Value::String(issue.body.clone()));
    map.insert(
        "status".into(),
        serde_json::Value::String(status_to_str(issue.status).to_owned()),
    );
    if let Some(a) = &issue.assignee {
        map.insert("assignee".into(), serde_json::Value::String(a.clone()));
    }
    map.insert(
        "labels".into(),
        serde_json::Value::Array(
            issue
                .labels
                .iter()
                .map(|l| serde_json::Value::String(l.clone()))
                .collect(),
        ),
    );
    Ok(serde_json::to_vec(&serde_json::Value::Object(map))?)
}

fn status_to_str(s: IssueStatus) -> &'static str {
    s.as_str()
}

#[async_trait]
impl IssueBackend for SimBackend {
    fn name(&self) -> &'static str {
        "simulator"
    }

    fn supports(&self, feature: BackendFeature) -> bool {
        // The in-process simulator implements the full matrix — it's the
        // reference backend that every other adapter is contract-tested
        // against.
        matches!(
            feature,
            BackendFeature::Delete
                | BackendFeature::Transitions
                | BackendFeature::StrongVersioning
                | BackendFeature::BulkEdit
                | BackendFeature::Workflows
        )
    }

    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>> {
        let url = format!("{}/projects/{}/issues", self.base(), project);
        let resp = self
            .http
            .request_with_headers(Method::GET, &url, &self.agent_only())
            .await?;
        decode_issues(resp, &url).await
    }

    async fn get_issue(&self, project: &str, id: IssueId) -> Result<Issue> {
        let url = format!("{}/projects/{}/issues/{}", self.base(), project, id.0);
        let resp = self
            .http
            .request_with_headers(Method::GET, &url, &self.agent_only())
            .await?;
        decode_issue(resp, &url).await
    }

    async fn create_issue(&self, project: &str, issue: Untainted<Issue>) -> Result<Issue> {
        let url = format!("{}/projects/{}/issues", self.base(), project);
        let body = render_create_body(issue.inner_ref())?;
        let resp = self
            .http
            .request_with_headers_and_body(Method::POST, &url, &self.json_headers(), Some(body))
            .await?;
        decode_issue(resp, &url).await
    }

    async fn update_issue(
        &self,
        project: &str,
        id: IssueId,
        patch: Untainted<Issue>,
        expected_version: Option<u64>,
    ) -> Result<Issue> {
        let url = format!("{}/projects/{}/issues/{}", self.base(), project, id.0);
        let body = render_patch_body(patch.inner_ref())?;
        // Build the If-Match header string in a local so the &str lifetime
        // outlives the `headers` vector passed into the HTTP layer.
        let if_match_val = expected_version.map(|v| format!("\"{v}\""));
        let mut headers = self.json_headers();
        if let Some(ref v) = if_match_val {
            headers.push(("If-Match", v.as_str()));
        }
        let resp = self
            .http
            .request_with_headers_and_body(Method::PATCH, &url, &headers, Some(body))
            .await?;
        let status = resp.status();
        if status == StatusCode::CONFLICT {
            let bytes = resp.bytes().await?;
            return Err(Error::Other(format!(
                "version mismatch: {}",
                String::from_utf8_lossy(&bytes)
            )));
        }
        decode_issue(resp, &url).await
    }

    async fn delete_or_close(
        &self,
        project: &str,
        id: IssueId,
        _reason: DeleteReason,
    ) -> Result<()> {
        // The sim performs a real DELETE regardless of reason — the reason
        // is meaningful only to backends (GitHub) that close with
        // state_reason. We preserve the argument in the signature so callers
        // can write backend-agnostic code.
        let url = format!("{}/projects/{}/issues/{}", self.base(), project, id.0);
        let resp = self
            .http
            .request_with_headers(Method::DELETE, &url, &self.agent_only())
            .await?;
        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!("not found: {url}")));
        }
        if !status.is_success() {
            let bytes = resp.bytes().await?;
            return Err(Error::Other(format!(
                "sim returned {status} for DELETE {url}: {}",
                String::from_utf8_lossy(&bytes)
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taint::{sanitize, ServerMetadata, Tainted};
    use chrono::{TimeZone, Utc};
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sample_issue_json(id: u64) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "title": "hello",
            "status": "open",
            "labels": [],
            "created_at": "2026-04-13T00:00:00Z",
            "updated_at": "2026-04-13T00:00:00Z",
            "version": 1,
            "body": ""
        })
    }

    fn sample_tainted() -> Tainted<Issue> {
        let t = Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        Tainted::new(Issue {
            id: IssueId(0),
            title: "agent authored".into(),
            status: IssueStatus::Open,
            assignee: None,
            labels: vec!["bug".into()],
            created_at: t,
            updated_at: t,
            version: 0,
            body: "body here".into(),
            parent_id: None,
        })
    }

    fn sample_untainted() -> Untainted<Issue> {
        let t = Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        sanitize(
            sample_tainted(),
            ServerMetadata {
                id: IssueId(42),
                created_at: t,
                updated_at: t,
                version: 3,
            },
        )
    }

    #[tokio::test]
    async fn list_builds_the_right_url() {
        let server = MockServer::start().await;
        // The agent header is always present (value varies by pid), so we
        // match only on method + path + response.
        Mock::given(method("GET"))
            .and(path("/projects/demo/issues"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                sample_issue_json(1),
                sample_issue_json(2),
            ])))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let issues = backend.list_issues("demo").await.expect("list");
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].id, IssueId(1));
        assert_eq!(issues[1].id, IssueId(2));
    }

    #[tokio::test]
    async fn get_builds_the_right_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/projects/demo/issues/7"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue_json(7)))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let issue = backend.get_issue("demo", IssueId(7)).await.expect("get");
        assert_eq!(issue.id, IssueId(7));
        assert_eq!(issue.title, "hello");
    }

    #[tokio::test]
    async fn get_maps_404_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/projects/demo/issues/9999"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let err = backend
            .get_issue("demo", IssueId(9999))
            .await
            .expect_err("404");
        match err {
            Error::Other(msg) => assert!(msg.starts_with("not found:"), "got {msg}"),
            other => panic!("expected Error::Other(not found), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn update_with_expected_version_attaches_if_match() {
        let server = MockServer::start().await;
        // Matcher asserts the If-Match header is exactly `"5"` (quoted etag).
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/42"))
            .and(header("If-Match", "\"5\""))
            .and(header("Content-Type", "application/json"))
            .and(body_partial_json(
                serde_json::json!({ "status": "in_progress" }),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue_json(42)))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        // Build an Untainted<Issue> with status=in_progress.
        let t = Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        let u = sanitize(
            Tainted::new(Issue {
                id: IssueId(0),
                title: "hello".into(),
                status: IssueStatus::InProgress,
                assignee: None,
                labels: vec![],
                created_at: t,
                updated_at: t,
                version: 0,
                body: String::new(),
                parent_id: None,
            }),
            ServerMetadata {
                id: IssueId(42),
                created_at: t,
                updated_at: t,
                version: 5,
            },
        );
        let out = backend
            .update_issue("demo", IssueId(42), u, Some(5))
            .await
            .expect("update");
        assert_eq!(out.id, IssueId(42));
    }

    #[tokio::test]
    async fn update_without_expected_version_is_wildcard() {
        // Strict negative proof: mount ONE matcher that explicitly requires
        // If-Match to be absent (via a closure matcher), with `.expect(1)` —
        // wiremock's `MockServer::drop` then panics if the mock wasn't hit
        // exactly once. Any leak of `If-Match` onto the wire turns this into
        // a verification failure rather than a false pass.
        use wiremock::Match;
        struct NoIfMatch;
        impl Match for NoIfMatch {
            fn matches(&self, request: &wiremock::Request) -> bool {
                !request.headers.contains_key("if-match")
            }
        }

        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/42"))
            .and(header("Content-Type", "application/json"))
            .and(NoIfMatch)
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue_json(42)))
            .expect(1)
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let u = sample_untainted();
        let out = backend
            .update_issue("demo", IssueId(42), u, None)
            .await
            .expect("update");
        assert_eq!(out.id, IssueId(42));
        // server drops here; .expect(1) panics if the no-If-Match matcher
        // didn't fire exactly once.
    }

    #[tokio::test]
    async fn supports_reports_full_matrix_for_sim() {
        let backend = SimBackend::new("http://127.0.0.1:7878".into()).expect("backend");
        for f in [
            BackendFeature::Delete,
            BackendFeature::Transitions,
            BackendFeature::StrongVersioning,
            BackendFeature::BulkEdit,
            BackendFeature::Workflows,
        ] {
            assert!(backend.supports(f), "expected sim to support {f:?}");
        }
        assert_eq!(backend.name(), "simulator");
    }
}
