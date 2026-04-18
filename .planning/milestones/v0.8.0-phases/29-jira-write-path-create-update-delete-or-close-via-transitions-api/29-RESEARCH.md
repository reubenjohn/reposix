# Phase 29: JIRA Write Path — Research

**Prepared:** 2026-04-16
**Based on:** Codebase analysis of Phase 28 JiraBackend + Confluence write path

---

## Research Summary

Phase 29 completes the JIRA write path. The Phase 28 implementation left clean stubs with clear extension points. The Confluence write path (`crates/reposix-confluence/src/lib.rs`) is the primary reference — JIRA's write path follows the same structural pattern with JIRA-specific API differences.

---

## Validation Architecture

The following validation strategy applies to Phase 29:

### Write Operations
- `create_issue` → POST /rest/api/3/issue → 201 with `{id, key, self}` → follow with GET to hydrate
- `update_issue` → PUT /rest/api/3/issue/{id} → 204 No Content → follow with GET to hydrate
- `delete_or_close` → GET transitions → POST transitions → 204; fallback DELETE → 204

### Security Validations (inherited from Phase 28, must remain intact)
- All HTTP through `reposix_core::http::client()` — no direct reqwest
- No credential values in audit log or error messages
- All mutations audited via `audit_event()`

### Test Coverage
- ≥3 wiremock write-path tests (6 planned per CONTEXT.md)
- Contract test extended with write invariants (create → get → assert; update → get → assert; delete → get → assert not found)
- Test 12 (`write_ops_return_not_supported`) deleted and replaced

---

## Key Findings

### 1. Infrastructure Already Present in JiraBackend

Phase 28 shipped all the infrastructure Phase 29 needs:

| Asset | Location | Status |
|-------|----------|--------|
| `write_headers()` | `JiraBackend::write_headers()` (line 237) | Ready |
| `audit_event()` | `JiraBackend::audit_event()` (line 391) | Ready |
| `standard_headers()` | `JiraBackend::standard_headers()` (line 226) | Ready |
| `await_rate_limit_gate()` | `JiraBackend::await_rate_limit_gate()` | Ready |
| `arm_rate_limit_backoff()` | exists, marked `#[allow(dead_code)]` — wire in write path | Ready |
| `get_issue_inner()` | for post-write hydration | Ready |
| `with_audit(conn)` | for attaching audit DB | Ready |
| `request_with_headers_and_body()` | via `self.http` (HttpClient) | Ready |

**No new infrastructure needed** — the write path implementation is pure logic wiring.

### 2. ADF Write Encoding

JIRA requires ADF JSON for `description` fields. The minimal single-paragraph wrapper:

```json
{
  "type": "doc",
  "version": 1,
  "content": [{
    "type": "paragraph",
    "content": [{
      "type": "text",
      "text": "<plain_body>"
    }]
  }]
}
```

This is what `serde_json::json!()` produces cleanly. No external crate needed.

**Edge cases:**
- Empty body → `"text": ""` produces valid ADF (JIRA accepts it)
- Multiline body → single paragraph; JIRA will preserve newlines as `hardBreak` nodes but plain text is acceptable for programmatic writes

### 3. issuetype Discovery

`GET /rest/api/3/issuetype?projectKeys=<KEY>` returns an array:
```json
[
  {"id": "10001", "name": "Story", ...},
  {"id": "10002", "name": "Task", ...},
  {"id": "10003", "name": "Bug", ...}
]
```

Cache with `std::sync::OnceLock<Vec<String>>` — initialized on first `create_issue` call:

```rust
// On JiraBackend struct:
issue_type_cache: Arc<OnceLock<Vec<String>>>,
```

Selection logic: prefer `"Task"` (case-insensitive), fallback to first entry. Serialize the chosen name into the POST body.

**Note:** `OnceLock` requires the cache to be populated exactly once per backend instance. Since the backend is `Clone`, the `Arc<OnceLock<...>>` pattern ensures all clones share the same initialized cache. Initialization uses `OnceLock::get_or_try_init()` (async-friendly via tokio's `OnceCell` if needed) or a fallback synchronous pattern.

**Alternative:** Use `tokio::sync::OnceCell<Vec<String>>` (async-safe `get_or_init`). This is cleaner for async backends since `std::sync::OnceLock::get_or_try_init()` is stable as of Rust 1.82.

**Recommendation:** `std::sync::OnceLock` + blocking init is fine since the HTTP call completes before the FUSE handler responds; use `tokio::sync::OnceCell` only if lint/tests demand fully async.

### 4. delete_or_close Transition Flow

**Step 1: GET /rest/api/3/issue/{id}/transitions**

Response:
```json
{
  "transitions": [
    {"id": "11", "name": "To Do", "to": {"statusCategory": {"key": "new"}}},
    {"id": "21", "name": "In Progress", "to": {"statusCategory": {"key": "indeterminate"}}},
    {"id": "31", "name": "Done", "to": {"statusCategory": {"key": "done"}}},
    {"id": "41", "name": "Won't Fix", "to": {"statusCategory": {"key": "done"}}}
  ]
}
```

**Step 2: Filter and select**

```rust
let done_transitions: Vec<_> = transitions
    .iter()
    .filter(|t| t.to.status_category.key == "done")
    .collect();

let chosen = match reason {
    DeleteReason::WontFix => {
        // prefer "won't"/"reject"/"not planned"/"invalid"/"duplicate" name
        done_transitions.iter()
            .find(|t| {
                let name = t.name.to_lowercase();
                name.contains("won't") || name.contains("wont") ||
                name.contains("reject") || name.contains("not planned") ||
                name.contains("invalid") || name.contains("duplicate")
            })
            .or_else(|| done_transitions.first())
    }
    _ => done_transitions.first()
};
```

**Step 3: POST /rest/api/3/issue/{id}/transitions**

Body: `{"transition": {"id": "<chosen.id>"}}`

Some JIRA screens require `fields.resolution`. On 400 response: retry with:
```json
{
  "transition": {"id": "<id>"},
  "fields": {"resolution": {"name": "Done"}}
}
```

**Step 4: DELETE fallback**

Only when `done_transitions.is_empty()`:
```rust
warn!("JIRA: no done transitions for issue {id}, falling back to DELETE");
// DELETE /rest/api/3/issue/{id}
// Only succeed on 204
```

### 5. update_issue Fields

JIRA's PUT body for field updates:
```json
{
  "fields": {
    "summary": "<title>",
    "description": <adf_wrapper(body)>,
    "labels": ["label1", "label2"],
    "assignee": null
  }
}
```

**Key constraints:**
- Status changes NOT via PUT — they require transition API
- `expected_version` silently ignored (no ETag in JIRA v3 REST)
- Response is 204 No Content — hydrate via `get_issue_inner(id)` for return value

### 6. create_issue Response and Hydration

`POST /rest/api/3/issue` returns 201:
```json
{"id": "10001", "key": "PROJ-1", "self": "https://.../rest/api/3/issue/10001"}
```

Parse `id` as `u64` → `IssueId`. Then call `get_issue_inner(IssueId(parsed_id))` to return the full `Issue` struct (same pattern as `ConfluenceBackend::create_issue`).

### 7. ADF Read Path Upgrade

The existing `adf_to_plain_text` in `reposix-jira/src/adf.rs` (line 31) strips all formatting. Phase 29 upgrades this to `adf_to_markdown` using the recursive visitor from `reposix-confluence/src/adf.rs::adf_to_markdown`.

**Key difference:** Confluence's ADF → Markdown handles `heading`, `bulletList`, `orderedList`, `codeBlock`, `inlineCode`, `strong`, `em`. The JIRA version can reuse the same logic since JIRA uses the same ADF spec.

**Implementation path:**
1. Add `adf_to_markdown(value: &Value) -> Result<String, Error>` to `reposix-jira/src/adf.rs`
2. Update `translate()` (the read path) to call `adf_to_markdown` instead of `adf_to_plain_text`
3. Keep `adf_to_plain_text` for backward compat or make it private

**Risk:** JIRA may produce ADF with additional node types not present in Confluence. The existing "unknown node: recurse into children" fallback handles this gracefully.

### 8. BackendConnector `supports()` Change

Current (Phase 28):
```rust
fn supports(&self, feature: BackendFeature) -> bool {
    matches!(feature, BackendFeature::Hierarchy)
}
```

Phase 29:
```rust
fn supports(&self, feature: BackendFeature) -> bool {
    matches!(
        feature,
        BackendFeature::Hierarchy | BackendFeature::Delete | BackendFeature::Transitions
    )
}
```

### 9. Test 12 Deletion

The existing test `write_ops_return_not_supported` (line ~1212 in lib.rs) asserts that all three write ops return the "not supported" error. This test MUST be deleted and replaced with the 6 new write-path wiremock tests from CONTEXT.md D-15.

### 10. Contract Test Extension

The existing `assert_contract` function in `tests/contract.rs` covers 5 read-only invariants. Phase 29 adds `assert_write_contract`:

```rust
async fn assert_write_contract<B: BackendConnector>(backend: &B, project: &str) {
    // Create
    let issue = make_untainted("test title", "test body", None);
    let created = backend.create_issue(project, issue).await.unwrap();
    assert_eq!(created.title, "test title");
    
    // Update  
    let patch = make_untainted("updated title", "updated body", None);
    let updated = backend.update_issue(project, created.id, patch, None).await.unwrap();
    assert_eq!(updated.title, "updated title");
    
    // Delete
    backend.delete_or_close(project, created.id, DeleteReason::Completed).await.unwrap();
    let gone = backend.get_issue(project, created.id).await;
    assert!(gone.is_err(), "deleted issue should return Err");
}
```

---

## File Map

| File | Role | Change Type |
|------|------|------------|
| `crates/reposix-jira/src/lib.rs` | Main backend — write stubs, `supports()`, struct | MODIFY |
| `crates/reposix-jira/src/adf.rs` | ADF encoder/decoder | MODIFY (add adf_to_markdown, add adf_paragraph_wrap) |
| `crates/reposix-jira/tests/contract.rs` | Contract test | MODIFY (add write contract) |
| `crates/reposix-jira/Cargo.toml` | Dependencies | CHECK (std OnceLock needs no new dep) |
| `.planning/phases/29-*/29-SUMMARY.md` | Ship docs | CREATE |

No new crates needed. No new workspace dependencies needed.

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| JIRA requires `fields.resolution` on close | High | D-10 retry logic handles this |
| `OnceLock::get_or_try_init` async compatibility | Medium | Use `tokio::sync::OnceCell` if needed |
| JIRA returns different ADF node types from Confluence | Low | Unknown-node fallback in adf_to_markdown handles this |
| Test 12 deletion breaks CI | Low | Phase 29 replaces it with write tests before ship |
| `issuetype` endpoint returns empty array | Low | Error surfaced to caller cleanly |

---

## RESEARCH COMPLETE

All technical questions for Phase 29 planning are answered. The write path is a straightforward extension of:
1. Existing `JiraBackend` infrastructure (write_headers, audit_event, rate_limit_gate)
2. Confluence write path pattern (create → hydrate, update → hydrate, delete with audit)
3. JIRA-specific: transitions API, issuetype discovery cache, ADF minimal wrapper

**Recommended wave split:**
- Wave A: ADF upgrade (`adf_to_markdown` + `adf_paragraph_wrap`) + issuetype cache struct
- Wave B: `create_issue` + `update_issue` implementations + wiremock tests
- Wave C: `delete_or_close` via transitions + fallback + remaining tests + `supports()` update + contract extension + docs/CHANGELOG
