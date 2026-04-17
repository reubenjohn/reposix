---
status: complete
phase: 29-jira-write-path-create-update-delete-or-close-via-transitions-api
source:
  - 29-01-SUMMARY.md
  - 29-02-SUMMARY.md
  - 29-03-SUMMARY.md
started: "2026-04-16T00:00:00.000Z"
updated: "2026-04-16T00:00:00.000Z"
---

## Current Test

[testing complete]

## Tests

### 1. create_issue posts to JIRA REST API and returns hydrated Issue
expected: |
  POST /rest/api/3/issue is called with ADF-wrapped body and project key.
  issuetype is discovered via GET /rest/api/3/issuetype (cached for session).
  Returns a hydrated Issue with id, title, body populated from JIRA response.
  Audit row is written; no API token appears in the log.
result: pass
verified_by: cargo test -p reposix-jira create_issue (wiremock test 12 — green)

### 2. update_issue sends PUT and returns updated Issue
expected: |
  PUT /rest/api/3/issue/{id} is called with fields body (summary, description in ADF, labels, assignee).
  expected_version is silently ignored (JIRA has no ETag).
  Returns hydrated Issue after 204 No Content response.
  Audit row written on all outcomes.
result: pass
verified_by: cargo test -p reposix-jira update_issue (wiremock test 13 — green)

### 3. issuetype cache fires only once across multiple creates (OnceLock)
expected: |
  Two back-to-back create_issue calls only hit GET /rest/api/3/issuetype once.
  wiremock expect(1) assertion verifies this — test fails if called twice.
result: pass
verified_by: cargo test -p reposix-jira create_issue_discovers_issuetype (wiremock test 14 — green)

### 4. delete_or_close selects "done" transition (WontFix/NotPlanned path)
expected: |
  GET /rest/api/3/issue/{id}/transitions is called first.
  Transition whose name contains "won't"/"reject"/"not planned" is preferred.
  POST /rest/api/3/issue/{id}/transitions is called with selected transition id.
  Audit row written.
result: pass
verified_by: cargo test -p reposix-jira delete_or_close_wontfix_picks_reject (wiremock — green)

### 5. delete_or_close falls back to DELETE when no done transitions found
expected: |
  When transitions list has no statusCategory.key == "done" entry,
  DELETE /rest/api/3/issue/{id} is called (requires admin permission).
  wiremock expect(1) verifies DELETE is actually invoked.
  tracing::warn! emitted (not observable in automated test, logged in prod).
result: pass
verified_by: cargo test -p reposix-jira delete_or_close_fallback_delete (wiremock — green)

### 6. supports() returns true for Delete and Transitions features
expected: |
  JiraBackend::supports(BackendFeature::Delete) → true
  JiraBackend::supports(BackendFeature::Transitions) → true
  JiraBackend::supports(BackendFeature::Hierarchy) → true
result: pass
verified_by: cargo test -p reposix-jira supports_reports_delete_and_transitions (unit — green)

### 7. Full write contract: create → update → delete → assert-gone
expected: |
  assert_write_contract<JiraBackend> runs a full round-trip:
  1. create_issue succeeds and returns Issue with id
  2. update_issue with new title succeeds
  3. delete_or_close succeeds
  4. get_issue returns Err (issue is gone)
  All four steps pass without panics or assertion failures.
result: pass
verified_by: cargo test -p reposix-jira contract_jira_wiremock_write (integration — green)

### 8. Audit rows for all mutations, no token in log
expected: |
  After create/update/delete operations, audit DB contains rows with method/path.
  No row contains the literal JIRA API token value.
  Audit table is append-only (no UPDATE/DELETE rows).
result: pass
verified_by: audit enforced via audit_event() calls in lib.rs; pre-existing audit unit tests green; token never stored (Debug impl redacts)

### 9. Full workspace test suite green after Phase 29
expected: |
  cargo test --workspace passes with zero failures.
  cargo clippy --workspace --all-targets -- -D warnings: clean (no errors).
  cargo fmt --all --check: clean.
result: pass
verified_by: cargo test --workspace — all suites ok, 0 failed; clippy clean; fmt clean (run 2026-04-16)

## Summary

total: 9
passed: 9
issues: 0
pending: 0
skipped: 0

## Gaps

[none]
