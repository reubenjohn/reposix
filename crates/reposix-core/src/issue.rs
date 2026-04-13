//! Issue (the unit a FUSE file represents) types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// A non-negative integer issue identifier within a project. We deliberately avoid u32 so the
/// type signals that the simulator may use a much larger ID space without API breakage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IssueId(pub u64);

impl std::fmt::Display for IssueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Workflow state. Modeled after a Jira-flavored superset of GitHub Issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    /// Newly filed, not yet triaged.
    Open,
    /// Actively being worked on.
    InProgress,
    /// Awaiting review.
    InReview,
    /// Closed successfully.
    Done,
    /// Closed without resolution.
    WontFix,
}

impl IssueStatus {
    /// Render to canonical YAML scalar form.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::InReview => "in_review",
            Self::Done => "done",
            Self::WontFix => "wont_fix",
        }
    }
}

/// A single issue. Serialized to disk as a Markdown file with YAML frontmatter; the body of the
/// markdown is `body`, everything else lives in the frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Project-scoped unique id.
    pub id: IssueId,
    /// Single-line summary.
    pub title: String,
    /// Workflow state.
    pub status: IssueStatus,
    /// Optional assignee (free-form string; e.g. `"agent-alpha"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// Free-form labels.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Server-managed creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Server-managed last-update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Optimistic-concurrency version. Bumped on every server-side update.
    #[serde(default)]
    pub version: u64,
    /// Free-form Markdown body.
    #[serde(default)]
    pub body: String,
}

/// Frontmatter helpers — round-trip an [`Issue`] through `---\n<yaml>\n---\n<body>` form.
pub mod frontmatter {
    use super::{DateTime, Error, Issue, Result, Utc};
    use serde::{Deserialize, Serialize};

    /// Subset of [`Issue`] that lives inside the frontmatter (everything except `body`).
    #[derive(Debug, Serialize, Deserialize)]
    struct Frontmatter {
        id: super::IssueId,
        title: String,
        status: super::IssueStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        assignee: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        labels: Vec<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        #[serde(default)]
        version: u64,
    }

    /// Render an [`Issue`] to its on-disk form.
    ///
    /// # Errors
    /// Returns [`Error::Yaml`] if the frontmatter cannot be serialized (e.g. a label contains
    /// a character no YAML representation can encode).
    pub fn render(issue: &Issue) -> Result<String> {
        let fm = Frontmatter {
            id: issue.id,
            title: issue.title.clone(),
            status: issue.status,
            assignee: issue.assignee.clone(),
            labels: issue.labels.clone(),
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            version: issue.version,
        };
        let yaml = serde_yaml::to_string(&fm)?;
        let mut out = String::with_capacity(yaml.len() + issue.body.len() + 16);
        out.push_str("---\n");
        out.push_str(&yaml);
        out.push_str("---\n");
        out.push_str(&issue.body);
        if !issue.body.ends_with('\n') {
            out.push('\n');
        }
        Ok(out)
    }

    /// Parse on-disk Markdown+frontmatter into an [`Issue`].
    ///
    /// # Errors
    /// Returns [`Error::InvalidIssue`] if the file does not start with a `---` fence or the
    /// fence is malformed; [`Error::Yaml`] if the frontmatter YAML is invalid.
    pub fn parse(text: &str) -> Result<Issue> {
        let body_start;
        let yaml = if let Some(rest) = text.strip_prefix("---\n") {
            // Find closing fence — accept either `---\n` or `---` at EOF.
            if let Some(end) = rest.find("\n---\n") {
                body_start = end + 5; // length of "\n---\n"
                &rest[..end]
            } else if let Some(end) = rest.find("\n---") {
                if rest[end + 4..].is_empty() {
                    body_start = end + 4;
                    &rest[..end]
                } else {
                    return Err(Error::InvalidIssue(
                        "frontmatter close fence not followed by newline".into(),
                    ));
                }
            } else {
                return Err(Error::InvalidIssue(
                    "frontmatter open without close fence".into(),
                ));
            }
        } else {
            return Err(Error::InvalidIssue("missing frontmatter open fence".into()));
        };
        let fm: Frontmatter = serde_yaml::from_str(yaml)?;
        let body = rest_after(text, body_start).to_owned();
        Ok(Issue {
            id: fm.id,
            title: fm.title,
            status: fm.status,
            assignee: fm.assignee,
            labels: fm.labels,
            created_at: fm.created_at,
            updated_at: fm.updated_at,
            version: fm.version,
            body,
        })
    }

    /// Re-extract just the YAML map and re-emit as canonical JSON. Useful for the remote helper
    /// when diffing frontmatter across versions.
    ///
    /// # Errors
    /// Propagates any error from [`parse`] or from JSON serialization.
    pub fn yaml_to_json_value(text: &str) -> Result<serde_json::Value> {
        let issue = parse(text)?;
        Ok(serde_json::to_value(issue)?)
    }

    fn rest_after(text: &str, body_start_in_rest: usize) -> &str {
        // `text` begins with "---\n", which is 4 bytes; the parse function recorded
        // body_start as offset within `rest = text[4..]`, so add 4 to index into `text`.
        let abs = 4 + body_start_in_rest;
        &text[abs.min(text.len())..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample() -> Issue {
        let t = Utc.with_ymd_and_hms(2026, 4, 13, 0, 0, 0).unwrap();
        Issue {
            id: IssueId(123),
            title: "thing is broken".into(),
            status: IssueStatus::InProgress,
            assignee: Some("agent-alpha".into()),
            labels: vec!["bug".into(), "p1".into()],
            created_at: t,
            updated_at: t,
            version: 3,
            body: "Steps to reproduce:\n1. do the thing\n2. observe brokenness\n".into(),
        }
    }

    #[test]
    fn frontmatter_roundtrips() {
        let original = sample();
        let rendered = frontmatter::render(&original).expect("render");
        assert!(rendered.starts_with("---\n"));
        let parsed = frontmatter::parse(&rendered).expect("parse");
        assert_eq!(parsed.id, original.id);
        assert_eq!(parsed.title, original.title);
        assert_eq!(parsed.status as u8, original.status as u8);
        assert_eq!(parsed.body, original.body);
        assert_eq!(parsed.version, original.version);
    }

    #[test]
    fn missing_open_fence_is_rejected() {
        let bad = "no frontmatter here\n";
        assert!(matches!(
            frontmatter::parse(bad),
            Err(Error::InvalidIssue(_))
        ));
    }
}
