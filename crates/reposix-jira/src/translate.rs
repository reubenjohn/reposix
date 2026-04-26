//! Pure helpers and DTO → [`Record`] translation.
//!
//! This module is the seam between the on-the-wire JIRA shapes (defined in
//! [`crate::types`]) and the canonical [`Record`] surface that
//! [`reposix_core::backend::BackendConnector`] consumers see. All functions
//! here are pure — no HTTP, no audit, no clock — so they're trivially
//! unit-testable without a mock server.

use std::collections::BTreeMap;

use reposix_core::{Error, Record, RecordId, RecordStatus, Result};

use crate::adf;
use crate::types::{JiraIssue, JiraResolution, JiraStatus};

/// Encode HTTP Basic auth header value.
///
/// # Panics
///
/// Does not panic — base64 encoding is infallible.
#[must_use]
pub fn basic_auth_header(email: &str, token: &str) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;
    format!("Basic {}", STANDARD.encode(format!("{email}:{token}")))
}

/// Validate a tenant subdomain against DNS-label rules.
///
/// Blocks SSRF patterns: empty string, length > 63, non-lowercase characters,
/// leading/trailing hyphens, and embedded dots that could escape the subdomain.
///
/// # Errors
///
/// Returns `Err(Error::Other(...))` if `tenant` fails any validation rule.
pub fn validate_tenant(tenant: &str) -> Result<()> {
    if tenant.is_empty() {
        return Err(Error::Other(
            "invalid jira tenant subdomain: empty string".into(),
        ));
    }
    if tenant.len() > 63 {
        return Err(Error::Other(format!(
            "invalid jira tenant subdomain: {tenant:?} (exceeds 63-char DNS label limit)"
        )));
    }
    // Must contain only lowercase alphanumeric and internal hyphens.
    // No dots (which would escape the subdomain), no leading/trailing hyphen.
    let all_valid = tenant
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    let no_leading_hyphen =
        tenant.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit());
    let no_trailing_hyphen =
        tenant.ends_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit());
    let no_dots = !tenant.contains('.');
    if !all_valid || !no_leading_hyphen || !no_trailing_hyphen || !no_dots {
        return Err(Error::Other(format!(
            "invalid jira tenant subdomain: {tenant:?} \
             (must be lowercase alphanumeric with internal hyphens only, no dots)"
        )));
    }
    Ok(())
}

/// Map a JIRA status + optional resolution to an [`RecordStatus`].
pub(crate) fn map_status(status: &JiraStatus, resolution: Option<&JiraResolution>) -> RecordStatus {
    // WontFix override: check resolution name first.
    if let Some(res) = resolution {
        let lower = res.name.to_lowercase();
        if lower.contains("won't")
            || lower.contains("wont")
            || lower.contains("not a bug")
            || lower.contains("duplicate")
            || lower.contains("cannot reproduce")
        {
            return RecordStatus::WontFix;
        }
    }
    // Primary mapping on statusCategory.key.
    match status.status_category.key.as_str() {
        "indeterminate" => {
            if status.name.to_lowercase().contains("review") {
                RecordStatus::InReview
            } else {
                RecordStatus::InProgress
            }
        }
        "done" => RecordStatus::Done,
        _ => RecordStatus::Open, // safe fallback for unknown categories
    }
}

/// Translate a raw `JiraIssue` (from network) into a canonical [`Record`].
///
/// Consumes the input — call this after `Tainted::into_inner()`.
///
/// # Errors
///
/// Returns `Err` if the JIRA numeric issue ID cannot be parsed as `u64`.
pub(crate) fn translate(raw: JiraIssue) -> Result<Record> {
    let id = RecordId(
        raw.id
            .parse::<u64>()
            .map_err(|e| Error::Other(format!("invalid jira id {:?}: {e}", raw.id)))?,
    );
    let title = raw.fields.summary.unwrap_or_default();
    let status = map_status(&raw.fields.status, raw.fields.resolution.as_ref());
    let description_json = raw.fields.description.unwrap_or(serde_json::Value::Null);
    // Use adf_to_markdown for rich read path; fall back to plain text on error.
    let body = if description_json.is_null() {
        String::new()
    } else {
        adf::adf_to_markdown(&description_json)
            .unwrap_or_else(|_| adf::adf_to_plain_text(&description_json))
    };
    let created_at = raw.fields.created.with_timezone(&chrono::Utc);
    let updated_at = raw.fields.updated.with_timezone(&chrono::Utc);
    #[allow(clippy::cast_sign_loss)]
    let version = raw.fields.updated.timestamp_millis().unsigned_abs();
    let assignee = raw.fields.assignee.map(|a| a.display_name);
    let labels = raw.fields.labels;
    let parent_id = raw
        .fields
        .parent
        .as_ref()
        .and_then(|p| p.id.parse::<u64>().ok())
        .map(RecordId);

    let mut extensions: BTreeMap<String, serde_yaml::Value> = BTreeMap::new();
    extensions.insert(
        "jira_key".to_string(),
        serde_yaml::Value::String(raw.key.clone()),
    );
    extensions.insert(
        "issue_type".to_string(),
        serde_yaml::Value::String(raw.fields.issuetype.name.clone()),
    );
    if let Some(priority) = &raw.fields.priority {
        extensions.insert(
            "priority".to_string(),
            serde_yaml::Value::String(priority.name.clone()),
        );
    }
    extensions.insert(
        "status_name".to_string(),
        serde_yaml::Value::String(raw.fields.status.name.clone()),
    );
    extensions.insert(
        "hierarchy_level".to_string(),
        serde_yaml::Value::from(raw.fields.issuetype.hierarchy_level),
    );

    Ok(Record {
        id,
        title,
        status,
        assignee,
        labels,
        created_at,
        updated_at,
        version,
        body,
        parent_id,
        extensions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{JiraFields, JiraIssueType, JiraParent, JiraStatusCategory};

    // ─── Test: status_mapping_matrix ────────────────────────────────────

    #[test]
    fn status_mapping_matrix() {
        let cases: &[(&str, &str, Option<&str>, RecordStatus)] = &[
            ("new", "Open", None, RecordStatus::Open),
            (
                "indeterminate",
                "In Progress",
                None,
                RecordStatus::InProgress,
            ),
            ("indeterminate", "In Review", None, RecordStatus::InReview),
            ("done", "Done", None, RecordStatus::Done),
            ("done", "Done", Some("Won't Fix"), RecordStatus::WontFix),
            ("done", "Done", Some("Duplicate"), RecordStatus::WontFix),
            ("unknown-cat", "Something", None, RecordStatus::Open),
        ];

        for (cat, name, res_name, expected) in cases {
            let status = JiraStatus {
                name: (*name).to_string(),
                status_category: JiraStatusCategory {
                    key: (*cat).to_string(),
                },
            };
            let resolution = res_name.map(|n| JiraResolution {
                name: n.to_string(),
            });
            let got = map_status(&status, resolution.as_ref());
            assert_eq!(
                got, *expected,
                "status mapping failed for cat={cat} name={name} res={res_name:?}: got {got:?}, expected {expected:?}"
            );
        }
    }

    // ─── Test: adf_description_strips_to_plain_text ─────────────────────

    #[test]
    fn adf_description_strips_to_plain_text() {
        let doc = serde_json::json!({
            "type": "doc", "version": 1,
            "content": [
                {"type": "paragraph", "content": [
                    {"type": "text", "text": "First paragraph"}
                ]},
                {"type": "codeBlock", "content": [
                    {"type": "text", "text": "fn main() {}"}
                ]}
            ]
        });
        let result = adf::adf_to_plain_text(&doc);
        assert!(result.contains("First paragraph"), "paragraph text missing");
        assert!(result.contains("fn main()"), "code block text missing");

        // null description
        let null_result = adf::adf_to_plain_text(&serde_json::Value::Null);
        assert_eq!(
            null_result, "",
            "null description must produce empty string"
        );
    }

    // ─── Test: parent_hierarchy ──────────────────────────────────────────

    #[test]
    fn parent_hierarchy() {
        // Subtask with parent
        let subtask = JiraIssue {
            id: "200".into(),
            key: "PROJ-200".into(),
            fields: JiraFields {
                summary: Some("subtask".into()),
                description: None,
                status: JiraStatus {
                    name: "Open".into(),
                    status_category: JiraStatusCategory { key: "new".into() },
                },
                resolution: None,
                assignee: None,
                labels: vec![],
                created: "2025-01-01T00:00:00.000+0000".parse().unwrap(),
                updated: "2025-06-01T00:00:00.000+0000".parse().unwrap(),
                parent: Some(JiraParent { id: "10000".into() }),
                issuetype: JiraIssueType {
                    name: "Subtask".into(),
                    hierarchy_level: -1,
                },
                priority: None,
            },
        };
        let issue = translate(subtask).expect("translate must succeed");
        assert_eq!(issue.parent_id, Some(RecordId(10000)));
        assert_eq!(
            issue.extensions.get("hierarchy_level"),
            Some(&serde_yaml::Value::from(-1_i64))
        );

        // Issue with no parent
        let standalone = JiraIssue {
            id: "300".into(),
            key: "PROJ-300".into(),
            fields: JiraFields {
                summary: Some("standalone".into()),
                description: None,
                status: JiraStatus {
                    name: "Open".into(),
                    status_category: JiraStatusCategory { key: "new".into() },
                },
                resolution: None,
                assignee: None,
                labels: vec![],
                created: "2025-01-01T00:00:00.000+0000".parse().unwrap(),
                updated: "2025-06-01T00:00:00.000+0000".parse().unwrap(),
                parent: None,
                issuetype: JiraIssueType {
                    name: "Story".into(),
                    hierarchy_level: 0,
                },
                priority: None,
            },
        };
        let issue2 = translate(standalone).expect("translate must succeed");
        assert_eq!(issue2.parent_id, None);
    }

    // ─── Test: tenant_validation_rejects_ssrf ────────────────────────────

    #[test]
    fn tenant_validation_rejects_ssrf() {
        let long_tenant = "a".repeat(64);
        let bad_inputs = vec![
            "",
            "-bad",
            "bad-",
            "a.evil.com",
            long_tenant.as_str(),
            "UPPERCASE",
            "has space",
        ];
        for input in bad_inputs {
            assert!(
                validate_tenant(input).is_err(),
                "validate_tenant({input:?}) must reject invalid input"
            );
        }
        // Valid inputs
        assert!(validate_tenant("mycompany").is_ok());
        assert!(validate_tenant("my-company-123").is_ok());
        assert!(validate_tenant("a").is_ok());
        assert!(validate_tenant(&"a".repeat(63)).is_ok());
    }

    // ─── Test: extensions_omitted_when_empty ─────────────────────────────

    #[test]
    fn extensions_omitted_when_empty() {
        // An Issue with empty extensions must not serialize the word "extensions"
        // in its frontmatter YAML.
        use chrono::TimeZone;
        let now = chrono::Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap();
        let issue = Record {
            id: RecordId(1),
            title: "test".into(),
            status: RecordStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: now,
            updated_at: now,
            version: 1,
            body: "body".into(),
            parent_id: None,
            extensions: BTreeMap::new(),
        };
        let rendered = reposix_core::frontmatter::render(&issue).expect("render must succeed");
        assert!(
            !rendered.contains("extensions"),
            "empty extensions must be omitted from YAML, got: {rendered}"
        );
    }
}
