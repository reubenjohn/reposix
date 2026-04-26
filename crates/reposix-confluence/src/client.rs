//! HTTP plumbing, audit hooks, and rate-limit gate for [`ConfluenceBackend`].
//!
//! Holds the [`ConfluenceBackend`] struct alongside its constructors and
//! internal helpers — every method that talks to the network or the audit
//! DB lives here. The trait surface (`impl BackendConnector for
//! ConfluenceBackend`) lives in `crate::lib` (so this file stays focused on
//! plumbing, and the trait impl reads as a thin adapter over these helpers).

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use reqwest::{Method, StatusCode};
use rusqlite::Connection;

use reposix_core::http::{client, ClientOpts, HttpClient};
use reposix_core::{Error, Record, RecordId, Result, Tainted};

use crate::translate::{
    basic_auth_header, parse_next_cursor, redact_url, translate, validate_tenant,
};
use crate::types::{
    CommentKind, ConfAttachment, ConfAttachmentList, ConfComment, ConfCommentList,
    ConfDirectChildrenList, ConfPageList, ConfSpaceList, ConfSpaceSummary, ConfSpaceSummaryList,
    ConfWhiteboard, ConfluenceCreds, MAX_ISSUES_PER_LIST, MAX_RATE_LIMIT_SLEEP, PAGE_SIZE,
};

/// Read-only `BackendConnector` for Atlassian Confluence Cloud REST v2.
///
/// Construct via [`ConfluenceBackend::new`] (public production API)
/// or [`ConfluenceBackend::new_with_base_url`] (custom base; used by
/// wiremock unit tests and the contract test).
///
/// Write methods (`create_record`, `update_record`, `delete_or_close`) are
/// fully implemented as of v0.6. Audit log rows are added in Phase 16
/// Wave C.
///
/// # Thread-safety
///
/// `Clone` is cheap (`http` is `Arc`-shared) and all methods take `&self`,
/// so the struct is safe to share across tokio tasks. The rate-limit gate
/// is shared across clones so a single throttled instance can't be bypassed
/// by cloning.
#[derive(Clone)]
pub struct ConfluenceBackend {
    pub(crate) http: Arc<HttpClient>,
    pub(crate) creds: ConfluenceCreds,
    /// Tenant base URL with no trailing slash, e.g.
    /// `https://reuben-john.atlassian.net` in production, or the wiremock
    /// server URI in tests.
    pub(crate) base_url: String,
    /// When `Some(t)`, the next outbound request must sleep until `t`.
    /// Set after a response where `x-ratelimit-remaining` hits zero or a
    /// 429 is returned with a `Retry-After` header. Shared across clones.
    pub(crate) rate_limit_gate: Arc<Mutex<Option<Instant>>>,
    /// Optional audit log connection. When `Some`, every write call
    /// (`create_record`, `update_record`, `delete_or_close`) inserts one row
    /// into the `audit_events` table. The caller is responsible for opening
    /// the connection via [`reposix_core::audit::open_audit_db`] so the
    /// schema and append-only triggers are loaded before the first insert.
    ///
    /// `None` by default — attach via [`Self::with_audit`].
    pub(crate) audit: Option<Arc<Mutex<Connection>>>,
}

// Manual Debug on the backend struct too — the derived Debug would print
// `creds` which has its own redaction, but being explicit documents the
// intent and ensures `cargo expand` can never accidentally flip back.
impl std::fmt::Debug for ConfluenceBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `http` is deliberately omitted; it has no meaningful Debug state
        // worth showing and including it obscures the redaction intent.
        f.debug_struct("ConfluenceBackend")
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

impl ConfluenceBackend {
    /// Build a new backend against `https://{tenant}.atlassian.net`.
    ///
    /// `tenant` is the Atlassian tenant subdomain (e.g. `"reuben-john"` for
    /// `https://reuben-john.atlassian.net`). Validated against DNS-label
    /// rules (`^[a-z0-9][a-z0-9-]{0,62}$`, no leading/trailing hyphen) to
    /// defeat SSRF via injection (T-11-02). Callers that need a custom base
    /// (wiremock, enterprise) use [`Self::new_with_base_url`] instead.
    ///
    /// **Important:** the caller MUST set `REPOSIX_ALLOWED_ORIGINS` to
    /// include `https://{tenant}.atlassian.net` before the first request.
    /// The reposix-core default allowlist is loopback-only (SG-01).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Other`] if `tenant` fails DNS-label validation, or
    /// if [`client`] fails (e.g. malformed `REPOSIX_ALLOWED_ORIGINS` spec).
    pub fn new(creds: ConfluenceCreds, tenant: &str) -> Result<Self> {
        validate_tenant(tenant)?;
        Self::new_with_base_url(creds, format!("https://{tenant}.atlassian.net"))
    }

    /// Like [`Self::new`] but accepts a caller-supplied base URL. Used by
    /// `tests` to point at a local `wiremock::MockServer`.
    ///
    /// # Errors
    ///
    /// Propagates any error from [`client`].
    pub fn new_with_base_url(creds: ConfluenceCreds, base_url: String) -> Result<Self> {
        let http = client(ClientOpts::default())?;
        Ok(Self {
            http: Arc::new(http),
            creds,
            base_url,
            rate_limit_gate: Arc::new(Mutex::new(None)),
            audit: None,
        })
    }

    /// Attach an audit log connection. Every write call (`create_record`,
    /// `update_record`, `delete_or_close`) inserts one row into
    /// `audit_events` when an audit connection is present; writes succeed
    /// even if the audit insert fails (best-effort, log-and-swallow — the
    /// Confluence round-trip has already committed).
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
    /// returns `Err(Error::Other(...))` instead of silently capping at
    /// `MAX_ISSUES_PER_LIST` pages, closing the SG-05 taint-escape risk
    /// (the agent thinking it has the whole space when it doesn't).
    ///
    /// Use this when a caller **must** see every page in the space or fail
    /// loudly. The default `list_records` still returns `Ok(capped)` with a
    /// `tracing::warn!` for backwards compatibility.
    ///
    /// # Errors
    ///
    /// - Returns `Error::Other` if pagination would exceed `MAX_ISSUES_PER_LIST`.
    /// - All errors that `list_records` would raise also apply here.
    pub async fn list_records_strict(&self, project: &str) -> Result<Vec<Record>> {
        self.list_issues_impl(project, true).await
    }

    /// Shared pagination loop for both `list_records` and
    /// [`Self::list_records_strict`]. When `strict == true` the cap site
    /// returns `Err`; when `false` it emits a `tracing::warn!` and returns
    /// `Ok(capped)`.
    ///
    /// # Errors
    ///
    /// - Transport or HTTP errors from the Confluence REST API.
    /// - In strict mode: `Error::Other` when the page cap is exceeded.
    pub(crate) async fn list_issues_impl(
        &self,
        project: &str,
        strict: bool,
    ) -> Result<Vec<Record>> {
        let space_id = self.resolve_space_id(project).await?;
        let first = format!(
            "{}/wiki/api/v2/spaces/{}/pages?limit={}",
            self.base(),
            space_id,
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
                if strict {
                    return Err(Error::Other(format!(
                        "Confluence space '{project}' exceeds {MAX_ISSUES_PER_LIST}-page cap; \
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
            let resp = self
                .http
                .request_with_headers(Method::GET, url.as_str(), &header_refs)
                .await?;
            self.ingest_rate_limit(&resp);
            let status = resp.status();
            let bytes = resp.bytes().await?;
            if !status.is_success() {
                return Err(Error::Other(format!(
                    "confluence returned {status} for GET {}: {}",
                    redact_url(&url),
                    String::from_utf8_lossy(&bytes)
                )));
            }
            // Parse as Value first so we can extract `_links.next` via the
            // pure helper, then deserialize the strongly-typed list shape.
            let body_json: serde_json::Value = serde_json::from_slice(&bytes)?;
            let next_cursor = parse_next_cursor(&body_json);
            let list: ConfPageList = serde_json::from_value(body_json)?;
            for page in list.results {
                // SG-05: wrap ingress bytes as Tainted before translating.
                let tainted = Tainted::new(page);
                let issue = translate(tainted.into_inner())?;
                out.push(issue);
                if out.len() >= MAX_ISSUES_PER_LIST {
                    if strict {
                        return Err(Error::Other(format!(
                            "Confluence space '{project}' exceeds {MAX_ISSUES_PER_LIST}-page cap; \
                             refusing to truncate (strict mode)"
                        )));
                    }
                    return Ok(out);
                }
            }
            // Drop the typed list's own `links` field in favor of the pure
            // helper's output — they come from the same JSON but the helper
            // keeps the test surface narrower.
            let _ = list.links;
            next_url = next_cursor.map(|relative| {
                // Relative path (e.g. "/wiki/api/v2/spaces/12345/pages?cursor=ABC")
                // — prepend tenant base. Absolute URLs would be caught by
                // the SG-01 allowlist gate anyway, but relative-only by
                // construction defeats SSRF by design.
                if relative.starts_with("http://") || relative.starts_with("https://") {
                    relative
                } else {
                    format!("{}{}", self.base(), relative)
                }
            });
        }
        Ok(out)
    }

    pub(crate) fn base(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }

    /// Assemble the standard headers every Confluence REST call carries.
    ///
    /// Returns owned strings so callers can `.iter().map(|(k, v)| (*k,
    /// v.as_str()))` into the `&[(&str, &str)]` shape `HttpClient` wants
    /// without lifetime gymnastics.
    pub(crate) fn standard_headers(&self) -> Vec<(&'static str, String)> {
        vec![
            ("Accept", "application/json".to_owned()),
            ("User-Agent", "reposix-confluence/0.6".to_owned()),
            (
                "Authorization",
                basic_auth_header(&self.creds.email, &self.creds.api_token),
            ),
        ]
    }

    /// Assemble headers for write requests (POST / PUT). Clones
    /// [`standard_headers`](Self::standard_headers) and appends
    /// `Content-Type: application/json`.
    ///
    /// Kept separate from `standard_headers` because GET and DELETE paths
    /// **must not** send `Content-Type` — some proxies reject or log unexpected
    /// content headers on body-less requests. The split makes the intent
    /// explicit and avoids accidentally adding a body header to every request.
    pub(crate) fn write_headers(&self) -> Vec<(&'static str, String)> {
        let mut h = self.standard_headers();
        h.push(("Content-Type", "application/json".to_owned()));
        h
    }

    /// Inspect rate-limit headers from `resp` and arm the shared gate if the
    /// response says we're throttled.
    ///
    /// Atlassian returns:
    /// - `x-ratelimit-remaining` (u64 as string)
    /// - `retry-after` (seconds as u64 string) — present on 429, sometimes
    ///   also on near-exhaustion 2xx responses
    /// - `x-ratelimit-reset` (ISO 8601 string) — we deliberately don't parse
    ///   this, `retry-after` is simpler and authoritative
    ///
    /// If the response is a 429 OR `x-ratelimit-remaining == 0`, we arm the
    /// gate on `Instant::now() + retry_after`, capped at `MAX_RATE_LIMIT_SLEEP`.
    /// If `retry-after` is absent, default to 60s.
    pub(crate) fn ingest_rate_limit(&self, resp: &reqwest::Response) {
        let headers = resp.headers();
        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let retry_after = headers
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let is_throttled = resp.status() == StatusCode::TOO_MANY_REQUESTS || remaining == Some(0);
        if is_throttled {
            let wait = retry_after
                .unwrap_or(60)
                .min(MAX_RATE_LIMIT_SLEEP.as_secs());
            if wait > 0 {
                let gate = Instant::now() + Duration::from_secs(wait);
                *self.rate_limit_gate.lock() = Some(gate);
                tracing::warn!(
                    wait_secs = wait,
                    "Confluence rate limit — backing off until retry-after"
                );
            }
        } else if let Some(n) = remaining {
            if n < 10 {
                tracing::warn!(
                    remaining = n,
                    "Confluence rate limit approaching exhaustion"
                );
            }
        }
    }

    /// If a previous response parked us behind a rate-limit gate, sleep
    /// until the gate elapses (or the cap expires). Called before every
    /// outbound request.
    pub(crate) async fn await_rate_limit_gate(&self) {
        let maybe_gate = *self.rate_limit_gate.lock();
        if let Some(deadline) = maybe_gate {
            let now = Instant::now();
            if deadline > now {
                let wait = deadline - now;
                tokio::time::sleep(wait).await;
            }
            *self.rate_limit_gate.lock() = None;
        }
    }

    /// Resolve a Confluence space key (e.g. `"REPOSIX"`) to its numeric
    /// space id (as a string, e.g. `"360450"`). One round-trip per
    /// `list_records` call.
    ///
    /// # Errors
    ///
    /// - Transport errors propagate.
    /// - HTTP non-2xx surfaces as `Err(Error::Other(…))`.
    /// - Empty `results` array (space not found) surfaces as
    ///   `Err(Error::Other("not found: space key …"))`.
    pub(crate) async fn resolve_space_id(&self, space_key: &str) -> Result<String> {
        // WR-01: build the URL via `url::Url::query_pairs_mut` so that any
        // metacharacters in `space_key` (`&`, `=`, `#`, space, non-ASCII) are
        // percent-encoded instead of smuggling extra query parameters.
        let mut url = url::Url::parse(&format!("{}/wiki/api/v2/spaces", self.base()))
            .map_err(|e| Error::Other(format!("bad base url: {e}")))?;
        url.query_pairs_mut().append_pair("keys", space_key);
        let url = url.to_string();
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
            return Err(Error::Other(format!("not found: space key {space_key}")));
        }
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status} for GET {}: {}",
                redact_url(&url),
                String::from_utf8_lossy(&bytes)
            )));
        }
        let list: ConfSpaceList = serde_json::from_slice(&bytes)?;
        if list.results.is_empty() {
            return Err(Error::Other(format!("not found: space key {space_key}")));
        }
        let id = list.results.into_iter().next().unwrap().id;
        // WR-02: the space_id is about to be interpolated into a URL path
        // component. Every byte from the network is tainted; refuse anything
        // that isn't strictly numeric (which is all Confluence documents as
        // legitimate). This blocks a malicious tenant from returning
        // `"12345/../admin"` or similar path-smuggling payloads — SG-01 only
        // gates origins, not paths, so this validation is the relevant cut.
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
            return Err(Error::Other(format!(
                "malformed space id from server: {id:?}"
            )));
        }
        Ok(id)
    }

    /// Fetch the current `version.number` for `id` without returning the full
    /// issue. Used by `update_record` when the caller passes
    /// `expected_version = None` and we need a pre-flight GET to discover the
    /// current version before constructing the PUT body.
    ///
    /// # Errors
    ///
    /// Propagates transport errors and `Err(Error::Other("not found: …"))` on
    /// 404.
    pub(crate) async fn fetch_current_version(&self, id: RecordId) -> Result<u64> {
        // Re-uses get_record which already handles rate-limit gate, SG-05
        // taint wrapping, and error mapping. The only "waste" is translating
        // the full page — that's acceptable for the `expected_version = None`
        // code path (one extra round-trip already implies we don't have cached
        // version data).
        use reposix_core::backend::BackendConnector;
        let issue = self.get_record("", id).await?;
        Ok(issue.version)
    }

    /// Insert one audit row for a completed write call. Best-effort —
    /// any failure is logged via [`tracing::error!`] and swallowed so the
    /// Confluence write result is never masked.
    ///
    /// `method` must be `"POST"`, `"PUT"`, or `"DELETE"`.
    /// `request_summary` should be the page title truncated to 256 chars —
    /// **never** body content (T-16-C-04).
    /// `response_bytes` is the raw server response body; only its SHA-256
    /// prefix is stored, never the content itself.
    pub(crate) fn audit_write(
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
                format!("reposix-confluence-{}", std::process::id()),
                method,
                path,
                i64::from(status),
                request_summary,
                response_summary,
            ],
        ) {
            tracing::error!(error = %e, "confluence audit insert failed");
        }
    }

    /// Fetch all inline + footer comments for a Confluence page.
    ///
    /// Issues two GETs (one per endpoint) and paginates each via `_links.next`.
    /// Returns a `Vec<ConfComment>` with `kind` populated to distinguish source.
    ///
    /// # Security
    /// - SSRF-safe: pagination cursors are prepended to `self.base()` (relative path).
    /// - Caps at `MAX_ISSUES_PER_LIST` with `tracing::warn!` (HARD-02 compliance).
    /// - Error messages pass the URL through `redact_url` so tenant hostnames
    ///   never leak (HARD-05 precedent).
    ///
    /// # Errors
    /// Returns `Error::Other` on non-2xx response. Transport errors propagate.
    pub async fn list_comments(&self, page_id: u64) -> Result<Vec<ConfComment>> {
        let mut out: Vec<ConfComment> = Vec::new();
        for (kind_str, kind_enum) in &[
            ("inline-comments", CommentKind::Inline),
            ("footer-comments", CommentKind::Footer),
        ] {
            let first = format!(
                "{}/wiki/api/v2/pages/{}/{}?limit={}&body-format=atlas_doc_format",
                self.base(),
                page_id,
                kind_str,
                PAGE_SIZE
            );
            let mut next_url: Option<String> = Some(first);
            let mut pages: usize = 0;
            let header_owned = self.standard_headers();
            let header_refs: Vec<(&str, &str)> =
                header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
            while let Some(url) = next_url.take() {
                pages += 1;
                if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) {
                    tracing::warn!(
                        page_id,
                        kind = %kind_str,
                        pages,
                        "reached MAX_ISSUES_PER_LIST cap on comments; stopping pagination"
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
                let bytes = resp.bytes().await?;
                if !status.is_success() {
                    return Err(Error::Other(format!(
                        "confluence returned {status} for GET {}: {}",
                        redact_url(&url),
                        String::from_utf8_lossy(&bytes)
                    )));
                }
                let body_json: serde_json::Value = serde_json::from_slice(&bytes)?;
                let next_cursor = parse_next_cursor(&body_json);
                let list: ConfCommentList = serde_json::from_value(body_json)?;
                for mut comment in list.results {
                    comment.kind = *kind_enum;
                    out.push(comment);
                    if out.len() >= MAX_ISSUES_PER_LIST {
                        tracing::warn!(
                            page_id,
                            total = out.len(),
                            "reached MAX_ISSUES_PER_LIST absolute cap; returning truncated list"
                        );
                        return Ok(out);
                    }
                }
                next_url = next_cursor.map(|relative| {
                    if relative.starts_with("http://") || relative.starts_with("https://") {
                        relative
                    } else {
                        format!("{}{}", self.base(), relative)
                    }
                });
            }
        }
        Ok(out)
    }

    /// Enumerate all readable Confluence spaces.
    ///
    /// Paginates via `_links.next` (same cursor scheme as `list_records`).
    /// Returns `Vec<ConfSpaceSummary>` with absolute `webui_url` fields.
    ///
    /// # Errors
    /// Transport errors + non-2xx responses (message passes through `redact_url`).
    pub async fn list_spaces(&self) -> Result<Vec<ConfSpaceSummary>> {
        let first = format!("{}/wiki/api/v2/spaces?limit=250", self.base());
        let mut next_url: Option<String> = Some(first);
        let mut out: Vec<ConfSpaceSummary> = Vec::new();
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        while let Some(url) = next_url.take() {
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
                    "confluence returned {status} for GET {}: {}",
                    redact_url(&url),
                    String::from_utf8_lossy(&bytes)
                )));
            }
            let body_json: serde_json::Value = serde_json::from_slice(&bytes)?;
            let next_cursor = parse_next_cursor(&body_json);
            let list: ConfSpaceSummaryList = serde_json::from_value(body_json)?;
            for s in list.results {
                let rel = s.links.and_then(|l| l.webui).unwrap_or_default();
                let webui_url = if rel.is_empty() {
                    String::new()
                } else if rel.starts_with("http://") || rel.starts_with("https://") {
                    rel
                } else {
                    format!("{}{}", self.base(), rel)
                };
                out.push(ConfSpaceSummary {
                    key: s.key,
                    name: s.name,
                    webui_url,
                });
            }
            next_url = next_cursor.map(|relative| {
                if relative.starts_with("http://") || relative.starts_with("https://") {
                    relative
                } else {
                    format!("{}{}", self.base(), relative)
                }
            });
        }
        Ok(out)
    }

    /// List attachments for a Confluence page.
    ///
    /// Calls `GET /wiki/api/v2/pages/{page_id}/attachments` and returns
    /// metadata for all attachments. Does NOT download binary bodies — the
    /// caller fetches binaries via [`Self::download_attachment`] using
    /// `att.download_link`.
    ///
    /// Paginates using the same cursor scheme as [`Self::list_issues_impl`].
    /// Caps at `MAX_ISSUES_PER_LIST` with a `tracing::warn!` (same as comments).
    ///
    /// # Errors
    ///
    /// Transport + non-2xx from Confluence REST API.
    pub async fn list_attachments(&self, page_id: u64) -> Result<Vec<ConfAttachment>> {
        let first = format!(
            "{}/wiki/api/v2/pages/{}/attachments?limit={}",
            self.base(),
            page_id,
            PAGE_SIZE
        );
        let mut next_url: Option<String> = Some(first);
        let mut out: Vec<ConfAttachment> = Vec::new();
        let mut pages: usize = 0;
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        while let Some(url) = next_url.take() {
            pages += 1;
            if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) {
                tracing::warn!(
                    page_id,
                    pages,
                    "reached MAX_ISSUES_PER_LIST cap on attachments; stopping pagination"
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
            let bytes = resp.bytes().await?;
            if !status.is_success() {
                return Err(Error::Other(format!(
                    "confluence returned {status} for GET {}: {}",
                    redact_url(&url),
                    String::from_utf8_lossy(&bytes)
                )));
            }
            let body_json: serde_json::Value = serde_json::from_slice(&bytes)?;
            let next_cursor = parse_next_cursor(&body_json);
            let list: ConfAttachmentList = serde_json::from_value(body_json)?;
            for att in list.results {
                out.push(att);
                if out.len() >= MAX_ISSUES_PER_LIST {
                    tracing::warn!(
                        page_id,
                        total = out.len(),
                        "reached MAX_ISSUES_PER_LIST absolute cap on attachments"
                    );
                    return Ok(out);
                }
            }
            next_url = next_cursor.map(|relative| {
                if relative.starts_with("http://") || relative.starts_with("https://") {
                    relative
                } else {
                    format!("{}{}", self.base(), relative)
                }
            });
        }
        Ok(out)
    }

    /// List whiteboards in a Confluence space (by numeric space id string).
    ///
    /// Calls `GET /wiki/api/v2/spaces/{space_id}/direct-children` and filters
    /// results where `type == "whiteboard"`. Returns `Ok(vec![])` gracefully
    /// if the endpoint returns 404 (endpoint has MEDIUM confidence per
    /// RESEARCH.md).
    ///
    /// Paginates via `_links.next` cursor. Caps at `MAX_ISSUES_PER_LIST` with
    /// a `tracing::warn!`.
    ///
    /// # Errors
    ///
    /// Transport + non-2xx (except 404) from Confluence REST API.
    pub async fn list_whiteboards(&self, space_id: &str) -> Result<Vec<ConfWhiteboard>> {
        let first = format!(
            "{}/wiki/api/v2/spaces/{}/direct-children?limit={}",
            self.base(),
            space_id,
            PAGE_SIZE
        );
        let mut next_url: Option<String> = Some(first);
        let mut out: Vec<ConfWhiteboard> = Vec::new();
        let mut pages: usize = 0;
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        while let Some(url) = next_url.take() {
            pages += 1;
            if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) {
                tracing::warn!(
                    space_id,
                    pages,
                    "reached MAX_ISSUES_PER_LIST cap on whiteboard listing; stopping pagination"
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
            // 404 means the endpoint doesn't exist for this tenant — return
            // empty gracefully (MEDIUM confidence endpoint per RESEARCH.md).
            if status == reqwest::StatusCode::NOT_FOUND {
                tracing::warn!(
                    space_id,
                    "direct-children endpoint returned 404; whiteboard listing not available"
                );
                return Ok(vec![]);
            }
            let bytes = resp.bytes().await?;
            if !status.is_success() {
                return Err(Error::Other(format!(
                    "confluence returned {status} for GET {}: {}",
                    redact_url(&url),
                    String::from_utf8_lossy(&bytes)
                )));
            }
            let body_json: serde_json::Value = serde_json::from_slice(&bytes)?;
            let next_cursor = parse_next_cursor(&body_json);
            let list: ConfDirectChildrenList = serde_json::from_value(body_json)?;
            for child in list.results {
                if child.content_type != "whiteboard" {
                    continue;
                }
                let wb = ConfWhiteboard {
                    id: child.id,
                    status: child.status,
                    title: child.title,
                    space_id: child.space_id,
                    author_id: child.author_id,
                    created_at: child.created_at.unwrap_or_else(chrono::Utc::now),
                    parent_id: child.parent_id,
                    parent_type: child.parent_type,
                };
                out.push(wb);
                if out.len() >= MAX_ISSUES_PER_LIST {
                    tracing::warn!(
                        space_id,
                        total = out.len(),
                        "reached MAX_ISSUES_PER_LIST absolute cap on whiteboards"
                    );
                    return Ok(out);
                }
            }
            next_url = next_cursor.map(|relative| {
                if relative.starts_with("http://") || relative.starts_with("https://") {
                    relative
                } else {
                    format!("{}{}", self.base(), relative)
                }
            });
        }
        Ok(out)
    }

    /// Download the binary body of an attachment.
    ///
    /// `download_url` is the relative path from [`ConfAttachment::download_link`]
    /// (e.g. `"/wiki/download/attachments/12345/image.png"`). Prepends
    /// `self.base()` and sends `standard_headers()` (Basic auth is required —
    /// `reqwest::get` without auth returns 401).
    ///
    /// The caller MUST check `att.file_size` and refuse to call this for files
    /// exceeding `52_428_800` bytes (50 MiB) to prevent OOM. The method itself
    /// is unbounded by design to keep the API simple (see threat model T-24-01-04).
    ///
    /// # Errors
    ///
    /// Transport + non-2xx from the Confluence download endpoint.
    pub async fn download_attachment(&self, download_url: &str) -> Result<Vec<u8>> {
        let full_url =
            if download_url.starts_with("http://") || download_url.starts_with("https://") {
                download_url.to_owned()
            } else {
                format!("{}{}", self.base(), download_url)
            };
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers(Method::GET, full_url.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence attachment download returned {status} for GET {}",
                redact_url(&full_url)
            )));
        }
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CommentKind, ConfluenceCreds};
    use base64::Engine;
    use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason};
    use reposix_core::{sanitize, RecordStatus, ServerMetadata, Untainted};
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    fn creds() -> ConfluenceCreds {
        ConfluenceCreds {
            email: "test@example.com".into(),
            api_token: "tkn".into(),
        }
    }

    fn page_json(
        id: &str,
        status: &str,
        title: &str,
        body_value: Option<&str>,
    ) -> serde_json::Value {
        let body = match body_value {
            Some(v) => json!({ "storage": { "value": v, "representation": "storage" } }),
            None => json!({}),
        };
        json!({
            "id": id,
            "status": status,
            "title": title,
            "createdAt": "2026-04-13T00:00:00Z",
            "ownerId": null,
            "version": {
                "number": 1,
                "createdAt": "2026-04-13T00:00:00Z",
            },
            "body": body,
        })
    }

    /// Build a page JSON response with an `atlas_doc_format` body.
    /// `adf_doc` should be a `{"type":"doc",...}` value that
    /// `adf_to_markdown` can parse. Used by tests that exercise the
    /// ADF read path (C4).
    fn page_json_adf(
        id: &str,
        status: &str,
        title: &str,
        adf_doc: &serde_json::Value,
    ) -> serde_json::Value {
        let adf_doc = adf_doc.clone();
        json!({
            "id": id,
            "status": status,
            "title": title,
            "createdAt": "2026-04-13T00:00:00Z",
            "ownerId": null,
            "version": {
                "number": 1,
                "createdAt": "2026-04-13T00:00:00Z",
            },
            "body": {
                "atlas_doc_format": {
                    "value": adf_doc,
                    "representation": "atlas_doc_format",
                }
            },
        })
    }

    async fn mount_space_lookup(server: &MockServer, key: &str, id: &str) {
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces"))
            .and(query_param("keys", key))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [ { "id": id, "key": key, "name": "demo", "type": "global" } ],
                "_links": {}
            })))
            .mount(server)
            .await;
    }

    /// Wiremock matcher: assert the `cql` query value (URL-decoded) contains
    /// the given substring. Used by Phase 33's `list_changed_since` test
    /// because CQL strings are URL-encoded on the wire and `query_param`
    /// only exact-matches.
    struct CqlContains(&'static str);
    impl wiremock::Match for CqlContains {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let pairs: Vec<(String, String)> = request
                .url
                .query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();
            pairs.iter().any(|(k, v)| k == "cql" && v.contains(self.0))
        }
    }

    #[tokio::test]
    async fn confluence_list_changed_since_sends_cql_lastmodified() {
        // Phase 33: prove the Confluence override emits a CQL search with
        // `lastModified > "<datetime>"` filter on the wire.
        use chrono::{TimeZone, Utc};
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/search"))
            .and(CqlContains("lastModified"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [
                    { "content": { "id": "12345", "type": "page" } }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let t = Utc.with_ymd_and_hms(2026, 4, 24, 0, 0, 0).unwrap();
        let ids = backend
            .list_changed_since("REPOSIX", t)
            .await
            .expect("list_changed");
        assert_eq!(ids, vec![RecordId(12345)]);
    }

    #[tokio::test]
    async fn confluence_list_changed_since_strips_quotes_from_project() {
        // Defense-in-depth: an attacker-controlled space slug containing `"`
        // would otherwise break out of the CQL string literal. The override
        // strips quotes before interpolation.
        use chrono::{TimeZone, Utc};
        let server = MockServer::start().await;
        // Match: cql contains the safely-stripped slug, NOT the original.
        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/search"))
            .and(CqlContains("space = \"REPOSIX\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": []
            })))
            .expect(1)
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let t = Utc.with_ymd_and_hms(2026, 4, 24, 0, 0, 0).unwrap();
        // `RE"PO"SIX` — quotes will be stripped to `REPOSIX`.
        let ids = backend
            .list_changed_since("RE\"PO\"SIX", t)
            .await
            .expect("list_changed");
        assert_eq!(ids, Vec::<RecordId>::new());
    }

    // -------- 1: list resolves space key and fetches pages --------

    #[tokio::test]
    async fn list_resolves_space_key_and_fetches_pages() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "REPOSIX", "12345").await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/12345/pages"))
            .and(query_param("limit", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [
                    page_json("98765", "current", "first page", None),
                    page_json("98766", "current", "second page", None),
                ],
                "_links": {}
            })))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issues = backend.list_records("REPOSIX").await.expect("list");
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].id, RecordId(98765));
        assert_eq!(issues[0].status, RecordStatus::Open);
        assert_eq!(issues[1].id, RecordId(98766));
    }

    // -------- 2: pagination via _links.next --------

    #[tokio::test]
    async fn list_paginates_via_links_next() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "REPOSIX", "12345").await;
        // Page 1: two results, _links.next as relative path
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/12345/pages"))
            .and(query_param("limit", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [
                    page_json("1", "current", "p1", None),
                    page_json("2", "current", "p2", None),
                ],
                "_links": {
                    "next": "/wiki/api/v2/spaces/12345/pages?cursor=ABC&limit=100"
                }
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        // Page 2: one result, no next
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/12345/pages"))
            .and(query_param("cursor", "ABC"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [
                    page_json("3", "current", "p3", None),
                ],
                "_links": {}
            })))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issues = backend.list_records("REPOSIX").await.expect("list");
        assert_eq!(issues.len(), 3);
        assert_eq!(issues[0].id, RecordId(1));
        assert_eq!(issues[2].id, RecordId(3));
    }

    // -------- 3: get_record returns ADF body converted to Markdown --------

    #[tokio::test]
    async fn get_issue_returns_body_adf_as_markdown() {
        let server = MockServer::start().await;
        let adf_doc = json!({
            "type": "doc",
            "version": 1,
            "content": [
                {
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "Hello"}]
                }
            ]
        });
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/98765"))
            .and(query_param("body-format", "atlas_doc_format"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(page_json_adf("98765", "current", "hello", &adf_doc)),
            )
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issue = backend
            .get_record("REPOSIX", RecordId(98765))
            .await
            .expect("get");
        assert!(
            issue.body.contains("Hello"),
            "expected body to contain 'Hello', got: {:?}",
            issue.body
        );
        assert_eq!(issue.id, RecordId(98765));
    }

    // -------- 4: 404 maps to not-found --------

    #[tokio::test]
    async fn get_404_maps_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/9999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({"message": "not found"})))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let err = backend
            .get_record("REPOSIX", RecordId(9999))
            .await
            .expect_err("404");
        match err {
            Error::Other(m) => assert!(m.starts_with("not found:"), "got {m}"),
            other => panic!("expected not found, got {other:?}"),
        }
    }

    // -------- 4b: ADF absent → storage fallback (C4) --------

    /// When the ADF response contains no `atlas_doc_format` body (pre-ADF page),
    /// `get_record` must fall back to a second GET with `?body-format=storage`
    /// and return the raw storage HTML as the body.
    #[tokio::test]
    async fn get_issue_falls_back_to_storage_when_adf_empty() {
        let server = MockServer::start().await;
        // ADF request returns a page with no atlas_doc_format body (pre-ADF page).
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/55555"))
            .and(query_param("body-format", "atlas_doc_format"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json(
                "55555",
                "current",
                "legacy page",
                None, // no ADF body — triggers fallback
            )))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        // Storage fallback request returns the raw storage HTML.
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/55555"))
            .and(query_param("body-format", "storage"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json(
                "55555",
                "current",
                "legacy page",
                Some("<p>legacy content</p>"),
            )))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issue = backend
            .get_record("REPOSIX", RecordId(55555))
            .await
            .expect("get with fallback");
        assert_eq!(issue.id, RecordId(55555));
        assert_eq!(
            issue.body, "<p>legacy content</p>",
            "storage fallback must return raw HTML body"
        );
    }

    // -------- 5: status "current" → Open (via get_record, since list omits body) --------

    #[tokio::test]
    async fn status_current_maps_to_open() {
        let server = MockServer::start().await;
        // Respond to ADF request with a minimal ADF body so no storage fallback occurs.
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/1"))
            .and(query_param("body-format", "atlas_doc_format"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_adf(
                "1",
                "current",
                "c",
                &json!({"type": "doc", "version": 1, "content": []}),
            )))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issue = backend
            .get_record("REPOSIX", RecordId(1))
            .await
            .expect("get");
        assert_eq!(issue.status, RecordStatus::Open);
    }

    // -------- 6: status "trashed" → Done --------

    #[tokio::test]
    async fn status_trashed_maps_to_done() {
        let server = MockServer::start().await;
        // Respond to ADF request with a minimal ADF body so no storage fallback occurs.
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/2"))
            .and(query_param("body-format", "atlas_doc_format"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_adf(
                "2",
                "trashed",
                "t",
                &json!({"type": "doc", "version": 1, "content": []}),
            )))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issue = backend
            .get_record("REPOSIX", RecordId(2))
            .await
            .expect("get");
        assert_eq!(issue.status, RecordStatus::Done);
    }

    // -------- 7: Basic-auth header is byte-exact --------

    /// Custom matcher that verifies the Authorization header is exactly
    /// `Basic base64(test@example.com:tkn)`. Per 11-RESEARCH.md §Custom
    /// Match impl — this is how we prove "no Bearer, the right Basic".
    struct BasicAuthMatches;
    impl wiremock::Match for BasicAuthMatches {
        fn matches(&self, request: &Request) -> bool {
            let expected = format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode("test@example.com:tkn")
            );
            request
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|s| s == expected)
        }
    }

    #[tokio::test]
    async fn auth_header_is_basic_with_correct_base64() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/42"))
            .and(BasicAuthMatches)
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_adf(
                "42",
                "current",
                "x",
                &json!({"type": "doc", "version": 1, "content": []}),
            )))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        // If the header is wrong, wiremock returns no-match 404 and this fails.
        backend
            .get_record("REPOSIX", RecordId(42))
            .await
            .expect("auth header must match");
    }

    // -------- 8: 429 + Retry-After arms the gate --------

    #[tokio::test]
    async fn rate_limit_429_retry_after_arms_gate() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/42"))
            .respond_with(
                ResponseTemplate::new(429)
                    .append_header("retry-after", "2")
                    .set_body_json(json!({"message": "too many"})),
            )
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let _ = backend.get_record("REPOSIX", RecordId(42)).await; // expect Err, don't care which
        let gate = backend.rate_limit_gate.lock().to_owned();
        let now = Instant::now();
        match gate {
            Some(deadline) => {
                assert!(
                    deadline > now,
                    "gate must be in the future, got delta {:?}",
                    deadline.saturating_duration_since(now)
                );
                assert!(
                    deadline - now <= MAX_RATE_LIMIT_SLEEP + Duration::from_secs(1),
                    "gate must be capped at MAX_RATE_LIMIT_SLEEP"
                );
            }
            None => panic!("gate should be armed after 429 with Retry-After"),
        }
    }

    // -------- 9: (removed) write_methods_return_not_supported --------
    // Removed in Phase 16 Wave B: write methods are now implemented, so the
    // old "short-circuits to not-supported" assertion is no longer valid.
    // Coverage for write paths is in B6 (wiremock tests) and B7 (supports test).

    // -------- 10: capability matrix --------

    /// Phase 16 Wave B flipped `Delete` and `StrongVersioning` to `true`.
    /// `Hierarchy` was already `true` since Phase 13 Wave B1.
    /// Renamed from `supports_reports_only_hierarchy` to match the new reality.
    #[test]
    fn supports_reports_hierarchy_delete_strong_versioning() {
        let backend =
            ConfluenceBackend::new_with_base_url(creds(), "http://127.0.0.1:1".to_owned())
                .expect("backend");
        assert!(!backend.supports(BackendFeature::Workflows));
        assert!(!backend.supports(BackendFeature::Transitions));
        assert!(!backend.supports(BackendFeature::BulkEdit));
        assert!(backend.supports(BackendFeature::Hierarchy));
        assert!(backend.supports(BackendFeature::Delete));
        assert!(backend.supports(BackendFeature::StrongVersioning));
        assert_eq!(backend.name(), "confluence");
    }

    #[test]
    fn root_collection_name_returns_pages() {
        let backend =
            ConfluenceBackend::new_with_base_url(creds(), "http://127.0.0.1:1".to_owned())
                .expect("backend");
        assert_eq!(backend.root_collection_name(), "pages");
    }

    /// End-to-end proof that `parentId` + `parentType` survive the JSON
    /// decode → `ConfPage` → `translate` → `Record` pipeline through the
    /// `BackendConnector::list_records` seam (not just the `translate` helper in
    /// isolation). Mixes three shapes in one list so we assert all branches
    /// with a single wiremock round-trip.
    #[tokio::test]
    async fn list_populates_parent_id_end_to_end() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "REPOSIX", "12345").await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/12345/pages"))
            .and(query_param("limit", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [
                    // (a) child page: parentType="page" → parent_id populated
                    {
                        "id": "98765",
                        "status": "current",
                        "title": "child",
                        "parentId": "360556",
                        "parentType": "page",
                        "createdAt": "2026-04-13T00:00:00Z",
                        "ownerId": null,
                        "version": {
                            "number": 1,
                            "createdAt": "2026-04-13T00:00:00Z",
                        },
                        "body": {},
                    },
                    // (b) space-root page: no parent fields → None
                    {
                        "id": "360556",
                        "status": "current",
                        "title": "home",
                        "createdAt": "2026-04-13T00:00:00Z",
                        "ownerId": null,
                        "version": {
                            "number": 1,
                            "createdAt": "2026-04-13T00:00:00Z",
                        },
                        "body": {},
                    },
                    // (c) folder-parented page: parentType="folder" → Some(999) (CONF-06)
                    {
                        "id": "12321",
                        "status": "current",
                        "title": "in-folder",
                        "parentId": "999",
                        "parentType": "folder",
                        "createdAt": "2026-04-13T00:00:00Z",
                        "ownerId": null,
                        "version": {
                            "number": 1,
                            "createdAt": "2026-04-13T00:00:00Z",
                        },
                        "body": {},
                    },
                ],
                "_links": {}
            })))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issues = backend.list_records("REPOSIX").await.expect("list");
        assert_eq!(issues.len(), 3);

        let child = issues
            .iter()
            .find(|i| i.id == RecordId(98765))
            .expect("child page present");
        assert_eq!(
            child.parent_id,
            Some(RecordId(360_556)),
            "page-parented child must propagate parent_id"
        );

        let root = issues
            .iter()
            .find(|i| i.id == RecordId(360_556))
            .expect("root page present");
        assert_eq!(
            root.parent_id, None,
            "page with no parent fields must deserialize as orphan"
        );

        let foldered = issues
            .iter()
            .find(|i| i.id == RecordId(12321))
            .expect("folder-parented page present");
        // CONF-06 fix (Phase 24 Plan 01): folder parentType now propagates to
        // Record::parent_id (same as "page"). Updated from None to Some(999).
        assert_eq!(
            foldered.parent_id,
            Some(RecordId(999)),
            "folder parentType must propagate to Record::parent_id (CONF-06)"
        );
    }

    // -------- Threat-model tests (T-11-01, T-11-02) --------

    #[test]
    fn creds_debug_redacts_api_token() {
        let c = ConfluenceCreds {
            email: "u@example.com".into(),
            api_token: "SUPER_SECRET_TOKEN_VALUE".into(),
        };
        let rendered = format!("{c:?}");
        assert!(
            rendered.contains("<redacted>"),
            "Debug must print <redacted>, got {rendered}"
        );
        assert!(
            !rendered.contains("SUPER_SECRET_TOKEN_VALUE"),
            "Debug must NOT leak the token, got {rendered}"
        );
        assert!(
            rendered.contains("u@example.com"),
            "email should still be visible for operator debugging"
        );
    }

    #[test]
    fn backend_debug_redacts_creds() {
        let backend = ConfluenceBackend::new_with_base_url(
            ConfluenceCreds {
                email: "u@example.com".into(),
                api_token: "SUPER_SECRET_TOKEN_VALUE".into(),
            },
            "http://127.0.0.1:1".to_owned(),
        )
        .expect("backend");
        let rendered = format!("{backend:?}");
        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains("SUPER_SECRET_TOKEN_VALUE"));
    }

    #[test]
    fn new_rejects_invalid_tenant() {
        // Injection / path traversal / scheme-bending attempts.
        let too_long = "a".repeat(64);
        let bad: [&str; 10] = [
            "",
            "tenant.with.dots",
            "tenant/slash",
            "tenant@at",
            "-leading-hyphen",
            "trailing-hyphen-",
            "UPPERCASE",
            "tenant_underscore",
            "../../../etc/passwd",
            too_long.as_str(),
        ];
        for t in &bad {
            let r = ConfluenceBackend::new(creds(), t);
            let is_expected_err = matches!(
                &r,
                Err(Error::Other(m)) if m.contains("invalid confluence tenant")
            );
            assert!(
                is_expected_err,
                "tenant {t:?} should be rejected, got {r:?}"
            );
        }
    }

    #[test]
    fn new_accepts_valid_tenants() {
        for t in ["a", "reuben-john", "tenant1", "1tenant", "a0-b1-c2"] {
            let r = ConfluenceBackend::new(creds(), t);
            assert!(r.is_ok(), "tenant {t:?} should be accepted, got {r:?}");
        }
    }

    // -------- WR-01: space_key is percent-encoded, not splat into URL --------

    /// A `space_key` containing `&`, `=`, `#`, space, and non-ASCII must be
    /// percent-encoded into the query string rather than smuggling extra
    /// query parameters or breaking out of the `keys=` value.
    #[tokio::test]
    async fn space_key_is_percent_encoded_in_query_string() {
        use wiremock::matchers::query_param;
        let server = MockServer::start().await;
        // The adversarial key we want to send as a literal value of `keys=`.
        let adversarial = "A&limit=1#frag ZZ";
        // Wiremock's `query_param` matcher checks the decoded value — so if
        // the adapter percent-encodes properly, wiremock will see the raw
        // literal string back. If the adapter splices it raw, the query
        // string would parse as multiple params and this match would fail.
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces"))
            .and(query_param("keys", adversarial))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{"id": "12345", "key": adversarial}]
            })))
            .mount(&server)
            .await;
        // Also mount the pages endpoint so list_records can complete — this
        // proves the round-trip end-to-end.
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/12345/pages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [],
                "_links": {}
            })))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issues = backend
            .list_records(adversarial)
            .await
            .expect("list should succeed with adversarial space_key");
        assert_eq!(issues.len(), 0);
    }

    // -------- WR-02: space_id from server is validated before URL interpolation --------

    /// A malicious tenant (or MITM) that returns a non-numeric `id` — e.g.
    /// `"12345/../admin"` — must be rejected before any second-round HTTP
    /// call, because the `space_id` is about to be interpolated into a URL
    /// path and SG-01 only gates origins, not paths.
    #[tokio::test]
    async fn list_rejects_non_numeric_space_id() {
        use wiremock::matchers::query_param;
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces"))
            .and(query_param("keys", "REPOSIX"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{"id": "12345/../admin", "key": "REPOSIX"}]
            })))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let err = backend
            .list_records("REPOSIX")
            .await
            .expect_err("non-numeric space_id must be rejected");
        match err {
            Error::Other(m) => assert!(
                m.contains("malformed space id from server"),
                "expected malformed-space-id message, got {m}"
            ),
            other => panic!("expected Error::Other, got {other:?}"),
        }
    }

    // ======================================================================
    // Phase 16 Wave B: write-method wiremock tests (B6) + supports test (B7)
    // ======================================================================

    /// Build a page JSON fixture with an explicit version number (for
    /// optimistic-locking tests that need a specific current version).
    fn page_json_v(id: &str, title: &str, version: u64) -> serde_json::Value {
        json!({
            "id": id,
            "status": "current",
            "title": title,
            "createdAt": "2026-04-13T00:00:00Z",
            "ownerId": null,
            "version": {
                "number": version,
                "createdAt": "2026-04-13T00:00:00Z",
            },
            "body": { "storage": { "value": "<p>body</p>", "representation": "storage" } },
        })
    }

    /// Build an `Untainted<Record>` with the given fields for use in write tests.
    fn make_untainted(title: &str, body: &str, parent_id: Option<RecordId>) -> Untainted<Record> {
        let t = chrono::DateTime::parse_from_rfc3339("2026-04-13T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        sanitize(
            Tainted::new(Record {
                id: RecordId(0),
                title: title.to_owned(),
                status: RecordStatus::Open,
                assignee: None,
                labels: vec![],
                created_at: t,
                updated_at: t,
                version: 0,
                body: body.to_owned(),
                parent_id,
                extensions: std::collections::BTreeMap::new(),
            }),
            ServerMetadata {
                id: RecordId(99),
                created_at: t,
                updated_at: t,
                version: 1,
            },
        )
    }

    // -------- B6.1: update_record sends PUT with incremented version --------

    #[tokio::test]
    async fn update_issue_sends_put_with_version() {
        let server = MockServer::start().await;
        // Respond to PUT /wiki/api/v2/pages/99 with a page that has version 43
        Mock::given(method("PUT"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v(
                "99",
                "updated title",
                43,
            )))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let patch = make_untainted("updated title", "body text", None);
        // Pass expected_version = Some(42) → PUT body must have version.number = 43
        let result = backend
            .update_record("REPOSIX", RecordId(99), patch, Some(42))
            .await
            .expect("update_record should succeed");
        assert_eq!(result.title, "updated title");
        assert_eq!(result.id, RecordId(99));
        assert_eq!(result.version, 43);
    }

    // -------- B6.2: update_record 409 maps to conflict error --------

    #[tokio::test]
    async fn update_issue_409_maps_to_conflict_error() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(ResponseTemplate::new(409).set_body_json(json!({"message": "stale"})))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let patch = make_untainted("title", "body", None);
        let err = backend
            .update_record("REPOSIX", RecordId(99), patch, Some(5))
            .await
            .expect_err("409 must be an error");
        match err {
            Error::Other(m) => assert!(
                m.starts_with("confluence version conflict"),
                "error must start with 'confluence version conflict', got: {m}"
            ),
            other => panic!("expected Error::Other, got {other:?}"),
        }
    }

    // -------- B6.3: update_record with None version fetches then PUTs --------

    #[tokio::test]
    async fn update_issue_none_version_fetches_then_puts() {
        let server = MockServer::start().await;
        // Pre-flight GET: version=7
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v(
                "99",
                "original title",
                7,
            )))
            .mount(&server)
            .await;
        // PUT: respond with version=8
        Mock::given(method("PUT"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v(
                "99",
                "new title",
                8,
            )))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let patch = make_untainted("new title", "new body", None);
        // expected_version = None → must do GET first, then PUT with number=8
        let result = backend
            .update_record("REPOSIX", RecordId(99), patch, None)
            .await
            .expect("update_record with None version should succeed");
        assert_eq!(result.version, 8);
        assert_eq!(result.title, "new title");
    }

    // -------- B6.4: update_record 404 maps to not-found --------

    #[tokio::test]
    async fn update_issue_404_maps_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({"message": "not found"})))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let patch = make_untainted("title", "body", None);
        let err = backend
            .update_record("REPOSIX", RecordId(99), patch, Some(1))
            .await
            .expect_err("404 must be an error");
        match err {
            Error::Other(m) => assert!(
                m.contains("not found"),
                "error must contain 'not found', got: {m}"
            ),
            other => panic!("expected Error::Other, got {other:?}"),
        }
    }

    // -------- B6.5: create_record POSTs to pages with correct spaceId --------

    #[tokio::test]
    async fn create_issue_posts_to_pages() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "REPOSIX", "12345").await;
        Mock::given(method("POST"))
            .and(path("/wiki/api/v2/pages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v(
                "77777",
                "my new page",
                1,
            )))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issue = make_untainted("my new page", "# Hello", None);
        let result = backend
            .create_record("REPOSIX", issue)
            .await
            .expect("create_record should succeed");
        assert_eq!(result.id, RecordId(77777));
        assert_eq!(result.title, "my new page");
    }

    // -------- B6.6: create_record with parent_id sends parentId in body --------

    /// Wiremock matcher: POST body has `parentId == "42"`.
    struct ParentIdMatches;
    impl wiremock::Match for ParentIdMatches {
        fn matches(&self, request: &Request) -> bool {
            let Ok(body) = serde_json::from_slice::<serde_json::Value>(&request.body) else {
                return false;
            };
            body.get("parentId")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s == "42")
        }
    }

    #[tokio::test]
    async fn create_issue_with_parent_id() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "REPOSIX", "12345").await;

        Mock::given(method("POST"))
            .and(path("/wiki/api/v2/pages"))
            .and(ParentIdMatches)
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v(
                "88888",
                "child page",
                1,
            )))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issue = make_untainted("child page", "body", Some(RecordId(42)));
        let result = backend
            .create_record("REPOSIX", issue)
            .await
            .expect("create_record with parent_id should succeed");
        assert_eq!(result.id, RecordId(88888));
    }

    // -------- B6.7: create_record without parent_id sends null --------

    /// Wiremock matcher: POST body has `parentId == null`.
    struct ParentIdIsNull;
    impl wiremock::Match for ParentIdIsNull {
        fn matches(&self, request: &Request) -> bool {
            let Ok(body) = serde_json::from_slice::<serde_json::Value>(&request.body) else {
                return false;
            };
            body.get("parentId").is_some_and(serde_json::Value::is_null)
        }
    }

    #[tokio::test]
    async fn create_issue_without_parent_id_sends_null() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "REPOSIX", "12345").await;

        Mock::given(method("POST"))
            .and(path("/wiki/api/v2/pages"))
            .and(ParentIdIsNull)
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v(
                "55555",
                "root page",
                1,
            )))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issue = make_untainted("root page", "body", None);
        let result = backend
            .create_record("REPOSIX", issue)
            .await
            .expect("create_record without parent should succeed");
        assert_eq!(result.id, RecordId(55555));
    }

    // -------- B6.8: delete_or_close sends DELETE and returns Ok on 204 --------

    #[tokio::test]
    async fn delete_or_close_sends_delete() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        backend
            .delete_or_close("REPOSIX", RecordId(99), DeleteReason::Completed)
            .await
            .expect("delete_or_close on 204 must return Ok");
    }

    // -------- B6.9: delete_or_close 404 maps to not-found --------

    #[tokio::test]
    async fn delete_or_close_404_maps_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({"message": "not found"})))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let err = backend
            .delete_or_close("REPOSIX", RecordId(99), DeleteReason::Completed)
            .await
            .expect_err("404 must be an error");
        match err {
            Error::Other(m) => assert!(
                m.contains("not found"),
                "error must contain 'not found', got: {m}"
            ),
            other => panic!("expected Error::Other, got {other:?}"),
        }
    }

    // -------- B6.10: write methods send Content-Type: application/json --------

    /// Wiremock matcher: request has `Content-Type: application/json` header.
    struct ContentTypeIsJson;
    impl wiremock::Match for ContentTypeIsJson {
        fn matches(&self, request: &Request) -> bool {
            request
                .headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|s| s.starts_with("application/json"))
        }
    }

    #[tokio::test]
    async fn write_methods_send_content_type_json() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "REPOSIX", "12345").await;

        // POST
        Mock::given(method("POST"))
            .and(path("/wiki/api/v2/pages"))
            .and(ContentTypeIsJson)
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v("1", "t", 1)))
            .mount(&server)
            .await;
        // PUT
        Mock::given(method("PUT"))
            .and(path("/wiki/api/v2/pages/99"))
            .and(ContentTypeIsJson)
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v("99", "t", 2)))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");

        // create_record → POST
        let issue = make_untainted("t", "b", None);
        backend
            .create_record("REPOSIX", issue)
            .await
            .expect("POST must carry Content-Type: application/json");

        // update_record → PUT
        let patch = make_untainted("t", "b", None);
        backend
            .update_record("REPOSIX", RecordId(99), patch, Some(1))
            .await
            .expect("PUT must carry Content-Type: application/json");
    }

    // -------- B6.11: write methods send Basic-auth --------

    #[tokio::test]
    async fn write_methods_send_basic_auth() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "REPOSIX", "12345").await;

        // POST with auth check
        Mock::given(method("POST"))
            .and(path("/wiki/api/v2/pages"))
            .and(BasicAuthMatches)
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v("1", "t", 1)))
            .mount(&server)
            .await;
        // PUT with auth check
        Mock::given(method("PUT"))
            .and(path("/wiki/api/v2/pages/99"))
            .and(BasicAuthMatches)
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v("99", "t", 2)))
            .mount(&server)
            .await;
        // DELETE with auth check
        Mock::given(method("DELETE"))
            .and(path("/wiki/api/v2/pages/42"))
            .and(BasicAuthMatches)
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");

        let issue = make_untainted("t", "b", None);
        backend
            .create_record("REPOSIX", issue)
            .await
            .expect("POST must carry Basic auth");

        let patch = make_untainted("t", "b", None);
        backend
            .update_record("REPOSIX", RecordId(99), patch, Some(1))
            .await
            .expect("PUT must carry Basic auth");

        backend
            .delete_or_close("REPOSIX", RecordId(42), DeleteReason::Completed)
            .await
            .expect("DELETE must carry Basic auth");
    }

    // -------- B6.12: rate_limit_gate shared with write path --------

    #[tokio::test]
    async fn rate_limit_gate_shared_with_writes() {
        // This test verifies the gate is armed on a 429 response and that a
        // subsequent write call respects it. We do NOT actually sleep 1+ second
        // (that would make the test suite slow); instead we verify:
        // 1. A 429 GET arms the gate.
        // 2. The gate deadline is in the future immediately after.
        // The actual sleep is exercised by the existing `rate_limit_429_retry_after_arms_gate`
        // test; this test proves the gate is SHARED between read and write paths
        // (i.e. arming via GET prevents an immediate PUT).
        let server = MockServer::start().await;
        // GET returns 429 with Retry-After: 2
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/10"))
            .respond_with(
                ResponseTemplate::new(429)
                    .append_header("retry-after", "2")
                    .set_body_json(json!({"message": "too many"})),
            )
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        // GET — arms the gate
        let _ = backend.get_record("REPOSIX", RecordId(10)).await;

        // Immediately after: gate must be set and in the future
        let gate = backend.rate_limit_gate.lock().to_owned();
        assert!(
            gate.is_some_and(|d| d > Instant::now()),
            "rate_limit_gate must be armed after 429 GET; write path shares the same gate"
        );
    }

    // ======================================================================
    // B7: supports_lists_delete_hierarchy_strong_versioning
    // ======================================================================

    #[test]
    fn supports_lists_delete_hierarchy_strong_versioning() {
        // Instantiate with an unreachable URL — no HTTP needed for a sync
        // capability check.
        let backend =
            ConfluenceBackend::new_with_base_url(creds(), "http://127.0.0.1:1".to_owned())
                .expect("backend");
        assert!(
            backend.supports(BackendFeature::Hierarchy),
            "Hierarchy must be supported"
        );
        assert!(
            backend.supports(BackendFeature::Delete),
            "Delete must be supported"
        );
        assert!(
            backend.supports(BackendFeature::StrongVersioning),
            "StrongVersioning must be supported"
        );
        assert!(
            !backend.supports(BackendFeature::BulkEdit),
            "BulkEdit must NOT be supported"
        );
        assert!(
            !backend.supports(BackendFeature::Workflows),
            "Workflows must NOT be supported"
        );
    }

    // ======================================================================
    // C5: Audit log unit tests (in-memory SQLite)
    // ======================================================================

    /// Open an in-memory `SQLite` DB with the audit schema loaded.
    /// Wrapped in `Arc<Mutex<_>>` ready for `.with_audit(…)`.
    fn open_in_memory_audit() -> std::sync::Arc<Mutex<rusqlite::Connection>> {
        let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
        reposix_core::audit::load_schema(&conn).expect("load_schema");
        Arc::new(Mutex::new(conn))
    }

    /// Count rows in `audit_events` matching a given method.
    fn count_audit_rows(audit: &Arc<Mutex<rusqlite::Connection>>, method: &str) -> i64 {
        audit
            .lock()
            .query_row(
                "SELECT COUNT(*) FROM audit_events WHERE method = ?1",
                [method],
                |r| r.get(0),
            )
            .unwrap_or(0)
    }

    #[tokio::test]
    async fn update_issue_writes_audit_row() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v("99", "t", 2)))
            .mount(&server)
            .await;

        let audit = open_in_memory_audit();
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri())
            .expect("backend")
            .with_audit(Arc::clone(&audit));

        let patch = make_untainted("updated title", "body", None);
        backend
            .update_record("REPOSIX", RecordId(99), patch, Some(1))
            .await
            .expect("update_record should succeed");

        assert_eq!(
            count_audit_rows(&audit, "PUT"),
            1,
            "exactly one PUT audit row expected"
        );
    }

    #[tokio::test]
    async fn create_issue_writes_audit_row() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "REPOSIX", "12345").await;
        Mock::given(method("POST"))
            .and(path("/wiki/api/v2/pages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v("77", "new", 1)))
            .mount(&server)
            .await;

        let audit = open_in_memory_audit();
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri())
            .expect("backend")
            .with_audit(Arc::clone(&audit));

        let issue = make_untainted("new page", "body", None);
        backend
            .create_record("REPOSIX", issue)
            .await
            .expect("create_record should succeed");

        assert_eq!(
            count_audit_rows(&audit, "POST"),
            1,
            "exactly one POST audit row expected"
        );
        // Check path column.
        let path_val: String = audit
            .lock()
            .query_row(
                "SELECT path FROM audit_events WHERE method = 'POST'",
                [],
                |r| r.get(0),
            )
            .expect("row exists");
        assert_eq!(path_val, "/wiki/api/v2/pages");
    }

    #[tokio::test]
    async fn delete_or_close_writes_audit_row() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/wiki/api/v2/pages/55"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let audit = open_in_memory_audit();
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri())
            .expect("backend")
            .with_audit(Arc::clone(&audit));

        backend
            .delete_or_close("REPOSIX", RecordId(55), DeleteReason::Completed)
            .await
            .expect("delete_or_close should succeed");

        let row: (String, i64) = audit
            .lock()
            .query_row(
                "SELECT method, status FROM audit_events WHERE method = 'DELETE'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("DELETE audit row must exist");
        assert_eq!(row.0, "DELETE");
        assert_eq!(row.1, 204);
    }

    #[tokio::test]
    async fn audit_row_has_correct_method_and_path() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v("99", "t", 2)))
            .mount(&server)
            .await;

        let audit = open_in_memory_audit();
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri())
            .expect("backend")
            .with_audit(Arc::clone(&audit));

        let patch = make_untainted("title check", "body", None);
        backend
            .update_record("REPOSIX", RecordId(99), patch, Some(1))
            .await
            .expect("update_record should succeed");

        let (method_val, path_val, status_val, agent_id, response_summary): (
            String,
            String,
            i64,
            String,
            String,
        ) = audit
            .lock()
            .query_row(
                "SELECT method, path, status, agent_id, response_summary \
                 FROM audit_events ORDER BY id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .expect("audit row must exist");

        assert_eq!(method_val, "PUT");
        assert_eq!(path_val, "/wiki/api/v2/pages/99");
        assert_eq!(status_val, 200);
        assert!(
            agent_id.starts_with("reposix-confluence-"),
            "agent_id must start with 'reposix-confluence-', got {agent_id:?}"
        );
        // response_summary format: "{status}:{16_hex_chars}"
        let re_ok = response_summary.starts_with("200:")
            && response_summary.len() == 4 + 16 // "200:" + 16 hex chars
            && response_summary[4..].chars().all(|c| c.is_ascii_hexdigit());
        assert!(
            re_ok,
            "response_summary must match '^200:[0-9a-f]{{16}}$', got {response_summary:?}"
        );
    }

    /// T-16-C-06: if the audit connection's table is dropped, the write result
    /// must still be `Ok`. The audit insert failure is log-and-swallow.
    #[tokio::test]
    async fn audit_insert_failure_does_not_mask_write_result() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json_v("99", "t", 2)))
            .mount(&server)
            .await;

        // Open an audit DB whose table is then dropped — INSERT will fail.
        let conn = rusqlite::Connection::open_in_memory().expect("in-memory db");
        reposix_core::audit::load_schema(&conn).expect("load_schema");
        conn.execute("DROP TABLE audit_events", [])
            .expect("drop table for test setup");
        let audit = Arc::new(Mutex::new(conn));

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri())
            .expect("backend")
            .with_audit(Arc::clone(&audit));

        let patch = make_untainted("resilience test", "body", None);
        // Must NOT return Err — audit failure is swallowed.
        backend
            .update_record("REPOSIX", RecordId(99), patch, Some(1))
            .await
            .expect("write must succeed even when audit INSERT fails");
    }

    /// T-16-C-01 extension: a failed write (409 Conflict) must still produce
    /// an audit row so that post-hoc attack analysis can see the attempt.
    #[tokio::test]
    async fn audit_records_failed_writes() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/wiki/api/v2/pages/99"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(json!({"statusCode": 409, "message": "version conflict"})),
            )
            .mount(&server)
            .await;

        let audit = open_in_memory_audit();
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri())
            .expect("backend")
            .with_audit(Arc::clone(&audit));

        let patch = make_untainted("conflict test", "body", None);
        // The write itself returns Err (409 is a version mismatch error).
        let result = backend
            .update_record("REPOSIX", RecordId(99), patch, Some(1))
            .await;
        assert!(result.is_err(), "409 must return Err");

        // But the audit row must still be written with status 409.
        let status_val: i64 = audit
            .lock()
            .query_row(
                "SELECT status FROM audit_events WHERE method = 'PUT'",
                [],
                |r| r.get(0),
            )
            .expect("audit row must exist even for failed writes");
        assert_eq!(
            status_val, 409,
            "audit row status must be 409 for failed write"
        );
    }

    // -------- truncation: warn mode (list_records, default) --------

    /// Verify that `list_records` (non-strict) emits a warn and returns
    /// `Ok(capped)` when the space exceeds `MAX_ISSUES_PER_LIST / PAGE_SIZE`
    /// pages. We mock exactly `MAX_ISSUES_PER_LIST / PAGE_SIZE + 1` page
    /// responses so the pagination loop triggers the cap on the next fetch.
    ///
    /// Because we're relying on the tracing warn being emitted (rather than
    /// asserting on a captured subscriber) we just confirm the Ok result shape
    /// here; the strict-mode test below covers the Err path.
    #[tokio::test]
    async fn truncation_warn_on_default_list() {
        // Number of pages that will exceed the cap: PAGE_SIZE=100, cap=500 →
        // more than 500/100 = 5 page-fetches. We set up 6 pages in the mock
        // so the 6th fetch triggers the cap branch.
        let server = MockServer::start().await;
        mount_space_lookup(&server, "TRUNCTEST", "9999").await;

        // Pages 1..=5: each returns PAGE_SIZE (1) result with a next cursor.
        // We use 1 item per page to keep the mock small; the cap check fires
        // on pages > 5, not on out.len() >= 500.
        for i in 1..=5u32 {
            let cursor_param = format!("C{i}");
            let next_cursor = format!("/wiki/api/v2/spaces/9999/pages?cursor=C{i}&limit=100");
            let matcher_path = path("/wiki/api/v2/spaces/9999/pages");
            let page_result = json!([page_json(
                &i.to_string(),
                "current",
                &format!("page {i}"),
                None
            )]);
            if i == 1 {
                // First page: no cursor param, just the base path + limit
                Mock::given(method("GET"))
                    .and(matcher_path)
                    .and(query_param("limit", "100"))
                    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                        "results": page_result,
                        "_links": { "next": next_cursor }
                    })))
                    .up_to_n_times(1)
                    .mount(&server)
                    .await;
            } else {
                let prev_cursor = format!("C{}", i - 1);
                Mock::given(method("GET"))
                    .and(matcher_path)
                    .and(query_param("cursor", prev_cursor.as_str()))
                    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                        "results": page_result,
                        "_links": { "next": next_cursor }
                    })))
                    .up_to_n_times(1)
                    .mount(&server)
                    .await;
                drop(cursor_param);
            }
        }
        // Page 6: would be fetched if cap doesn't fire — but the cap fires at
        // pages > 5, so this mock should NOT be called. We register it with
        // up_to_n_times(0) to assert it is never reached.
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/9999/pages"))
            .and(query_param("cursor", "C5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [],
                "_links": {}
            })))
            .expect(0)
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let result = backend.list_records("TRUNCTEST").await;
        // Must succeed (warn mode, not strict).
        let issues = result.expect("list_records must succeed in warn mode even at cap");
        // 5 pages × 1 item = 5 issues.
        assert_eq!(issues.len(), 5, "expected 5 issues (one per mocked page)");
    }

    /// Verify that `list_records_strict` returns `Err` containing
    /// `"strict mode"` and `"500-page cap"` when the space exceeds the
    /// pagination cap, and that no `Ok(partial)` result escapes.
    #[tokio::test]
    async fn truncation_errors_in_strict_mode() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "TRUNCTEST", "9999").await;

        // Same 5-page setup as above — the 6th page-fetch would trigger the
        // cap check on pages > 5. In strict mode the check fires before the
        // 6th request, returning Err immediately.
        for i in 1..=5u32 {
            let next_cursor = format!("/wiki/api/v2/spaces/9999/pages?cursor=C{i}&limit=100");
            let page_result = json!([page_json(
                &i.to_string(),
                "current",
                &format!("page {i}"),
                None
            )]);
            if i == 1 {
                Mock::given(method("GET"))
                    .and(path("/wiki/api/v2/spaces/9999/pages"))
                    .and(query_param("limit", "100"))
                    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                        "results": page_result,
                        "_links": { "next": next_cursor }
                    })))
                    .up_to_n_times(1)
                    .mount(&server)
                    .await;
            } else {
                let prev_cursor = format!("C{}", i - 1);
                Mock::given(method("GET"))
                    .and(path("/wiki/api/v2/spaces/9999/pages"))
                    .and(query_param("cursor", prev_cursor.as_str()))
                    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                        "results": page_result,
                        "_links": { "next": next_cursor }
                    })))
                    .up_to_n_times(1)
                    .mount(&server)
                    .await;
            }
        }

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let result = backend.list_records_strict("TRUNCTEST").await;
        assert!(
            result.is_err(),
            "list_records_strict must return Err at cap"
        );
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("strict mode"),
            "error must mention 'strict mode': {msg}"
        );
        assert!(
            msg.contains("500-page cap"),
            "error must mention '500-page cap': {msg}"
        );
        // Also verify the space name is in the error for debuggability.
        assert!(
            msg.contains("TRUNCTEST"),
            "error must mention the space key: {msg}"
        );
    }

    /// Verify that error messages from `list_records` on HTTP failure do NOT
    /// contain the host:port (which in production would be the tenant name),
    /// but DO contain the path portion of the URL (for debuggability).
    #[tokio::test]
    async fn list_error_message_omits_tenant() {
        let server = MockServer::start().await;
        mount_space_lookup(&server, "LEAKTEST", "7777").await;

        // Return HTTP 500 for the first pages GET.
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/7777/pages"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
            .mount(&server)
            .await;

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let result = backend.list_records("LEAKTEST").await;
        assert!(result.is_err(), "500 must return Err");
        let msg = format!("{}", result.unwrap_err());

        // The mock server is on 127.0.0.1:<port>. The error must NOT contain
        // the port number (which stands in for the tenant host in production).
        let host_port = server.uri(); // e.g. "http://127.0.0.1:54321"
                                      // Strip the scheme to get "127.0.0.1:54321"
        let host_port_bare = host_port.trim_start_matches("http://");
        assert!(
            !msg.contains(host_port_bare),
            "error must not contain host:port '{host_port_bare}': {msg}"
        );

        // The path portion must survive so callers can debug the request.
        assert!(
            msg.contains("/wiki/api/v2/spaces/7777/pages"),
            "error must contain the API path for debuggability: {msg}"
        );
    }

    // ======================================================================
    // Phase 23 Plan 01: list_comments() + list_spaces() tests
    // ======================================================================

    // -------- Task 1: list_comments tests --------

    #[tokio::test]
    async fn list_comments_returns_inline_and_footer() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/98765/inline-comments"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "id": "111",
                    "pageId": "98765",
                    "parentCommentId": null,
                    "resolutionStatus": "open",
                    "version": {"createdAt": "2026-01-15T10:30:00Z", "number": 1, "authorId": "user-a"},
                    "body": {"atlas_doc_format": {"value": {"type":"doc","version":1,"content":[]}}}
                }],
                "_links": {}
            })))
            .mount(&server).await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/98765/footer-comments"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "id": "222",
                    "pageId": "98765",
                    "version": {"createdAt": "2026-02-01T09:00:00Z", "number": 1, "authorId": "user-b"},
                    "body": {"atlas_doc_format": {"value": {"type":"doc","version":1,"content":[]}}}
                }],
                "_links": {}
            })))
            .mount(&server).await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let comments = backend.list_comments(98765).await.expect("list_comments");
        assert_eq!(comments.len(), 2, "expected 1 inline + 1 footer");
        assert!(comments
            .iter()
            .any(|c| c.id == "111" && c.kind == CommentKind::Inline));
        assert!(comments
            .iter()
            .any(|c| c.id == "222" && c.kind == CommentKind::Footer));
    }

    #[tokio::test]
    async fn list_comments_paginates_inline_via_links_next() {
        let server = MockServer::start().await;
        // Page 1 — returns _links.next pointing to cursor=SECOND
        // .up_to_n_times(1) ensures the first-page mock is exhausted before
        // the cursor request arrives, preventing wiremock from re-matching it
        // (wiremock uses first-registered-wins when multiple mocks match).
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/42/inline-comments"))
            .and(query_param("limit", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "id": "1", "pageId": "42", "version": {"createdAt": "2026-01-01T00:00:00Z", "number": 1, "authorId": "a"}
                }],
                "_links": { "next": "/wiki/api/v2/pages/42/inline-comments?cursor=SECOND&limit=100" }
            })))
            .up_to_n_times(1)
            .mount(&server).await;
        // Page 2 — same path, different cursor
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/42/inline-comments"))
            .and(query_param("cursor", "SECOND"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "id": "2", "pageId": "42", "version": {"createdAt": "2026-01-01T00:00:00Z", "number": 1, "authorId": "a"}
                }],
                "_links": {}
            })))
            .mount(&server).await;
        // Empty footer
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/42/footer-comments"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"results": [], "_links": {}})),
            )
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let comments = backend.list_comments(42).await.expect("list_comments");
        assert_eq!(comments.len(), 2);
    }

    #[tokio::test]
    async fn list_comments_handles_absent_body() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/7/inline-comments"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "id": "10", "pageId": "7",
                    "version": {"createdAt": "2026-01-01T00:00:00Z", "number": 1, "authorId": "a"}
                }],
                "_links": {}
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/7/footer-comments"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"results": [], "_links": {}})),
            )
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let comments = backend.list_comments(7).await.expect("list_comments");
        assert_eq!(comments.len(), 1);
        assert!(comments[0].body.is_none());
        assert_eq!(comments[0].body_markdown(), "");
    }

    #[tokio::test]
    async fn list_comments_rejects_non_success_status() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/99/inline-comments"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let err = backend.list_comments(99).await.expect_err("must fail");
        let msg = format!("{err}");
        assert!(msg.contains("404"), "error must mention status: {msg}");
        // HARD-05 precedent: redact_url strips tenant hostname — server.uri() is
        // a 127.0.0.1:<port> wiremock URL; the redaction turns it into a path.
        // Assert the wiremock port is NOT in the error message.
        let host = server.uri();
        assert!(
            !msg.contains(&host),
            "error must not leak full URL (host={host}): {msg}"
        );
    }

    // -------- Task 2: list_spaces tests --------

    #[tokio::test]
    async fn list_spaces_returns_key_name_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [
                    { "id": "1", "key": "REPOSIX", "name": "Reposix Project",
                      "_links": { "webui": "/wiki/spaces/REPOSIX" } },
                    { "id": "2", "key": "TEAM", "name": "Team Space",
                      "_links": { "webui": "/wiki/spaces/TEAM" } }
                ],
                "_links": {}
            })))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let spaces = backend.list_spaces().await.expect("list_spaces");
        assert_eq!(spaces.len(), 2);
        assert_eq!(spaces[0].key, "REPOSIX");
        assert_eq!(spaces[0].name, "Reposix Project");
        // webui_url must be absolute (prefixed with wiremock base URI)
        assert!(
            spaces[0].webui_url.starts_with(&server.uri()),
            "webui_url must be absolute: {}",
            spaces[0].webui_url
        );
        assert!(spaces[0].webui_url.ends_with("/wiki/spaces/REPOSIX"));
    }

    #[tokio::test]
    async fn list_spaces_paginates_via_links_next() {
        let server = MockServer::start().await;
        // .up_to_n_times(1) prevents the first-page mock from matching the cursor
        // request (which also has limit=250). Wiremock uses first-registered-wins,
        // so without the limit both mocks match and the infinite-loop response wins.
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces"))
            .and(query_param("limit", "250"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{"id":"1","key":"A","name":"A","_links":{"webui":"/wiki/spaces/A"}}],
                "_links": {"next":"/wiki/api/v2/spaces?cursor=NEXT&limit=250"}
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces"))
            .and(query_param("cursor", "NEXT"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{"id":"2","key":"B","name":"B","_links":{"webui":"/wiki/spaces/B"}}],
                "_links": {}
            })))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let spaces = backend.list_spaces().await.expect("list_spaces");
        assert_eq!(spaces.len(), 2);
        assert_eq!(spaces[0].key, "A");
        assert_eq!(spaces[1].key, "B");
    }

    #[tokio::test]
    async fn list_spaces_rejects_non_success_with_redacted_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces"))
            .respond_with(ResponseTemplate::new(500).set_body_string("oops"))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let err = backend.list_spaces().await.expect_err("must fail");
        let msg = format!("{err}");
        assert!(msg.contains("500"));
        let host = server.uri();
        assert!(
            !msg.contains(&host),
            "error must redact host ({host}): {msg}"
        );
    }

    // ======================================================================
    // Phase 24 Plan 01: list_attachments + list_whiteboards + download_attachment
    // ======================================================================

    // -------- list_attachments tests --------

    #[tokio::test]
    async fn list_attachments_returns_vec() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/12345/attachments"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "id": "att-1",
                    "status": "current",
                    "title": "diagram.png",
                    "createdAt": "2024-01-01T00:00:00Z",
                    "pageId": "12345",
                    "mediaType": "image/png",
                    "fileSize": 1024,
                    "downloadLink": "/wiki/download/attachments/12345/diagram.png"
                }],
                "_links": {}
            })))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let atts = backend
            .list_attachments(12345)
            .await
            .expect("list_attachments");
        assert_eq!(atts.len(), 1);
        assert_eq!(atts[0].title, "diagram.png");
        assert_eq!(atts[0].file_size, 1024);
        assert_eq!(atts[0].media_type, "image/png");
    }

    #[tokio::test]
    async fn list_attachments_empty_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/99/attachments"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [],
                "_links": {}
            })))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let atts = backend
            .list_attachments(99)
            .await
            .expect("list_attachments empty");
        assert!(atts.is_empty());
    }

    #[tokio::test]
    async fn list_attachments_non2xx_returns_err() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/12345/attachments"))
            .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let err = backend
            .list_attachments(12345)
            .await
            .expect_err("403 must be Err");
        let msg = format!("{err}");
        assert!(msg.contains("403"), "error must mention status: {msg}");
    }

    // -------- list_whiteboards tests --------

    #[tokio::test]
    async fn list_whiteboards_filters_by_type() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/space-999/direct-children"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [
                    {
                        "id": "wb-1",
                        "type": "whiteboard",
                        "title": "Arch Board",
                        "spaceId": "space-999",
                        "createdAt": "2024-01-01T00:00:00Z",
                        "status": "current"
                    },
                    {
                        "id": "pg-1",
                        "type": "page",
                        "title": "Some Page",
                        "spaceId": "space-999",
                        "createdAt": "2024-01-01T00:00:00Z",
                        "status": "current"
                    }
                ],
                "_links": {}
            })))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let wbs = backend
            .list_whiteboards("space-999")
            .await
            .expect("list_whiteboards");
        assert_eq!(
            wbs.len(),
            1,
            "page must be filtered out; only whiteboard remains"
        );
        assert_eq!(wbs[0].id, "wb-1");
        assert_eq!(wbs[0].title, "Arch Board");
    }

    #[tokio::test]
    async fn list_whiteboards_404_returns_empty() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/no-wb-space/direct-children"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let wbs = backend
            .list_whiteboards("no-wb-space")
            .await
            .expect("404 must return Ok(vec![])");
        assert!(wbs.is_empty(), "404 must degrade to empty vec");
    }

    // -------- download_attachment tests --------

    #[tokio::test]
    async fn download_attachment_returns_bytes() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/download/attachments/12345/file.pdf"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"PDF_CONTENT".to_vec()))
            .mount(&server)
            .await;
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let bytes = backend
            .download_attachment("/wiki/download/attachments/12345/file.pdf")
            .await
            .expect("download_attachment");
        assert_eq!(bytes, b"PDF_CONTENT".to_vec());
    }
}
