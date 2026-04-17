# Phase 29 — Pattern Map

**Extracted from:** codebase analysis during Phase 29 planning
**Primary analog:** `crates/reposix-confluence/src/lib.rs` write path

---

## Files Modified

| File | Role | Closest Analog |
|------|------|----------------|
| `crates/reposix-jira/src/lib.rs` | Write ops + supports() + issuetype cache | `crates/reposix-confluence/src/lib.rs` |
| `crates/reposix-jira/src/adf.rs` | Add adf_to_markdown + adf_paragraph_wrap | `crates/reposix-confluence/src/adf.rs` |
| `crates/reposix-jira/tests/contract.rs` | Write contract tests | `crates/reposix-jira/tests/contract.rs` |

---

## Pattern 1: create_issue (from ConfluenceBackend)

```rust
// crates/reposix-confluence/src/lib.rs:1582
async fn create_issue(&self, project: &str, issue: Untainted<Issue>) -> Result<Issue> {
    let post_body = serde_json::json!({ ... });
    let post_body_bytes = serde_json::to_vec(&post_body)?;
    let url = format!("{}/wiki/api/v2/pages", self.base());
    let header_owned = self.write_headers();
    let header_refs: Vec<(&str, &str)> =
        header_owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
    self.await_rate_limit_gate().await;
    let resp = self.http.request_with_headers_and_body(
        Method::POST, url.as_str(), &header_refs, Some(post_body_bytes),
    ).await?;
    self.ingest_rate_limit(&resp);
    let status = resp.status();
    let bytes = resp.bytes().await?;
    let req_summary: String = issue.inner_ref().title.chars().take(256).collect();
    self.audit_write("POST", "/wiki/api/v2/pages", status.as_u16(), &req_summary, &bytes);
    if !status.is_success() {
        return Err(Error::Other(format!("... returned {status}: {}", ...)));
    }
    let page: ConfPage = serde_json::from_slice(&bytes)?;
    let tainted = Tainted::new(page);
    translate(tainted.into_inner())
}
```

**JIRA adaptation:**
- URL: `{base}/rest/api/3/issue`
- POST body: `{"fields": {"project": {"key": project}, "summary": title, "issuetype": {"name": type}, "description": adf_wrapper(body)}}`
- Response: 201 with `{"id": "10001", "key": "PROJ-1"}` → parse id → `get_issue_inner(IssueId(id))`
- Audit call: `self.audit_event("POST", "/rest/api/3/issue", status_u16, &req_summary, &bytes)`

---

## Pattern 2: update_issue (from ConfluenceBackend)

```rust
// crates/reposix-confluence/src/lib.rs:1637
async fn update_issue(&self, _project, id, patch, expected_version) -> Result<Issue> {
    let put_body = serde_json::json!({
        "id": id.0.to_string(),
        "version": { "number": current_version + 1 },
        "body": { "representation": "storage", "value": storage_xhtml },
    });
    // ... PUT request pattern same as create_issue
    // Response: 200 with updated page → translate
}
```

**JIRA adaptation:**
- URL: `{base}/rest/api/3/issue/{id.0}`
- PUT body: `{"fields": {"summary": title, "description": adf_wrapper(body), "labels": labels}}`
- Response: 204 No Content → `get_issue_inner(id)` for hydration
- `expected_version` silently ignored (JIRA has no ETag)
- Audit: `self.audit_event("PUT", &format!("/rest/api/3/issue/{}", id.0), ...)`

---

## Pattern 3: delete_or_close (from ConfluenceBackend)

```rust
// crates/reposix-confluence/src/lib.rs:1708
async fn delete_or_close(&self, _project, id, _reason) -> Result<()> {
    let url = format!("{}/wiki/api/v2/pages/{}", self.base(), id.0);
    let header_owned = self.standard_headers();
    // ... DELETE request
    if status == StatusCode::NO_CONTENT {
        self.audit_write("DELETE", &audit_path, status_u16, "", &[]);
        return Ok(());
    }
}
```

**JIRA adaptation (two-step transitions):**
1. `GET {base}/rest/api/3/issue/{id.0}/transitions` (standard_headers)
2. Parse transitions, select by statusCategory.key == "done" + reason preference
3. `POST {base}/rest/api/3/issue/{id.0}/transitions` (write_headers) with `{"transition":{"id":"..."}}`
4. On 400 (required fields): retry with `{"transition":{"id":"..."},"fields":{"resolution":{"name":"Done"}}}`
5. Fallback DELETE only if transitions list is empty
6. Audit on all paths

---

## Pattern 4: supports() change

```rust
// Current (Phase 28):
fn supports(&self, feature: BackendFeature) -> bool {
    matches!(feature, BackendFeature::Hierarchy)
}

// Phase 29:
fn supports(&self, feature: BackendFeature) -> bool {
    matches!(
        feature,
        BackendFeature::Hierarchy | BackendFeature::Delete | BackendFeature::Transitions
    )
}
```

---

## Pattern 5: issuetype cache (new, no analog)

```rust
// Add to JiraBackend struct:
issue_type_cache: Arc<std::sync::OnceLock<Vec<String>>>,

// In new()/new_with_base_url():
issue_type_cache: Arc::new(std::sync::OnceLock::new()),

// In create_issue:
let issue_types = match self.issue_type_cache.get() {
    Some(types) => types,
    None => {
        let types = self.fetch_issue_types(project).await?;
        let _ = self.issue_type_cache.set(types);
        self.issue_type_cache.get().expect("just set")
    }
};
let chosen_type = issue_types.iter()
    .find(|t| t.eq_ignore_ascii_case("Task"))
    .or_else(|| issue_types.first())
    .cloned()
    .unwrap_or_else(|| "Task".to_owned());
```

---

## Pattern 6: adf_paragraph_wrap (new function in adf.rs)

```rust
// Add to crates/reposix-jira/src/adf.rs:
pub fn adf_paragraph_wrap(text: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "doc",
        "version": 1,
        "content": [{
            "type": "paragraph",
            "content": [{
                "type": "text",
                "text": text
            }]
        }]
    })
}
```

---

## Pattern 7: adf_to_markdown (copy-adapt from confluence)

See `crates/reposix-confluence/src/adf.rs::adf_to_markdown` (line 105).
Key function signature to match:
```rust
pub fn adf_to_markdown(adf: &serde_json::Value) -> Result<String, Error>
```

The recursive visitor pattern is identical — copy with JIRA-appropriate error type.
