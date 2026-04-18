---
id: 29-01
status: complete
commit: 10d24ba
---

# Plan 29-01 Summary — ADF write encoder + issuetype cache infrastructure

## What shipped

### Task T1: `adf_paragraph_wrap`
Added `pub fn adf_paragraph_wrap(text: &str) -> serde_json::Value` to `crates/reposix-jira/src/adf.rs`. Wraps plain text in minimal ADF doc structure (`doc > paragraph > text`) for JIRA REST v3 write requests. Two unit tests + one doctest all pass.

### Task T2: `adf_to_markdown`
Added `pub fn adf_to_markdown(adf: &serde_json::Value) -> Result<String, reposix_core::Error>` with full node visitor (paragraph, heading, codeBlock, bulletList, orderedList, listItem, text with marks, hardBreak). Copy-adapted from `reposix-confluence/src/adf.rs`. `MAX_ADF_DEPTH = 50`, `FALLBACK_PREFIX = "[unknown-adf:"`. Seven new unit tests pass including deep-nesting overflow safety.

### Task T3: `JiraBackend` struct + read path upgrade
- Added `use std::sync::OnceLock` import to `lib.rs`
- Added `issue_type_cache: Arc<OnceLock<Vec<String>>>` field to `JiraBackend` struct (with `#[allow(dead_code)]` until 29-02 wires it in)
- Initialized field in `new_with_base_url()` constructor
- Updated `translate()` read path: `adf_to_markdown` with `adf_to_plain_text` fallback for non-null descriptions; null → empty string directly

## Test results
- 26 unit tests pass, 2 doc tests pass, 2 contract tests pass
- `cargo clippy -p reposix-jira -- -D warnings`: clean
- `cargo fmt --check -p reposix-jira`: clean

## Files modified
- `crates/reposix-jira/src/adf.rs` (+300 lines: new functions + private helpers + tests)
- `crates/reposix-jira/src/lib.rs` (+10 lines: import, struct field, constructor init, read path)
