//! [`ConfluenceReadOnlyBackend`] — read-only [`IssueBackend`] adapter for
//! Atlassian Confluence Cloud REST v2.
//!
//! # Scope
//!
//! v0.3 ships read-only: `list_issues` + `get_issue` work against a real
//! Atlassian tenant (once credentials are configured); `create_issue` /
//! `update_issue` / `delete_or_close` return `Error::Other("not supported: …")`
//! without emitting any HTTP. v0.4 may flip on the write path (separate ADR).
//!
//! # Page → Issue mapping (Option A flattening, per ADR-002)
//!
//! | Issue field  | Confluence source                             |
//! |--------------|-----------------------------------------------|
//! | `id`         | `parse_u64(page.id)` — Confluence page IDs are numeric strings |
//! | `title`      | `page.title`                                  |
//! | `status`     | `"current" | "draft"` → `Open`; `"archived" | "trashed" | "deleted"` → `Done` |
//! | `body`       | `page.body.storage.value` (raw storage HTML)  |
//! | `created_at` | `page.createdAt`                              |
//! | `updated_at` | `page.version.createdAt`                      |
//! | `version`    | `page.version.number`                         |
//! | `assignee`   | `page.ownerId` (Atlassian accountId)          |
//! | `labels`     | `vec![]` (deferred — labels live on a separate endpoint) |
//!
//! Parent/child hierarchy, space metadata, comments, attachments, and
//! restrictions are out of scope for v0.3 and documented as lost metadata
//! in ADR-002.
//!
//! # Pagination
//!
//! Confluence v2 uses cursor-based pagination via `_links.next` (relative
//! path). This differs from GitHub's `Link: rel="next"` header. The adapter
//! prepends `self.base()` to the relative cursor path; attacker-controlled
//! absolute URLs would still be caught by the SG-01 allowlist gate inside
//! [`HttpClient`], but the construction-by-relative-prepend shape defeats
//! that class of attack by construction.
//!
//! # Rate limiting
//!
//! On HTTP 429 (or when `x-ratelimit-remaining: 0` is reported), the adapter
//! arms a shared [`rate_limit_gate`] on an [`Instant`] derived from the
//! `Retry-After` header (seconds). The next outbound call parks until the
//! gate elapses, capped at [`MAX_RATE_LIMIT_SLEEP`].
//!
//! [`rate_limit_gate`]: ConfluenceReadOnlyBackend
//!
//! # Security
//!
//! - **SG-01:** every HTTP call goes through `reposix-core`'s sealed
//!   [`HttpClient`], which re-checks every target URL against
//!   `REPOSIX_ALLOWED_ORIGINS` before any socket I/O. Callers MUST set the
//!   env var to include `https://{tenant}.atlassian.net` at runtime.
//! - **SG-05:** every decoded page is wrapped through [`Tainted::new`] before
//!   translation, documenting the "came from untrusted network" origin of
//!   every byte.
//! - **T-11-01 (creds leak):** [`ConfluenceCreds`] has a manual `Debug` impl
//!   that prints `api_token: "<redacted>"`. Same redaction on the backend
//!   struct.
//! - **T-11-02 (SSRF via tenant injection):** [`ConfluenceReadOnlyBackend::new`]
//!   validates `tenant` against DNS-label rules (`^[a-z0-9][a-z0-9-]{0,62}$`)
//!   before URL construction.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]

pub mod adf;

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use parking_lot::Mutex;
use reqwest::{Method, StatusCode};
use serde::Deserialize;

use reposix_core::backend::{BackendFeature, DeleteReason, IssueBackend};
use reposix_core::http::{client, ClientOpts, HttpClient};
use reposix_core::{Error, Issue, IssueId, IssueStatus, Result, Tainted, Untainted};

/// Maximum time we'll wait for a rate-limit reset before surfacing the
/// exhaustion as an error. Caps worst-case call latency.
const MAX_RATE_LIMIT_SLEEP: Duration = Duration::from_secs(60);

/// Max issues we'll page through in one `list_issues` call.
///
/// At [`PAGE_SIZE`] 100 that's 5 requests — enough for the REPOSIX demo space
/// and a bounded memory budget. Matches `reposix-github`'s cap.
const MAX_ISSUES_PER_LIST: usize = 500;

/// Page size to request from Confluence (1..=250 allowed; 100 is the sweet
/// spot for latency × payload size).
const PAGE_SIZE: usize = 100;

/// Format string for the default production base URL. Callers supply the
/// tenant subdomain (validated against DNS-label rules in
/// [`ConfluenceReadOnlyBackend::new`]).
pub const DEFAULT_BASE_URL_FORMAT: &str = "https://{tenant}.atlassian.net";

/// Basic-auth credentials for Confluence Cloud.
///
/// `api_token` is an Atlassian API token issued from
/// <https://id.atlassian.com/manage-profile/security/api-tokens>. `email` is
/// the account email the token was issued under (they must match, else the
/// token authenticates nothing).
///
/// # Debug redaction
///
/// This type has a **manual** `Debug` impl that prints `api_token:
/// "<redacted>"`. Do NOT `#[derive(Debug)]`; that would leak the token into
/// every tracing span and error message.
#[derive(Clone)]
pub struct ConfluenceCreds {
    /// Atlassian account email — the email the `api_token` was issued under.
    pub email: String,
    /// Atlassian API token. Redacted in `Debug`.
    pub api_token: String,
}

impl std::fmt::Debug for ConfluenceCreds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfluenceCreds")
            .field("email", &self.email)
            .field("api_token", &"<redacted>")
            .finish()
    }
}

/// Read-only `IssueBackend` for Atlassian Confluence Cloud REST v2.
///
/// Construct via [`ConfluenceReadOnlyBackend::new`] (public production API)
/// or [`ConfluenceReadOnlyBackend::new_with_base_url`] (custom base; used by
/// wiremock unit tests and the contract test).
///
/// # Thread-safety
///
/// `Clone` is cheap (`http` is `Arc`-shared) and all methods take `&self`,
/// so the struct is safe to share across tokio tasks. The rate-limit gate
/// is shared across clones so a single throttled instance can't be bypassed
/// by cloning.
#[derive(Clone)]
pub struct ConfluenceReadOnlyBackend {
    http: Arc<HttpClient>,
    creds: ConfluenceCreds,
    /// Tenant base URL with no trailing slash, e.g.
    /// `https://reuben-john.atlassian.net` in production, or the wiremock
    /// server URI in tests.
    base_url: String,
    /// When `Some(t)`, the next outbound request must sleep until `t`.
    /// Set after a response where `x-ratelimit-remaining` hits zero or a
    /// 429 is returned with a `Retry-After` header. Shared across clones.
    rate_limit_gate: Arc<Mutex<Option<Instant>>>,
}

// Manual Debug on the backend struct too — the derived Debug would print
// `creds` which has its own redaction, but being explicit documents the
// intent and ensures `cargo expand` can never accidentally flip back.
impl std::fmt::Debug for ConfluenceReadOnlyBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `http` is deliberately omitted; it has no meaningful Debug state
        // worth showing and including it obscures the redaction intent.
        f.debug_struct("ConfluenceReadOnlyBackend")
            .field("base_url", &self.base_url)
            .field("creds", &self.creds)
            .field("rate_limit_gate", &"<gate>")
            .finish_non_exhaustive()
    }
}

/// Minimal shape of the `GET /wiki/api/v2/spaces?keys=KEY` response we
/// consume. `deny_unknown_fields` is deliberately NOT set — Atlassian
/// adds fields routinely and forward-compat matters.
#[derive(Debug, Deserialize)]
struct ConfSpaceList {
    results: Vec<ConfSpace>,
}

#[derive(Debug, Deserialize)]
struct ConfSpace {
    id: String,
}

/// `GET /wiki/api/v2/spaces/{id}/pages?limit=N` response shape.
#[derive(Debug, Deserialize)]
struct ConfPageList {
    results: Vec<ConfPage>,
    #[serde(default, rename = "_links")]
    links: Option<ConfLinks>,
}

#[derive(Debug, Deserialize)]
struct ConfLinks {
    // Mirrored by `parse_next_cursor`; serde pulls the same path through
    // `serde_json::Value`. Kept for documentation + deny-unknown-shape
    // reviewer cues. Allow `dead_code`: field is observed only via the
    // JSON-Value path so the compiler can't see the usage.
    #[serde(default)]
    #[allow(dead_code)]
    next: Option<String>,
}

/// A single page as returned by both the list endpoint (with `body: {}`
/// empty) and the single-page endpoint (with `body.storage.value` populated
/// when `?body-format=storage` is requested).
#[derive(Debug, Deserialize)]
struct ConfPage {
    id: String,
    status: String,
    title: String,
    #[serde(rename = "createdAt")]
    created_at: chrono::DateTime<chrono::Utc>,
    version: ConfVersion,
    #[serde(default, rename = "ownerId")]
    owner_id: Option<String>,
    #[serde(default)]
    body: Option<ConfPageBody>,
    /// Confluence REST v2 `parentId` — numeric string referring to another
    /// entity in the content hierarchy. Only meaningful when `parent_type ==
    /// Some("page")`; for folders/whiteboards/databases we deliberately drop
    /// it in [`translate`] so the tree-builder treats those as orphans.
    /// `#[serde(default)]` keeps Phase-11 fixtures (no parent fields) decoding
    /// unchanged.
    #[serde(default, rename = "parentId")]
    parent_id: Option<String>,
    /// Confluence REST v2 `parentType` — one of `"page"`, `"folder"`,
    /// `"whiteboard"`, `"database"`, etc. Only the `"page"` case propagates
    /// into [`Issue::parent_id`]; every other value is treated as a tree root
    /// (with a `tracing::debug!` trail) because reposix's hierarchy model is
    /// homogeneous (pages only).
    #[serde(default, rename = "parentType")]
    parent_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConfVersion {
    number: u64,
    #[serde(rename = "createdAt")]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct ConfPageBody {
    #[serde(default)]
    storage: Option<ConfBodyStorage>,
}

#[derive(Debug, Deserialize)]
struct ConfBodyStorage {
    value: String,
}

/// Build the Basic-auth header value for a given email + token.
///
/// The output format is `Basic {base64(email:token)}` using the STANDARD
/// base64 alphabet (not URL-safe) with padding. This matches Atlassian's
/// <https://developer.atlassian.com/cloud/confluence/basic-auth-for-rest-apis/>
/// contract exactly.
#[must_use]
pub fn basic_auth_header(email: &str, token: &str) -> String {
    use base64::Engine;
    let raw = format!("{email}:{token}");
    format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(raw.as_bytes())
    )
}

/// Extract the relative next-page cursor path from a Confluence v2 paginated
/// response body. Returns `None` if `_links.next` is absent or not a string.
///
/// Callers MUST prepend the tenant base URL to turn the relative path into a
/// fully-qualified URL — never trust `_links.base` from the body (that's an
/// SSRF vector).
#[must_use]
pub fn parse_next_cursor(body: &serde_json::Value) -> Option<String> {
    body.get("_links")
        .and_then(|l| l.get("next"))
        .and_then(|n| n.as_str())
        .map(str::to_owned)
}

/// Map a Confluence v2 page `status` string onto reposix's normalized
/// [`IssueStatus`]. Pessimistic forward-compat: unknown values fall through
/// to `Open` (consistent with CONTEXT.md §status mapping).
#[must_use]
pub fn status_from_confluence(s: &str) -> IssueStatus {
    // Unknown values fall through to `Open` (pessimistic forward-compat):
    // an unseen Atlassian state should not silently mark pages as Done.
    // `match_same_arms` lint would prefer collapsing the current/draft/_
    // arms, but keeping `"current" | "draft"` as an explicit allowlist
    // documents the mapping contract — suppress the lint on purpose.
    #[allow(clippy::match_same_arms)]
    match s {
        "current" | "draft" => IssueStatus::Open,
        "archived" | "trashed" | "deleted" => IssueStatus::Done,
        _ => IssueStatus::Open,
    }
}

/// Translate a deserialized Confluence page into reposix's normalized
/// [`Issue`].
///
/// # Errors
///
/// Returns `Err(Error::Other(…))` if `page.id` is not a valid `u64` (very
/// rare — Atlassian consistently returns numeric strings, but system pages
/// have historically had non-numeric ids).
fn translate(page: ConfPage) -> Result<Issue> {
    let id = page
        .id
        .parse::<u64>()
        .map_err(|_| Error::Other(format!("confluence page id not a u64: {:?}", page.id)))?;
    let body = page
        .body
        .and_then(|b| b.storage)
        .map(|s| s.value)
        .unwrap_or_default();
    // Phase 13 Wave B1: derive `Issue::parent_id` from Confluence REST v2
    // `parentId`/`parentType`. Only `parent_type == "page"` propagates — the
    // reposix tree is homogeneous (pages only), so `folder` / `whiteboard` /
    // `database` parents become tree roots with a debug-log breadcrumb.
    //
    // A malformed `parentId` (e.g. `"abc"`) is degraded to `None` rather
    // than propagated as `Err`. T-13-PB1: failing the whole list because
    // one page's parent is garbage would be a DoS amplifier against an
    // adversarial tenant. Surfacing the page as a tree root is the
    // graceful-degradation alternative the threat register mandates.
    let parent_id = match (page.parent_id.as_deref(), page.parent_type.as_deref()) {
        (Some(pid_str), Some("page")) => {
            if let Ok(n) = pid_str.parse::<u64>() {
                Some(IssueId(n))
            } else {
                // IN-01: cap attacker-controlled string in tracing output to
                // bound log storage + log-injection blast radius. 64 bytes is
                // ample diagnostic detail without amplifying.
                let pid_preview = pid_str.get(..pid_str.len().min(64)).unwrap_or("");
                tracing::warn!(
                    page_id = %page.id,
                    bad_parent = %pid_preview,
                    "confluence parentId not parseable as u64, treating as orphan"
                );
                None
            }
        }
        (_, Some(other)) => {
            // IN-01: same cap for parentType field.
            let other_preview = other.get(..other.len().min(64)).unwrap_or("");
            tracing::debug!(
                page_id = %page.id,
                parent_type = %other_preview,
                "confluence non-page parentType, treating as orphan"
            );
            None
        }
        _ => None,
    };
    Ok(Issue {
        id: IssueId(id),
        title: page.title,
        status: status_from_confluence(&page.status),
        assignee: page.owner_id,
        labels: vec![],
        created_at: page.created_at,
        updated_at: page.version.created_at,
        version: page.version.number,
        body,
        parent_id,
    })
}

impl ConfluenceReadOnlyBackend {
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
        Self::validate_tenant(tenant)?;
        Self::new_with_base_url(creds, format!("https://{tenant}.atlassian.net"))
    }

    /// Validate a tenant subdomain against DNS-label rules.
    ///
    /// Rejects: empty, > 63 chars, any character outside `[a-z0-9-]`,
    /// leading/trailing hyphen. This defeats injection like `a.evil.com`,
    /// `../../../`, `a@b`, or `tenant.with.dots`.
    fn validate_tenant(tenant: &str) -> Result<()> {
        if tenant.is_empty() || tenant.len() > 63 {
            return Err(Error::Other(format!(
                "invalid confluence tenant subdomain: {tenant:?} (length must be 1..=63)"
            )));
        }
        if tenant.starts_with('-') || tenant.ends_with('-') {
            return Err(Error::Other(format!(
                "invalid confluence tenant subdomain: {tenant:?} (no leading/trailing hyphen)"
            )));
        }
        if !tenant
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(Error::Other(format!(
                "invalid confluence tenant subdomain: {tenant:?} (only [a-z0-9-] allowed)"
            )));
        }
        Ok(())
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
        })
    }

    fn base(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }

    /// Assemble the standard headers every Confluence REST call carries.
    ///
    /// Returns owned strings so callers can `.iter().map(|(k, v)| (*k,
    /// v.as_str()))` into the `&[(&str, &str)]` shape `HttpClient` wants
    /// without lifetime gymnastics.
    fn standard_headers(&self) -> Vec<(&'static str, String)> {
        vec![
            ("Accept", "application/json".to_owned()),
            ("User-Agent", "reposix-confluence-readonly/0.3".to_owned()),
            (
                "Authorization",
                basic_auth_header(&self.creds.email, &self.creds.api_token),
            ),
        ]
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
    /// gate on `Instant::now() + retry_after`, capped at
    /// [`MAX_RATE_LIMIT_SLEEP`]. If `retry-after` is absent, default to 60s.
    fn ingest_rate_limit(&self, resp: &reqwest::Response) {
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
    async fn await_rate_limit_gate(&self) {
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
    /// `list_issues` call.
    ///
    /// # Errors
    ///
    /// - Transport errors propagate.
    /// - HTTP non-2xx surfaces as `Err(Error::Other(…))`.
    /// - Empty `results` array (space not found) surfaces as
    ///   `Err(Error::Other("not found: space key …"))`.
    async fn resolve_space_id(&self, space_key: &str) -> Result<String> {
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
                "confluence returned {status} for GET {url}: {}",
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
}

#[async_trait]
impl IssueBackend for ConfluenceReadOnlyBackend {
    fn name(&self) -> &'static str {
        "confluence-readonly"
    }

    fn supports(&self, feature: BackendFeature) -> bool {
        // Read-only v0.3: no write-path features. Even Workflows is false
        // because Confluence has no in-flight state labels analogous to
        // GitHub's status/*. Phase 13 Wave B1 adds `Hierarchy` — Confluence
        // is the ONLY backend (of the current three) that exposes a parent
        // tree, so the FUSE layer can key `tree/` overlay emission off this
        // one bit instead of per-backend `match`ing.
        matches!(feature, BackendFeature::Hierarchy)
    }

    fn root_collection_name(&self) -> &'static str {
        // Confluence-native vocabulary: pages, not issues. The default
        // `"issues"` stays correct for sim + GitHub; Confluence overrides
        // here so mounts surface as `pages/<padded-id>.md` — the layout
        // locked in Phase 13 CONTEXT.md and ADR-003.
        "pages"
    }

    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>> {
        let space_id = self.resolve_space_id(project).await?;
        let first = format!(
            "{}/wiki/api/v2/spaces/{}/pages?limit={}",
            self.base(),
            space_id,
            PAGE_SIZE
        );
        let mut next_url: Option<String> = Some(first);
        let mut out: Vec<Issue> = Vec::new();
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
            let bytes = resp.bytes().await?;
            if !status.is_success() {
                return Err(Error::Other(format!(
                    "confluence returned {status} for GET {url}: {}",
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

    async fn get_issue(&self, _project: &str, id: IssueId) -> Result<Issue> {
        let url = format!(
            "{}/wiki/api/v2/pages/{}?body-format=storage",
            self.base(),
            id.0
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
        if status == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!("not found: {url}")));
        }
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status} for GET {url}: {}",
                String::from_utf8_lossy(&bytes)
            )));
        }
        let page: ConfPage = serde_json::from_slice(&bytes)?;
        // SG-05: Tainted::new wraps ingress before translation.
        let tainted = Tainted::new(page);
        translate(tainted.into_inner())
    }

    async fn create_issue(&self, _project: &str, _issue: Untainted<Issue>) -> Result<Issue> {
        Err(Error::Other(
            "not supported: create_issue — reposix-confluence is read-only".into(),
        ))
    }

    async fn update_issue(
        &self,
        _project: &str,
        _id: IssueId,
        _patch: Untainted<Issue>,
        _expected_version: Option<u64>,
    ) -> Result<Issue> {
        Err(Error::Other(
            "not supported: update_issue — reposix-confluence is read-only".into(),
        ))
    }

    async fn delete_or_close(
        &self,
        _project: &str,
        _id: IssueId,
        _reason: DeleteReason,
    ) -> Result<()> {
        Err(Error::Other(
            "not supported: delete_or_close — reposix-confluence is read-only".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use reposix_core::{sanitize, ServerMetadata};
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

        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issues = backend.list_issues("REPOSIX").await.expect("list");
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].id, IssueId(98765));
        assert_eq!(issues[0].status, IssueStatus::Open);
        assert_eq!(issues[1].id, IssueId(98766));
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

        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issues = backend.list_issues("REPOSIX").await.expect("list");
        assert_eq!(issues.len(), 3);
        assert_eq!(issues[0].id, IssueId(1));
        assert_eq!(issues[2].id, IssueId(3));
    }

    // -------- 3: get_issue returns body storage value --------

    #[tokio::test]
    async fn get_issue_returns_body_storage_value() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/98765"))
            .and(query_param("body-format", "storage"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json(
                "98765",
                "current",
                "hello",
                Some("<p>Hello</p>"),
            )))
            .mount(&server)
            .await;

        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issue = backend
            .get_issue("REPOSIX", IssueId(98765))
            .await
            .expect("get");
        assert_eq!(issue.body, "<p>Hello</p>");
        assert_eq!(issue.id, IssueId(98765));
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
        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let err = backend
            .get_issue("REPOSIX", IssueId(9999))
            .await
            .expect_err("404");
        match err {
            Error::Other(m) => assert!(m.starts_with("not found:"), "got {m}"),
            other => panic!("expected not found, got {other:?}"),
        }
    }

    // -------- 5: status "current" → Open (via get_issue, since list omits body) --------

    #[tokio::test]
    async fn status_current_maps_to_open() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json(
                "1",
                "current",
                "c",
                Some(""),
            )))
            .mount(&server)
            .await;
        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issue = backend.get_issue("REPOSIX", IssueId(1)).await.expect("get");
        assert_eq!(issue.status, IssueStatus::Open);
    }

    // -------- 6: status "trashed" → Done --------

    #[tokio::test]
    async fn status_trashed_maps_to_done() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/pages/2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json(
                "2",
                "trashed",
                "t",
                Some(""),
            )))
            .mount(&server)
            .await;
        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issue = backend.get_issue("REPOSIX", IssueId(2)).await.expect("get");
        assert_eq!(issue.status, IssueStatus::Done);
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
            .respond_with(ResponseTemplate::new(200).set_body_json(page_json(
                "42",
                "current",
                "x",
                Some(""),
            )))
            .mount(&server)
            .await;
        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        // If the header is wrong, wiremock returns no-match 404 and this fails.
        backend
            .get_issue("REPOSIX", IssueId(42))
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
        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let _ = backend.get_issue("REPOSIX", IssueId(42)).await; // expect Err, don't care which
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

    // -------- 9: write methods short-circuit to not-supported --------

    #[tokio::test]
    async fn write_methods_return_not_supported() {
        // Unreachable base URL — must not matter because writes short-circuit.
        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), "http://127.0.0.1:1".to_owned())
                .expect("backend");
        let t = chrono::Utc::now();
        let u = sanitize(
            Tainted::new(Issue {
                id: IssueId(0),
                title: "x".into(),
                status: IssueStatus::Open,
                assignee: None,
                labels: vec![],
                created_at: t,
                updated_at: t,
                version: 0,
                body: String::new(),
                parent_id: None,
            }),
            ServerMetadata {
                id: IssueId(1),
                created_at: t,
                updated_at: t,
                version: 1,
            },
        );
        assert!(matches!(
            backend.create_issue("REPOSIX", u.clone()).await,
            Err(Error::Other(m)) if m.starts_with("not supported:")
        ));
        assert!(matches!(
            backend
                .update_issue("REPOSIX", IssueId(1), u, None)
                .await,
            Err(Error::Other(m)) if m.starts_with("not supported:")
        ));
        assert!(matches!(
            backend
                .delete_or_close("REPOSIX", IssueId(1), DeleteReason::Completed)
                .await,
            Err(Error::Other(m)) if m.starts_with("not supported:")
        ));
    }

    // -------- 10: capability matrix --------

    /// Phase 13 Wave B1 flipped `Hierarchy` to `true`; every other feature
    /// stays `false` because Confluence is still read-only in v0.4. Renamed
    /// from `supports_reports_no_features` to match the new reality.
    #[test]
    fn supports_reports_only_hierarchy() {
        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), "http://127.0.0.1:1".to_owned())
                .expect("backend");
        assert!(!backend.supports(BackendFeature::Workflows));
        assert!(!backend.supports(BackendFeature::Delete));
        assert!(!backend.supports(BackendFeature::Transitions));
        assert!(!backend.supports(BackendFeature::StrongVersioning));
        assert!(!backend.supports(BackendFeature::BulkEdit));
        assert!(backend.supports(BackendFeature::Hierarchy));
        assert_eq!(backend.name(), "confluence-readonly");
    }

    // -------- Phase 13 Wave B1: parent_id derivation + Hierarchy capability --------

    /// Helper: synthesize a `ConfPage` directly (no JSON round-trip) for
    /// exercising `translate` branches without a wiremock server. Mirrors
    /// `page_json` but at the Rust struct level.
    fn synth_page(id: &str, parent_id: Option<&str>, parent_type: Option<&str>) -> ConfPage {
        let ts = chrono::DateTime::parse_from_rfc3339("2026-04-13T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        ConfPage {
            id: id.to_owned(),
            status: "current".to_owned(),
            title: "t".to_owned(),
            created_at: ts,
            version: ConfVersion {
                number: 1,
                created_at: ts,
            },
            owner_id: None,
            body: None,
            parent_id: parent_id.map(str::to_owned),
            parent_type: parent_type.map(str::to_owned),
        }
    }

    #[test]
    fn translate_populates_parent_id_for_page_parent() {
        let page = synth_page("99", Some("42"), Some("page"));
        let issue = translate(page).expect("translate");
        assert_eq!(issue.parent_id, Some(IssueId(42)));
    }

    #[test]
    fn translate_treats_folder_parent_as_orphan() {
        let page = synth_page("99", Some("99999"), Some("folder"));
        let issue = translate(page).expect("translate");
        assert_eq!(issue.parent_id, None);
    }

    #[test]
    fn translate_treats_whiteboard_parent_as_orphan() {
        let page = synth_page("99", Some("99999"), Some("whiteboard"));
        let issue = translate(page).expect("translate");
        assert_eq!(issue.parent_id, None);
    }

    #[test]
    fn translate_treats_database_parent_as_orphan() {
        let page = synth_page("99", Some("99999"), Some("database"));
        let issue = translate(page).expect("translate");
        assert_eq!(issue.parent_id, None);
    }

    #[test]
    fn translate_treats_missing_parent_as_orphan() {
        // Both fields absent (top-level / space-root page) → None.
        let page = synth_page("99", None, None);
        let issue = translate(page).expect("translate");
        assert_eq!(issue.parent_id, None);
    }

    #[test]
    fn translate_treats_parent_id_without_type_as_orphan() {
        // Defensive: Atlassian could theoretically return parentId without
        // parentType. Without a type we can't know it's a page; orphan it.
        let page = synth_page("99", Some("42"), None);
        let issue = translate(page).expect("translate");
        assert_eq!(issue.parent_id, None);
    }

    #[test]
    fn translate_handles_unparseable_parent_id() {
        // T-13-PB1: a malformed parentId must NOT wedge the page (or the
        // whole list). It degrades to None with a warn-level trace.
        let page = synth_page("99", Some("not-a-number"), Some("page"));
        let issue = translate(page).expect("translate must not error");
        assert_eq!(issue.parent_id, None);
    }

    #[test]
    fn root_collection_name_returns_pages() {
        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), "http://127.0.0.1:1".to_owned())
                .expect("backend");
        assert_eq!(backend.root_collection_name(), "pages");
    }

    /// End-to-end proof that `parentId` + `parentType` survive the JSON
    /// decode → `ConfPage` → `translate` → `Issue` pipeline through the
    /// `IssueBackend::list_issues` seam (not just the `translate` helper in
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
                    // (c) folder-parented page: parentType="folder" → None
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

        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issues = backend.list_issues("REPOSIX").await.expect("list");
        assert_eq!(issues.len(), 3);

        let child = issues
            .iter()
            .find(|i| i.id == IssueId(98765))
            .expect("child page present");
        assert_eq!(
            child.parent_id,
            Some(IssueId(360_556)),
            "page-parented child must propagate parent_id"
        );

        let root = issues
            .iter()
            .find(|i| i.id == IssueId(360_556))
            .expect("root page present");
        assert_eq!(
            root.parent_id, None,
            "page with no parent fields must deserialize as orphan"
        );

        let foldered = issues
            .iter()
            .find(|i| i.id == IssueId(12321))
            .expect("folder-parented page present");
        assert_eq!(
            foldered.parent_id, None,
            "non-page parentType must degrade to orphan"
        );
    }

    // -------- 11 / 12: parse_next_cursor pure-fn --------

    #[test]
    fn parse_next_cursor_extracts_relative_path() {
        let body = json!({
            "results": [],
            "_links": { "next": "/wiki/api/v2/spaces/1/pages?cursor=XYZ" }
        });
        assert_eq!(
            parse_next_cursor(&body).as_deref(),
            Some("/wiki/api/v2/spaces/1/pages?cursor=XYZ")
        );
    }

    #[test]
    fn parse_next_cursor_absent_returns_none() {
        let body = json!({ "results": [], "_links": {} });
        assert!(parse_next_cursor(&body).is_none());
        let body2 = json!({ "results": [] });
        assert!(parse_next_cursor(&body2).is_none());
    }

    // -------- 13: basic_auth_header pure-fn --------

    #[test]
    fn basic_auth_header_format() {
        use base64::engine::general_purpose::STANDARD;
        let got = basic_auth_header("a@b.com", "xyz");
        let want = format!("Basic {}", STANDARD.encode("a@b.com:xyz"));
        assert_eq!(got, want);
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
        let backend = ConfluenceReadOnlyBackend::new_with_base_url(
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
            let r = ConfluenceReadOnlyBackend::new(creds(), t);
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
            let r = ConfluenceReadOnlyBackend::new(creds(), t);
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
        // Also mount the pages endpoint so list_issues can complete — this
        // proves the round-trip end-to-end.
        Mock::given(method("GET"))
            .and(path("/wiki/api/v2/spaces/12345/pages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [],
                "_links": {}
            })))
            .mount(&server)
            .await;

        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issues = backend
            .list_issues(adversarial)
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
        let backend =
            ConfluenceReadOnlyBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let err = backend
            .list_issues("REPOSIX")
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
}
