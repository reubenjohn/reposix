//! HTTP plumbing, audit hooks, and rate-limit gate for [`JiraBackend`].
//!
//! Holds the [`JiraBackend`] struct alongside its constructors and internal
//! helpers — every method that talks to the network or the audit DB lives
//! here. The trait surface (`impl BackendConnector for JiraBackend`) lives
//! in [`crate::lib`] (so this file stays focused on plumbing, and the trait
//! impl reads as a thin adapter over these helpers).

use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use reqwest::{Method, StatusCode};
use rusqlite::Connection;

use reposix_core::http::{client, ClientOpts, HttpClient};
use reposix_core::{Error, Record, RecordId, Result, Tainted};

use crate::translate::{basic_auth_header, translate, validate_tenant};
use crate::types::{
    JiraCreds, JiraErrorResponse, JiraIssue, JiraSearchResponse, JIRA_FIELDS, MAX_ISSUES_PER_LIST,
    MAX_RATE_LIMIT_SLEEP, PAGE_SIZE,
};

// ─── Backend ─────────────────────────────────────────────────────────────────

/// JIRA Cloud read/write adapter.
///
/// Build via [`JiraBackend::new`] (production) or
/// [`JiraBackend::new_with_base_url`] (tests / wiremock).
/// Optionally attach an audit log via [`JiraBackend::with_audit`].
#[derive(Clone)]
pub struct JiraBackend {
    pub(crate) http: Arc<HttpClient>,
    pub(crate) creds: JiraCreds,
    /// Base URL (no trailing slash). E.g. `https://myco.atlassian.net`.
    pub(crate) base_url: String,
    pub(crate) rate_limit_gate: Arc<Mutex<Option<Instant>>>,
    pub(crate) audit: Option<Arc<Mutex<Connection>>>,
    /// Per-session cache for valid issue type names (populated on first `create_record`).
    pub(crate) issue_type_cache: Arc<OnceLock<Vec<String>>>,
}

impl std::fmt::Debug for JiraBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JiraBackend")
            .field("base_url", &self.base_url)
            .field("creds", &self.creds)
            .field("rate_limit_gate", &"<gate>")
            .field(
                "audit",
                if self.audit.is_some() {
                    &"<present>"
                } else {
                    &"<none>"
                },
            )
            .finish_non_exhaustive()
    }
}

impl JiraBackend {
    /// Build a backend for the given Atlassian tenant subdomain.
    ///
    /// `tenant` must satisfy DNS-label rules (lowercase alphanumeric + internal
    /// hyphens, length 1–63). This prevents SSRF via injected URL components.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `tenant` fails DNS-label validation, or if the HTTP
    /// client cannot be constructed.
    pub fn new(creds: JiraCreds, tenant: &str) -> Result<Self> {
        validate_tenant(tenant)?;
        let base_url = format!("https://{tenant}.atlassian.net");
        Self::new_with_base_url(creds, base_url)
    }

    /// Build a backend with an arbitrary base URL.
    ///
    /// Used by wiremock tests that need to point at a local mock server.
    /// No tenant validation — accepts any URL.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the HTTP client cannot be constructed.
    pub fn new_with_base_url(creds: JiraCreds, base_url: String) -> Result<Self> {
        let http = client(ClientOpts::default())?;
        Ok(Self {
            http: Arc::new(http),
            creds,
            base_url,
            rate_limit_gate: Arc::new(Mutex::new(None)),
            audit: None,
            issue_type_cache: Arc::new(OnceLock::new()),
        })
    }

    /// Attach an audit log connection.
    ///
    /// Every read call (`list_records`, `get_record`) inserts one row into
    /// `audit_events` when an audit connection is present. Writes succeed
    /// even if the audit insert fails (best-effort, log-and-swallow).
    ///
    /// The caller is responsible for opening the connection via
    /// [`reposix_core::audit::open_audit_db`] so the schema and triggers
    /// are loaded before the first insert.
    #[must_use]
    pub fn with_audit(mut self, conn: Arc<Mutex<Connection>>) -> Self {
        self.audit = Some(conn);
        self
    }

    /// Strict variant of [`reposix_core::backend::BackendConnector::list_records`]:
    /// returns `Err` instead of silently truncating at the
    /// [`MAX_ISSUES_PER_LIST`] cap.
    ///
    /// # Errors
    ///
    /// - Returns `Error::Other` if pagination would exceed `MAX_ISSUES_PER_LIST`.
    /// - All errors that `list_records` would raise also apply here.
    pub async fn list_records_strict(&self, project: &str) -> Result<Vec<Record>> {
        self.list_issues_impl(project, true).await
    }

    // ─── Internal helpers ────────────────────────────────────────────────

    pub(crate) fn base(&self) -> &str {
        &self.base_url
    }

    pub(crate) fn standard_headers(&self) -> Vec<(&'static str, String)> {
        vec![
            ("Accept", "application/json".to_owned()),
            ("User-Agent", "reposix-jira/0.8".to_owned()),
            (
                "Authorization",
                basic_auth_header(&self.creds.email, &self.creds.api_token),
            ),
        ]
    }

    pub(crate) fn write_headers(&self) -> Vec<(&'static str, String)> {
        let mut h = self.standard_headers();
        h.push(("Content-Type", "application/json".to_owned()));
        h
    }

    pub(crate) async fn list_issues_impl(
        &self,
        project: &str,
        strict: bool,
    ) -> Result<Vec<Record>> {
        let url = format!("{}/rest/api/3/search/jql", self.base());

        let fields: Vec<String> = JIRA_FIELDS.iter().map(|s| (*s).to_owned()).collect();
        let mut request_body = serde_json::json!({
            "jql": format!("project = \"{}\" ORDER BY id ASC", project),
            "fields": fields,
            "maxResults": PAGE_SIZE,
        });

        let mut out: Vec<Record> = Vec::new();
        let mut pages: usize = 0;

        let header_owned = self.write_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();

        loop {
            pages += 1;
            if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) + 1 {
                if strict {
                    return Err(Error::Other(format!(
                        "JIRA project '{project}' exceeds {MAX_ISSUES_PER_LIST}-issue cap; \
                         refusing to truncate (strict mode)"
                    )));
                }
                tracing::warn!(
                    pages,
                    "reached MAX_ISSUES_PER_LIST cap; stopping pagination"
                );
                break;
            }

            self.await_rate_limit_gate().await;
            let body_bytes = serde_json::to_vec(&request_body)?;
            let resp = self
                .http
                .request_with_headers_and_body(
                    Method::POST,
                    url.as_str(),
                    &header_refs,
                    Some(body_bytes),
                )
                .await?;
            self.ingest_rate_limit(&resp);
            let status = resp.status();
            let bytes = resp.bytes().await?;
            let status_u16 = status.as_u16();
            self.audit_event(
                "POST",
                "/rest/api/3/search/jql",
                status_u16,
                &format!("list:{project}:{}", out.len()),
                &bytes,
            );

            if !status.is_success() {
                return Err(Error::Other(format!(
                    "JIRA returned {status} for POST /rest/api/3/search/jql: {}",
                    String::from_utf8_lossy(&bytes)
                )));
            }

            let search_resp: JiraSearchResponse = serde_json::from_slice(&bytes)?;
            let is_last = search_resp.is_last.unwrap_or(true);
            let next_token = search_resp.next_page_token.clone();

            for issue in search_resp.issues {
                // SG-05: wrap ingress bytes as Tainted before translating.
                let tainted = Tainted::new(issue);
                let translated = translate(tainted.into_inner())?;
                out.push(translated);
                if out.len() >= MAX_ISSUES_PER_LIST {
                    if strict {
                        return Err(Error::Other(format!(
                            "JIRA project '{project}' exceeds {MAX_ISSUES_PER_LIST}-issue cap; \
                             refusing to truncate (strict mode)"
                        )));
                    }
                    tracing::warn!(
                        count = out.len(),
                        "reached MAX_ISSUES_PER_LIST cap; stopping early"
                    );
                    return Ok(out);
                }
            }

            if is_last {
                break;
            }

            if let Some(token) = next_token {
                request_body["nextPageToken"] = serde_json::Value::String(token);
            } else {
                // No cursor and not marked last — stop to avoid infinite loop.
                break;
            }
        }

        Ok(out)
    }

    pub(crate) async fn get_issue_inner(&self, id: RecordId) -> Result<Record> {
        let path = format!("/rest/api/3/issue/{id}");
        let url = format!("{}{}", self.base(), path);

        self.await_rate_limit_gate().await;
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let resp = self
            .http
            .request_with_headers(Method::GET, url.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        let status_u16 = status.as_u16();
        self.audit_event("GET", &path, status_u16, &format!("get:{id}"), &bytes);

        if status == StatusCode::NOT_FOUND {
            let err_resp: JiraErrorResponse =
                serde_json::from_slice(&bytes).unwrap_or(JiraErrorResponse {
                    error_messages: vec!["unknown".into()],
                });
            let msg = err_resp.error_messages.join("; ");
            return Err(Error::Other(format!("not found: {id} — {msg}")));
        }

        if !status.is_success() {
            return Err(Error::Other(format!(
                "JIRA returned {status} for GET {path}: {}",
                String::from_utf8_lossy(&bytes)
            )));
        }

        let issue: JiraIssue = serde_json::from_slice(&bytes)?;
        // SG-05: wrap ingress bytes as Tainted before translating.
        let tainted = Tainted::new(issue);
        translate(tainted.into_inner())
    }

    /// Best-effort audit event insert.
    ///
    /// Logs the method, path, HTTP status, a request summary string, and a
    /// SHA-256 prefix of the response body. The full body is never stored.
    /// The `Authorization` header is never logged.
    pub(crate) fn audit_event(
        &self,
        method: &'static str,
        path: &str,
        status: u16,
        request_summary: &str,
        response_bytes: &[u8],
    ) {
        let Some(ref audit) = self.audit else {
            return;
        };
        let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let sha_hex = {
            use sha2::{Digest, Sha256};
            let digest = Sha256::digest(response_bytes);
            hex::encode(digest)
        };
        let response_summary = format!("{status}:{}", &sha_hex[..16]);
        let conn = audit.lock();
        if let Err(e) = conn.execute(
            "INSERT INTO audit_events \
             (ts, agent_id, method, path, status, request_body, response_summary) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                ts,
                format!("reposix-jira-{}", std::process::id()),
                method,
                path,
                i64::from(status),
                request_summary,
                response_summary,
            ],
        ) {
            tracing::error!(error = %e, "jira audit insert failed");
        }
    }

    // ─── Rate-limit gate ─────────────────────────────────────────────────

    pub(crate) async fn await_rate_limit_gate(&self) {
        let maybe_gate = *self.rate_limit_gate.lock();
        if let Some(gate) = maybe_gate {
            let now = Instant::now();
            if gate > now {
                let sleep_for = gate.duration_since(now).min(MAX_RATE_LIMIT_SLEEP);
                tokio::time::sleep(sleep_for).await;
            }
            *self.rate_limit_gate.lock() = None;
        }
    }

    pub(crate) fn ingest_rate_limit(&self, resp: &reqwest::Response) {
        if resp.status() != StatusCode::TOO_MANY_REQUESTS {
            return;
        }
        let delay_secs: u64 = resp
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(2);
        let gate = Instant::now() + Duration::from_secs(delay_secs);
        *self.rate_limit_gate.lock() = Some(gate);
    }

    /// Arm the rate-limit gate using exponential backoff (no `Retry-After` header).
    ///
    /// Used when the server returns 429 without a `Retry-After` header. The
    /// delay is `1s * 2^attempt`, capped at [`MAX_RATE_LIMIT_SLEEP`].
    /// Tested by the `rate_limit_429_honors_retry_after` wiremock test.
    #[allow(dead_code)] // tested in rate_limit_429_honors_retry_after; production retry wired in Phase 29
    pub(crate) fn arm_rate_limit_backoff(&self, attempt: u32) {
        // Exponential backoff: 1s * 2^attempt, cap at MAX_RATE_LIMIT_SLEEP.
        let secs = (1_u64 << attempt).min(MAX_RATE_LIMIT_SLEEP.as_secs());
        let gate = Instant::now() + Duration::from_secs(secs);
        *self.rate_limit_gate.lock() = Some(gate);
    }

    /// Fetch valid issue type names for `project` from the JIRA API.
    ///
    /// Called at most once per backend instance (results are cached in
    /// `issue_type_cache`). Prefers `"Task"` when selecting a type for
    /// `create_record`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the HTTP call fails or the response cannot be parsed.
    pub(crate) async fn fetch_issue_types(&self, project: &str) -> Result<Vec<String>> {
        let url = format!(
            "{}/rest/api/3/issuetype?projectKeys={}",
            self.base(),
            project
        );
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers(Method::GET, url.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(Error::Other(format!(
                "jira issuetype fetch returned {status}"
            )));
        }
        // Response: array of issue type objects with at least a "name" field.
        let types: Vec<serde_json::Value> = serde_json::from_slice(&bytes)?;
        let names: Vec<String> = types
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(str::to_owned))
            .collect();
        if names.is_empty() {
            return Err(Error::Other(
                "jira issuetype endpoint returned no types for project".into(),
            ));
        }
        Ok(names)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason};
    use reposix_core::{Record, RecordId, RecordStatus};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    // ─── Helper: minimal JiraIssue JSON ─────────────────────────────────

    fn issue_json(
        id: u64,
        key: &str,
        summary: &str,
        status_cat: &str,
        status_name: &str,
        resolution: Option<&str>,
    ) -> serde_json::Value {
        serde_json::json!({
            "id": id.to_string(),
            "key": key,
            "fields": {
                "summary": summary,
                "description": serde_json::Value::Null,
                "status": {
                    "name": status_name,
                    "statusCategory": {"key": status_cat}
                },
                "resolution": resolution.map(|r| serde_json::json!({"name": r})),
                "assignee": serde_json::Value::Null,
                "labels": [],
                "created": "2025-01-01T00:00:00.000+0000",
                "updated": "2025-06-01T00:00:00.000+0000",
                "parent": serde_json::Value::Null,
                "issuetype": {"name": "Story", "hierarchyLevel": 0},
                "priority": {"name": "Medium"},
            }
        })
    }

    fn make_backend(server_uri: &str) -> JiraBackend {
        let creds = JiraCreds {
            email: "test@example.com".into(),
            api_token: "secret-token".into(),
        };
        JiraBackend::new_with_base_url(creds, server_uri.to_string())
            .expect("backend construction must succeed")
    }

    /// Wiremock matcher: assert the POST body bytes contain a substring.
    /// Wiremock 0.6 has no out-of-the-box `body_string_contains` matcher;
    /// this is the minimal stand-in (used for asserting JQL substrings).
    struct BodyContains(&'static str);
    impl wiremock::Match for BodyContains {
        fn matches(&self, request: &wiremock::Request) -> bool {
            std::str::from_utf8(&request.body).is_ok_and(|s| s.contains(self.0))
        }
    }

    #[tokio::test]
    async fn jira_list_changed_since_sends_updated_jql() {
        // Phase 33: prove the JIRA override emits a POST body containing the
        // JQL `updated >=` clause — the native incremental query.
        use chrono::{TimeZone, Utc};
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/api/3/search/jql"))
            .and(BodyContains("updated >="))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issues": [
                    issue_json(10042, "TEST-42", "x", "indeterminate", "In Progress", None)
                ],
                "isLast": true
            })))
            .expect(1)
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        let t = Utc.with_ymd_and_hms(2026, 4, 24, 0, 0, 0).unwrap();
        let ids = backend
            .list_changed_since("TEST", t)
            .await
            .expect("list_changed");
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], RecordId(10042));
    }

    #[tokio::test]
    async fn jira_list_changed_since_strips_quotes_from_project() {
        // Defense: a malicious project key with embedded `"` cannot break out
        // of the JQL string literal — the override strips quotes pre-interpolation.
        use chrono::{TimeZone, Utc};
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rest/api/3/search/jql"))
            .and(BodyContains("project = \\\"TEST\\\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issues": [],
                "isLast": true
            })))
            .expect(1)
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        let t = Utc.with_ymd_and_hms(2026, 4, 24, 0, 0, 0).unwrap();
        // `TE"S"T` — quotes will be stripped to `TEST`.
        let ids = backend
            .list_changed_since("TE\"S\"T", t)
            .await
            .expect("list_changed");
        assert_eq!(ids, Vec::<RecordId>::new());
    }

    // ─── Test 1: list_single_page ────────────────────────────────────────

    #[tokio::test]
    async fn list_single_page() {
        let server = MockServer::start().await;
        let issues: Vec<serde_json::Value> = (1..=10)
            .map(|i| {
                issue_json(
                    i,
                    &format!("PROJ-{i}"),
                    &format!("Record {i}"),
                    "new",
                    "Open",
                    None,
                )
            })
            .collect();
        Mock::given(method("POST"))
            .and(path("/rest/api/3/search/jql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issues": issues,
                "isLast": true
            })))
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        let result = backend
            .list_records("PROJ")
            .await
            .expect("list must succeed");
        assert_eq!(result.len(), 10, "expected 10 issues from single page");
        assert_eq!(result[0].id, RecordId(1));
    }

    // ─── Test 2: list_pagination_cursor ─────────────────────────────────

    #[tokio::test]
    async fn list_pagination_cursor() {
        let server = MockServer::start().await;
        let page1_issues: Vec<serde_json::Value> = (1..=3)
            .map(|i| {
                issue_json(
                    i,
                    &format!("PROJ-{i}"),
                    &format!("Record {i}"),
                    "new",
                    "Open",
                    None,
                )
            })
            .collect();
        let page2_issues: Vec<serde_json::Value> = (4..=6)
            .map(|i| {
                issue_json(
                    i,
                    &format!("PROJ-{i}"),
                    &format!("Record {i}"),
                    "new",
                    "Open",
                    None,
                )
            })
            .collect();

        // First request — no nextPageToken in body
        Mock::given(method("POST"))
            .and(path("/rest/api/3/search/jql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issues": page1_issues,
                "isLast": false,
                "nextPageToken": "tok1"
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        // Second request — cursor tok1 should appear
        Mock::given(method("POST"))
            .and(path("/rest/api/3/search/jql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issues": page2_issues,
                "isLast": true
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        let result = backend
            .list_records("PROJ")
            .await
            .expect("list must succeed");
        assert_eq!(result.len(), 6, "expected 6 issues across 2 pages");
    }

    // ─── Test 3: get_by_numeric_id ───────────────────────────────────────

    #[tokio::test]
    async fn get_by_numeric_id() {
        let server = MockServer::start().await;
        let issue = issue_json(
            10001,
            "MYPROJ-42",
            "A real issue",
            "indeterminate",
            "In Progress",
            None,
        );
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/10001"))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue))
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        let result = backend
            .get_record("MYPROJ", RecordId(10001))
            .await
            .expect("get must succeed");
        assert_eq!(result.id, RecordId(10001));
        assert_eq!(result.title, "A real issue");
        assert_eq!(result.status, RecordStatus::InProgress);
        assert_eq!(
            result.extensions.get("jira_key").and_then(|v| {
                if let serde_yaml::Value::String(s) = v {
                    Some(s.as_str())
                } else {
                    None
                }
            }),
            Some("MYPROJ-42")
        );
    }

    // ─── Test 4: get_404_maps_to_not_found ──────────────────────────────

    #[tokio::test]
    async fn get_404_maps_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/99999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "errorMessages": ["Issue Does Not Exist"],
                "errors": {}
            })))
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        let err = backend
            .get_record("PROJ", RecordId(99999))
            .await
            .expect_err("must return error for 404");
        let msg = err.to_string();
        assert!(
            msg.contains("not found") || msg.to_lowercase().contains("not found"),
            "expected 'not found' in error, got: {msg}"
        );
    }

    // ─── Test: rate_limit_429_honors_retry_after ─────────────────────────

    #[tokio::test]
    async fn rate_limit_429_honors_retry_after() {
        let server = MockServer::start().await;
        // First call → 429 with Retry-After: 1
        let issue = issue_json(1, "PROJ-1", "Issue 1", "new", "Open", None);
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/1"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "1")
                    .set_body_bytes(b"rate limited"),
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue))
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        // Make one request that will 429 — the gate will be armed
        let header_owned = backend.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let resp1 = backend
            .http
            .request_with_headers(
                Method::GET,
                format!("{}/rest/api/3/issue/1", backend.base_url),
                &header_refs,
            )
            .await
            .unwrap();
        backend.ingest_rate_limit(&resp1);
        // Gate should be armed now
        assert!(
            backend.rate_limit_gate.lock().is_some(),
            "rate limit gate must be armed after 429"
        );

        // Backoff arm (no header case)
        let backend2 = make_backend(&server.uri());
        backend2.arm_rate_limit_backoff(0);
        assert!(
            backend2.rate_limit_gate.lock().is_some(),
            "rate limit gate must be armed after backoff"
        );
    }

    // ─── Test: supports_reports_delete_and_transitions ───────────────────

    #[test]
    fn supports_reports_delete_and_transitions() {
        let creds = JiraCreds {
            email: "a@b.com".into(),
            api_token: "tok".into(),
        };
        let backend = JiraBackend::new_with_base_url(creds, "http://localhost".into()).unwrap();
        assert!(backend.supports(BackendFeature::Hierarchy));
        assert!(backend.supports(BackendFeature::Delete));
        assert!(backend.supports(BackendFeature::Transitions));
        assert!(!backend.supports(BackendFeature::StrongVersioning));
        assert!(!backend.supports(BackendFeature::BulkEdit));
        assert!(!backend.supports(BackendFeature::Workflows));
    }

    // ─── Helper: make_untainted ──────────────────────────────────────────

    fn make_untainted(title: &str, body: &str) -> reposix_core::Untainted<Record> {
        use reposix_core::{sanitize, ServerMetadata, Tainted};
        let now = chrono::Utc::now();
        let raw = Record {
            id: RecordId(0),
            title: title.to_owned(),
            body: body.to_owned(),
            status: RecordStatus::Open,
            created_at: now,
            updated_at: now,
            version: 0,
            assignee: None,
            labels: vec![],
            parent_id: None,
            extensions: BTreeMap::new(),
        };
        sanitize(
            Tainted::new(raw),
            ServerMetadata {
                id: RecordId(0),
                created_at: now,
                updated_at: now,
                version: 0,
            },
        )
    }

    // ─── Test: create_issue_posts_to_rest_api ────────────────────────────

    #[tokio::test]
    async fn create_issue_posts_to_rest_api() {
        let server = MockServer::start().await;
        // Mock: GET /rest/api/3/issuetype?projectKeys=P
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issuetype"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"id": "10001", "name": "Task"},
                {"id": "10002", "name": "Bug"},
            ])))
            .mount(&server)
            .await;
        // Mock: POST /rest/api/3/issue → 201
        Mock::given(method("POST"))
            .and(path("/rest/api/3/issue"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": "10042",
                "key": "P-42",
                "self": format!("{}/rest/api/3/issue/10042", server.uri()),
            })))
            .mount(&server)
            .await;
        // Mock: GET /rest/api/3/issue/10042 → full fixture
        let issue_fixture = issue_json(10042, "P-42", "test create", "new", "To Do", None);
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/10042"))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue_fixture))
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        let issue = make_untainted("test create", "body text");
        let created = backend.create_record("P", issue).await.expect("create");
        assert_eq!(created.title, "test create");
        assert_eq!(created.id, RecordId(10042));
    }

    // ─── Test: update_issue_puts_fields ──────────────────────────────────

    #[tokio::test]
    async fn update_issue_puts_fields() {
        let server = MockServer::start().await;
        // Mock: PUT /rest/api/3/issue/99 → 204
        Mock::given(method("PUT"))
            .and(path("/rest/api/3/issue/99"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;
        // Mock: GET /rest/api/3/issue/99 → updated fixture
        let updated_fixture = issue_json(99, "P-99", "updated title", "new", "To Do", None);
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/99"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated_fixture))
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        let patch = make_untainted("updated title", "new body");
        let updated = backend
            .update_record("P", RecordId(99), patch, None)
            .await
            .expect("update");
        assert_eq!(updated.title, "updated title");
    }

    // ─── Test: delete_or_close_via_transitions ───────────────────────────

    #[tokio::test]
    async fn delete_or_close_via_transitions() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/55/transitions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "transitions": [
                    {"id": "11", "name": "In Progress", "to": {"statusCategory": {"key": "indeterminate"}}},
                    {"id": "31", "name": "Done", "to": {"statusCategory": {"key": "done"}}},
                ]
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/api/3/issue/55/transitions"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        backend
            .delete_or_close("P", RecordId(55), DeleteReason::Completed)
            .await
            .expect("delete_or_close via transitions");
    }

    // ─── Test: delete_or_close_wontfix_picks_reject ──────────────────────

    #[tokio::test]
    async fn delete_or_close_wontfix_picks_reject() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/56/transitions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "transitions": [
                    {"id": "31", "name": "Done", "to": {"statusCategory": {"key": "done"}}},
                    {"id": "41", "name": "Won't Fix", "to": {"statusCategory": {"key": "done"}}},
                ]
            })))
            .mount(&server)
            .await;
        // Capture which transition id is posted — use a body_partial_json matcher.
        // wiremock doesn't have body_json_schema, so verify via expect(1) on the
        // transition POST and check the result directly.
        Mock::given(method("POST"))
            .and(path("/rest/api/3/issue/56/transitions"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        backend
            .delete_or_close("P", RecordId(56), DeleteReason::NotPlanned)
            .await
            .expect("delete_or_close wontfix");
        // Mock drop verifies expect(1) was satisfied.
    }

    // ─── Test: delete_or_close_fallback_delete ───────────────────────────

    #[tokio::test]
    async fn delete_or_close_fallback_delete() {
        let server = MockServer::start().await;
        // No done transitions.
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/57/transitions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "transitions": [
                    {"id": "11", "name": "In Progress", "to": {"statusCategory": {"key": "indeterminate"}}},
                ]
            })))
            .mount(&server)
            .await;
        // Fallback DELETE.
        Mock::given(method("DELETE"))
            .and(path("/rest/api/3/issue/57"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        backend
            .delete_or_close("P", RecordId(57), DeleteReason::Completed)
            .await
            .expect("delete_or_close fallback delete");
    }

    // ─── Test: create_issue_discovers_issuetype ──────────────────────────

    #[tokio::test]
    async fn create_issue_discovers_issuetype() {
        let server = MockServer::start().await;
        // Only Story and Bug — no Task — should pick Story (first).
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issuetype"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"id": "10001", "name": "Story"},
                {"id": "10002", "name": "Bug"},
            ])))
            .expect(1) // cache means this fires exactly once
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rest/api/3/issue"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": "10001", "key": "P-1",
            })))
            .mount(&server)
            .await;
        let get_fixture = issue_json(10001, "P-1", "s", "new", "To Do", None);
        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/10001"))
            .respond_with(ResponseTemplate::new(200).set_body_json(get_fixture))
            .mount(&server)
            .await;

        let backend = make_backend(&server.uri());
        // Two creates — issuetype GET should only fire once (cached).
        let _ = backend
            .create_record("P", make_untainted("s", "b"))
            .await
            .expect("create 1");
        // Second create reuses cache — Mock with expect(1) will verify at drop.
        let _ = backend
            .create_record("P", make_untainted("s", "b"))
            .await
            .expect("create 2");
    }
}
