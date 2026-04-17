---
id: 29-02
status: complete
commit: 7318588
---

# Plan 29-02 Summary — create_issue + update_issue write path

## What shipped

### Task T1: `fetch_issue_types`
Private async method on `JiraBackend`. GET `/rest/api/3/issuetype?projectKeys=<KEY>`, parses name array, returns `Vec<String>`. Returns `Err` if HTTP fails, if no types returned, or if JSON cannot be parsed.

### Task T2: `create_issue`
Replaced "not supported" stub. Flow: OnceLock cache init (fetch once, share across clones) → prefer "Task" type or first available → POST `/rest/api/3/issue` with ADF-wrapped body → audit_event → parse `CreateResp.id` → `get_issue_inner` to hydrate full Issue. Clippy-clean: `CreateResp` struct hoisted before statements; `if let` instead of `match`.

### Task T3: `update_issue`
Replaced "not supported" stub. PUT `/rest/api/3/issue/{id}` with fields body → 204 No Content → `get_issue_inner(id)`. `expected_version` silently ignored (JIRA has no ETag). Audit row on all outcomes.

### Task T4: Test changes
- Deleted test 12 (`write_ops_return_not_supported`) — behavior explicitly reversed
- Added `make_untainted` helper for test fixtures
- Added test 12 `create_issue_posts_to_rest_api` (wiremock: issuetype + POST + GET hydrate)
- Added test 13 `update_issue_puts_fields` (wiremock: PUT 204 + GET hydrate)
- Added test 14 `create_issue_discovers_issuetype` (wiremock: `expect(1)` verifies OnceLock fires once across two creates)

## Test results
- 28 unit tests pass, 2 doc tests, 2 contract tests
- `cargo clippy -p reposix-jira -- -D warnings`: clean
- `cargo fmt --check -p reposix-jira`: clean

## Files modified
- `crates/reposix-jira/src/lib.rs` (+296/-57 lines)
