---
plan: 28-01
phase: 28
status: complete
commit: d44d312
wave: 1
---

# Plan 28-01 Summary: reposix-jira crate — JiraBackend + BackendConnector impl

## What Was Built

Created the `crates/reposix-jira/` crate from scratch with a complete read-only JIRA Cloud REST v3 adapter.

**Key artifacts:**
- `crates/reposix-jira/Cargo.toml` — workspace member with all required deps (serde_yaml added alongside confluence pattern deps)
- `crates/reposix-jira/src/adf.rs` — ADF plain-text extractor (`adf_to_plain_text`), 5 unit tests
- `crates/reposix-jira/src/lib.rs` — `JiraBackend` implementing `BackendConnector`, 12 wiremock tests

## Security Invariants Confirmed

| Invariant | Status |
|-----------|--------|
| `JiraCreds` Debug redacts `api_token` | ✓ Manual Debug impl |
| `JiraBackend` Debug redacts `api_token` | ✓ `finish_non_exhaustive()` + `<redacted>` via creds |
| All JIRA responses wrapped in `Tainted::new()` | ✓ Both list and get paths |
| HTTP client via `reposix_core::http::client()` only | ✓ Zero direct `reqwest::Client::new()` calls |
| Tenant validation blocks SSRF | ✓ `validate_tenant()` enforces DNS-label rules |
| Audit log covers reads | ✓ `audit_event()` called in `list_issues_impl` and `get_issue_inner` |
| Write stubs return "not supported" | ✓ All 3 write ops |

## Test Results

```
running 17 tests
test adf::tests::code_block_extracted ... ok
test adf::tests::hard_break_becomes_newline ... ok
test adf::tests::null_returns_empty ... ok
test adf::tests::simple_paragraph ... ok
test adf::tests::unknown_node_type_recurses ... ok
test tests::adf_description_strips_to_plain_text ... ok
test tests::extensions_omitted_when_empty ... ok
test tests::get_404_maps_to_not_found ... ok
test tests::get_by_numeric_id ... ok
test tests::list_pagination_cursor ... ok
test tests::list_single_page ... ok
test tests::parent_hierarchy ... ok
test tests::rate_limit_429_honors_retry_after ... ok
test tests::status_mapping_matrix ... ok
test tests::supports_reports_hierarchy_only ... ok
test tests::tenant_validation_rejects_ssrf ... ok
test tests::write_ops_return_not_supported ... ok

test result: ok. 17 passed; 0 failed
```

`cargo clippy -p reposix-jira --all-targets -- -D warnings`: clean  
Workspace: 0 failures

## Deviations

- `serde_yaml` added to `crates/reposix-jira/Cargo.toml` (not in plan's dep list but required for `Issue.extensions: BTreeMap<String, serde_yaml::Value>`)
- `arm_rate_limit_backoff` marked `#[allow(dead_code)]` — tested in wiremock test 8, production retry wired in Phase 29
- `"new"` arm removed from `map_status` match (merged with `_` wildcard per clippy `match_same_arms`) — semantics unchanged

## Self-Check: PASSED
