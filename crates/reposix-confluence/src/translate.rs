//! Pure helpers and DTO → [`Record`] translation.
//!
//! This module is the seam between the on-the-wire Confluence shapes
//! (defined in [`crate::types`]) and the canonical [`Record`] surface that
//! [`reposix_core::backend::BackendConnector`] consumers see. All functions
//! here are pure — no HTTP, no audit, no clock — so they're trivially
//! unit-testable without a mock server.

use reposix_core::{Error, Record, RecordId, RecordStatus, Result};

use crate::types::ConfPage;

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

/// Validate a tenant subdomain against DNS-label rules.
///
/// Rejects: empty, > 63 chars, any character outside `[a-z0-9-]`,
/// leading/trailing hyphen. This defeats injection like `a.evil.com`,
/// `../../../`, `a@b`, or `tenant.with.dots`.
///
/// # Errors
///
/// Returns `Err(Error::Other(...))` if `tenant` fails any validation rule.
pub fn validate_tenant(tenant: &str) -> Result<()> {
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
/// [`RecordStatus`]. Pessimistic forward-compat: unknown values fall through
/// to `Open` (consistent with CONTEXT.md §status mapping).
#[must_use]
pub fn status_from_confluence(s: &str) -> RecordStatus {
    // Unknown values fall through to `Open` (pessimistic forward-compat):
    // an unseen Atlassian state should not silently mark pages as Done.
    // `match_same_arms` lint would prefer collapsing the current/draft/_
    // arms, but keeping `"current" | "draft"` as an explicit allowlist
    // documents the mapping contract — suppress the lint on purpose.
    #[allow(clippy::match_same_arms)]
    match s {
        "current" | "draft" => RecordStatus::Open,
        "archived" | "trashed" | "deleted" => RecordStatus::Done,
        _ => RecordStatus::Open,
    }
}

/// Translate a deserialized Confluence page into reposix's normalized
/// [`Record`].
///
/// # Errors
///
/// Returns `Err(Error::Other(…))` if `page.id` is not a valid `u64` (very
/// rare — Atlassian consistently returns numeric strings, but system pages
/// have historically had non-numeric ids).
pub(crate) fn translate(page: ConfPage) -> Result<Record> {
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
    // Phase 13 Wave B1: derive `Record::parent_id` from Confluence REST v2
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
                Some(RecordId(n))
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
            // to Record::parent_id so the tree/ overlay shows folder structure.
            if let Ok(n) = pid_str.parse::<u64>() {
                Some(RecordId(n))
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
    Ok(Record {
        id: RecordId(id),
        title: page.title,
        status: status_from_confluence(&page.status),
        assignee: page.owner_id,
        labels: vec![],
        created_at: page.created_at,
        updated_at: page.version.created_at,
        version: page.version.number,
        body,
        parent_id,
        extensions: std::collections::BTreeMap::new(),
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
pub(crate) fn redact_url(raw: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ConfPage, ConfVersion};
    use serde_json::json;

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
        assert_eq!(issue.parent_id, Some(RecordId(42)));
    }

    // CONF-06: folder parents now propagate to Record::parent_id (changed in
    // Phase 24 Plan 01). Updated from "orphan" to "propagates".
    #[test]
    fn translate_treats_folder_parent_as_orphan() {
        // CONF-06 (Phase 24 Plan 01): folder parents now propagate so the tree/
        // overlay can represent folder hierarchy. This test was originally written
        // when folder → orphan; updated to match the new behavior.
        let page = synth_page("99", Some("99999"), Some("folder"));
        let issue = translate(page).expect("translate");
        // CONF-06 fix: folder parentType now propagates (same as "page").
        assert_eq!(issue.parent_id, Some(RecordId(99999)));
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

    #[test]
    fn basic_auth_header_format() {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine as _;
        let got = basic_auth_header("a@b.com", "xyz");
        let want = format!("Basic {}", STANDARD.encode("a@b.com:xyz"));
        assert_eq!(got, want);
    }

    // -------- translate folder-parent tests (CONF-06) --------

    #[test]
    fn translate_folder_parent_propagates() {
        // CONF-06: folder parentType + valid parentId → parent_id Some(RecordId)
        let page = synth_page("77", Some("99999"), Some("folder"));
        let issue = translate(page).expect("translate must succeed");
        assert_eq!(
            issue.parent_id,
            Some(RecordId(99999)),
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
