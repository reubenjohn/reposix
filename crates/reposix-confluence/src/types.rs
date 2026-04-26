//! Confluence wire types and credentials.
//!
//! Holds the deserialization shapes for Confluence Cloud REST v2 responses,
//! the [`ConfluenceCreds`] struct (with redacted `Debug` impl), and the
//! public constants advertised by the connector ([`CAPABILITIES`],
//! [`DEFAULT_BASE_URL_FORMAT`]).

use std::time::Duration;

use serde::Deserialize;

/// Maximum time we'll wait for a rate-limit reset before surfacing the
/// exhaustion as an error. Caps worst-case call latency.
pub(crate) const MAX_RATE_LIMIT_SLEEP: Duration = Duration::from_secs(60);

/// Max issues we'll page through in one `list_records` call.
///
/// At [`PAGE_SIZE`] 100 that's 5 requests — enough for the REPOSIX demo space
/// and a bounded memory budget. Matches `reposix-github`'s cap.
pub(crate) const MAX_ISSUES_PER_LIST: usize = 500;

/// Page size to request from Confluence (1..=250 allowed; 100 is the sweet
/// spot for latency × payload size).
pub(crate) const PAGE_SIZE: usize = 100;

/// Format string for the default production base URL. Callers supply the
/// tenant subdomain (validated against DNS-label rules in
/// `ConfluenceBackend::new`).
pub const DEFAULT_BASE_URL_FORMAT: &str = "https://{tenant}.atlassian.net";

/// Capability matrix row published by this backend for `reposix doctor`.
///
/// Confluence Cloud supports the full read/write/delete surface for pages.
/// Comments live behind a separate REST endpoint (not the body) and are
/// not round-tripped in `git diff` today. Concurrency is strong via
/// `version.number` — PUT bodies carry `current + 1` and the server returns
/// 409 on conflict (optimistic locking).
pub const CAPABILITIES: reposix_core::BackendCapabilities = reposix_core::BackendCapabilities::new(
    true,
    true,
    true,
    true,
    reposix_core::CommentSupport::SeparateApi,
    reposix_core::VersioningModel::Strong,
);

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

/// Minimal shape of the `GET /wiki/api/v2/spaces?keys=KEY` response we
/// consume. `deny_unknown_fields` is deliberately NOT set — Atlassian
/// adds fields routinely and forward-compat matters.
#[derive(Debug, Deserialize)]
pub(crate) struct ConfSpaceList {
    pub(crate) results: Vec<ConfSpace>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ConfSpace {
    pub(crate) id: String,
}

/// `GET /wiki/api/v2/spaces/{id}/pages?limit=N` response shape.
#[derive(Debug, Deserialize)]
pub(crate) struct ConfPageList {
    pub(crate) results: Vec<ConfPage>,
    #[serde(default, rename = "_links")]
    pub(crate) links: Option<ConfLinks>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ConfLinks {
    // Mirrored by `parse_next_cursor`; serde pulls the same path through
    // `serde_json::Value`. Kept for documentation + deny-unknown-shape
    // reviewer cues. Allow `dead_code`: field is observed only via the
    // JSON-Value path so the compiler can't see the usage.
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) next: Option<String>,
}

/// A single page as returned by both the list endpoint (with `body: {}`
/// empty) and the single-page endpoint (with `body.storage.value` populated
/// when `?body-format=storage` is requested).
#[derive(Debug, Deserialize)]
pub(crate) struct ConfPage {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) title: String,
    #[serde(rename = "createdAt")]
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    pub(crate) version: ConfVersion,
    #[serde(default, rename = "ownerId")]
    pub(crate) owner_id: Option<String>,
    #[serde(default)]
    pub(crate) body: Option<ConfPageBody>,
    /// Confluence REST v2 `parentId` — numeric string referring to another
    /// entity in the content hierarchy. Only meaningful when `parent_type ==
    /// Some("page")`; for folders/whiteboards/databases we deliberately drop
    /// it in `translate` so the tree-builder treats those as orphans.
    /// `#[serde(default)]` keeps Phase-11 fixtures (no parent fields) decoding
    /// unchanged.
    #[serde(default, rename = "parentId")]
    pub(crate) parent_id: Option<String>,
    /// Confluence REST v2 `parentType` — one of `"page"`, `"folder"`,
    /// `"whiteboard"`, `"database"`, etc. Only the `"page"` case propagates
    /// into [`reposix_core::Record::parent_id`]; every other value is treated
    /// as a tree root (with a `tracing::debug!` trail) because reposix's
    /// hierarchy model is homogeneous (pages only).
    #[serde(default, rename = "parentType")]
    pub(crate) parent_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ConfVersion {
    pub(crate) number: u64,
    #[serde(rename = "createdAt")]
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
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
    /// that [`crate::adf::adf_to_markdown`] can walk it without a second
    /// parse step.
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
pub(crate) struct ConfCommentList {
    pub(crate) results: Vec<ConfComment>,
    #[serde(default, rename = "_links")]
    #[allow(dead_code)]
    pub(crate) links: Option<ConfLinks>,
}

/// A Confluence v2 attachment on a page.
///
/// Deserialized from `GET /wiki/api/v2/pages/{id}/attachments`.
/// The `download_link` field is a relative path — prepend `self.base()` before
/// issuing a download request (see `ConfluenceBackend::download_attachment`).
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
pub(crate) struct ConfAttachmentList {
    pub(crate) results: Vec<ConfAttachment>,
    #[serde(default, rename = "_links")]
    #[allow(dead_code)]
    pub(crate) links: Option<ConfLinks>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ConfDirectChildrenList {
    pub(crate) results: Vec<ConfDirectChild>,
    #[serde(default, rename = "_links")]
    #[allow(dead_code)]
    pub(crate) links: Option<ConfLinks>,
}

/// A single item from `GET /wiki/api/v2/spaces/{id}/direct-children`.
/// We keep only the fields needed to identify and reconstruct `ConfWhiteboard`;
/// unknown fields are silently ignored for forward-compat.
#[derive(Debug, Deserialize)]
pub(crate) struct ConfDirectChild {
    pub(crate) id: String,
    #[serde(rename = "type", default)]
    pub(crate) content_type: String,
    #[serde(default)]
    pub(crate) title: String,
    #[serde(rename = "spaceId", default)]
    pub(crate) space_id: String,
    #[serde(rename = "authorId", default)]
    pub(crate) author_id: Option<String>,
    #[serde(rename = "createdAt", default)]
    pub(crate) created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "parentId", default)]
    pub(crate) parent_id: Option<String>,
    #[serde(rename = "parentType", default)]
    pub(crate) parent_type: Option<String>,
    #[serde(default)]
    pub(crate) status: String,
}

/// Summary of a readable Confluence space, as returned by
/// `ConfluenceBackend::list_spaces`. The `webui_url` is already joined
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
pub(crate) struct ConfSpaceSummaryList {
    pub(crate) results: Vec<ConfSpaceRaw>,
    #[serde(default, rename = "_links")]
    #[allow(dead_code)]
    pub(crate) links: Option<ConfLinks>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ConfSpaceRaw {
    pub(crate) key: String,
    pub(crate) name: String,
    #[serde(default, rename = "_links")]
    pub(crate) links: Option<ConfSpaceRawLinks>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ConfSpaceRawLinks {
    #[serde(default)]
    pub(crate) webui: Option<String>,
}
