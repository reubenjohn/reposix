//! P73 CONNECTOR-GAP-03: assert JIRA `list_records` does NOT leak
//! `fields.attachment` or `fields.comment.comments` into the rendered
//! `Record` (body OR extensions). Per
//! `docs/decisions/005-jira-issue-mapping.md:79-87`.
//!
//! D-03 is load-bearing: this asserts at the RENDERING boundary, not at
//! the JSON parse layer. Even if the wiremock response carries the
//! adversarial fields, `JiraBackend::list_records` must produce a
//! `Record` whose user-visible surfaces (`body` markdown + `extensions`
//! map) name neither `attachment` nor `comment`.
//!
//! Why this is interesting: `JIRA_FIELDS` (the explicit allowlist sent
//! in the search request, see `crates/reposix-jira/src/types.rs`) does
//! NOT include `attachment` or `comment`. Today, JIRA still tolerates
//! extra fields in the response body (the deserializer ignores unknown
//! keys), so this test seeds a response containing them anyway. The
//! contract claim is "the rendered Record never names them" — that
//! holds whether the parse layer drops them or carries them; this test
//! catches a future change that widens `JiraFields` AND inserts the new
//! key into `extensions` without explicit allowlisting.

use reposix_core::backend::BackendConnector;
use reposix_jira::{JiraBackend, JiraCreds};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn issue_with_attachments_and_comments() -> serde_json::Value {
    json!({
        "id": "10042",
        "key": "PROJ-42",
        "fields": {
            "summary": "Issue with attachments and comments",
            "description": serde_json::Value::Null,
            "status": {
                "name": "Open",
                "statusCategory": {"key": "new"}
            },
            "resolution": serde_json::Value::Null,
            "assignee": serde_json::Value::Null,
            "labels": [],
            "created": "2025-01-01T00:00:00.000+0000",
            "updated": "2025-12-01T10:30:00.000+0000",
            "parent": serde_json::Value::Null,
            "issuetype": {"name": "Task", "hierarchyLevel": 0},
            "priority": {"name": "Medium"},
            // Adversarial: these fields MUST NOT leak into the Record.
            "attachment": [
                {
                    "id": "99",
                    "filename": "secret-payload.txt",
                    "content": "https://example.invalid/secret-payload.txt"
                }
            ],
            "comment": {
                "comments": [
                    {
                        "id": "1001",
                        "body": "do-not-leak-this-comment-body-into-record",
                        "author": {"displayName": "Mallory"}
                    }
                ],
                "total": 1
            }
        }
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_records_excludes_attachments_and_comments() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "issues": [issue_with_attachments_and_comments()],
            "isLast": true
        })))
        .mount(&server)
        .await;

    let creds = JiraCreds {
        email: "test@example.com".into(),
        api_token: "token".into(),
    };
    let backend = JiraBackend::new_with_base_url(creds, server.uri())
        .expect("JiraBackend::new_with_base_url");

    let records = backend.list_records("PROJ").await.expect("list_records ok");
    assert_eq!(records.len(), 1, "expected single issue from wiremock");
    let record = &records[0];

    // (1) Body markdown must not name attachments.
    assert!(
        !record.body.to_lowercase().contains("attachment"),
        "Record.body leaked attachment data: {:?}",
        record.body
    );
    assert!(
        !record.body.contains("secret-payload.txt"),
        "Record.body leaked attachment filename: {:?}",
        record.body
    );

    // (2) Body markdown must not name comments.
    assert!(
        !record.body.to_lowercase().contains("comment"),
        "Record.body leaked comment data: {:?}",
        record.body
    );
    assert!(
        !record
            .body
            .contains("do-not-leak-this-comment-body-into-record"),
        "Record.body leaked comment body: {:?}",
        record.body
    );

    // (3) Extensions map must not name attachments or comments.
    let leaked_keys: Vec<&String> = record
        .extensions
        .keys()
        .filter(|k| {
            let lower = k.to_lowercase();
            lower.contains("attachment") || lower.contains("comment")
        })
        .collect();
    assert!(
        leaked_keys.is_empty(),
        "Record.extensions leaked attachment/comment keys: {:?}",
        leaked_keys
    );

    // (4) Extensions VALUES must not name the adversarial content
    //     (defense in depth; current allowlist is jira_key, issue_type,
    //     priority, status_name, hierarchy_level — none could legitimately
    //     contain the seeded strings).
    for (k, v) in &record.extensions {
        let s = serde_yaml::to_string(v).unwrap_or_default();
        assert!(
            !s.contains("secret-payload.txt"),
            "Record.extensions[{k}] leaked attachment filename: {s:?}"
        );
        assert!(
            !s.contains("do-not-leak-this-comment-body-into-record"),
            "Record.extensions[{k}] leaked comment body: {s:?}"
        );
    }
}
