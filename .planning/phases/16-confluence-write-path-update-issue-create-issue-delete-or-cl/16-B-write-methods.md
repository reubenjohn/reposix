# Plan: Wave B — Confluence write methods (`create_issue` / `update_issue` / `delete_or_close`) + struct rename

## Goal

Replace the three `Err(Error::Other("not supported: …"))` stubs on `ConfluenceReadOnlyBackend` with real Confluence REST v2 implementations, rename the struct to `ConfluenceBackend`, update the `supports()` capability matrix, and cover every new branch with `wiremock` unit tests — still no audit log (Wave C) and no read-path ADF switch (Wave D).

## Wave

B (depends on: Wave A merged — `markdown_to_storage` is used by `create_issue` + `update_issue`; unblocks: Wave C + Wave D).

## Addresses

- **Requirements:** WRITE-01 (create), WRITE-02 (update with optimistic locking), WRITE-03 (delete).
- **Locked decisions:**
  - **LD-16-01** — write path routes through `IssueBackend` trait (we only implement the trait; no new public methods).
  - **LD-16-02** — `Untainted<Issue>` parameter type enforces `sanitize()` was called upstream; we don't re-sanitize.

## Tasks

### B1. Rename `ConfluenceReadOnlyBackend` → `ConfluenceBackend` across the workspace

- **Files (edit — at minimum):**
  - `crates/reposix-confluence/src/lib.rs` — struct definition, impl blocks, all internal references, `name()` return value (`"confluence-readonly"` → `"confluence"`), module doc-comments, every intra-doc link (`[`ConfluenceReadOnlyBackend`]` → `[`ConfluenceBackend`]`), crate-level `//!` doc, the `User-Agent` header string (`"reposix-confluence-readonly/0.3"` → `"reposix-confluence/0.6"`).
  - `crates/reposix-confluence/tests/contract.rs` — `use reposix_confluence::{ConfluenceCreds, ConfluenceReadOnlyBackend};` → `ConfluenceBackend`; every constructor call.
  - `crates/reposix-cli/src/list.rs` — lines 22, 84, 85 (`.context("build ConfluenceReadOnlyBackend")`).
  - `crates/reposix-fuse/src/main.rs` — lines 24, 135.
  - `crates/reposix-fuse/tests/nested_layout.rs` — lines 36, 115, 129.
  - `crates/reposix-fuse/Cargo.toml` — comment on line 19.
- **Action:** Global rename. Use `rg -l 'ConfluenceReadOnlyBackend' crates/` first to confirm the call-site set, then rename. No backward-compat type alias — v0.6.0 is pre-1.0 (RESEARCH §Codebase Patterns §Struct rename).
- **Expected line impact:** ~40 edited lines across ~6 files; net diff near-zero.
- **Verification:** `cargo check --workspace` passes; `rg 'ConfluenceReadOnlyBackend' crates/` returns 0 hits.

### B2. Update `supports()` + add JSON `write_headers()` helper

- **File:** `crates/reposix-confluence/src/lib.rs`
- **Action:**
  - Change `supports()` to return `true` for `Hierarchy | Delete | StrongVersioning` (RESEARCH §Codebase Patterns §`supports()` matrix update):
    ```rust
    fn supports(&self, feature: BackendFeature) -> bool {
        matches!(
            feature,
            BackendFeature::Hierarchy
                | BackendFeature::Delete
                | BackendFeature::StrongVersioning
        )
    }
    ```
  - Add a private helper `fn write_headers(&self) -> Vec<(&'static str, String)>` that clones `standard_headers()` and pushes `("Content-Type", "application/json".to_owned())`. Document why it's separate from `standard_headers()` (GET paths don't need Content-Type).
- **Expected line impact:** ~15 lines.
- **Verification:** `cargo test -p reposix-confluence supports` (if an existing test asserts the supports matrix; otherwise this becomes a new unit test `supports_lists_delete_hierarchy_strong_versioning`).

### B3. Implement `update_issue` against `PUT /wiki/api/v2/pages/{id}`

- **File:** `crates/reposix-confluence/src/lib.rs` (replace the stub at ~L707–717)
- **Action:** Wire the PUT call with optimistic locking. Pseudocode (follow exactly the pattern from `list_issues`/`get_issue` for rate-limit gate + header construction):
  1. **Pre-flight version resolution.** If `expected_version` is `Some(v)`, trust it; if `None`, do a GET via the existing `get_issue` logic (extract to a private helper `fetch_current_version(&self, id: IssueId) -> Result<u64>` that just returns `issue.version`).
  2. Convert `patch.inner_ref().body` from Markdown to storage XHTML via `crate::adf::markdown_to_storage(&patch.inner_ref().body)?`.
  3. Build the PUT body:
     ```rust
     let put_body = serde_json::json!({
         "id": id.0.to_string(),
         "status": "current",
         "title": patch.inner_ref().title,
         "version": { "number": current_version + 1 },
         "body": { "representation": "storage", "value": storage_xhtml },
     });
     ```
  4. `await_rate_limit_gate()`; `request_with_headers(Method::PUT, url, &write_headers_refs)` — but `HttpClient::request_with_headers` does not accept a body. **Check the `HttpClient` API first**: if a body-accepting method like `put_json` or `send_json` exists, use it; if only `request_with_headers` exists, extend the trait / client with a new method `request_with_json(method, url, headers, body: &serde_json::Value)` that internally calls `reqwest::RequestBuilder::json(body)`. Implementation note: this extension lives in `crates/reposix-core/src/http.rs` if needed — it's a one-method addition; do NOT bypass the sealed client.
  5. `ingest_rate_limit(&resp)`.
  6. Branch on status: `404 → Err(Error::Other("not found: {url}"))`, `409 → Err(Error::Other("version mismatch: <bytes>"))`, non-success → `Err(Error::Other("confluence returned {status} for PUT {url}: <bytes>"))`, success → `serde_json::from_slice::<ConfPage>(&bytes)` → `Tainted::new(page)` → `translate(tainted.into_inner())`.
- **Expected line impact:** ~60 lines (method body) + ~15 lines if extending `HttpClient`.
- **Verification:** `cargo test -p reposix-confluence update_issue` — new tests (B6) green; `cargo check --workspace` clean.

### B4. Implement `create_issue` against `POST /wiki/api/v2/pages`

- **File:** `crates/reposix-confluence/src/lib.rs`
- **Action:** Replace stub at ~L701–705.
  1. `let space_id = self.resolve_space_id(project).await?;` — `project` is the space key (matches `list_issues` convention).
  2. Convert body: `let storage_xhtml = crate::adf::markdown_to_storage(&issue.inner_ref().body)?;`.
  3. Build POST body:
     ```rust
     let post_body = serde_json::json!({
         "spaceId": space_id,
         "status": "current",
         "title": issue.inner_ref().title,
         "parentId": issue.inner_ref().parent_id.map(|id| id.0.to_string()),
         "body": { "representation": "storage", "value": storage_xhtml },
     });
     ```
  4. Same rate-limit + headers pattern as B3. URL: `{base}/wiki/api/v2/pages`.
  5. Branch: non-success → `Err(Error::Other(…))`; success → `ConfPage` → `Tainted::new` → `translate(…)`.
- **Expected line impact:** ~50 lines.
- **Verification:** `cargo test -p reposix-confluence create_issue` green.

### B5. Implement `delete_or_close` against `DELETE /wiki/api/v2/pages/{id}`

- **File:** `crates/reposix-confluence/src/lib.rs`
- **Action:** Replace stub at ~L719–728.
  1. URL: `{base}/wiki/api/v2/pages/{id.0}`. `reason: DeleteReason` is ignored (documented in a `/// Note:` on the method body — Confluence has no reason field; status becomes `"trashed"` which the read path already maps to `Done`).
  2. No body; `await_rate_limit_gate`; `request_with_headers(Method::DELETE, url, &header_refs)` with `standard_headers` (no Content-Type needed for DELETE with empty body).
  3. Branch: `204 → Ok(())`, `404 → Err(Error::Other("not found: {url}"))`, other → `Err(Error::Other("confluence returned {status} for DELETE {url}: <bytes>"))`.
- **Expected line impact:** ~35 lines.
- **Verification:** `cargo test -p reposix-confluence delete_or_close` green.

### B6. Wiremock unit tests for all three methods

- **File:** `crates/reposix-confluence/src/lib.rs` (extend the existing `#[cfg(test)] mod tests` block — same location as the existing list/get tests that already use `MockServer`)
- **Action:** Author the 10 tests from RESEARCH.md §Backend Write Method Wiremock Tests (exact names preserved for traceability to the VALIDATION map):

  | Test fn name | Wiremock mock | Assertion |
  |---|---|---|
  | `update_issue_sends_put_with_version` | `Mock::given(method("PUT")).and(path("/wiki/api/v2/pages/99")).respond_with(200 + page_json)` | Request body JSON has `version.number == 43` when pre-flight GET returned version 42; response `Issue::title` round-trips |
  | `update_issue_409_maps_to_version_mismatch` | PUT → 409 with `{"message":"stale"}` body | `Err` whose `to_string()` starts with `"version mismatch"` |
  | `update_issue_none_version_fetches_then_puts` | `GET /wiki/api/v2/pages/99` → 200 (version=7) + `PUT /wiki/api/v2/pages/99` → 200 | With `expected_version = None`, PUT body carries `version.number == 8` |
  | `update_issue_404_maps_to_not_found` | PUT → 404 | `Err` message contains `"not found"` |
  | `create_issue_posts_to_pages` | `GET /wiki/api/v2/spaces?keys=REPOSIX` → 200 (space id 12345) + `POST /wiki/api/v2/pages` → 200 | POST body has `spaceId == "12345"`, `title == issue.title`; returned `Issue` has the server-assigned id |
  | `create_issue_with_parent_id` | POST → 200; request body matcher | `parent_id: Some(IssueId(42))` → POST body has `parentId == "42"` |
  | `create_issue_without_parent_id_sends_null` | POST → 200; request body matcher | `parent_id: None` → POST body has `parentId == null` (or field absent — pick one and lock it) |
  | `delete_or_close_sends_delete` | `DELETE /wiki/api/v2/pages/99` → 204 | `Ok(())` |
  | `delete_or_close_404_maps_to_not_found` | DELETE → 404 | `Err` message contains `"not found"` |
  | `write_methods_send_content_type_json` | Custom matcher on each of POST/PUT | Header `Content-Type: application/json` present |
  | `write_methods_send_basic_auth` | Reuse existing `BasicAuthMatches` | Header `Authorization: Basic …` present on all three |
  | `rate_limit_gate_shared_with_writes` | `GET` returns 429 with `Retry-After: 1`; subsequent PUT succeeds | Elapsed wall time ≥ 1 second between the GET completing and the PUT starting |

  **Test helpers:** Reuse the existing `creds()` helper and `MockServer`. If `BasicAuthMatches` doesn't exist yet as a shared matcher, grep for the existing header-assertion pattern in this file and re-use.

- **Expected line impact:** ~250–350 lines of test code.
- **Verification:** `cargo test -p reposix-confluence` runs ≥12 new tests (green); total crate test count jumps by at least 12.

### B7. `supports_lists_delete_hierarchy_strong_versioning` unit test

- **File:** `crates/reposix-confluence/src/lib.rs` (tests mod)
- **Action:** One tiny unit test that constructs a `ConfluenceBackend` with `new_with_base_url("http://127.0.0.1:1")` (no HTTP needed — just to instantiate) and asserts `supports(Hierarchy)`, `supports(Delete)`, `supports(StrongVersioning)` are all true, while `supports(BulkEdit)` and `supports(Workflows)` are false.
- **Expected line impact:** ~20 lines.
- **Verification:** Runs as part of `cargo test -p reposix-confluence`.

## Verification

Before merging Wave B:

```bash
cargo test -p reposix-confluence                                     # crate: ADF + write-method tests green
cargo test --workspace                                               # no regressions elsewhere
cargo clippy --workspace --all-targets -- -D warnings                # clean
cargo fmt --all --check                                              # formatted
rg 'ConfluenceReadOnlyBackend' crates/                               # 0 hits
```

All must pass. Test count expected after Wave B ≈ 293 (post-A) + ≥13 (B6) + 1 (B7) = **307+**.

## Threat model

| Threat ID | STRIDE | Component | Disposition | Mitigation |
|---|---|---|---|---|
| T-16-B-01 | Tampering | PUT body echoes Markdown from a tainted source | Mitigate | Signature is `Untainted<Issue>` → `sanitize()` was called upstream (LD-16-02). The FUSE write path already wraps in `Untainted` before calling the trait method. No re-sanitization inside the backend. |
| T-16-B-02 | Tampering | SSRF via page-id string injection | Mitigate | `IssueId(u64)` is numeric by construction — `format!("/wiki/api/v2/pages/{}", id.0)` cannot smuggle `/../`. Enforced by the type system. No test needed beyond the existing `validate_issue_filename` coverage. |
| T-16-B-03 | Tampering | Request lands on wrong origin | Mitigate | All calls go through the sealed `HttpClient` which re-checks `REPOSIX_ALLOWED_ORIGINS` on every request (SG-01). No new URL-construction paths bypass it. |
| T-16-B-04 | Repudiation | Version mismatch silently succeeds | Mitigate | `version.number = current + 1` is covered by `update_issue_sends_put_with_version` and `update_issue_none_version_fetches_then_puts`. 409 → typed error covered by `update_issue_409_maps_to_version_mismatch`. |
| T-16-B-05 | DoS | Write-path rate-limit not honored | Mitigate | Reuse the existing `rate_limit_gate` / `await_rate_limit_gate` / `ingest_rate_limit` helpers on every write call. `rate_limit_gate_shared_with_writes` test locks it in. |
| T-16-B-06 | Information Disclosure | Credential leak in error-body log | Accept | Error messages include the URL but not the `Authorization` header. Existing `ConfluenceCreds` `Debug` redaction carries forward to the renamed struct (RESEARCH §T-11-01). |
| T-16-B-07 | Tampering | Body converted via `markdown_to_storage` emits unsafe XHTML | Accept | Documented in Wave A T-16-A-04; Confluence server-side strips unknown tags. Future hardening in v0.7.0. |

## Success criteria

1. `cargo test -p reposix-confluence` reports ≥28 tests green (Wave A ≥15 + Wave B ≥13) and zero `not supported: create/update/delete` strings remain in the codebase (`rg 'not supported: (create|update|delete)' crates/reposix-confluence/` → 0).
2. `rg 'ConfluenceReadOnlyBackend' crates/` returns 0 hits; `rg 'ConfluenceBackend' crates/` returns >10 hits (struct + callers).
3. `supports(BackendFeature::Delete)` and `supports(BackendFeature::StrongVersioning)` both return `true`, locked by B7's test.
4. All write methods' request bodies carry `Content-Type: application/json`, locked by `write_methods_send_content_type_json`.
5. `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean.
6. Workspace test count ≥ 307 (baseline 278 + ≥15 from A + ≥14 from B).
