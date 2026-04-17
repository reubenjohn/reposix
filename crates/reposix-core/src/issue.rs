//! Issue (the unit a FUSE file represents) types.

use std::collections::BTreeMap;

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
    /// Parent in a hierarchy-supporting backend (currently Confluence only).
    ///
    /// Always `None` for sim and GitHub. When `Some`, is the parent page/issue
    /// id as reported by the backend. Used by `reposix-fuse` to synthesize the
    /// `tree/` overlay (Phase 13).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<IssueId>,
    /// Backend-specific metadata that does not fit the canonical 5-field schema.
    ///
    /// Keys are backend-defined strings (e.g. `"jira_key"`, `"issue_type"`).
    /// Values are arbitrary YAML-compatible scalars or nested structures.
    /// Empty map is omitted from serialized frontmatter (does not appear in `.md` files).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, serde_yaml::Value>,
}

/// Frontmatter helpers — round-trip an [`Issue`] through `---\n<yaml>\n---\n<body>` form.
pub mod frontmatter {
    use std::collections::BTreeMap;

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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_id: Option<super::IssueId>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        extensions: BTreeMap<String, serde_yaml::Value>,
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
            parent_id: issue.parent_id,
            extensions: issue.extensions.clone(),
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
            parent_id: fm.parent_id,
            extensions: fm.extensions,
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
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
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

    #[test]
    fn parent_id_roundtrips_through_json_when_some() {
        let mut iss = sample();
        iss.parent_id = Some(IssueId(42));
        let json = serde_json::to_string(&iss).unwrap();
        assert!(
            json.contains("\"parent_id\":42"),
            "expected `\"parent_id\":42` in JSON, got: {json}"
        );
        let back: Issue = serde_json::from_str(&json).unwrap();
        assert_eq!(back.parent_id, Some(IssueId(42)));
    }

    #[test]
    fn parent_id_omitted_when_none() {
        let iss = sample(); // parent_id: None
        let json = serde_json::to_string(&iss).unwrap();
        assert!(
            !json.contains("parent_id"),
            "parent_id should be omitted when None, got: {json}"
        );
    }

    #[test]
    fn parent_id_default_on_missing_field() {
        // Old JSON payload, no parent_id field at all — must deserialize with None.
        let json = r#"{"id":1,"title":"t","status":"open","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}"#;
        let iss: Issue = serde_json::from_str(json).unwrap();
        assert_eq!(iss.parent_id, None);
    }

    #[test]
    fn parent_id_roundtrips_through_frontmatter_when_some() {
        let mut iss = sample();
        iss.parent_id = Some(IssueId(777));
        let rendered = frontmatter::render(&iss).expect("render");
        assert!(
            rendered.contains("parent_id: 777"),
            "expected `parent_id: 777` in YAML, got: {rendered}"
        );
        let parsed = frontmatter::parse(&rendered).expect("parse");
        assert_eq!(parsed.parent_id, Some(IssueId(777)));
    }

    #[test]
    fn parent_id_omitted_from_frontmatter_when_none() {
        let iss = sample(); // parent_id: None
        let rendered = frontmatter::render(&iss).expect("render");
        assert!(
            !rendered.contains("parent_id"),
            "parent_id should be omitted from YAML when None, got: {rendered}"
        );
    }

    #[test]
    fn frontmatter_renders_parent_id_when_some() {
        // Plan 13-B3 SC-required test: the rendered YAML contains the exact line
        // `parent_id: 42` (serde_yaml emits numeric scalars unquoted).
        let mut iss = sample();
        iss.parent_id = Some(IssueId(42));
        let rendered = frontmatter::render(&iss).expect("render");
        assert!(
            rendered.contains("parent_id: 42\n"),
            "expected exact line `parent_id: 42` in YAML, got: {rendered}"
        );
    }

    #[test]
    fn frontmatter_parses_parent_id_when_present() {
        // A frontmatter block authored by a hierarchy-aware backend (e.g. Confluence)
        // round-trips through parse with the numeric id preserved.
        let text = "---\n\
id: 1\n\
title: child page\n\
status: open\n\
created_at: 2026-04-14T00:00:00Z\n\
updated_at: 2026-04-14T00:00:00Z\n\
version: 1\n\
parent_id: 42\n\
---\n\
body here.\n";
        let iss = frontmatter::parse(text).expect("parse");
        assert_eq!(iss.parent_id, Some(IssueId(42)));
        assert_eq!(iss.id, IssueId(1));
        assert_eq!(iss.title, "child page");
    }

    #[test]
    fn frontmatter_parses_legacy_without_parent_id() {
        // Fixture shape matches a pre-Phase-13 on-disk file: no `parent_id:` key at
        // all. The `#[serde(default)]` attribute must fill it in with `None` rather
        // than erroring. This is the load-bearing backward-compat test.
        let text = "---\n\
id: 1\n\
title: Legacy issue\n\
status: open\n\
created_at: 2025-01-01T00:00:00Z\n\
updated_at: 2025-01-01T00:00:00Z\n\
version: 1\n\
---\n\
Body goes here.\n";
        let iss = frontmatter::parse(text).expect("legacy frontmatter must parse");
        assert_eq!(iss.parent_id, None);
        assert_eq!(iss.title, "Legacy issue");
    }

    #[test]
    fn frontmatter_roundtrip_with_parent() {
        // Deep-equality roundtrip: parse(render(issue)) yields the same Issue.
        // Catches any drift between the public `Issue` struct and the private
        // `Frontmatter` DTO.
        let mut original = sample();
        original.parent_id = Some(IssueId(131_192));
        let rendered = frontmatter::render(&original).expect("render");
        let parsed = frontmatter::parse(&rendered).expect("parse");
        assert_eq!(parsed.id, original.id);
        assert_eq!(parsed.title, original.title);
        assert_eq!(parsed.status as u8, original.status as u8);
        assert_eq!(parsed.assignee, original.assignee);
        assert_eq!(parsed.labels, original.labels);
        assert_eq!(parsed.created_at, original.created_at);
        assert_eq!(parsed.updated_at, original.updated_at);
        assert_eq!(parsed.version, original.version);
        assert_eq!(parsed.body, original.body);
        assert_eq!(parsed.parent_id, Some(IssueId(131_192)));
    }

    #[test]
    fn frontmatter_roundtrip_without_parent() {
        // Deep-equality roundtrip for the None branch — verifies `skip_serializing_if`
        // doesn't accidentally drop other fields and that the deserialize default
        // kicks in on the return trip.
        let original = sample(); // parent_id: None
        let rendered = frontmatter::render(&original).expect("render");
        let parsed = frontmatter::parse(&rendered).expect("parse");
        assert_eq!(parsed.id, original.id);
        assert_eq!(parsed.title, original.title);
        assert_eq!(parsed.status as u8, original.status as u8);
        assert_eq!(parsed.assignee, original.assignee);
        assert_eq!(parsed.labels, original.labels);
        assert_eq!(parsed.created_at, original.created_at);
        assert_eq!(parsed.updated_at, original.updated_at);
        assert_eq!(parsed.version, original.version);
        assert_eq!(parsed.body, original.body);
        assert_eq!(parsed.parent_id, None);
    }

    #[test]
    fn extensions_empty_omitted_from_yaml() {
        // An Issue with no extensions must not emit the word "extensions" in YAML.
        let iss = sample(); // extensions: BTreeMap::new()
        let rendered = frontmatter::render(&iss).expect("render");
        assert!(
            !rendered.contains("extensions"),
            "empty extensions must be omitted from YAML, got: {rendered}"
        );
    }

    #[test]
    fn extensions_roundtrip() {
        // Non-empty extensions survive a render→parse cycle with value equality.
        let mut iss = sample();
        iss.extensions.insert("foo".into(), serde_yaml::Value::from(42_i64));
        iss.extensions.insert("bar".into(), serde_yaml::Value::from("x"));
        let rendered = frontmatter::render(&iss).expect("render");
        assert!(
            rendered.contains("extensions"),
            "non-empty extensions must appear in YAML, got: {rendered}"
        );
        let parsed = frontmatter::parse(&rendered).expect("parse");
        assert_eq!(
            parsed.extensions, iss.extensions,
            "extensions must round-trip through render/parse"
        );
    }

    #[test]
    fn extensions_defaults_to_empty_on_parse() {
        // A legacy frontmatter without an `extensions:` key must parse to an empty map.
        let text = "---\n\
id: 1\n\
title: Legacy issue\n\
status: open\n\
created_at: 2025-01-01T00:00:00Z\n\
updated_at: 2025-01-01T00:00:00Z\n\
version: 1\n\
---\n\
Body goes here.\n";
        let iss = frontmatter::parse(text).expect("legacy frontmatter must parse");
        assert!(
            iss.extensions.is_empty(),
            "extensions must default to empty on legacy parse, got: {:?}",
            iss.extensions
        );
    }
}
