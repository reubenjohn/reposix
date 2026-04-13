//! Backend fetch helpers: two functions that wrap the HTTP I/O the
//! Filesystem impl performs on `readdir` and `read`.
//!
//! Security & reliability contract:
//!
//! - All I/O goes through the sealed [`reposix_core::http::HttpClient`] via
//!   [`HttpClient::request_with_headers`]; we attach
//!   `X-Reposix-Agent: <agent>` so the Phase-2 audit middleware attributes.
//! - Every call is wrapped in a 5-second [`tokio::time::timeout`] — belt and
//!   suspenders on top of the reqwest-layer 5s `ClientOpts::total_timeout`.
//!   On elapsed timeout we return [`FetchError::Timeout`], which the FUSE
//!   callback maps to `libc::EIO` so the kernel never hangs (SG-07).
//! - A non-allowlisted origin surfaces as [`FetchError::Origin`]; the caller
//!   should also map this to `EIO` (leaking the reason in the `stat` errno
//!   is fine, but we do not expose the origin string to the kernel).

use std::time::Duration;

use reposix_core::http::HttpClient;
use reposix_core::{Issue, IssueId, Untainted};
use reqwest::Method;
use serde::Serialize;
use thiserror::Error;

/// The 5-second wall-clock ceiling enforced by `fetch_*` independent of
/// whatever the [`HttpClient`] timeout is configured to.
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(5);

/// Errors reachable from the fetch helpers. Intentionally opaque to callers
/// — the FUSE callback path ultimately collapses all of these to `EIO` or
/// `ENOENT`, but keeping the variants distinct aids tests and logs.
#[derive(Debug, Error)]
pub enum FetchError {
    /// Wall-clock 5s elapsed before the backend responded. The FUSE callback
    /// MUST map this to `libc::EIO`.
    #[error("backend did not respond within {:?}", FETCH_TIMEOUT)]
    Timeout,
    /// Backend returned 404 — the issue does not exist at `{origin}/projects/{project}/issues/{id}`.
    #[error("issue not found")]
    NotFound,
    /// Backend returned a non-404 4xx or any 5xx.
    #[error("backend responded with status {0}")]
    Status(reqwest::StatusCode),
    /// Transport-level failure (TCP refused, TLS handshake failure, etc).
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    /// The target origin is not allowlisted. Bubbles up from the sealed
    /// [`HttpClient`] newtype.
    #[error("origin not allowlisted: {0}")]
    Origin(String),
    /// The backend returned 200 but the JSON body did not match our
    /// [`Issue`] schema.
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
    /// Other core errors (e.g. allowlist env var un-parseable).
    #[error("core: {0}")]
    Core(String),
    /// Backend returned 409 on a PATCH — the `If-Match` version did not
    /// match the current server version. The FUSE callback maps this to
    /// `libc::EIO`; the user must `git pull --rebase` to reconcile.
    #[error("version mismatch: current={current}")]
    Conflict {
        /// Current server version, parsed from the 409 response body.
        current: u64,
    },
    /// Backend returned a 4xx on POST (e.g. validation failure). The body
    /// message, if any, is captured verbatim.
    #[error("bad request: {0}")]
    BadRequest(String),
}

/// JSON payload shape for outbound PATCH / POST — the subset of [`Issue`]
/// fields the sim's `PatchIssueBody` / `CreateIssueBody` allow. Using a
/// dedicated struct means `id`/`version`/`created_at`/`updated_at` cannot
/// accidentally appear in the wire format even if a future `Issue` field is
/// added (SG-03 defence in depth on top of [`reposix_core::sanitize`]).
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

/// Body shape for 409 Conflict responses from the sim's PATCH handler.
/// Matches `{"error":"version_mismatch","current":N,"sent":"..."}` — we only
/// need `current` to surface back into [`FetchError::Conflict`].
#[derive(serde::Deserialize)]
struct ConflictBody {
    current: u64,
}

fn from_core(e: reposix_core::Error) -> FetchError {
    match e {
        reposix_core::Error::InvalidOrigin(o) => FetchError::Origin(o),
        reposix_core::Error::Http(t) => FetchError::Transport(t),
        other => FetchError::Core(other.to_string()),
    }
}

/// Fetch the full issue list for `project`.
///
/// Wire: `GET {origin}/projects/{project}/issues` → `[Issue, ...]`.
///
/// # Errors
/// See [`FetchError`]. In particular, the 5-second cap is enforced here.
pub async fn fetch_issues(
    http: &HttpClient,
    origin: &str,
    project: &str,
    agent: &str,
) -> Result<Vec<Issue>, FetchError> {
    let url = format!(
        "{}/projects/{}/issues",
        origin.trim_end_matches('/'),
        project
    );
    let resp = tokio::time::timeout(
        FETCH_TIMEOUT,
        http.request_with_headers(Method::GET, &url, &[("X-Reposix-Agent", agent)]),
    )
    .await
    .map_err(|_| FetchError::Timeout)?
    .map_err(from_core)?;

    let status = resp.status();
    if !status.is_success() {
        return Err(FetchError::Status(status));
    }
    let bytes = tokio::time::timeout(FETCH_TIMEOUT, resp.bytes())
        .await
        .map_err(|_| FetchError::Timeout)?
        .map_err(FetchError::Transport)?;
    let issues: Vec<Issue> = serde_json::from_slice(&bytes)?;
    Ok(issues)
}

/// Fetch a single issue by ID.
///
/// Wire: `GET {origin}/projects/{project}/issues/{id}` → `Issue`.
///
/// # Errors
/// See [`FetchError`]. 404 surfaces as [`FetchError::NotFound`]; other 4xx/5xx
/// as [`FetchError::Status`].
pub async fn fetch_issue(
    http: &HttpClient,
    origin: &str,
    project: &str,
    id: IssueId,
    agent: &str,
) -> Result<Issue, FetchError> {
    let url = format!(
        "{}/projects/{}/issues/{}",
        origin.trim_end_matches('/'),
        project,
        id.0
    );
    let resp = tokio::time::timeout(
        FETCH_TIMEOUT,
        http.request_with_headers(Method::GET, &url, &[("X-Reposix-Agent", agent)]),
    )
    .await
    .map_err(|_| FetchError::Timeout)?
    .map_err(from_core)?;

    let status = resp.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(FetchError::NotFound);
    }
    if !status.is_success() {
        return Err(FetchError::Status(status));
    }
    let bytes = tokio::time::timeout(FETCH_TIMEOUT, resp.bytes())
        .await
        .map_err(|_| FetchError::Timeout)?
        .map_err(FetchError::Transport)?;
    let issue: Issue = serde_json::from_slice(&bytes)?;
    Ok(issue)
}

/// PATCH an issue with optimistic concurrency. Body is the sanitized issue
/// re-serialized as an [`EgressPayload`] (server-controlled fields physically
/// cannot appear in the wire shape).
///
/// Wire: `PATCH {origin}/projects/{project}/issues/{id}` with
/// `If-Match: {version}` → 200 + Issue, 409 + `{"current": N}`, or 4xx/5xx.
///
/// # Errors
/// - [`FetchError::Conflict`] on HTTP 409 (parses `current` from body).
/// - [`FetchError::Timeout`] on 5s elapsed.
/// - [`FetchError::Status`] on other non-success responses.
/// - See [`FetchError`] for the full variant set.
pub async fn patch_issue(
    http: &HttpClient,
    origin: &str,
    project: &str,
    id: IssueId,
    version: u64,
    sanitized: Untainted<Issue>,
    agent: &str,
) -> Result<Issue, FetchError> {
    let url = format!(
        "{}/projects/{}/issues/{}",
        origin.trim_end_matches('/'),
        project,
        id.0
    );
    let payload = EgressPayload::from_untainted(&sanitized);
    let body = serde_json::to_vec(&payload)?;
    let version_str = version.to_string();
    let headers = [
        ("If-Match", version_str.as_str()),
        ("Content-Type", "application/json"),
        ("X-Reposix-Agent", agent),
    ];
    let resp = tokio::time::timeout(
        FETCH_TIMEOUT,
        http.request_with_headers_and_body(Method::PATCH, &url, &headers, Some(body)),
    )
    .await
    .map_err(|_| FetchError::Timeout)?
    .map_err(from_core)?;

    let status = resp.status();
    if status == reqwest::StatusCode::CONFLICT {
        let bytes = tokio::time::timeout(FETCH_TIMEOUT, resp.bytes())
            .await
            .map_err(|_| FetchError::Timeout)?
            .map_err(FetchError::Transport)?;
        let parsed: ConflictBody =
            serde_json::from_slice(&bytes).unwrap_or(ConflictBody { current: 0 });
        return Err(FetchError::Conflict {
            current: parsed.current,
        });
    }
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(FetchError::NotFound);
    }
    if !status.is_success() {
        return Err(FetchError::Status(status));
    }
    let bytes = tokio::time::timeout(FETCH_TIMEOUT, resp.bytes())
        .await
        .map_err(|_| FetchError::Timeout)?
        .map_err(FetchError::Transport)?;
    let issue: Issue = serde_json::from_slice(&bytes)?;
    Ok(issue)
}

/// POST a new issue. Body is the sanitized issue re-serialized as an
/// [`EgressPayload`]. Returns the authoritative Issue from the 201 response.
///
/// Wire: `POST {origin}/projects/{project}/issues` → 201 + Issue, or 4xx/5xx.
///
/// # Errors
/// - [`FetchError::BadRequest`] on 4xx (with the response body as message).
/// - [`FetchError::Timeout`] on 5s elapsed.
/// - See [`FetchError`] for the full variant set.
pub async fn post_issue(
    http: &HttpClient,
    origin: &str,
    project: &str,
    sanitized: Untainted<Issue>,
    agent: &str,
) -> Result<Issue, FetchError> {
    let url = format!(
        "{}/projects/{}/issues",
        origin.trim_end_matches('/'),
        project
    );
    let payload = EgressPayload::from_untainted(&sanitized);
    let body = serde_json::to_vec(&payload)?;
    let headers = [
        ("Content-Type", "application/json"),
        ("X-Reposix-Agent", agent),
    ];
    let resp = tokio::time::timeout(
        FETCH_TIMEOUT,
        http.request_with_headers_and_body(Method::POST, &url, &headers, Some(body)),
    )
    .await
    .map_err(|_| FetchError::Timeout)?
    .map_err(from_core)?;

    let status = resp.status();
    if status.is_client_error() {
        let bytes = tokio::time::timeout(FETCH_TIMEOUT, resp.bytes())
            .await
            .map_err(|_| FetchError::Timeout)?
            .map_err(FetchError::Transport)?;
        let msg = String::from_utf8_lossy(&bytes).into_owned();
        return Err(FetchError::BadRequest(msg));
    }
    if !status.is_success() {
        return Err(FetchError::Status(status));
    }
    let bytes = tokio::time::timeout(FETCH_TIMEOUT, resp.bytes())
        .await
        .map_err(|_| FetchError::Timeout)?
        .map_err(FetchError::Transport)?;
    let issue: Issue = serde_json::from_slice(&bytes)?;
    Ok(issue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use reposix_core::http::{client, ClientOpts};
    use reposix_core::IssueStatus;
    use std::time::Instant;
    use wiremock::matchers::{any, header, method, path};
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
    async fn fetch_issues_parses_list() {
        let server = MockServer::start().await;
        let issues = vec![sample_issue(1), sample_issue(2), sample_issue(3)];
        Mock::given(method("GET"))
            .and(path("/projects/demo/issues"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
            .mount(&server)
            .await;
        let http = client(ClientOpts::default()).unwrap();
        let got = fetch_issues(&http, &server.uri(), "demo", "reposix-fuse-1")
            .await
            .unwrap();
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].id, IssueId(1));
    }

    #[tokio::test]
    async fn fetch_issue_parses_one() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/projects/demo/issues/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(1)))
            .mount(&server)
            .await;
        let http = client(ClientOpts::default()).unwrap();
        let got = fetch_issue(&http, &server.uri(), "demo", IssueId(1), "reposix-fuse-1")
            .await
            .unwrap();
        assert_eq!(got.id, IssueId(1));
    }

    #[tokio::test]
    async fn fetch_issue_404_is_not_found() {
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let http = client(ClientOpts::default()).unwrap();
        let err = fetch_issue(&http, &server.uri(), "demo", IssueId(9), "reposix-fuse-1")
            .await
            .unwrap_err();
        assert!(matches!(err, FetchError::NotFound), "got {err:?}");
    }

    #[tokio::test]
    async fn fetch_issue_500_is_status() {
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;
        let http = client(ClientOpts::default()).unwrap();
        let err = fetch_issue(&http, &server.uri(), "demo", IssueId(1), "reposix-fuse-1")
            .await
            .unwrap_err();
        assert!(matches!(err, FetchError::Status(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn fetch_issues_attaches_agent_header() {
        let server = MockServer::start().await;
        let issues = vec![sample_issue(1)];
        Mock::given(method("GET"))
            .and(path("/projects/demo/issues"))
            .and(header("X-Reposix-Agent", "reposix-fuse-42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&issues))
            .mount(&server)
            .await;
        let http = client(ClientOpts::default()).unwrap();
        let got = fetch_issues(&http, &server.uri(), "demo", "reposix-fuse-42")
            .await
            .unwrap();
        assert_eq!(got.len(), 1);
    }

    #[tokio::test]
    async fn fetch_issue_origin_rejected() {
        // evil.example is not in the default allowlist — expect Origin error.
        let http = client(ClientOpts::default()).unwrap();
        let err = fetch_issue(&http, "http://evil.example", "demo", IssueId(1), "agent")
            .await
            .unwrap_err();
        assert!(matches!(err, FetchError::Origin(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn patch_issue_sends_if_match_header() {
        use reposix_core::{sanitize, ServerMetadata, Tainted};
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/1"))
            .and(header("If-Match", "3"))
            .and(header("X-Reposix-Agent", "reposix-fuse-42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(sample_issue(1)))
            .mount(&server)
            .await;
        let http = client(ClientOpts::default()).unwrap();
        let t = chrono::Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        let meta = ServerMetadata {
            id: IssueId(1),
            created_at: t,
            updated_at: t,
            version: 3,
        };
        let untainted = sanitize(Tainted::new(sample_issue(1)), meta);
        let got = patch_issue(
            &http,
            &server.uri(),
            "demo",
            IssueId(1),
            3,
            untainted,
            "reposix-fuse-42",
        )
        .await
        .unwrap();
        assert_eq!(got.id, IssueId(1));
    }

    #[tokio::test]
    async fn patch_issue_409_returns_conflict() {
        use reposix_core::{sanitize, ServerMetadata, Tainted};
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/projects/demo/issues/1"))
            .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
                "error": "version_mismatch",
                "current": 7,
                "sent": "1"
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
    async fn patch_issue_times_out_within_budget() {
        use reposix_core::{sanitize, ServerMetadata, Tainted};
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
        let t0 = Instant::now();
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
    async fn post_issue_sends_egress_shape_only() {
        use reposix_core::{sanitize, ServerMetadata, Tainted};
        use wiremock::matchers::body_string_contains;
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/projects/demo/issues"))
            .and(body_string_contains("\"title\""))
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

    #[tokio::test]
    async fn fetch_issue_times_out_within_budget() {
        // 10s backend delay + 5s fetch timeout → Timeout within ~5.5s.
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
            .mount(&server)
            .await;
        let http = client(ClientOpts::default()).unwrap();
        let t0 = Instant::now();
        let err = fetch_issue(&http, &server.uri(), "demo", IssueId(1), "agent")
            .await
            .unwrap_err();
        let elapsed = t0.elapsed();
        // Either our timeout fired first (FetchError::Timeout) or reqwest's
        // 5s ClientOpts timeout fired first (FetchError::Transport with
        // is_timeout()); both prove the 5s ceiling holds.
        let ok = matches!(err, FetchError::Timeout)
            || matches!(&err, FetchError::Transport(e) if e.is_timeout());
        assert!(ok, "expected timeout-flavored error, got {err:?}");
        assert!(
            elapsed < Duration::from_millis(5_800),
            "should return within 5.5s; took {elapsed:?}"
        );
    }
}
