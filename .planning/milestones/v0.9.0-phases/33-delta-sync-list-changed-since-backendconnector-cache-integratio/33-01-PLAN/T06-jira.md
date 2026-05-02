[index](./index.md)

# Task 01-T06 — `JiraBackend::list_changed_since` — JQL `updated >= "<iso>"`

<read_first>
- `crates/reposix-jira/src/lib.rs:200-350` (list_issues_impl, JQL construction pattern)
- `crates/reposix-jira/src/lib.rs:522-540` (trait impl)
</read_first>

<action>
Edit `crates/reposix-jira/src/lib.rs`:

1. Inside `impl BackendConnector for JiraBackend`, after `list_issues` (line 534), add:

```rust
async fn list_changed_since(
    &self,
    project: &str,
    since: chrono::DateTime<chrono::Utc>,
) -> reposix_core::Result<Vec<reposix_core::IssueId>> {
    // JQL `updated >= "yyyy-MM-dd HH:mm"` (JQL does not accept full ISO8601
    // with timezone; use the canonical two-field form).
    let jql_time = since.format("%Y-%m-%d %H:%M").to_string();
    let url = format!("{}/rest/api/3/search/jql", self.base());
    // Strip quotes from project slug defensively before interpolation.
    let safe_project = project.replace('"', "");
    let fields: Vec<String> = JIRA_FIELDS.iter().map(|s| (*s).to_owned()).collect();
    let mut request_body = serde_json::json!({
        "jql": format!("project = \"{}\" AND updated >= \"{}\" ORDER BY id ASC", safe_project, jql_time),
        "fields": fields,
        "maxResults": PAGE_SIZE,
    });

    let mut out: Vec<reposix_core::IssueId> = Vec::new();
    let mut pages: usize = 0;
    let header_owned = self.write_headers();
    let header_refs: Vec<(&str, &str)> =
        header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();

    loop {
        pages += 1;
        if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) + 1 {
            tracing::warn!(pages, "reached MAX_ISSUES_PER_LIST cap; stopping pagination");
            break;
        }
        self.await_rate_limit_gate().await;
        let body_bytes = serde_json::to_vec(&request_body)?;
        let resp = self.http.request_with_headers_and_body(
            Method::POST, url.as_str(), &header_refs, Some(body_bytes)
        ).await?;
        self.ingest_rate_limit(&resp);
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(reposix_core::Error::Other(format!(
                "JIRA returned {status} for POST /rest/api/3/search/jql: {}",
                String::from_utf8_lossy(&bytes)
            )));
        }
        let search_resp: JiraSearchResponse = serde_json::from_slice(&bytes)?;
        let is_last = search_resp.is_last.unwrap_or(true);
        let next_token = search_resp.next_page_token.clone();
        for issue in search_resp.issues {
            // Translate just to get the IssueId — then discard the full Issue.
            let translated = translate(issue)?;
            out.push(translated.id);
            if out.len() >= MAX_ISSUES_PER_LIST {
                return Ok(out);
            }
        }
        if is_last { break; }
        if let Some(token) = next_token {
            request_body["nextPageToken"] = serde_json::Value::String(token);
        } else {
            break;
        }
    }
    Ok(out)
}
```

2. Add wiremock contract test patterned on the existing JIRA list tests in the same file. Match on the JQL substring `"updated >="`:

```rust
#[tokio::test]
async fn jira_list_changed_since_sends_updated_jql() {
    use chrono::{TimeZone, Utc};
    let server = MockServer::start().await;
    // Match POST body JSON contains "updated >=".
    Mock::given(method("POST"))
        .and(path("/rest/api/3/search/jql"))
        .and(wiremock::matchers::body_string_contains("updated >="))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "issues": [
                { "id": "10042", "key": "TEST-42", "fields": {
                    "summary": "x", "status": {"name": "Open"},
                    "created": "2026-04-24T01:00:00.000+0000",
                    "updated": "2026-04-24T02:00:00.000+0000",
                    "labels": []
                }}
            ],
            "isLast": true
        })))
        .mount(&server)
        .await;

    let backend = make_test_jira_backend(server.uri()); // follow existing helper
    let t = Utc.with_ymd_and_hms(2026, 4, 24, 0, 0, 0).unwrap();
    let ids = backend.list_changed_since("TEST", t).await.expect("list");
    assert_eq!(ids.len(), 1);
}
```

If `body_string_contains` is not a wiremock matcher in this version, write a `struct BodyContains(&'static str)` implementing `wiremock::Match` — inspect `request.body` for the substring.
</action>

<acceptance_criteria>
- `cargo build -p reposix-jira` exits 0.
- `cargo test -p reposix-jira jira_list_changed_since_sends_updated_jql` exits 0.
- `grep -n 'list_changed_since' crates/reposix-jira/src/lib.rs` finds exactly one override.
- JQL `project` value is NOT attacker-controllable (interpolated after `.replace('"', "")`).
</acceptance_criteria>

<threat_model>
Same defensive `project.replace('"', "")` pre-interpolation against JQL injection as Confluence. Auth + egress gating inherited from `self.http`. The POST body is serde_json-serialized (not string-concatenated) so no JSON injection.
</threat_model>
