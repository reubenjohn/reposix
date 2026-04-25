//! Round-trip integration test (WRITE-04 end-to-end).
//!
//! Exercises the full create → read → body-matches stack:
//!
//! 1. POST a new page (wiremock stub returns `id: "777"`).
//! 2. GET the page by id with `?body-format=atlas_doc_format` (wiremock
//!    returns hand-crafted ADF for "# Title\nhello world\n").
//! 3. Assert the returned body (converted by `adf_to_markdown`) contains
//!    the heading and paragraph from the original Markdown input.
//! 4. Assert the audit table has exactly 1 row with `method = 'POST'`.
//!
//! This proves WRITE-04 without a real Atlassian tenant.

use std::sync::Arc;

use parking_lot::Mutex;
use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};
use reposix_core::backend::{BackendConnector, DeleteReason};
use reposix_core::{Record, RecordId, IssueStatus, Tainted, Untainted};
use rusqlite::Connection;
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn creds() -> ConfluenceCreds {
    ConfluenceCreds {
        email: "test@example.com".into(),
        api_token: "token".into(),
    }
}

/// Open an in-memory `SQLite` DB with the audit schema loaded and return an
/// `Arc<Mutex<Connection>>` ready for [`ConfluenceBackend::with_audit`].
fn open_audit_db() -> Arc<Mutex<Connection>> {
    let conn = Connection::open_in_memory().expect("in-memory db");
    reposix_core::audit::load_schema(&conn).expect("load audit schema");
    Arc::new(Mutex::new(conn))
}

/// Build an [`Untainted<Record>`] for write-path tests.
fn make_issue(title: &str, body: &str) -> Untainted<Record> {
    use reposix_core::{sanitize, ServerMetadata};
    let t = chrono::DateTime::parse_from_rfc3339("2026-04-13T00:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    sanitize(
        Tainted::new(Record {
            id: RecordId(0),
            title: title.to_owned(),
            status: IssueStatus::Open,
            assignee: None,
            labels: vec![],
            created_at: t,
            updated_at: t,
            version: 0,
            body: body.to_owned(),
            parent_id: None,
            extensions: std::collections::BTreeMap::new(),
        }),
        ServerMetadata {
            id: RecordId(0),
            created_at: t,
            updated_at: t,
            version: 1,
        },
    )
}

/// WRITE-04: create a page then read it back; assert body round-trip and
/// audit log row.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_then_get_roundtrip_with_audit() {
    let server = MockServer::start().await;

    // 1. Space-key resolver — create_issue needs the space id.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/spaces"))
        .and(query_param("keys", "REPOSIX"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [{"id": "9999", "key": "REPOSIX"}]
        })))
        .mount(&server)
        .await;

    // 2. POST /wiki/api/v2/pages → 200 with id "777". Body is the
    //    storage-XHTML created by markdown_to_storage; we echo a minimal
    //    page shape back (the caller uses the returned id for get_issue).
    Mock::given(method("POST"))
        .and(path("/wiki/api/v2/pages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "777",
            "status": "current",
            "title": "Title",
            "createdAt": "2026-04-13T00:00:00Z",
            "ownerId": null,
            "version": {"number": 1, "createdAt": "2026-04-13T00:00:00Z"},
            "body": {}
        })))
        .mount(&server)
        .await;

    // 3. GET /wiki/api/v2/pages/777?body-format=atlas_doc_format → ADF JSON.
    //    Represents "# Title\nhello world\n" in ADF.
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/777"))
        .and(query_param("body-format", "atlas_doc_format"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "777",
            "status": "current",
            "title": "Title",
            "createdAt": "2026-04-13T00:00:00Z",
            "ownerId": null,
            "version": {"number": 1, "createdAt": "2026-04-13T00:00:00Z"},
            "body": {
                "atlas_doc_format": {
                    "representation": "atlas_doc_format",
                    "value": {
                        "type": "doc",
                        "version": 1,
                        "content": [
                            {
                                "type": "heading",
                                "attrs": {"level": 1},
                                "content": [{"type": "text", "text": "Title"}]
                            },
                            {
                                "type": "paragraph",
                                "content": [{"type": "text", "text": "hello world"}]
                            }
                        ]
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    let audit = open_audit_db();
    let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri())
        .expect("backend")
        .with_audit(Arc::clone(&audit));

    // 4. create_issue: POST the page, get back RecordId(777).
    let issue_to_create = make_issue("Title", "# Title\nhello world\n");
    let created = backend
        .create_issue("REPOSIX", issue_to_create)
        .await
        .expect("create_issue must succeed");
    assert_eq!(created.id, RecordId(777), "created page id must be 777");

    // 5. get_issue: GET the page by id, receive ADF, convert to Markdown.
    let fetched = backend
        .get_issue("REPOSIX", RecordId(777))
        .await
        .expect("get_issue must succeed");
    assert_eq!(fetched.id, RecordId(777));

    // 6. Body must contain the heading and paragraph text from the ADF.
    assert!(
        fetched.body.contains("Title"),
        "body must contain 'Title', got: {:?}",
        fetched.body
    );
    assert!(
        fetched.body.contains("hello world"),
        "body must contain 'hello world', got: {:?}",
        fetched.body
    );

    // 7. Audit table must have exactly 1 POST row (create_issue only).
    let post_count: i64 = audit
        .lock()
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE method = 'POST'",
            [],
            |r| r.get(0),
        )
        .expect("audit query must succeed");
    assert_eq!(
        post_count, 1,
        "exactly 1 POST audit row expected (create_issue), got {post_count}"
    );
}

/// Verify that `delete_or_close` is also audited as a sanity-check that the
/// audit wiring is complete across all three write methods in an integration
/// context (not just unit tests).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_or_close_is_audited_in_integration_context() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/wiki/api/v2/pages/42"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let audit = open_audit_db();
    let backend = ConfluenceBackend::new_with_base_url(creds(), server.uri())
        .expect("backend")
        .with_audit(Arc::clone(&audit));

    backend
        .delete_or_close("REPOSIX", RecordId(42), DeleteReason::Completed)
        .await
        .expect("delete_or_close must succeed");

    let del_count: i64 = audit
        .lock()
        .query_row(
            "SELECT COUNT(*) FROM audit_events WHERE method = 'DELETE'",
            [],
            |r| r.get(0),
        )
        .expect("audit query must succeed");
    assert_eq!(
        del_count, 1,
        "exactly 1 DELETE audit row expected, got {del_count}"
    );
}
