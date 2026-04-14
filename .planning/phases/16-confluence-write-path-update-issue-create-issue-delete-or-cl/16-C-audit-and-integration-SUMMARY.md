---
phase: 16
wave: C
slug: audit-and-integration
status: SHIPPED
completed: 2026-04-14
duration_approx: 60m
tasks_completed: 6
tasks_total: 6
test_count_before: 308
test_count_after: 317
tests_added: 9
commits:
  - hash: b4f538a
    message: "feat(16-C): add rusqlite+sha2 deps to reposix-confluence"
  - hash: 34a704c
    message: "feat(16-C): add audit field + with_audit builder; add audit_write helper; wire into create/update/delete"
  - hash: 6504713
    message: "feat(16-C): switch get_issue to ADF body format with storage fallback"
  - hash: c4614a0
    message: "feat(16-C): add audit unit tests (6) — update/create/delete write audit rows"
  - hash: 3918452
    message: "feat(16-C): add roundtrip integration test (WRITE-04 end-to-end) + cargo fmt"
key_files:
  modified:
    - Cargo.toml
    - crates/reposix-confluence/Cargo.toml
    - crates/reposix-confluence/src/lib.rs
    - crates/reposix-confluence/tests/contract.rs
  created:
    - crates/reposix-confluence/tests/roundtrip.rs
subsystem: reposix-confluence
tags: [audit-log, adf, read-path, write-path, confluence, roundtrip, wave-c]
---

# Phase 16 Wave C: Audit and Integration SUMMARY

## One-liner

`ConfluenceBackend` now writes an audit row to a caller-supplied `rusqlite` connection on every `create_issue`/`update_issue`/`delete_or_close` call, the read path requests ADF body format (falling back to storage for pre-ADF pages), and a wiremock round-trip integration test proves WRITE-04 end-to-end.

## Goal

Wire the Phase 1 audit log schema (SG-06) into `ConfluenceBackend`, flip the read path to `?body-format=atlas_doc_format` with a storage fallback, and prove WRITE-04 with a create→read→body-matches round-trip integration test.

## Status: SHIPPED

All 6 tasks completed; all verification gates passed.

## Tasks Completed

| Task | Description | Commit |
|------|-------------|--------|
| C1 | Add `rusqlite` + `sha2` to workspace deps; add both to `reposix-confluence/Cargo.toml` | b4f538a |
| C2 | Add `audit: Option<Arc<Mutex<Connection>>>` field; `with_audit` builder; update `Debug` impl | 34a704c |
| C3 | Add private `audit_write` helper (best-effort, log-and-swallow); wire into create/update/delete on both success and failure paths | 34a704c |
| C4 | Switch `get_issue` to `?body-format=atlas_doc_format` with storage fallback; add `ConfBodyAdf` struct; update `translate`; update all mocks; add `get_issue_falls_back_to_storage_when_adf_empty` test | 6504713 |
| C5 | 6 audit unit tests with in-memory SQLite: update/create/delete write rows, correct method+path shape, insert failure does not mask write, failed writes still get audited | c4614a0 |
| C6 | `tests/roundtrip.rs`: `create_then_get_roundtrip_with_audit` (POST→GET→body-matches+audit), plus delete sanity test | 3918452 |

## Verification Gates

| Gate | Result |
|------|--------|
| `cargo test -p reposix-confluence` | 65 passed, 0 failed |
| `cargo test -p reposix-confluence --test roundtrip` | 2 passed, 0 failed |
| `cargo test -p reposix-confluence --test contract` | 5 passed, 0 failed, 2 ignored |
| `cargo test --workspace` | 317 total, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | passes |

## Test Count

- Before: **308** (after Wave B)
- After: **317** (+9 net new)
- Required minimum: 315 — **exceeded**

## Success Criteria Verification

1. `with_audit` ≥2 hits in lib.rs — PASS (9 hits: definition, doc, builder body, test call sites)
2. `audit_write(` ≥3 hits across write methods — PASS (4 hits: create, update, delete success, delete failure)
3. `body-format=atlas_doc_format` in `get_issue` — PASS (line 822)
4. `cargo test --test roundtrip` green — PASS
5. Workspace test count ≥315 — PASS (317)
6. `cargo clippy` clean — PASS
7. No existing read-path test regressed — PASS (all contract.rs + inline get_issue tests green after mock updates)

## Implementation Notes

### ADF body format and serde shape

Confluence returns `body.atlas_doc_format.value` as a JSON object when `?body-format=atlas_doc_format` is requested. Added `ConfBodyAdf { value: serde_json::Value }` and extended `ConfPageBody` with `#[serde(default, rename = "atlas_doc_format")] adf: Option<ConfBodyAdf>`. The `translate` function checks for ADF first; if absent or null (pre-ADF pages), it falls back to `body.storage.value`.

### Fallback condition

The fallback to storage is triggered when `body.atlas_doc_format` is absent (`None` after deserialization) or when `value` is a JSON `null`. This correctly handles both pre-ADF pages and pages where Confluence returns an empty ADF object.

### audit_write path format

Stable path format: `/wiki/api/v2/pages` for POST, `/wiki/api/v2/pages/{id}` for PUT and DELETE. No query strings in the audit path column (matches the sim audit middleware convention).

### Test mock updates

All existing `get_issue` tests that previously matched `?body-format=storage` were updated to respond to `?body-format=atlas_doc_format` with `page_json_adf(...)`. The SSRF contract tests were similarly updated: the adversarial URL fields are now embedded in ADF text nodes rather than storage HTML, and the `body-exfil` assertion still passes because `adf_to_markdown` renders text nodes verbatim.

### page_json_adf helper

Added a test helper `fn page_json_adf(id, status, title, adf_doc: &Value)` alongside the existing `page_json`. Takes the ADF doc by reference (clippy `needless_pass_by_value` compliance) and clones internally for the `json!` macro.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Style] Clippy `needless_pass_by_value` on `page_json_adf`**
- **Found during:** Verification gate (clippy)
- **Issue:** Test helper `page_json_adf` took `adf_doc: serde_json::Value` by value but clippy flagged it as `needless_pass_by_value` since the macro can take a reference.
- **Fix:** Changed to `adf_doc: &serde_json::Value`, clone inside function.
- **Files modified:** `crates/reposix-confluence/src/lib.rs`
- **Commit:** 3918452

**2. [Rule 1 - Style] `cargo fmt` reformatting**
- **Found during:** Verification gate (fmt)
- **Issue:** `audit_write` call in `create_issue` exceeded line width; `page_json_adf` signature needed multi-line format.
- **Fix:** `cargo fmt --all` applied.
- **Files modified:** `crates/reposix-confluence/src/lib.rs`
- **Commit:** 3918452

**3. [Rule 1 - Style] Clippy `doc_markdown` on test helper doc comment**
- **Found during:** Verification gate (clippy)
- **Issue:** Doc comment said "SQLite" without backticks; clippy pedantic requires `` `SQLite` ``.
- **Fix:** Added backticks.
- **Files modified:** `crates/reposix-confluence/src/lib.rs`
- **Commit:** c4614a0

### Plan Deviations

**C6 has 2 tests instead of 1.** The plan specified 1 integration test (`create_then_get_roundtrip_with_audit`). A second test (`delete_or_close_is_audited_in_integration_context`) was added as an integration-level sanity check that DELETE audit wiring is complete. This is additional coverage, not a regression.

## Known Stubs

None.

## Threat Flags

No new network endpoints introduced. All write methods route through `HttpClient` which re-checks `REPOSIX_ALLOWED_ORIGINS` (SG-01). The audit connection is a local SQLite handle — no new network surface.

## Self-Check: PASSED

- `crates/reposix-confluence/src/lib.rs` modified with C2/C3/C4/C5 changes
- `crates/reposix-confluence/tests/roundtrip.rs` created (C6)
- `crates/reposix-confluence/tests/contract.rs` updated (C4 mock updates)
- Commits b4f538a, 34a704c, 6504713, c4614a0, 3918452 all present in git log
- 317 total workspace tests (verified by `cargo test --workspace`)
- Clippy and fmt gates both clean

## Unblocks

Wave D (documentation, changelog, version bump) can now proceed. All three write methods are audited, the read path uses ADF, and the round-trip is proven end-to-end.
