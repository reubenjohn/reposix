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
        // adf_to_markdown returns Err only when the ADF root type is not "doc".
        // Item 4b (design §6): NEVER substitute String::new() on failure — an
        // empty body flows into the working tree, the agent commits, and the
        // export path PATCHes the SoT body to empty → silent destruction of real
        // (attacker-influenceable) Confluence content. Fail closed instead:
        // (1) prefer the raw storage HTML when present; (2) otherwise substitute
        // a conspicuous NON-EMPTY teaching sentinel the export path refuses to
        // push. The single record degrades loudly; the rest of list_records
        // still returns (T-16-C-05 graceful-degradation / DoS mitigation intact).
        match crate::adf::adf_to_markdown(&adf_body.value) {
            Ok(md) => md,
            Err(e) => {
                if let Some(storage) = page.body.as_ref().and_then(|b| b.storage.as_ref()) {
                    tracing::warn!(
                        error = %e,
                        page_id = %page.id,
                        "adf_to_markdown failed; falling back to raw storage HTML"
                    );
                    storage.value.clone()
                } else {
                    let root_type = crate::adf::adf_root_type(&adf_body.value);
                    tracing::warn!(
                        error = %e,
                        page_id = %page.id,
                        adf_root_type = %root_type,
                        "adf_to_markdown failed and no storage fallback; substituting fail-closed unreadable-ADF sentinel"
                    );
                    crate::adf::unreadable_adf_body(root_type, &page.id)
                }
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
    use crate::types::{
        ConfBodyAdf, ConfBodyStorage, ConfPage, ConfPageBody, ConfPageList, ConfVersion,
    };
    use serde_json::json;

    /// Build a `ConfPage` carrying a non-`doc` ADF body (which `adf_to_markdown`
    /// rejects), optionally with a raw storage HTML fallback. Used to exercise
    /// the item-4b fail-closed body-substitution paths.
    fn synth_page_with_bad_adf(id: &str, storage_html: Option<&str>) -> ConfPage {
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
            body: Some(ConfPageBody {
                storage: storage_html.map(|v| ConfBodyStorage {
                    value: v.to_owned(),
                }),
                // Non-"doc" root → adf_to_markdown returns Err.
                adf: Some(ConfBodyAdf {
                    value: json!({
                        "type": "paragraph",
                        "content": [{"type": "text", "text": "smuggled"}]
                    }),
                }),
            }),
            parent_id: None,
            parent_type: None,
        }
    }

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

    // -------- item 4b: fail-closed body substitution (design §6) --------

    #[test]
    fn translate_non_doc_adf_with_storage_uses_storage_html() {
        // A non-doc ADF root WITH a storage fallback → uses the storage HTML,
        // never empty and never the sentinel.
        let page = synth_page_with_bad_adf("7766017", Some("<p>real content</p>"));
        let record = translate(page).expect("translate must not fail on bad ADF");
        assert_eq!(record.body, "<p>real content</p>");
        assert!(
            !crate::adf::is_unreadable_adf_sentinel(&record.body),
            "storage-backed body must not be the sentinel"
        );
    }

    #[test]
    fn translate_non_doc_adf_without_storage_yields_sentinel_never_empty() {
        // The security invariant: a non-doc ADF root WITHOUT storage must NEVER
        // produce String::new() — it degrades to the non-empty teaching sentinel
        // the export path refuses. This is what stops a silent empty-body PATCH.
        let page = synth_page_with_bad_adf("7766017", None);
        let record = translate(page).expect("translate must not fail on bad ADF");
        assert!(
            !record.body.is_empty(),
            "unreadable body must NEVER be an empty String (silent-blank hazard)"
        );
        assert_ne!(record.body, String::new());
        assert!(
            crate::adf::is_unreadable_adf_sentinel(&record.body),
            "unreadable body must be the detectable sentinel, got: {}",
            record.body
        );
        // Teaching content: names the offending root type and the page id.
        assert!(record.body.contains("\"paragraph\""), "must name root type");
        assert!(record.body.contains("7766017"), "must name the page id");
    }

    // -------- v0.14.0 item 5: string-encoded ADF value decode (DP-2) --------

    #[test]
    // test-name-honesty: ok — "real markdown" = real ADF CONTENT (vs the item-4b
    // sentinel), NOT a real backend; this is a pure serde_json decode + translate
    // unit test with no network (the live-TokenWorld twin is the #[ignore] smoke
    // get_record_real_confluence_body_is_not_unreadable_sentinel in agent_flow_real.rs).
    fn translate_decodes_string_encoded_adf_value_to_real_markdown() {
        // The Confluence Cloud v2 API STRING-encodes `body.atlas_doc_format.value`
        // (a JSON string whose contents are the ADF document). A prior bug typed
        // `value` as an object, so every real page read `""` for its root type and
        // was replaced by the item-4b unreadable-ADF sentinel — blocking push
        // round-trips (p93 + the vision litmus). This locks the real wire shape:
        // deserialize a string-encoded value through `ConfPage` and assert it
        // translates to REAL Markdown, never the sentinel.
        let adf_string = json!({
            "type": "doc",
            "version": 1,
            "content": [
                {"type": "heading", "attrs": {"level": 1},
                 "content": [{"type": "text", "text": "Real Title"}]},
                {"type": "paragraph",
                 "content": [{"type": "text", "text": "Real body content."}]}
            ]
        })
        .to_string();
        let wire = json!({
            "id": "2818063",
            "status": "current",
            "title": "reposix demo space Home",
            "createdAt": "2026-04-13T00:00:00Z",
            "version": {"number": 7, "createdAt": "2026-04-13T00:00:00Z"},
            "body": {
                "atlas_doc_format": {
                    "value": adf_string,
                    "representation": "atlas_doc_format"
                }
            }
        });
        let page: ConfPage = serde_json::from_value(wire).expect("deserialize real-shape page");
        let record = translate(page).expect("translate");
        assert!(
            record.body.contains("# Real Title"),
            "string-encoded ADF must decode to real markdown, got: {}",
            record.body
        );
        assert!(
            record.body.contains("Real body content."),
            "expected the real paragraph text, got: {}",
            record.body
        );
        assert!(
            !crate::adf::is_unreadable_adf_sentinel(&record.body),
            "a valid real ADF body must NOT become the fail-closed sentinel, got: {}",
            record.body
        );
    }

    #[test]
    fn translate_non_json_string_adf_value_still_sentinels() {
        // Fail-closed preserved: a `value` string that is NOT decodable JSON
        // (genuinely malformed ADF) must still degrade to the item-4b sentinel,
        // never silently become real-looking content. The string decode only
        // fixes the parse; it does not weaken the guard.
        let wire = json!({
            "id": "999",
            "status": "current",
            "title": "t",
            "createdAt": "2026-04-13T00:00:00Z",
            "version": {"number": 1, "createdAt": "2026-04-13T00:00:00Z"},
            "body": {"atlas_doc_format": {"value": "not json at all"}}
        });
        let page: ConfPage = serde_json::from_value(wire).expect("deserialize");
        let record = translate(page).expect("translate must not error");
        assert!(
            crate::adf::is_unreadable_adf_sentinel(&record.body),
            "a malformed ADF value must still hit the fail-closed sentinel, got: {}",
            record.body
        );
    }

    #[test]
    fn conf_body_adf_deserializes_string_encoded_value_to_object() {
        // Unit-level lock on the decode itself: the wrapper accepts the live
        // API's string-encoded value and exposes the decoded OBJECT.
        let wire = json!({"value": "{\"type\":\"doc\",\"version\":1,\"content\":[]}"});
        let adf: ConfBodyAdf = serde_json::from_value(wire).expect("decode string-encoded adf");
        assert_eq!(
            crate::adf::adf_root_type(&adf.value),
            "doc",
            "string-encoded value must decode to an object with root type doc"
        );
    }

    #[test]
    fn conf_page_list_with_empty_or_missing_adf_value_degrades_not_dos() {
        // Regression (v0.14.0 item 5c — pre-existing list-wide DoS): a single
        // page whose `atlas_doc_format` is `{}` (empty) or omits `value` must
        // NOT fail the ENTIRE `ConfPageList` deserialize (client.rs
        // `serde_json::from_value`) — that would blank the whole space off one
        // malformed page. `#[serde(default)]` on the inner `Raw.value` defaults
        // it to Null so the list stays intact and each bad page degrades to the
        // fail-closed item-4b sentinel while its siblings list normally.
        let good_value = json!({
            "type": "doc", "version": 1,
            "content": [{"type": "paragraph",
                         "content": [{"type": "text", "text": "good content"}]}]
        })
        .to_string();
        let wire = json!({
            "results": [
                {
                    "id": "1001", "status": "current", "title": "good",
                    "createdAt": "2026-04-13T00:00:00Z",
                    "version": {"number": 3, "createdAt": "2026-04-13T00:00:00Z"},
                    "body": {"atlas_doc_format": {"value": good_value,
                                                  "representation": "atlas_doc_format"}}
                },
                {
                    // Empty ADF wrapper `{}` — no `value` key at all.
                    "id": "1002", "status": "current", "title": "empty-adf",
                    "createdAt": "2026-04-13T00:00:00Z",
                    "version": {"number": 1, "createdAt": "2026-04-13T00:00:00Z"},
                    "body": {"atlas_doc_format": {}}
                },
                {
                    // ADF wrapper present but `value` omitted (only representation).
                    "id": "1003", "status": "current", "title": "missing-value",
                    "createdAt": "2026-04-13T00:00:00Z",
                    "version": {"number": 1, "createdAt": "2026-04-13T00:00:00Z"},
                    "body": {"atlas_doc_format": {"representation": "atlas_doc_format"}}
                }
            ],
            "_links": {}
        });

        // The WHOLE list must deserialize — one bad page never DoSes the rest.
        let list: ConfPageList =
            serde_json::from_value(wire).expect("empty/missing ADF value must NOT fail the list");
        assert_eq!(
            list.results.len(),
            3,
            "all three pages must survive the deserialize"
        );

        let mut pages = list.results.into_iter();
        // Good page → real decoded markdown, never the sentinel.
        let good = translate(pages.next().unwrap()).expect("translate good page");
        assert!(
            good.body.contains("good content")
                && !crate::adf::is_unreadable_adf_sentinel(&good.body),
            "good page must decode to real markdown, got: {}",
            good.body
        );
        // Empty `{}` and missing-`value` pages → fail-closed sentinel (Null ADF
        // root → adf_to_markdown Err → non-empty teaching placeholder), NEVER an
        // empty body and NEVER a whole-list error.
        for (id, page) in [
            ("1002", pages.next().unwrap()),
            ("1003", pages.next().unwrap()),
        ] {
            let rec =
                translate(page).unwrap_or_else(|e| panic!("translate {id} must not error: {e:?}"));
            assert!(
                crate::adf::is_unreadable_adf_sentinel(&rec.body),
                "page {id} (empty/missing ADF value) must degrade to the fail-closed \
                 sentinel, got: {}",
                rec.body
            );
        }
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
