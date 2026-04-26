//! [`SimBackend`] — concrete [`BackendConnector`] implementation that speaks to
//! the in-process `reposix-sim` HTTP routes.
//!
//! Every method is a thin shim:
//! 1. Build `{origin}/projects/{project}/issues[/<id>]`.
//! 2. Send via the shared [`HttpClient`] (re-routed through the egress
//!    allowlist gate, SG-01).
//! 3. Deserialize the response into [`Record`] via `serde_json::from_slice`.
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

use crate::backend::{BackendConnector, BackendFeature, DeleteReason};
use crate::http::{client, ClientOpts, HttpClient};
use crate::record::{Record, RecordId, RecordStatus};
use crate::taint::Untainted;
use crate::{Error, Result};

/// `BackendConnector` implementation for the in-process simulator.
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
/// `Record`, mapping HTTP errors to `Error::*`. 404 becomes
/// `Error::NotFound { project, id }` (project + id parsed from the request URL).
async fn decode_issue(resp: reqwest::Response, context: &str) -> Result<Record> {
    let status = resp.status();
    let bytes = resp.bytes().await?;
    if status == StatusCode::NOT_FOUND {
        let (project, id) = parse_project_id_from_url(context);
        return Err(Error::NotFound { project, id });
    }
    if !status.is_success() {
        return Err(Error::Other(format!(
            "sim returned {status} for {context}: {}",
            String::from_utf8_lossy(&bytes)
        )));
    }
    let issue: Record = serde_json::from_slice(&bytes)?;
    Ok(issue)
}

/// Best-effort parse of `project` and `id` from a sim URL of the form
/// `<origin>/projects/<project>/issues[/<id>]`. Falls back to the raw URL
/// when the shape doesn't match — `Error::NotFound { project, id }` is for
/// callers that want structured fields, never for routing logic, so a lossy
/// fallback is acceptable.
fn parse_project_id_from_url(url: &str) -> (String, String) {
    // Strip any query string before splitting (e.g. `?since=...`).
    let path = url.split('?').next().unwrap_or(url);
    if let Some(rest) = path.split("/projects/").nth(1) {
        let mut parts = rest.splitn(3, '/');
        let project = parts.next().unwrap_or("").to_owned();
        // parts.next() is "issues" (or similar); skip it.
        let _ = parts.next();
        let id = parts.next().unwrap_or("").to_owned();
        return (project, id);
    }
    (String::new(), url.to_owned())
}

async fn decode_issues(resp: reqwest::Response, context: &str) -> Result<Vec<Record>> {
    let status = resp.status();
    let bytes = resp.bytes().await?;
    if !status.is_success() {
        return Err(Error::Other(format!(
            "sim returned {status} for {context}: {}",
            String::from_utf8_lossy(&bytes)
        )));
    }
    let list: Vec<Record> = serde_json::from_slice(&bytes)?;
    Ok(list)
}

/// Render a `PatchIssueBody`-compatible JSON payload from an [`Record`].
///
/// The sim's `PatchIssueBody` has `deny_unknown_fields`, so this only emits
/// the mutable-field subset: `title`, `body`, `status`, `assignee`, `labels`.
/// `assignee = None` is emitted as JSON `null` (clear); `Some(_)` as set.
fn render_patch_body(issue: &Record) -> Result<Vec<u8>> {
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
fn render_create_body(issue: &Record) -> Result<Vec<u8>> {
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

fn status_to_str(s: RecordStatus) -> &'static str {
    s.as_str()
}

/// Best-effort parse of the sim's HTTP 409 body shape into
/// `(current, requested)` strings for `Error::VersionMismatch`. Returns
/// empty strings when the body isn't the expected JSON; callers that need
/// the raw bytes use `Error::VersionMismatch.body` directly.
fn parse_version_mismatch_body(body: &str, expected: Option<u64>) -> (String, String) {
    let parsed: Option<serde_json::Value> = serde_json::from_str(body).ok();
    let current = parsed
        .as_ref()
        .and_then(|v| v.get("current"))
        .and_then(|c| {
            c.as_u64()
                .map(|n| n.to_string())
                .or_else(|| c.as_str().map(ToOwned::to_owned))
        })
        .unwrap_or_default();
    let requested = parsed
        .as_ref()
        .and_then(|v| v.get("sent"))
        .and_then(|s| {
            s.as_str()
                .map(ToOwned::to_owned)
                .or_else(|| s.as_u64().map(|n| n.to_string()))
        })
        .or_else(|| expected.map(|v| v.to_string()))
        .unwrap_or_default();
    (current, requested)
}

#[async_trait]
impl BackendConnector for SimBackend {
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

    async fn list_records(&self, project: &str) -> Result<Vec<Record>> {
        let url = format!("{}/projects/{}/issues", self.base(), project);
        let resp = self
            .http
            .request_with_headers(Method::GET, &url, &self.agent_only())
            .await?;
        decode_issues(resp, &url).await
    }

    async fn list_changed_since(
        &self,
        project: &str,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<RecordId>> {
        // RFC3339-with-Z form contains only digits, `T`, `-`, `:`, `Z`;
        // none require percent-encoding in a query value, so format!()
        // is safe. If callers ever pass non-UTC or fractional seconds
        // that widen the charset, switch to `url::form_urlencoded`.
        let since_iso = since.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let url = format!(
            "{}/projects/{}/issues?since={}",
            self.base(),
            project,
            since_iso
        );
        let resp = self
            .http
            .request_with_headers(Method::GET, &url, &self.agent_only())
            .await?;
        let issues = decode_issues(resp, &url).await?;
        Ok(issues.into_iter().map(|i| i.id).collect())
    }

    async fn get_record(&self, project: &str, id: RecordId) -> Result<Record> {
        let url = format!("{}/projects/{}/issues/{}", self.base(), project, id.0);
        let resp = self
            .http
            .request_with_headers(Method::GET, &url, &self.agent_only())
            .await?;
        decode_issue(resp, &url).await
    }

    async fn create_record(&self, project: &str, issue: Untainted<Record>) -> Result<Record> {
        let url = format!("{}/projects/{}/issues", self.base(), project);
        let body = render_create_body(issue.inner_ref())?;
        let resp = self
            .http
            .request_with_headers_and_body(Method::POST, &url, &self.json_headers(), Some(body))
            .await?;
        decode_issue(resp, &url).await
    }

    async fn update_record(
        &self,
        project: &str,
        id: RecordId,
        patch: Untainted<Record>,
        expected_version: Option<u64>,
    ) -> Result<Record> {
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
            let body = String::from_utf8_lossy(&bytes).into_owned();
            // Best-effort parse of the sim's 409 body shape:
            // `{"error":"version_mismatch","current":<u64>,"sent":"<str>"}`.
            // Falls back to empty strings when the body isn't the expected
            // JSON — callers wanting the raw bytes use `body` directly.
            let (current, requested) = parse_version_mismatch_body(&body, expected_version);
            return Err(Error::VersionMismatch {
                current,
                requested,
                body,
            });
        }
        decode_issue(resp, &url).await
    }

    async fn delete_or_close(
        &self,
        project: &str,
        id: RecordId,
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
            let (project_p, id_p) = parse_project_id_from_url(&url);
            return Err(Error::NotFound {
                project: project_p,
                id: id_p,
            });
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
    use wiremock::matchers::{body_partial_json, header, method, path, query_param};
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

    fn sample_tainted() -> Tainted<Record> {
        let t = Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        Tainted::new(Record {
            id: RecordId(0),
            title: "agent authored".into(),
            status: RecordStatus::Open,
            assignee: None,
            labels: vec!["bug".into()],
            created_at: t,
            updated_at: t,
            version: 0,
            body: "body here".into(),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        })
    }

    fn sample_untainted() -> Untainted<Record> {
        let t = Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        sanitize(
            sample_tainted(),
            ServerMetadata {
                id: RecordId(42),
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
        let issues = backend.list_records("demo").await.expect("list");
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].id, RecordId(1));
        assert_eq!(issues[1].id, RecordId(2));
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
        let issue = backend.get_record("demo", RecordId(7)).await.expect("get");
        assert_eq!(issue.id, RecordId(7));
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
            .get_record("demo", RecordId(9999))
            .await
            .expect_err("404");
        match err {
            Error::NotFound { project, id } => {
                assert_eq!(project, "demo", "expected project='demo', got {project}");
                assert_eq!(id, "9999", "expected id='9999', got {id}");
            }
            other => panic!("expected Error::NotFound, got {other:?}"),
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
        // Build an Untainted<Record> with status=in_progress.
        let t = Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        let u = sanitize(
            Tainted::new(Record {
                id: RecordId(0),
                title: "hello".into(),
                status: RecordStatus::InProgress,
                assignee: None,
                labels: vec![],
                created_at: t,
                updated_at: t,
                version: 0,
                body: String::new(),
                parent_id: None,
                extensions: std::collections::BTreeMap::new(),
            }),
            ServerMetadata {
                id: RecordId(42),
                created_at: t,
                updated_at: t,
                version: 5,
            },
        );
        let out = backend
            .update_record("demo", RecordId(42), u, Some(5))
            .await
            .expect("update");
        assert_eq!(out.id, RecordId(42));
    }

    #[tokio::test]
    async fn update_issue_409_prefix_is_version_mismatch() {
        // R13 mitigation: pin the sim's 409 VersionMismatch body shape before
        // Wave B1's `fs.rs::backend_err_to_fetch` string-matches the
        // `"version mismatch: "` prefix and JSON-parses the tail to recover
        // `current` into `FetchError::Conflict { current }`. See
        // 14-RESEARCH.md#Q1 and 14-PLAN.md risk R13. If the sim ever changes
        // the `{"error":"version_mismatch","current":N,"sent":"..."}` body
        // shape, this fires in the core crate rather than silently degrading
        // the FUSE write path's conflict-current log line.
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/42"))
            .and(header("If-Match", "\"1\""))
            .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
                "error": "version_mismatch",
                "current": 7,
                "sent": "1",
            })))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let u = sample_untainted();
        let err = backend
            .update_record("demo", RecordId(42), u, Some(1))
            .await
            .expect_err("409");
        match err {
            Error::VersionMismatch {
                ref current,
                ref requested,
                ref body,
            } => {
                assert_eq!(current, "7", "expected current='7', got {current}");
                assert_eq!(requested, "1", "expected requested='1', got {requested}");
                assert!(
                    body.contains("\"current\":7"),
                    "expected body to contain '\"current\":7', got {body}"
                );
                // Display string still surfaces the `version mismatch:`
                // prefix — keeps `reposix-swarm`'s ErrorKind::classify
                // substring-matching working during the migration.
                assert!(
                    err.to_string().starts_with("version mismatch:"),
                    "expected display prefix 'version mismatch:', got {err}"
                );
            }
            other => panic!("expected Error::VersionMismatch, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn update_issue_409_current_field_present_as_json() {
        // POLISH2-09: closes the stringly-typed protocol. Was: parse JSON
        // out of `Error::Other("version mismatch: {body}")` via
        // `strip_prefix("version mismatch: ")` + `serde_json::from_str(tail)`.
        // Now: pattern-match `Error::VersionMismatch { body, .. }` and parse
        // `body` directly. Code-quality audit P1-5.
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/42"))
            .and(header("If-Match", "\"1\""))
            .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
                "error": "version_mismatch",
                "current": 7,
                "sent": "1",
            })))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let u = sample_untainted();
        let err = backend
            .update_record("demo", RecordId(42), u, Some(1))
            .await
            .expect_err("409");
        let Error::VersionMismatch { body, .. } = err else {
            panic!("expected Error::VersionMismatch, got {err:?}");
        };
        // The body field carries the raw 409 response — parse it directly,
        // no string slicing required.
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("body parses as JSON");
        let current = parsed
            .get("current")
            .expect("'current' key present")
            .as_u64()
            .expect("'current' is a u64");
        assert_eq!(current, 7, "expected current=7, got {current}");
    }

    #[tokio::test]
    async fn version_mismatch_round_trips_typed_body() {
        // POLISH2-09 happy path: prove `Error::VersionMismatch { body, .. }`
        // exposes the raw 409 body without substring fallback. Companion to
        // `update_issue_409_current_field_present_as_json`; here the body is
        // a deliberately exotic string that *contains* the legacy prefix so
        // any re-introduction of `strip_prefix("version mismatch: ")` would
        // mis-parse and fail.
        let server = MockServer::start().await;
        let hostile_body = "version mismatch: not really, just a label";
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/42"))
            .and(header("If-Match", "\"1\""))
            .respond_with(ResponseTemplate::new(409).set_body_string(hostile_body))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let u = sample_untainted();
        let err = backend
            .update_record("demo", RecordId(42), u, Some(1))
            .await
            .expect_err("409");
        match err {
            Error::VersionMismatch { body, .. } => {
                assert_eq!(
                    body, hostile_body,
                    "body field must carry the raw 409 response verbatim"
                );
            }
            other => panic!("expected Error::VersionMismatch, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn not_found_round_trips_typed() {
        // POLISH2-09 happy path: prove the 404 → `Error::NotFound { project,
        // id }` mapping populates structured fields, not just a stringly
        // wrapped message. Companion to `get_maps_404_to_not_found`.
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/projects/proj-x/issues/777"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let err = backend
            .get_record("proj-x", RecordId(777))
            .await
            .expect_err("404");
        match err {
            Error::NotFound { project, id } => {
                assert_eq!(project, "proj-x");
                assert_eq!(id, "777");
            }
            other => panic!("expected Error::NotFound, got {other:?}"),
        }
        // Display string keeps the `not found:` prefix for swarm classifier
        // back-compat during the v0.12.0 wider migration.
        let again = backend
            .get_record("proj-x", RecordId(777))
            .await
            .expect_err("404");
        assert!(
            again.to_string().starts_with("not found:"),
            "expected display prefix 'not found:', got {again}"
        );
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
            .update_record("demo", RecordId(42), u, None)
            .await
            .expect("update");
        assert_eq!(out.id, RecordId(42));
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

    // ----- Phase 14 Wave B1 re-homes -----
    //
    // The tests below were formerly in `crates/reposix-fuse/src/fetch.rs::tests`
    // and `crates/reposix-fuse/tests/write.rs`. They exercised
    // `patch_issue`/`post_issue` directly; after Wave B1 those helpers are
    // deleted and the FUSE write path routes through `BackendConnector`, so the
    // same wire assertions move here and pin the same contract at the trait
    // impl layer. See `.planning/phases/14-.../14-RESEARCH.md` §Q10.

    #[tokio::test]
    async fn update_issue_sends_quoted_if_match() {
        // Re-home of fetch.rs::patch_issue_sends_if_match_header. Variant of
        // the existing `update_with_expected_version_attaches_if_match` at
        // version=3 (previous test pins version=5); both prove the quoted-etag
        // form matches the sim's handler invariant.
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/1"))
            .and(header("If-Match", "\"3\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue_json(1)))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let t = Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        let u = sanitize(
            Tainted::new(Record {
                id: RecordId(0),
                title: "hello".into(),
                status: RecordStatus::Open,
                assignee: None,
                labels: vec![],
                created_at: t,
                updated_at: t,
                version: 0,
                body: String::new(),
                parent_id: None,
                extensions: std::collections::BTreeMap::new(),
            }),
            ServerMetadata {
                id: RecordId(1),
                created_at: t,
                updated_at: t,
                version: 3,
            },
        );
        let out = backend
            .update_record("demo", RecordId(1), u, Some(3))
            .await
            .expect("update");
        assert_eq!(out.id, RecordId(1));
    }

    #[tokio::test]
    async fn update_issue_attaches_agent_header() {
        // Re-home of fetch.rs::fetch_issues_attaches_agent_header (SG-05 audit
        // attribution proof). Value is process-specific
        // (`reposix-core-simbackend-<pid>`), so we assert "header present,
        // any value" via a closure matcher — the same pattern
        // `update_without_expected_version_is_wildcard` (line 550) uses to
        // assert absence.
        use wiremock::Match;
        struct HasAgentHeader;
        impl Match for HasAgentHeader {
            fn matches(&self, request: &wiremock::Request) -> bool {
                request.headers.contains_key("x-reposix-agent")
            }
        }

        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/1"))
            .and(HasAgentHeader)
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue_json(1)))
            .expect(1)
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let u = sample_untainted();
        let _ = backend
            .update_record("demo", RecordId(1), u, Some(1))
            .await
            .expect("update");
        // Dropping `server` verifies the .expect(1) — panics if the
        // HasAgentHeader matcher didn't fire exactly once.
    }

    #[tokio::test]
    async fn create_issue_omits_server_fields() {
        // Re-home of fetch.rs::post_issue_sends_egress_shape_only. Prove the
        // POST body contains the mutable-field subset and lacks the server-
        // owned fields `version`, `id`, `created_at`, `updated_at`.
        // `render_create_body` (sim.rs:173) mechanically guarantees this; the
        // test is a regression guard against future field drift.
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/projects/demo/issues"))
            .respond_with(ResponseTemplate::new(201).set_body_json(sample_issue_json(4)))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let u = sample_untainted();
        let got = backend.create_record("demo", u).await.expect("create");
        assert_eq!(got.id, RecordId(4));

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let body = String::from_utf8_lossy(&requests[0].body);
        assert!(
            body.contains("\"title\""),
            "egress body lacks title: {body}"
        );
        assert!(
            body.contains("\"labels\""),
            "egress body lacks labels: {body}"
        );
        assert!(
            !body.contains("\"version\""),
            "egress body leaked version: {body}"
        );
        assert!(!body.contains("\"id\""), "egress body leaked id: {body}");
        assert!(
            !body.contains("\"created_at\""),
            "egress body leaked created_at: {body}"
        );
        assert!(
            !body.contains("\"updated_at\""),
            "egress body leaked updated_at: {body}"
        );
    }

    #[tokio::test]
    async fn update_issue_respects_untainted_sanitization() {
        // **SG-03 proof.** Re-home of tests/write.rs::
        // sanitize_strips_server_fields_on_egress (flagged critical in
        // 14-RESEARCH.md#Q10). A hostile tainted issue carrying an inflated
        // `version=999_999` flows through `sanitize()` and into
        // `SimBackend::update_record`; the wire body must contain the
        // mutable-field subset only. This proves the Untainted<Record>
        // discipline holds at the trait-impl layer too, not just in the old
        // `EgressPayload` struct.
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue_json(1)))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let t = Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        let meta = ServerMetadata {
            id: RecordId(1),
            created_at: t,
            updated_at: t,
            version: 1,
        };
        let hostile = Record {
            id: RecordId(1),
            title: "hello".into(),
            status: RecordStatus::Open,
            assignee: None,
            labels: vec![],
            // The attacker-influenced input tries to forge a version.
            // sanitize() strips it and replaces with the server-authoritative
            // value from `meta` — but even the server's version must NOT
            // appear in the wire body (only in the If-Match header).
            created_at: t,
            updated_at: t,
            version: 999_999,
            body: String::new(),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        };
        let u = sanitize(Tainted::new(hostile), meta);
        backend
            .update_record("demo", RecordId(1), u, Some(1))
            .await
            .expect("update");

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let body = String::from_utf8_lossy(&requests[0].body);
        assert!(
            !body.contains("\"version\""),
            "egress body leaked version: {body}"
        );
        // id is allowed in the URL path; NOT in the body.
        assert!(!body.contains("\"id\""), "egress body leaked id: {body}");
        assert!(
            !body.contains("\"created_at\""),
            "egress body leaked created_at: {body}"
        );
        assert!(
            !body.contains("\"updated_at\""),
            "egress body leaked updated_at: {body}"
        );
        assert!(body.contains("\"title\""));
        assert!(body.contains("\"status\""));
    }

    #[tokio::test]
    async fn create_issue_400_preserves_body_in_error() {
        // Per 14-RESEARCH.md#Q2: the old `FetchError::BadRequest(String)`
        // variant is dropped; 4xx responses now surface as `Error::Other(msg)`
        // where `msg` contains the body bytes via `decode_issue`'s format
        // string. Pin this as a regression guard — the content-preservation
        // property is what kept information lossless across the refactor.
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/projects/demo/issues"))
            .respond_with(ResponseTemplate::new(400).set_body_string("invalid title"))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let u = sample_untainted();
        let err = backend.create_record("demo", u).await.expect_err("400");
        match err {
            Error::Other(msg) => {
                assert!(
                    msg.contains("invalid title"),
                    "expected body substring 'invalid title', got {msg}"
                );
                assert!(
                    msg.contains("400"),
                    "expected status substring '400', got {msg}"
                );
            }
            other => panic!("expected Error::Other, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn sim_backend_rejects_non_allowlisted_origin() {
        // Re-home of fetch.rs::fetch_issue_origin_rejected (SG-01 allowlist
        // gate proof). The allowlist check lives in `http.rs` and fires at
        // request time (NOT construction time); a non-allowlisted origin
        // surfaces as `Error::InvalidOrigin(_)` on the first method call.
        // Default allowlist (empty env var) is 127.0.0.1/localhost, so
        // `http://evil.example` is rejected.
        let backend = SimBackend::new("http://evil.example".into()).expect("backend");
        let err = backend.list_records("demo").await.expect_err("evil origin");
        assert!(
            matches!(err, Error::InvalidOrigin(_)),
            "expected InvalidOrigin, got {err:?}"
        );
    }

    #[tokio::test]
    async fn get_issue_500_surfaces_error_other() {
        // Re-home of fetch.rs::fetch_issue_500_is_status. The old FetchError
        // had a dedicated Status variant; the trait surfaces all non-success
        // non-404s as Error::Other with the status substring preserved.
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/projects/demo/issues/1"))
            .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let err = backend
            .get_record("demo", RecordId(1))
            .await
            .expect_err("500");
        match err {
            Error::Other(msg) => {
                assert!(
                    msg.contains("sim returned 500"),
                    "expected 'sim returned 500' substring, got {msg}"
                );
            }
            other => panic!("expected Error::Other(500), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_issue_returns_authoritative_issue() {
        // Positive companion to `create_issue_omits_server_fields`: after a
        // 201, the returned `Record` carries the server-assigned id/version/
        // created_at from the response body — this is what the FUSE `create`
        // callback relies on to populate its inode cache.
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/projects/demo/issues"))
            .respond_with(ResponseTemplate::new(201).set_body_json(sample_issue_json(42)))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let u = sample_untainted();
        let got = backend.create_record("demo", u).await.expect("create");
        assert_eq!(got.id, RecordId(42));
        assert_eq!(got.version, 1);
    }

    #[tokio::test]
    async fn delete_or_close_succeeds_on_200() {
        // No prior sim.rs test covered the delete happy path; add coverage
        // to partially offset the net test-count reduction from deleting
        // `fetch.rs::tests` + `tests/write.rs`.
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/projects/demo/issues/1"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        backend
            .delete_or_close("demo", RecordId(1), DeleteReason::Completed)
            .await
            .expect("delete");
    }

    #[tokio::test]
    async fn list_changed_since_sends_since_query_param() {
        // Phase 33: prove the SimBackend override emits `?since=<RFC3339>` on
        // the wire. The mock fails-loud (.expect(1)) if the param is missing.
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/projects/demo/issues"))
            .and(query_param("since", "2026-04-24T00:00:00Z"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([sample_issue_json(42)])),
            )
            .expect(1)
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let t = Utc.with_ymd_and_hms(2026, 4, 24, 0, 0, 0).unwrap();
        let ids = backend
            .list_changed_since("demo", t)
            .await
            .expect("list_changed");
        assert_eq!(ids, vec![RecordId(42)]);
    }

    #[tokio::test]
    async fn list_changed_since_returns_ids_only() {
        // The override returns only IDs (not full Issues) — symmetric with
        // the Phase 31 lazy-blob design. Confirm by feeding 3 issue JSON
        // bodies and asserting the returned Vec is exactly their IDs.
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/projects/demo/issues"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                sample_issue_json(1),
                sample_issue_json(2),
                sample_issue_json(3),
            ])))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let t = chrono::Utc::now();
        let ids = backend
            .list_changed_since("demo", t)
            .await
            .expect("list_changed");
        assert_eq!(ids, vec![RecordId(1), RecordId(2), RecordId(3)]);
    }

    #[tokio::test]
    async fn delete_or_close_404_maps_to_not_found() {
        // Companion to get_maps_404_to_not_found but for DELETE — the sim's
        // DELETE handler returns 404 for unknown ids and SimBackend renders
        // that as `Error::NotFound { project, id }`. Covered separately
        // because the code path in `delete_or_close` is distinct from the
        // shared `decode_issue` helper. POLISH2-09: was Error::Other.
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/projects/demo/issues/9999"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let backend = SimBackend::new(server.uri()).expect("backend");
        let err = backend
            .delete_or_close("demo", RecordId(9999), DeleteReason::Completed)
            .await
            .expect_err("404");
        match err {
            Error::NotFound { project, id } => {
                assert_eq!(project, "demo");
                assert_eq!(id, "9999");
            }
            other => panic!("expected Error::NotFound, got {other:?}"),
        }
    }
}
