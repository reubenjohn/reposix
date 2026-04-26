//! [`GithubReadOnlyBackend`] — a read-only [`BackendConnector`] that adapts
//! GitHub's REST v3 Issues API onto reposix's normalized `Record` /
//! `RecordStatus` shape.
//!
//! # Scope
//!
//! v0.1 ships **read-only**: `list_records` and `get_record` work against real
//! GitHub; `create_record` / `update_record` / `delete_or_close` return
//! `Error::Other("not supported: ...")`. The v0.2 cut will flip on the write
//! path once credentials-handling UX lands (see ADR-001 for the write-side of
//! the state mapping).
//!
//! # State mapping
//!
//! GitHub's 2-valued `state` + optional `state_reason` + label conventions
//! collapse onto reposix's 5-valued `RecordStatus` via
//! [`docs/decisions/001-github-state-mapping.md`](../docs/decisions/001-github-state-mapping.md).
//! Summary:
//!
//! - `open` + no `status/*` label → `Open`.
//! - `open` + `status/in-progress` → `InProgress`.
//! - `open` + `status/in-review` → `InReview`.
//! - `closed` + `state_reason = completed | null | other` → `Done`.
//! - `closed` + `state_reason = not_planned` → `WontFix`.
//!
//! # Contract test
//!
//! [`tests/contract.rs`](../../tests/contract.rs) runs the same 5 invariants
//! against both `SimBackend` and `GithubReadOnlyBackend`, proving
//! normalized-shape parity. The GitHub half is `#[ignore]`-gated so
//! unauthenticated CI stays green even when GitHub's 60 req/hr anonymous
//! ceiling is exhausted.
//!
//! # Security
//!
//! Every HTTP call goes through `reposix-core`'s sealed [`HttpClient`], which
//! re-checks every target URL against `REPOSIX_ALLOWED_ORIGINS` before any
//! socket I/O (SG-01). Because GitHub's production origin is
//! `https://api.github.com`, callers MUST set the env var explicitly at
//! runtime, e.g.:
//!
//! ```bash
//! REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:*,https://api.github.com
//! ```
//!
//! We deliberately do NOT compile-time-add `https://api.github.com` to the
//! default allowlist — the reposix-core default is loopback-only and that
//! invariant matters more than the "zero-config GitHub" UX. A future v0.2
//! feature flag may revisit this once the allowlist gains a named-preset
//! mechanism.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use parking_lot::Mutex;
use reqwest::{Method, StatusCode};
use serde::Deserialize;

use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason};
use reposix_core::http::{client, ClientOpts, HttpClient};
use reposix_core::{Error, Record, RecordId, RecordStatus, Result, Untainted};

/// Maximum time we'll wait for a rate-limit reset before surfacing the
/// exhaustion as an error. Caps the worst-case call latency; a well-behaved
/// GitHub token should never hit this.
const MAX_RATE_LIMIT_SLEEP: Duration = Duration::from_secs(60);

/// Default GitHub REST API base. Override via
/// [`GithubReadOnlyBackend::new_with_base_url`] for tests that point at a
/// `wiremock::MockServer` (or a corporate GitHub Enterprise tenant).
pub const DEFAULT_BASE_URL: &str = "https://api.github.com";

/// Capability matrix row published by this backend for `reposix doctor`.
///
/// GitHub Issues supports the full read/write surface alongside the sim,
/// with comments routed in-body (the read path collapses comments into the
/// body's frontmatter+body), delete-as-close (issues are closed with a
/// state reason rather than removed), and ETag-based optimistic concurrency
/// for write conflicts.
pub const CAPABILITIES: reposix_core::BackendCapabilities = reposix_core::BackendCapabilities::new(
    true,
    true,
    true,
    true,
    reposix_core::CommentSupport::InBody,
    reposix_core::VersioningModel::Etag,
);

/// Label prefix that encodes the two "open-but-active" variants (see
/// ADR-001). Only `status/in-progress` and `status/in-review` participate;
/// any other `status/*` label is ignored and the issue falls through to
/// `Open`.
const STATUS_LABEL_PREFIX: &str = "status/";
const STATUS_LABEL_IN_PROGRESS: &str = "status/in-progress";
const STATUS_LABEL_IN_REVIEW: &str = "status/in-review";

/// Max issues we'll page through in one `list_records` call. GitHub allows
/// 100/page; at 5 pages that's 500 issues — enough for the `octocat/Hello-World`
/// fixture and a bounded memory budget for pathological repos. Adjustable in
/// a future version if users hit the cap.
const MAX_ISSUES_PER_LIST: usize = 500;

/// Page size GitHub accepts (1..=100). We pick the maximum to minimize HTTP
/// round-trips within the 5-page cap above.
const PAGE_SIZE: usize = 100;

/// `BackendConnector` implementation for GitHub's REST v3 Issues API.
///
/// Construct via [`GithubReadOnlyBackend::new`] (uses the public production
/// API) or [`GithubReadOnlyBackend::new_with_base_url`] (accepts a custom
/// base; used by the unit tests to point at `wiremock`).
///
/// # Thread-safety
///
/// `Clone` is cheap (the inner `HttpClient` is `Arc`-shared) and all methods
/// are `&self`, so the struct is safe to share across tokio tasks.
#[derive(Clone)]
pub struct GithubReadOnlyBackend {
    http: Arc<HttpClient>,
    token: Option<String>,
    base_url: String,
    /// When `Some(t)`, the next outbound request must sleep until `t` to
    /// respect GitHub's rate limit. Set after a response where
    /// `x-ratelimit-remaining` hits zero; cleared when the reset elapses.
    /// Shared across clones so a single depleted token can't starve the
    /// backend just because two tasks happen to hold separate handles.
    rate_limit_gate: Arc<Mutex<Option<Instant>>>,
}

// Hand-rolled `Debug` that REDACTS the `GITHUB_TOKEN`. The auto-derived
// `Debug` would emit `Some("ghp_…")` — a credential leak the moment a
// `tracing::error!("{:?}", backend)` or a panic backtrace catches it.
// Mirrors the same redaction pattern in `reposix-confluence` and
// `reposix-jira`. Surfaced by the v0.11.1 security-persona audit.
impl std::fmt::Debug for GithubReadOnlyBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GithubReadOnlyBackend")
            .field("http", &self.http)
            .field("token", &self.token.as_ref().map(|_| "<redacted>"))
            .field("base_url", &self.base_url)
            .field("rate_limit_gate", &self.rate_limit_gate)
            .finish()
    }
}

/// Minimal GitHub issue shape we actually consume. `deny_unknown_fields` is
/// deliberately NOT set — GitHub adds fields routinely and we'd rather stay
/// forward-compatible than crash on a new column.
#[derive(Debug, Deserialize)]
struct GhIssue {
    number: u64,
    title: String,
    state: String,
    #[serde(default)]
    state_reason: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    labels: Vec<GhLabel>,
    #[serde(default)]
    assignee: Option<GhUser>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct GhLabel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GhUser {
    login: String,
}

impl GithubReadOnlyBackend {
    /// Build a new backend targeting the public GitHub REST API.
    ///
    /// `token` is optional: `Some(..)` authenticates via `Authorization:
    /// Bearer ..` and buys the 5000 req/hr ceiling; `None` uses anonymous
    /// access, capped at 60 req/hr per IP. Callers that expect heavy load
    /// (CI contract tests) should set a token.
    ///
    /// **Important:** because this backend speaks to `https://api.github.com`,
    /// the caller MUST set `REPOSIX_ALLOWED_ORIGINS` to include
    /// `https://api.github.com` before the first request. The default
    /// reposix-core allowlist is loopback-only (SG-01).
    ///
    /// # Errors
    ///
    /// Propagates any error from [`client`] (e.g. a malformed
    /// `REPOSIX_ALLOWED_ORIGINS` spec at construction time).
    pub fn new(token: Option<String>) -> Result<Self> {
        Self::new_with_base_url(token, DEFAULT_BASE_URL.to_owned())
    }

    /// Like [`Self::new`] but with a caller-supplied base URL. Used by
    /// `tests/contract.rs` to point at a local `wiremock::MockServer`.
    ///
    /// # Errors
    ///
    /// Same as [`Self::new`].
    pub fn new_with_base_url(token: Option<String>, base_url: String) -> Result<Self> {
        let http = client(ClientOpts::default())?;
        Ok(Self {
            http: Arc::new(http),
            token,
            base_url,
            rate_limit_gate: Arc::new(Mutex::new(None)),
        })
    }

    fn base(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }

    /// Assemble the standard headers every GitHub REST call carries.
    ///
    /// Returns owned strings so callers can `.iter().map(|(k, v)| (k.as_str(),
    /// v.as_str()))` into the `&[(&str, &str)]` shape `HttpClient` wants
    /// without borrow-lifetime gymnastics.
    fn standard_headers(&self) -> Vec<(&'static str, String)> {
        let mut h = vec![
            ("Accept", "application/vnd.github+json".to_owned()),
            ("User-Agent", "reposix-github-readonly/0.1".to_owned()),
            ("X-GitHub-Api-Version", "2022-11-28".to_owned()),
        ];
        if let Some(ref tok) = self.token {
            h.push(("Authorization", format!("Bearer {tok}")));
        }
        h
    }

    /// Inspect `x-ratelimit-remaining` + `x-ratelimit-reset` and:
    ///  - WARN at `remaining < 10` so operators see the ceiling approaching.
    ///  - When `remaining == 0`, set the shared [`rate_limit_gate`] so the
    ///    next outbound call sleeps until the reset (capped at
    ///    [`MAX_RATE_LIMIT_SLEEP`]).
    ///
    /// Returning the gate state to the caller would also be reasonable, but
    /// keeping it inside the struct means `await_rate_limit_gate` is the
    /// single choke-point that enforces it.
    ///
    /// [`rate_limit_gate`]: Self::rate_limit_gate
    fn ingest_rate_limit(&self, resp: &reqwest::Response) {
        let headers = resp.headers();
        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let reset = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        match remaining {
            Some(0) => {
                let now_unix = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let wait = reset
                    .map_or(0, |r| r.saturating_sub(now_unix))
                    .min(MAX_RATE_LIMIT_SLEEP.as_secs());
                if wait > 0 {
                    let gate = Instant::now() + Duration::from_secs(wait);
                    *self.rate_limit_gate.lock() = Some(gate);
                    tracing::warn!(
                        wait_secs = wait,
                        "GitHub rate limit exhausted — backing off until reset"
                    );
                }
            }
            Some(n) if n < 10 => {
                tracing::warn!(remaining = n, "GitHub rate limit approaching exhaustion");
            }
            _ => {}
        }
    }

    /// If a previous response parked us behind a rate-limit gate, sleep
    /// until the gate elapses (or the cap expires). Called before every
    /// outbound request. Cheap when the gate is `None` — just a
    /// parking-lot read.
    async fn await_rate_limit_gate(&self) {
        let maybe_gate = *self.rate_limit_gate.lock();
        if let Some(deadline) = maybe_gate {
            let now = Instant::now();
            if deadline > now {
                let wait = deadline - now;
                tokio::time::sleep(wait).await;
            }
            // Clear the gate unconditionally — either we slept through it
            // or it had already elapsed. If GitHub's still exhausted, the
            // next response will re-arm it.
            *self.rate_limit_gate.lock() = None;
        }
    }
}

/// Translate a GitHub issue payload into reposix's normalized [`Record`],
/// applying ADR-001's state mapping.
fn translate(gh: GhIssue) -> Record {
    // Determine the status via ADR-001:
    // - closed + not_planned → WontFix.
    // - closed + (completed | null | other)  → Done.
    // - open + status/in-review label → InReview.
    // - open + status/in-progress label → InProgress.
    // - open + no status/* label → Open.
    let status = if gh.state == "closed" {
        match gh.state_reason.as_deref() {
            Some("not_planned") => RecordStatus::WontFix,
            // Completed, reopened, duplicate, null, unknown — pessimistic
            // fallback per ADR-001.
            _ => RecordStatus::Done,
        }
    } else if gh.labels.iter().any(|l| l.name == STATUS_LABEL_IN_REVIEW) {
        RecordStatus::InReview
    } else if gh.labels.iter().any(|l| l.name == STATUS_LABEL_IN_PROGRESS) {
        RecordStatus::InProgress
    } else {
        RecordStatus::Open
    };

    // Strip the status/* markers from the user-visible label list — they're
    // an implementation detail of the mapping convention, not user content.
    let labels: Vec<String> = gh
        .labels
        .into_iter()
        .filter(|l| !l.name.starts_with(STATUS_LABEL_PREFIX))
        .map(|l| l.name)
        .collect();

    Record {
        id: RecordId(gh.number),
        title: gh.title,
        status,
        assignee: gh.assignee.map(|u| u.login),
        labels,
        created_at: gh.created_at,
        updated_at: gh.updated_at,
        // GitHub doesn't carry a monotonic version; sanitize() treats 1 as
        // "first revision" which matches our round-trip expectation.
        version: 1,
        body: gh.body.unwrap_or_default(),
        // GitHub Issues doesn't expose a parent hierarchy; always None.
        parent_id: None,
        extensions: std::collections::BTreeMap::new(),
    }
}

/// Extract the `rel="next"` URL from a `Link:` header, if present. GitHub
/// formats these as `<url1>; rel="first", <url2>; rel="next"` etc.
fn parse_next_link(link_header: &str) -> Option<String> {
    for entry in link_header.split(',') {
        let entry = entry.trim();
        // Each entry: <url>; rel="next"
        let Some(end) = entry.find('>') else { continue };
        if !entry.starts_with('<') {
            continue;
        }
        let url = &entry[1..end];
        let rest = &entry[end + 1..];
        if rest.contains("rel=\"next\"") {
            return Some(url.to_owned());
        }
    }
    None
}

#[async_trait]
impl BackendConnector for GithubReadOnlyBackend {
    fn name(&self) -> &'static str {
        "github-readonly"
    }

    fn supports(&self, feature: BackendFeature) -> bool {
        // GitHub's 2-valued state + labels DO give us named workflow
        // transitions, so we claim Workflows. Read-only v0.1 cannot Delete,
        // Transitions (writes), StrongVersioning (no etags plumbed yet), or
        // BulkEdit.
        matches!(feature, BackendFeature::Workflows)
    }

    async fn list_records(&self, project: &str) -> Result<Vec<Record>> {
        let first = format!(
            "{}/repos/{}/issues?state=all&per_page={}",
            self.base(),
            project,
            PAGE_SIZE
        );
        let mut next_url: Option<String> = Some(first);
        let mut out: Vec<Record> = Vec::new();
        let mut pages: usize = 0;

        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();

        while let Some(url) = next_url.take() {
            pages += 1;
            if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) {
                tracing::warn!(
                    pages,
                    "reached MAX_ISSUES_PER_LIST cap; stopping pagination"
                );
                break;
            }
            self.await_rate_limit_gate().await;
            let resp = self
                .http
                .request_with_headers(Method::GET, url.as_str(), &header_refs)
                .await?;
            self.ingest_rate_limit(&resp);
            let status = resp.status();
            let link_hdr = resp
                .headers()
                .get("link")
                .and_then(|v| v.to_str().ok())
                .map(std::string::ToString::to_string);
            let bytes = resp.bytes().await?;
            if !status.is_success() {
                return Err(Error::Other(format!(
                    "github returned {status} for GET {url}: {}",
                    String::from_utf8_lossy(&bytes)
                )));
            }
            let page: Vec<GhIssue> = serde_json::from_slice(&bytes)?;
            for gh in page {
                out.push(translate(gh));
                if out.len() >= MAX_ISSUES_PER_LIST {
                    return Ok(out);
                }
            }
            next_url = link_hdr.as_deref().and_then(parse_next_link);
        }
        Ok(out)
    }

    async fn get_record(&self, project: &str, id: RecordId) -> Result<Record> {
        let url = format!("{}/repos/{}/issues/{}", self.base(), project, id.0);
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
        if status == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!("not found: {url}")));
        }
        if !status.is_success() {
            return Err(Error::Other(format!(
                "github returned {status} for GET {url}: {}",
                String::from_utf8_lossy(&bytes)
            )));
        }
        let gh: GhIssue = serde_json::from_slice(&bytes)?;
        Ok(translate(gh))
    }

    async fn list_changed_since(
        &self,
        project: &str,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<RecordId>> {
        // GitHub's `GET /repos/{owner}/{repo}/issues` natively accepts
        // `?since=<ISO8601>`; reuse the same pagination loop as `list_records`
        // but emit IDs only.
        let since_iso = since.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let first = format!(
            "{}/repos/{}/issues?state=all&per_page={}&since={}",
            self.base(),
            project,
            PAGE_SIZE,
            since_iso
        );
        let mut next_url: Option<String> = Some(first);
        let mut out: Vec<RecordId> = Vec::new();
        let mut pages: usize = 0;

        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();

        while let Some(url) = next_url.take() {
            pages += 1;
            if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) {
                tracing::warn!(
                    pages,
                    "reached MAX_ISSUES_PER_LIST cap; stopping pagination"
                );
                break;
            }
            self.await_rate_limit_gate().await;
            let resp = self
                .http
                .request_with_headers(Method::GET, url.as_str(), &header_refs)
                .await?;
            self.ingest_rate_limit(&resp);
            let status = resp.status();
            let link_hdr = resp
                .headers()
                .get("link")
                .and_then(|v| v.to_str().ok())
                .map(std::string::ToString::to_string);
            let bytes = resp.bytes().await?;
            if !status.is_success() {
                return Err(Error::Other(format!(
                    "github returned {status} for GET {url}: {}",
                    String::from_utf8_lossy(&bytes)
                )));
            }
            let page: Vec<GhIssue> = serde_json::from_slice(&bytes)?;
            for gh in page {
                let issue = translate(gh);
                out.push(issue.id);
                if out.len() >= MAX_ISSUES_PER_LIST {
                    return Ok(out);
                }
            }
            next_url = link_hdr.as_deref().and_then(parse_next_link);
        }
        Ok(out)
    }

    async fn create_record(&self, _project: &str, _issue: Untainted<Record>) -> Result<Record> {
        Err(Error::Other(
            "not supported: create_record — reposix-github is read-only in v0.1".into(),
        ))
    }

    async fn update_record(
        &self,
        _project: &str,
        _id: RecordId,
        _patch: Untainted<Record>,
        _expected_version: Option<u64>,
    ) -> Result<Record> {
        Err(Error::Other(
            "not supported: update_record — reposix-github is read-only in v0.1".into(),
        ))
    }

    async fn delete_or_close(
        &self,
        _project: &str,
        _id: RecordId,
        _reason: DeleteReason,
    ) -> Result<()> {
        Err(Error::Other(
            "not supported: delete_or_close — reposix-github is read-only in v0.1".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reposix_core::{sanitize, ServerMetadata, Tainted};
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn gh_issue_json(
        number: u64,
        state: &str,
        state_reason: Option<&str>,
        labels: &[&str],
    ) -> serde_json::Value {
        json!({
            "number": number,
            "title": format!("issue {number}"),
            "state": state,
            "state_reason": state_reason,
            "body": "some body",
            "labels": labels.iter().map(|l| json!({"name": l})).collect::<Vec<_>>(),
            "assignee": null,
            "created_at": "2026-04-13T00:00:00Z",
            "updated_at": "2026-04-13T00:00:00Z",
        })
    }

    /// The default reposix-core allowlist is `http://127.0.0.1:*,http://localhost:*`.
    /// `wiremock::MockServer` binds to `127.0.0.1` so the default allowlist
    /// accepts it and we don't need to touch env vars in tests.

    #[tokio::test]
    async fn list_builds_the_right_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/octocat/hello/issues"))
            .and(query_param("state", "all"))
            .and(query_param("per_page", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                gh_issue_json(1, "open", None, &[]),
                gh_issue_json(2, "closed", Some("completed"), &[]),
            ])))
            .mount(&server)
            .await;

        let backend =
            GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
        let issues = backend.list_records("octocat/hello").await.expect("list");
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].id, RecordId(1));
        assert_eq!(issues[0].status, RecordStatus::Open);
        assert_eq!(issues[1].id, RecordId(2));
        assert_eq!(issues[1].status, RecordStatus::Done);
    }

    #[tokio::test]
    async fn get_builds_the_right_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/foo/bar/issues/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(gh_issue_json(
                42,
                "open",
                None,
                &[],
            )))
            .mount(&server)
            .await;

        let backend =
            GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
        let issue = backend
            .get_record("foo/bar", RecordId(42))
            .await
            .expect("get");
        assert_eq!(issue.id, RecordId(42));
        assert_eq!(issue.title, "issue 42");
    }

    #[tokio::test]
    async fn list_paginates_via_link_header() {
        let server = MockServer::start().await;
        // Page 2 URL points back at the same mock server so the allowlist
        // accepts it (127.0.0.1 is always allowed by default).
        let next_url = format!(
            "{}/repos/o/r/issues?state=all&per_page=100&page=2",
            server.uri()
        );
        let link_val = format!("<{next_url}>; rel=\"next\"");

        // Page 1: 2 issues, Link header says "page 2 available".
        // Registered FIRST so wiremock matches most-specific-first rule:
        // `up_to_n_times(1)` means this only answers one request then the
        // next matcher takes over.
        Mock::given(method("GET"))
            .and(path("/repos/o/r/issues"))
            .and(query_param("state", "all"))
            .and(query_param("per_page", "100"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("Link", link_val.as_str())
                    .set_body_json(json!([
                        gh_issue_json(1, "open", None, &[]),
                        gh_issue_json(2, "open", None, &[]),
                    ])),
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;

        // Page 2: one issue, no next link. More specific matcher (includes
        // `page=2`).
        Mock::given(method("GET"))
            .and(path("/repos/o/r/issues"))
            .and(query_param("page", "2"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!([gh_issue_json(
                    3,
                    "open",
                    None,
                    &[]
                ),])),
            )
            .mount(&server)
            .await;

        let backend =
            GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
        let issues = backend.list_records("o/r").await.expect("list");
        assert_eq!(issues.len(), 3, "expected all 3 pages combined");
        assert_eq!(issues[0].id, RecordId(1));
        assert_eq!(issues[2].id, RecordId(3));
    }

    #[tokio::test]
    async fn closed_with_completed_reason_maps_to_done() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/x/y/issues/99"))
            .respond_with(ResponseTemplate::new(200).set_body_json(gh_issue_json(
                99,
                "closed",
                Some("completed"),
                &[],
            )))
            .mount(&server)
            .await;

        let backend =
            GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
        let issue = backend.get_record("x/y", RecordId(99)).await.expect("get");
        assert_eq!(issue.status, RecordStatus::Done);
    }

    #[tokio::test]
    async fn closed_with_not_planned_maps_to_wontfix() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/x/y/issues/100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(gh_issue_json(
                100,
                "closed",
                Some("not_planned"),
                &[],
            )))
            .mount(&server)
            .await;

        let backend =
            GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
        let issue = backend.get_record("x/y", RecordId(100)).await.expect("get");
        assert_eq!(issue.status, RecordStatus::WontFix);
    }

    #[tokio::test]
    async fn open_with_in_progress_label_maps_to_in_progress() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/x/y/issues/7"))
            .respond_with(ResponseTemplate::new(200).set_body_json(gh_issue_json(
                7,
                "open",
                None,
                &["bug", "status/in-progress"],
            )))
            .mount(&server)
            .await;

        let backend =
            GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
        let issue = backend.get_record("x/y", RecordId(7)).await.expect("get");
        assert_eq!(issue.status, RecordStatus::InProgress);
        // Status label is stripped from the user-visible labels.
        assert_eq!(issue.labels, vec!["bug".to_string()]);
    }

    #[tokio::test]
    async fn open_with_in_review_label_maps_to_in_review() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/x/y/issues/8"))
            .respond_with(ResponseTemplate::new(200).set_body_json(gh_issue_json(
                8,
                "open",
                None,
                &["status/in-review"],
            )))
            .mount(&server)
            .await;

        let backend =
            GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
        let issue = backend.get_record("x/y", RecordId(8)).await.expect("get");
        assert_eq!(issue.status, RecordStatus::InReview);
    }

    #[tokio::test]
    async fn get_404_maps_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/x/y/issues/9999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({"message":"Not Found"})))
            .mount(&server)
            .await;

        let backend =
            GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
        let err = backend
            .get_record("x/y", RecordId(9999))
            .await
            .expect_err("404 should surface as error");
        match err {
            Error::Other(m) => assert!(m.starts_with("not found:"), "got {m}"),
            other => panic!("expected Error::Other(not found:..), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn rate_limit_zero_remaining_arms_the_gate() {
        // A response with `x-ratelimit-remaining: 0` and a reset in the near
        // future should park the gate on an Instant roughly that far out.
        let server = MockServer::start().await;
        let reset_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("unix time")
            .as_secs()
            + 2;
        Mock::given(method("GET"))
            .and(path("/repos/octocat/Hello-World/issues/42"))
            .respond_with(
                ResponseTemplate::new(200)
                    .append_header("x-ratelimit-remaining", "0")
                    .append_header("x-ratelimit-reset", reset_secs.to_string().as_str())
                    .set_body_json(gh_issue_json(42, "open", None, &[])),
            )
            .mount(&server)
            .await;
        let backend =
            GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
        let _ = backend
            .get_record("octocat/Hello-World", RecordId(42))
            .await
            .expect("get");
        // Now the gate must be set to within ~2s of now. A follow-up call
        // would sleep until the reset; we assert the armed state directly.
        let gate = backend.rate_limit_gate.lock().to_owned();
        let now = std::time::Instant::now();
        match gate {
            Some(deadline) => {
                assert!(
                    deadline > now,
                    "gate must be in the future, got {:?}",
                    deadline.saturating_duration_since(now)
                );
                assert!(
                    deadline - now
                        <= std::time::Duration::from_secs(MAX_RATE_LIMIT_SLEEP.as_secs() + 1),
                    "gate must be capped at MAX_RATE_LIMIT_SLEEP"
                );
            }
            None => panic!("gate should be armed after remaining=0 response"),
        }
    }

    #[tokio::test]
    async fn create_returns_not_supported() {
        // No mock needed — write methods short-circuit before any HTTP.
        let backend = GithubReadOnlyBackend::new_with_base_url(None, DEFAULT_BASE_URL.to_owned())
            .expect("backend");
        let t = chrono::Utc::now();
        let u = sanitize(
            Tainted::new(Record {
                id: RecordId(0),
                title: "x".into(),
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
                version: 1,
            },
        );
        let err = backend
            .create_record("x/y", u)
            .await
            .expect_err("read-only should reject create");
        match err {
            Error::Other(m) => assert!(m.starts_with("not supported:"), "got {m}"),
            other => panic!("expected Error::Other(not supported:..), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn update_returns_not_supported() {
        let backend = GithubReadOnlyBackend::new_with_base_url(None, DEFAULT_BASE_URL.to_owned())
            .expect("backend");
        let t = chrono::Utc::now();
        let u = sanitize(
            Tainted::new(Record {
                id: RecordId(0),
                title: "x".into(),
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
                version: 1,
            },
        );
        assert!(matches!(
            backend.update_record("x/y", RecordId(1), u, None).await,
            Err(Error::Other(m)) if m.starts_with("not supported:")
        ));
    }

    #[tokio::test]
    async fn delete_returns_not_supported() {
        let backend = GithubReadOnlyBackend::new_with_base_url(None, DEFAULT_BASE_URL.to_owned())
            .expect("backend");
        assert!(matches!(
            backend
                .delete_or_close("x/y", RecordId(1), DeleteReason::Completed)
                .await,
            Err(Error::Other(m)) if m.starts_with("not supported:")
        ));
    }

    #[test]
    fn supports_reports_workflows_only() {
        let backend = GithubReadOnlyBackend::new_with_base_url(None, DEFAULT_BASE_URL.to_owned())
            .expect("backend");
        assert!(backend.supports(BackendFeature::Workflows));
        assert!(!backend.supports(BackendFeature::Delete));
        assert!(!backend.supports(BackendFeature::Transitions));
        assert!(!backend.supports(BackendFeature::StrongVersioning));
        assert!(!backend.supports(BackendFeature::BulkEdit));
        assert_eq!(backend.name(), "github-readonly");
    }

    #[tokio::test]
    async fn github_list_changed_since_sends_since_param_and_returns_ids() {
        // Phase 33: prove the GitHub override emits ?since=<RFC3339>
        // alongside the existing state=all query param.
        use chrono::{TimeZone, Utc};
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/octo/r/issues"))
            .and(query_param("since", "2026-04-24T00:00:00Z"))
            .and(query_param("state", "all"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {
                    "number": 7,
                    "title": "x",
                    "state": "open",
                    "body": "",
                    "labels": [],
                    "assignee": null,
                    "created_at": "2026-04-24T01:00:00Z",
                    "updated_at": "2026-04-24T02:00:00Z",
                }
            ])))
            .expect(1)
            .mount(&server)
            .await;

        let backend =
            GithubReadOnlyBackend::new_with_base_url(None, server.uri()).expect("backend");
        let t = Utc.with_ymd_and_hms(2026, 4, 24, 0, 0, 0).unwrap();
        let ids = backend.list_changed_since("octo/r", t).await.expect("list");
        assert_eq!(ids, vec![RecordId(7)]);
    }

    #[test]
    fn parse_next_link_extracts_url() {
        let h = r#"<https://api.github.com/repositories/1/issues?page=2>; rel="next", <https://api.github.com/repositories/1/issues?page=5>; rel="last""#;
        assert_eq!(
            parse_next_link(h).as_deref(),
            Some("https://api.github.com/repositories/1/issues?page=2")
        );
    }

    #[test]
    fn parse_next_link_absent_returns_none() {
        let h = r#"<https://api.github.com/repositories/1/issues?page=5>; rel="last""#;
        assert!(parse_next_link(h).is_none());
    }
}
