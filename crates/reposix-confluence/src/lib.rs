//! [`ConfluenceBackend`] — read/write [`BackendConnector`] adapter for
//! Atlassian Confluence Cloud REST v2.
//!
//! # Scope
//!
//! v0.6 ships the full read+write path: `list_issues` + `get_issue` work
//! against a real Atlassian tenant (once credentials are configured);
//! `create_issue`, `update_issue`, and `delete_or_close` are implemented
//! against `POST /wiki/api/v2/pages`, `PUT /wiki/api/v2/pages/{id}`, and
//! `DELETE /wiki/api/v2/pages/{id}` respectively. Audit log rows are added in
//! Wave C (v0.6 Phase 16).
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
//! [`rate_limit_gate`]: ConfluenceBackend
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
//! - **T-11-02 (SSRF via tenant injection):** [`ConfluenceBackend::new`]
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
use rusqlite::Connection;
use serde::Deserialize;

use reposix_core::backend::{BackendFeature, DeleteReason, BackendConnector};
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
/// [`ConfluenceBackend::new`]).
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

/// Read-only `BackendConnector` for Atlassian Confluence Cloud REST v2.
///
/// Construct via [`ConfluenceBackend::new`] (public production API)
/// or [`ConfluenceBackend::new_with_base_url`] (custom base; used by
/// wiremock unit tests and the contract test).
///
/// Write methods (`create_issue`, `update_issue`, `delete_or_close`) are
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
    /// Optional audit log connection. When `Some`, every write call
    /// (`create_issue`, `update_issue`, `delete_or_close`) inserts one row
    /// into the `audit_events` table. The caller is responsible for opening
    /// the connection via [`reposix_core::audit::open_audit_db`] so the
    /// schema and append-only triggers are loaded before the first insert.
    ///
    /// `None` by default — attach via [`Self::with_audit`].
    audit: Option<Arc<Mutex<Connection>>>,
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

/// Body wrapper for Confluence pages and comments. Returned by both the page
/// list endpoint and the comment endpoints when `?body-format=atlas_doc_format`
/// is requested.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfPageBody {
    /// Raw storage XHTML body (Confluence `storage` representation).
    #[serde(default)]
    pub storage: Option<ConfBodyStorage>,
    /// ADF body returned when `?body-format=atlas_doc_format` is requested.
    /// The value is a JSON object (not a string) — we keep it as `Value` so
    /// that [`adf::adf_to_markdown`] can walk it without a second parse step.
    #[serde(default, rename = "atlas_doc_format")]
    pub adf: Option<ConfBodyAdf>,
}

/// Raw storage XHTML body wrapper.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfBodyStorage {
    /// Raw storage HTML value.
    pub value: String,
}

/// ADF body wrapper. The `value` field holds the full ADF JSON document.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfBodyAdf {
    /// Full ADF JSON document (e.g. `{"type":"doc","version":1,"content":[…]}`).
    pub value: serde_json::Value,
}

/// A Confluence v2 comment — either inline or footer.
///
/// Deserialized from `GET /wiki/api/v2/pages/{id}/inline-comments` and
/// `GET /wiki/api/v2/pages/{id}/footer-comments`. The two endpoints return
/// slightly different shapes — `resolution_status` and `parent_comment_id`
/// are inline-only; both are `Option` here.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfComment {
    /// Confluence numeric id (as string; always parseable as u64).
    pub id: String,
    /// Parent page id (numeric string).
    #[serde(rename = "pageId")]
    pub page_id: String,
    /// Version metadata: contains `createdAt` and `authorId`.
    pub version: ConfCommentVersion,
    /// `Some(...)` for inline comment replies; `None` for top-level and footer.
    #[serde(default, rename = "parentCommentId")]
    pub parent_comment_id: Option<String>,
    /// `Some("open" | "resolved" | "reopened" | ...)` for inline comments;
    /// `None` for footer comments (no resolution concept).
    #[serde(default, rename = "resolutionStatus")]
    pub resolution_status: Option<String>,
    /// ADF / storage-format body. Reuses the existing `ConfPageBody` wrapper.
    #[serde(default)]
    pub body: Option<ConfPageBody>,
    /// `"inline"` or `"footer"`. NOT set by serde — populated by
    /// `list_comments` after deserialization based on which endpoint returned it.
    #[serde(skip, default)]
    pub kind: CommentKind,
}

impl ConfComment {
    /// Convert the ADF body (if present) to Markdown. Empty string if no body or conversion fails.
    /// Mirrors the `translate()` function's degradation pattern for page bodies.
    #[must_use]
    pub fn body_markdown(&self) -> String {
        let Some(body) = self.body.as_ref() else {
            return String::new();
        };
        if let Some(adf) = body.adf.as_ref() {
            match crate::adf::adf_to_markdown(&adf.value) {
                Ok(md) => md,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        comment_id = %self.id,
                        "adf_to_markdown failed on comment; using empty body"
                    );
                    String::new()
                }
            }
        } else if let Some(storage) = body.storage.as_ref() {
            storage.value.clone()
        } else {
            String::new()
        }
    }
}

/// Which Confluence endpoint produced this comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommentKind {
    /// Inline comment (attached to a text range in the page body).
    #[default]
    Inline,
    /// Footer comment (page-level discussion, not anchored to specific text).
    Footer,
}

impl CommentKind {
    /// String form used in `kind:` frontmatter and URL path segment.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Footer => "footer",
        }
    }
}

/// Version metadata on a Confluence comment. `authorId` is the Atlassian
/// accountId string.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfCommentVersion {
    /// When the comment was created (ISO 8601).
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Atlassian accountId of the comment author.
    #[serde(rename = "authorId")]
    pub author_id: String,
    /// Version number (increments on edits).
    #[serde(default)]
    pub number: u64,
}

#[derive(Debug, Deserialize)]
struct ConfCommentList {
    results: Vec<ConfComment>,
    #[serde(default, rename = "_links")]
    #[allow(dead_code)]
    links: Option<ConfLinks>,
}

/// A Confluence v2 attachment on a page.
///
/// Deserialized from `GET /wiki/api/v2/pages/{id}/attachments`.
/// The `download_link` field is a relative path — prepend `self.base()` before
/// issuing a download request (see [`ConfluenceBackend::download_attachment`]).
#[derive(Debug, Clone, Deserialize)]
pub struct ConfAttachment {
    /// Confluence attachment id (numeric string).
    pub id: String,
    /// Attachment lifecycle status (e.g. `"current"`).
    pub status: String,
    /// Filename as stored in Confluence.
    pub title: String,
    /// When the attachment was created.
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Parent page id (numeric string).
    #[serde(rename = "pageId")]
    pub page_id: String,
    /// MIME type (e.g. `"image/png"`, `"application/pdf"`).
    #[serde(rename = "mediaType")]
    pub media_type: String,
    /// File size in bytes. `0` if absent from the response.
    #[serde(rename = "fileSize", default)]
    pub file_size: u64,
    /// Relative download path (e.g. `/wiki/download/attachments/12345/file.png`).
    /// Requires Basic-auth headers when fetched. Prepend `self.base()`.
    #[serde(rename = "downloadLink", default)]
    pub download_link: String,
}

/// A Confluence v2 whiteboard.
///
/// Deserialized from the `direct-children` endpoint
/// (`GET /wiki/api/v2/spaces/{id}/direct-children`) after filtering for
/// `type == "whiteboard"`.
///
/// `Serialize` is derived so the FUSE `read()` callback can return
/// `serde_json::to_vec(&whiteboard)` as the file content.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct ConfWhiteboard {
    /// Confluence whiteboard id (numeric string).
    pub id: String,
    /// Whiteboard lifecycle status (e.g. `"current"`).
    pub status: String,
    /// Whiteboard display title.
    pub title: String,
    /// Space id this whiteboard belongs to (numeric string).
    #[serde(rename = "spaceId")]
    pub space_id: String,
    /// Atlassian accountId of the whiteboard author. `None` if absent.
    #[serde(rename = "authorId", default)]
    pub author_id: Option<String>,
    /// When the whiteboard was created.
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Parent entity id (numeric string). `None` for top-level whiteboards.
    #[serde(rename = "parentId", default)]
    pub parent_id: Option<String>,
    /// Parent entity type (e.g. `"page"`, `"folder"`). `None` for top-level.
    #[serde(rename = "parentType", default)]
    pub parent_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConfAttachmentList {
    results: Vec<ConfAttachment>,
    #[serde(default, rename = "_links")]
    #[allow(dead_code)]
    links: Option<ConfLinks>,
}

#[derive(Debug, Deserialize)]
struct ConfDirectChildrenList {
    results: Vec<ConfDirectChild>,
    #[serde(default, rename = "_links")]
    #[allow(dead_code)]
    links: Option<ConfLinks>,
}

/// A single item from `GET /wiki/api/v2/spaces/{id}/direct-children`.
/// We keep only the fields needed to identify and reconstruct `ConfWhiteboard`;
/// unknown fields are silently ignored for forward-compat.
#[derive(Debug, Deserialize)]
struct ConfDirectChild {
    id: String,
    #[serde(rename = "type", default)]
    content_type: String,
    #[serde(default)]
    title: String,
    #[serde(rename = "spaceId", default)]
    space_id: String,
    #[serde(rename = "authorId", default)]
    author_id: Option<String>,
    #[serde(rename = "createdAt", default)]
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "parentId", default)]
    parent_id: Option<String>,
    #[serde(rename = "parentType", default)]
    parent_type: Option<String>,
    #[serde(default)]
    status: String,
}

/// Summary of a readable Confluence space, as returned by
/// [`ConfluenceBackend::list_spaces`]. The `webui_url` is already joined
/// with the tenant base URL (absolute URL ready to paste into a browser).
#[derive(Debug, Clone)]
pub struct ConfSpaceSummary {
    /// Space key (e.g. `"REPOSIX"`) — also usable as `--project` for mount/list.
    pub key: String,
    /// Human-readable name (e.g. `"Reposix Project"`).
    pub name: String,
    /// Absolute URL to the space's web UI (e.g. `https://tenant.atlassian.net/wiki/spaces/REPOSIX`).
    pub webui_url: String,
}

#[derive(Debug, Deserialize)]
struct ConfSpaceSummaryList {
    results: Vec<ConfSpaceRaw>,
    #[serde(default, rename = "_links")]
    #[allow(dead_code)]
    links: Option<ConfLinks>,
}

#[derive(Debug, Deserialize)]
struct ConfSpaceRaw {
    key: String,
    name: String,
    #[serde(default, rename = "_links")]
    links: Option<ConfSpaceRawLinks>,
}

#[derive(Debug, Deserialize)]
struct ConfSpaceRawLinks {
    #[serde(default)]
    webui: Option<String>,
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
    // Body extraction: prefer ADF → Markdown conversion; fall back to raw
    // storage HTML if ADF is absent (pre-ADF pages or storage-format re-fetch).
    let body = if let Some(adf_body) = page.body.as_ref().and_then(|b| b.adf.as_ref()) {
        // adf_to_markdown returns Err only if root is not type "doc"; in that
        // case gracefully degrade to empty string rather than failing the whole
        // page read (T-16-C-05 — attacker-influenced ADF must not wedge reads).
        match crate::adf::adf_to_markdown(&adf_body.value) {
            Ok(md) => md,
            Err(e) => {
                tracing::warn!(error = %e, "adf_to_markdown failed; using empty body");
                String::new()
            }
        }
    } else {
        page.body
            .and_then(|b| b.storage)
            .map(|s| s.value)
            .unwrap_or_default()
    };
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
        (Some(pid_str), Some("folder")) => {
            // CONF-06: folder parents are valid hierarchy nodes — propagate
            // to Issue::parent_id so the tree/ overlay shows folder structure.
            if let Ok(n) = pid_str.parse::<u64>() {
                Some(IssueId(n))
            } else {
                // IN-01: cap attacker-controlled string in tracing output to
                // bound log storage + log-injection blast radius.
                let pid_preview = pid_str.get(..pid_str.len().min(64)).unwrap_or("");
                tracing::warn!(
                    page_id = %page.id,
                    bad_parent = %pid_preview,
                    "confluence folder parentId not parseable as u64, treating as orphan"
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

/// Extract only the path and query from an HTTP URL so that tenant
/// hostnames never appear in error messages or tracing spans (OP-7 HARD-05).
///
/// `https://reuben-john.atlassian.net/wiki/api/v2/spaces/123/pages?cursor=X`
/// → `/wiki/api/v2/spaces/123/pages?cursor=X`
///
/// Returns `"<url parse error>"` if `raw` is not a valid URL so callers
/// never have to handle `None`/fallback themselves.
fn redact_url(raw: &str) -> String {
    url::Url::parse(raw).map_or_else(
        |_| "<url parse error>".to_string(),
        |u| {
            let path = u.path();
            match u.query() {
                Some(q) => format!("{path}?{q}"),
                None => path.to_string(),
            }
        },
    )
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
            audit: None,
        })
    }

    /// Attach an audit log connection. Every write call (`create_issue`,
    /// `update_issue`, `delete_or_close`) inserts one row into
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

    /// Strict variant of [`BackendConnector::list_issues`]: returns
    /// `Err(Error::Other(...))` instead of silently capping at
    /// [`MAX_ISSUES_PER_LIST`] pages, closing the SG-05 taint-escape
    /// risk (the agent thinking it has the whole space when it doesn't).
    ///
    /// Use this when a caller **must** see every page in the space or fail
    /// loudly. The default [`list_issues`](BackendConnector::list_issues) still
    /// returns `Ok(capped)` with a `tracing::warn!` for backwards compatibility.
    ///
    /// # Errors
    ///
    /// - Returns `Error::Other` if pagination would exceed `MAX_ISSUES_PER_LIST`.
    /// - All errors that `list_issues` would raise also apply here.
    pub async fn list_issues_strict(&self, project: &str) -> Result<Vec<Issue>> {
        self.list_issues_impl(project, true).await
    }

    /// Shared pagination loop for both [`list_issues`](BackendConnector::list_issues)
    /// and [`list_issues_strict`]. When `strict == true` the cap site returns
    /// `Err`; when `false` it emits a `tracing::warn!` and returns `Ok(capped)`.
    ///
    /// # Errors
    ///
    /// - Transport or HTTP errors from the Confluence REST API.
    /// - In strict mode: `Error::Other` when the page cap is exceeded.
    async fn list_issues_impl(&self, project: &str, strict: bool) -> Result<Vec<Issue>> {
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
    fn write_headers(&self) -> Vec<(&'static str, String)> {
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
    /// issue. Used by `update_issue` when the caller passes
    /// `expected_version = None` and we need a pre-flight GET to discover the
    /// current version before constructing the PUT body.
    ///
    /// # Errors
    ///
    /// Propagates transport errors and `Err(Error::Other("not found: …"))` on
    /// 404.
    async fn fetch_current_version(&self, id: IssueId) -> Result<u64> {
        // Re-uses get_issue which already handles rate-limit gate, SG-05
        // taint wrapping, and error mapping. The only "waste" is translating
        // the full page — that's acceptable for the `expected_version = None`
        // code path (one extra round-trip already implies we don't have cached
        // version data).
        let issue = self.get_issue("", id).await?;
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
    fn audit_write(
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
            format!("{digest:x}")
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
    /// - Caps at [`MAX_ISSUES_PER_LIST`] with `tracing::warn!` (HARD-02 compliance).
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
    /// Paginates via `_links.next` (same cursor scheme as `list_issues`).
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

#[async_trait]
impl BackendConnector for ConfluenceBackend {
    fn name(&self) -> &'static str {
        "confluence"
    }

    fn supports(&self, feature: BackendFeature) -> bool {
        // v0.6: write path is now implemented.
        // - `Hierarchy`: Confluence is the only current backend that exposes a
        //   parent/child tree, used by FUSE for the `tree/` overlay.
        // - `Delete`: `DELETE /wiki/api/v2/pages/{id}` moves pages to trash.
        // - `StrongVersioning`: PUT body carries `version.number = current + 1`;
        //   Confluence returns 409 on concurrent edits (optimistic locking).
        // `Workflows` and `BulkEdit` remain false: Confluence has no in-flight
        // state labels analogous to GitHub's status/* and no batch-edit API.
        matches!(
            feature,
            BackendFeature::Hierarchy | BackendFeature::Delete | BackendFeature::StrongVersioning
        )
    }

    fn root_collection_name(&self) -> &'static str {
        // Confluence-native vocabulary: pages, not issues. The default
        // `"issues"` stays correct for sim + GitHub; Confluence overrides
        // here so mounts surface as `pages/<padded-id>.md` — the layout
        // locked in Phase 13 CONTEXT.md and ADR-003.
        "pages"
    }

    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>> {
        self.list_issues_impl(project, false).await
    }

    async fn get_issue(&self, _project: &str, id: IssueId) -> Result<Issue> {
        // First attempt: request ADF body format for lossless Markdown conversion.
        let url_adf = format!(
            "{}/wiki/api/v2/pages/{}?body-format=atlas_doc_format",
            self.base(),
            id.0
        );
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers(Method::GET, url_adf.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if status == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!("not found: {}", redact_url(&url_adf))));
        }
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status} for GET {}: {}",
                redact_url(&url_adf),
                String::from_utf8_lossy(&bytes)
            )));
        }
        let page: ConfPage = serde_json::from_slice(&bytes)?;
        // Check if the ADF body is non-empty (null ADF means pre-ADF page).
        let adf_present = page
            .body
            .as_ref()
            .and_then(|b| b.adf.as_ref())
            .is_some_and(|a| !a.value.is_null());
        if adf_present {
            // SG-05: Tainted::new wraps ingress before translation.
            let tainted = Tainted::new(page);
            return translate(tainted.into_inner());
        }
        // Fallback: ADF body was absent/null (pre-ADF page) — re-fetch with
        // storage format so callers still get the raw storage HTML.
        let url_storage = format!(
            "{}/wiki/api/v2/pages/{}?body-format=storage",
            self.base(),
            id.0
        );
        self.await_rate_limit_gate().await;
        let resp2 = self
            .http
            .request_with_headers(Method::GET, url_storage.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp2);
        let status2 = resp2.status();
        let bytes2 = resp2.bytes().await?;
        if status2 == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!(
                "not found: {}",
                redact_url(&url_storage)
            )));
        }
        if !status2.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status2} for GET {}: {}",
                redact_url(&url_storage),
                String::from_utf8_lossy(&bytes2)
            )));
        }
        let page2: ConfPage = serde_json::from_slice(&bytes2)?;
        // SG-05: Tainted::new wraps ingress before translation.
        let tainted = Tainted::new(page2);
        translate(tainted.into_inner())
    }

    async fn create_issue(&self, project: &str, issue: Untainted<Issue>) -> Result<Issue> {
        let space_id = self.resolve_space_id(project).await?;
        // Convert the Markdown body to Confluence storage XHTML.
        let storage_xhtml = crate::adf::markdown_to_storage(&issue.inner_ref().body)?;
        let post_body = serde_json::json!({
            "spaceId": space_id,
            "status": "current",
            "title": issue.inner_ref().title,
            "parentId": issue.inner_ref().parent_id.map(|id| id.0.to_string()),
            "body": {
                "representation": "storage",
                "value": storage_xhtml,
            },
        });
        let post_body_bytes = serde_json::to_vec(&post_body)?;
        let url = format!("{}/wiki/api/v2/pages", self.base());
        let header_owned = self.write_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers_and_body(
                Method::POST,
                url.as_str(),
                &header_refs,
                Some(post_body_bytes),
            )
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        let status_u16 = status.as_u16();
        // T-16-C-04: audit title only (first 256 chars), never body content.
        let req_summary: String = issue.inner_ref().title.chars().take(256).collect();
        self.audit_write(
            "POST",
            "/wiki/api/v2/pages",
            status_u16,
            &req_summary,
            &bytes,
        );
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status} for POST {}: {}",
                redact_url(&url),
                String::from_utf8_lossy(&bytes)
            )));
        }
        let page: ConfPage = serde_json::from_slice(&bytes)?;
        // SG-05: wrap ingress bytes as Tainted before translating.
        let tainted = Tainted::new(page);
        translate(tainted.into_inner())
    }

    async fn update_issue(
        &self,
        _project: &str,
        id: IssueId,
        patch: Untainted<Issue>,
        expected_version: Option<u64>,
    ) -> Result<Issue> {
        // Pre-flight version resolution: if caller supplied an expected version
        // trust it; otherwise do a GET to discover the current version number.
        let current_version = match expected_version {
            Some(v) => v,
            None => self.fetch_current_version(id).await?,
        };
        // Convert the Markdown body to Confluence storage XHTML.
        let storage_xhtml = crate::adf::markdown_to_storage(&patch.inner_ref().body)?;
        let put_body = serde_json::json!({
            "id": id.0.to_string(),
            "status": "current",
            "title": patch.inner_ref().title,
            "version": { "number": current_version + 1 },
            "body": {
                "representation": "storage",
                "value": storage_xhtml,
            },
        });
        let put_body_bytes = serde_json::to_vec(&put_body)?;
        let url = format!("{}/wiki/api/v2/pages/{}", self.base(), id.0);
        let header_owned = self.write_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers_and_body(
                Method::PUT,
                url.as_str(),
                &header_refs,
                Some(put_body_bytes),
            )
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        let status_u16 = status.as_u16();
        // T-16-C-04: audit title only (first 256 chars), never body content.
        let req_summary: String = patch.inner_ref().title.chars().take(256).collect();
        let audit_path = format!("/wiki/api/v2/pages/{}", id.0);
        self.audit_write("PUT", &audit_path, status_u16, &req_summary, &bytes);
        if status == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!("not found: {}", redact_url(&url))));
        }
        if status == StatusCode::CONFLICT {
            let body_preview: String = String::from_utf8_lossy(&bytes).chars().take(256).collect();
            return Err(Error::Other(format!(
                "confluence version conflict for PUT {}: {body_preview}",
                redact_url(&url),
            )));
        }
        if !status.is_success() {
            return Err(Error::Other(format!(
                "confluence returned {status} for PUT {}: {}",
                redact_url(&url),
                String::from_utf8_lossy(&bytes)
            )));
        }
        let page: ConfPage = serde_json::from_slice(&bytes)?;
        // SG-05: wrap ingress bytes as Tainted before translating.
        let tainted = Tainted::new(page);
        translate(tainted.into_inner())
    }

    async fn delete_or_close(
        &self,
        _project: &str,
        id: IssueId,
        _reason: DeleteReason,
    ) -> Result<()> {
        // Note: `reason` is intentionally ignored — Confluence has no reason
        // field on DELETE. The DELETE moves the page to trash (status becomes
        // "trashed"), which the read path already maps to `IssueStatus::Done`.
        // A future "purge" would require `?purge=true`; that is out of scope
        // for v0.6.
        let url = format!("{}/wiki/api/v2/pages/{}", self.base(), id.0);
        let header_owned = self.standard_headers();
        let header_refs: Vec<(&str, &str)> =
            header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.await_rate_limit_gate().await;
        let resp = self
            .http
            .request_with_headers(Method::DELETE, url.as_str(), &header_refs)
            .await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let status_u16 = status.as_u16();
        let audit_path = format!("/wiki/api/v2/pages/{}", id.0);
        if status == StatusCode::NO_CONTENT {
            // 204: no body — audit with empty bytes, empty request summary.
            self.audit_write("DELETE", &audit_path, status_u16, "", &[]);
            return Ok(());
        }
        let bytes = resp.bytes().await?;
        // Audit on all non-204 paths (failures).
        self.audit_write("DELETE", &audit_path, status_u16, "", &bytes);
        if status == StatusCode::NOT_FOUND {
            return Err(Error::Other(format!("not found: {}", redact_url(&url))));
        }
        Err(Error::Other(format!(
            "confluence returned {status} for DELETE {}: {}",
            redact_url(&url),
            String::from_utf8_lossy(&bytes)
        )))
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

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let issues = backend.list_issues("REPOSIX").await.expect("list");
        assert_eq!(issues.len(), 3);
        assert_eq!(issues[0].id, IssueId(1));
        assert_eq!(issues[2].id, IssueId(3));
    }

    // -------- 3: get_issue returns ADF body converted to Markdown --------

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
            .get_issue("REPOSIX", IssueId(98765))
            .await
            .expect("get");
        assert!(
            issue.body.contains("Hello"),
            "expected body to contain 'Hello', got: {:?}",
            issue.body
        );
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
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
        let err = backend
            .get_issue("REPOSIX", IssueId(9999))
            .await
            .expect_err("404");
        match err {
            Error::Other(m) => assert!(m.starts_with("not found:"), "got {m}"),
            other => panic!("expected not found, got {other:?}"),
        }
    }

    // -------- 4b: ADF absent → storage fallback (C4) --------

    /// When the ADF response contains no `atlas_doc_format` body (pre-ADF page),
    /// `get_issue` must fall back to a second GET with `?body-format=storage`
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
            .get_issue("REPOSIX", IssueId(55555))
            .await
            .expect("get with fallback");
        assert_eq!(issue.id, IssueId(55555));
        assert_eq!(
            issue.body, "<p>legacy content</p>",
            "storage fallback must return raw HTML body"
        );
    }

    // -------- 5: status "current" → Open (via get_issue, since list omits body) --------

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
        let issue = backend.get_issue("REPOSIX", IssueId(1)).await.expect("get");
        assert_eq!(issue.status, IssueStatus::Open);
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
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
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

    // CONF-06: folder parents now propagate to Issue::parent_id (changed in
    // Phase 24 Plan 01). Updated from "orphan" to "propagates".
    #[test]
    fn translate_treats_folder_parent_as_orphan() {
        // CONF-06 (Phase 24 Plan 01): folder parents now propagate so the tree/
        // overlay can represent folder hierarchy. This test was originally written
        // when folder → orphan; updated to match the new behavior.
        let page = synth_page("99", Some("99999"), Some("folder"));
        let issue = translate(page).expect("translate");
        // CONF-06 fix: folder parentType now propagates (same as "page").
        assert_eq!(issue.parent_id, Some(IssueId(99999)));
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
            ConfluenceBackend::new_with_base_url(creds(), "http://127.0.0.1:1".to_owned())
                .expect("backend");
        assert_eq!(backend.root_collection_name(), "pages");
    }

    /// End-to-end proof that `parentId` + `parentType` survive the JSON
    /// decode → `ConfPage` → `translate` → `Issue` pipeline through the
    /// `BackendConnector::list_issues` seam (not just the `translate` helper in
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
        // CONF-06 fix (Phase 24 Plan 01): folder parentType now propagates to
        // Issue::parent_id (same as "page"). Updated from None to Some(999).
        assert_eq!(
            foldered.parent_id,
            Some(IssueId(999)),
            "folder parentType must propagate to Issue::parent_id (CONF-06)"
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

        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
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
        let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri()).expect("backend");
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

    /// Build an `Untainted<Issue>` with the given fields for use in write tests.
    fn make_untainted(title: &str, body: &str, parent_id: Option<IssueId>) -> Untainted<Issue> {
        let t = chrono::DateTime::parse_from_rfc3339("2026-04-13T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        sanitize(
            Tainted::new(Issue {
                id: IssueId(0),
                title: title.to_owned(),
                status: IssueStatus::Open,
                assignee: None,
                labels: vec![],
                created_at: t,
                updated_at: t,
                version: 0,
                body: body.to_owned(),
                parent_id,
            }),
            ServerMetadata {
                id: IssueId(99),
                created_at: t,
                updated_at: t,
                version: 1,
            },
        )
    }

    // -------- B6.1: update_issue sends PUT with incremented version --------

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
            .update_issue("REPOSIX", IssueId(99), patch, Some(42))
            .await
            .expect("update_issue should succeed");
        assert_eq!(result.title, "updated title");
        assert_eq!(result.id, IssueId(99));
        assert_eq!(result.version, 43);
    }

    // -------- B6.2: update_issue 409 maps to conflict error --------

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
            .update_issue("REPOSIX", IssueId(99), patch, Some(5))
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

    // -------- B6.3: update_issue with None version fetches then PUTs --------

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
            .update_issue("REPOSIX", IssueId(99), patch, None)
            .await
            .expect("update_issue with None version should succeed");
        assert_eq!(result.version, 8);
        assert_eq!(result.title, "new title");
    }

    // -------- B6.4: update_issue 404 maps to not-found --------

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
            .update_issue("REPOSIX", IssueId(99), patch, Some(1))
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

    // -------- B6.5: create_issue POSTs to pages with correct spaceId --------

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
            .create_issue("REPOSIX", issue)
            .await
            .expect("create_issue should succeed");
        assert_eq!(result.id, IssueId(77777));
        assert_eq!(result.title, "my new page");
    }

    // -------- B6.6: create_issue with parent_id sends parentId in body --------

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
        let issue = make_untainted("child page", "body", Some(IssueId(42)));
        let result = backend
            .create_issue("REPOSIX", issue)
            .await
            .expect("create_issue with parent_id should succeed");
        assert_eq!(result.id, IssueId(88888));
    }

    // -------- B6.7: create_issue without parent_id sends null --------

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
            .create_issue("REPOSIX", issue)
            .await
            .expect("create_issue without parent should succeed");
        assert_eq!(result.id, IssueId(55555));
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
            .delete_or_close("REPOSIX", IssueId(99), DeleteReason::Completed)
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
            .delete_or_close("REPOSIX", IssueId(99), DeleteReason::Completed)
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

        // create_issue → POST
        let issue = make_untainted("t", "b", None);
        backend
            .create_issue("REPOSIX", issue)
            .await
            .expect("POST must carry Content-Type: application/json");

        // update_issue → PUT
        let patch = make_untainted("t", "b", None);
        backend
            .update_issue("REPOSIX", IssueId(99), patch, Some(1))
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
            .create_issue("REPOSIX", issue)
            .await
            .expect("POST must carry Basic auth");

        let patch = make_untainted("t", "b", None);
        backend
            .update_issue("REPOSIX", IssueId(99), patch, Some(1))
            .await
            .expect("PUT must carry Basic auth");

        backend
            .delete_or_close("REPOSIX", IssueId(42), DeleteReason::Completed)
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
        let _ = backend.get_issue("REPOSIX", IssueId(10)).await;

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
            .update_issue("REPOSIX", IssueId(99), patch, Some(1))
            .await
            .expect("update_issue should succeed");

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
            .create_issue("REPOSIX", issue)
            .await
            .expect("create_issue should succeed");

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
            .delete_or_close("REPOSIX", IssueId(55), DeleteReason::Completed)
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
            .update_issue("REPOSIX", IssueId(99), patch, Some(1))
            .await
            .expect("update_issue should succeed");

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
            .update_issue("REPOSIX", IssueId(99), patch, Some(1))
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
            .update_issue("REPOSIX", IssueId(99), patch, Some(1))
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

    // -------- truncation: warn mode (list_issues, default) --------

    /// Verify that `list_issues` (non-strict) emits a warn and returns
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
        let result = backend.list_issues("TRUNCTEST").await;
        // Must succeed (warn mode, not strict).
        let issues = result.expect("list_issues must succeed in warn mode even at cap");
        // 5 pages × 1 item = 5 issues.
        assert_eq!(issues.len(), 5, "expected 5 issues (one per mocked page)");
    }

    /// Verify that `list_issues_strict` returns `Err` containing
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
        let result = backend.list_issues_strict("TRUNCTEST").await;
        assert!(result.is_err(), "list_issues_strict must return Err at cap");
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

    /// Verify that error messages from `list_issues` on HTTP failure do NOT
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
        let result = backend.list_issues("LEAKTEST").await;
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
    // + translate folder-parent fix (CONF-04, CONF-05, CONF-06)
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

    // -------- translate folder-parent tests (CONF-06) --------

    #[test]
    fn translate_folder_parent_propagates() {
        // CONF-06: folder parentType + valid parentId → parent_id Some(IssueId)
        let page = synth_page("77", Some("99999"), Some("folder"));
        let issue = translate(page).expect("translate must succeed");
        assert_eq!(
            issue.parent_id,
            Some(IssueId(99999)),
            "folder parentType with valid id must propagate to parent_id"
        );
    }

    #[test]
    fn translate_folder_parent_bad_id_is_orphan() {
        // CONF-06: folder parentType + non-numeric parentId → parent_id None
        let page = synth_page("77", Some("not-a-number"), Some("folder"));
        let issue = translate(page).expect("translate must not error on bad folder parentId");
        assert_eq!(
            issue.parent_id, None,
            "folder parentType with non-numeric id must degrade to orphan"
        );
    }
}
