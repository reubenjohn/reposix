//! JIRA wire types and credentials.
//!
//! Holds the deserialization shapes for JIRA REST v3 responses, the
//! [`JiraCreds`] struct (with redacted `Debug` impl), and the public
//! constants advertised by the connector ([`CAPABILITIES`],
//! [`DEFAULT_BASE_URL_FORMAT`]).

use std::time::Duration;

use serde::Deserialize;

/// Maximum time we'll wait for a rate-limit reset before surfacing the
/// exhaustion as an error. Caps worst-case call latency.
pub(crate) const MAX_RATE_LIMIT_SLEEP: Duration = Duration::from_secs(60);

/// Max issues we'll page through in one `list_records` call.
pub(crate) const MAX_ISSUES_PER_LIST: usize = 500;

/// Page size for the JIRA search endpoint (max 100 per request).
pub(crate) const PAGE_SIZE: usize = 100;

/// Format string for the default production base URL.
pub const DEFAULT_BASE_URL_FORMAT: &str = "https://{tenant}.atlassian.net";

/// Capability matrix row published by this backend for `reposix doctor`.
///
/// JIRA Cloud is read-only in v0.11.x: the connector lists, gets, and
/// surfaces issues, but `create_record` / `update_record` / `delete_or_close`
/// still return `Error::Other("not supported: ...")`. Comments are not
/// round-tripped (JIRA exposes them through a separate comments API rather
/// than a body field), and concurrency is timestamp-based — write-after-read
/// would race against concurrent edits if the write path were enabled.
pub const CAPABILITIES: reposix_core::BackendCapabilities = reposix_core::BackendCapabilities::new(
    true,
    false,
    false,
    false,
    reposix_core::CommentSupport::None,
    reposix_core::VersioningModel::Timestamp,
);

/// JIRA fields to request in search and get-issue requests.
pub(crate) const JIRA_FIELDS: &[&str] = &[
    "id",
    "key",
    "summary",
    "description",
    "status",
    "resolution",
    "assignee",
    "labels",
    "created",
    "updated",
    "parent",
    "issuetype",
    "priority",
];

// ─── Credentials ─────────────────────────────────────────────────────────────

/// Credentials for Atlassian JIRA Cloud.
///
/// `api_token` is excluded from `Debug` output — see [`JiraCreds`] `impl Debug`.
#[derive(Clone)]
pub struct JiraCreds {
    /// Atlassian account email address.
    pub email: String,
    /// JIRA API token. Never printed in debug output.
    pub api_token: String,
}

impl std::fmt::Debug for JiraCreds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JiraCreds")
            .field("email", &self.email)
            .field("api_token", &"<redacted>")
            .finish()
    }
}

// ─── JIRA API response structs ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct JiraSearchResponse {
    pub(crate) issues: Vec<JiraIssue>,
    #[serde(rename = "isLast")]
    pub(crate) is_last: Option<bool>,
    #[serde(rename = "nextPageToken")]
    pub(crate) next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JiraIssue {
    pub(crate) id: String,
    pub(crate) key: String,
    pub(crate) fields: JiraFields,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JiraFields {
    pub(crate) summary: Option<String>,
    pub(crate) description: Option<serde_json::Value>,
    pub(crate) status: JiraStatus,
    pub(crate) resolution: Option<JiraResolution>,
    pub(crate) assignee: Option<JiraAssignee>,
    #[serde(default)]
    pub(crate) labels: Vec<String>,
    pub(crate) created: chrono::DateTime<chrono::FixedOffset>,
    pub(crate) updated: chrono::DateTime<chrono::FixedOffset>,
    pub(crate) parent: Option<JiraParent>,
    pub(crate) issuetype: JiraIssueType,
    pub(crate) priority: Option<JiraPriority>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JiraStatus {
    pub(crate) name: String,
    #[serde(rename = "statusCategory")]
    pub(crate) status_category: JiraStatusCategory,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JiraStatusCategory {
    pub(crate) key: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JiraResolution {
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JiraAssignee {
    #[serde(rename = "displayName")]
    pub(crate) display_name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JiraParent {
    pub(crate) id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JiraIssueType {
    pub(crate) name: String,
    #[serde(rename = "hierarchyLevel")]
    pub(crate) hierarchy_level: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JiraPriority {
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct JiraErrorResponse {
    #[serde(rename = "errorMessages", default)]
    pub(crate) error_messages: Vec<String>,
}
