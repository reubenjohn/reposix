# Phase 16: Confluence Write Path — Research

**Researched:** 2026-04-14
**Domain:** Confluence Cloud REST v2 write path + ADF↔Markdown converter in Rust
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **LD-16-01:** Write path routes through `IssueBackend` trait, not direct REST calls.
- **LD-16-02:** `Tainted<T>` wraps all bytes from Confluence responses; `sanitize()` strips server-controlled frontmatter fields (`id`, `created_at`, `version`, `updated_at`) on the inbound path.
- **LD-16-03:** Every write call gets an audit log row (sim WAL pattern).

### Claude's Discretion
- Scope of write methods (all 3 vs just update)
- ADF round-trip fidelity level
- Optimistic locking implementation style
- Rate-limit retry-after handling mechanics
- How space key / page ID flow through `IssueBackend::create_issue`

### Deferred Ideas (OUT OF SCOPE)
- Confluence comments write path (Phase 23 OP-9a)
- Attachment upload
- Subprocess/JSON-RPC connector ABI (Phase 12)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WRITE-01 | Agent can create a new Confluence page by writing a new `.md` file in the FUSE mount (`ConfluenceBackend::create_issue`) | POST `/wiki/api/v2/pages` with `spaceId` from `project`, ADF or storage body |
| WRITE-02 | Agent can update a Confluence page by editing its `.md` file in the FUSE mount (`ConfluenceBackend::update_issue`) | PUT `/wiki/api/v2/pages/{id}` with `version.number` for optimistic locking |
| WRITE-03 | Agent can delete/close a Confluence page by unlinking its `.md` file (`ConfluenceBackend::delete_or_close`) | DELETE `/wiki/api/v2/pages/{id}` → 204 No Content |
| WRITE-04 | Page bodies round-trip through `atlas_doc_format` ↔ Markdown conversion with no data loss for headings, paragraphs, and code blocks | Hand-rolled ADF converter module; no external crate needed |
</phase_requirements>

---

## Summary

Phase 16 adds write capability to `ConfluenceReadOnlyBackend` (rename to `ConfluenceBackend`) by implementing all three `IssueBackend` write methods against the Confluence Cloud REST v2 API. The Confluence write API uses standard POST/PUT/DELETE endpoints with JSON bodies containing either `storage` (XHTML) or `atlas_doc_format` (ADF) representation. Optimistic locking is achieved by passing the current `version.number` in PUT bodies — a 409 is returned on conflict, which maps to the existing `"version mismatch: ..."` error convention.

The ADF ↔ Markdown converter is the most novel piece: it must be hand-rolled as a small module inside `reposix-confluence` (no suitable Rust ADF crate exists with compatible licensing and maintenance status). The converter only needs to handle a constrained set of constructs (headings H1–H6, paragraphs, fenced code blocks with optional language, bullet lists, ordered lists, inline code) — which maps cleanly to pulldown-cmark events for the Markdown→ADF direction and a recursive ADF-node visitor for the ADF→Markdown direction.

The audit log requirement (LD-16-03) is implemented by inserting rows into a local `rusqlite` connection carried on the backend struct — mirroring how the sim's audit middleware works, but client-side since there is no reposix server in the Confluence path.

**Primary recommendation:** Implement all three write methods (create / update / delete) plus a minimal ADF converter. Split into Wave A (ADF converter + unit tests), Wave B (write methods on the backend + wiremock tests), Wave C (audit row + integration proof), Wave D (rename struct, `BackendFeature::Delete|StrongVersioning`, version bump + docs).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ADF ↔ Markdown conversion | `reposix-confluence` crate | — | Format conversion lives at the adapter boundary where Confluence's storage shape is first/last seen |
| Optimistic locking (version number) | `reposix-confluence` crate | `IssueBackend` trait (caller supplies `expected_version`) | Confluence owns version semantics; trait already carries `Option<u64>` for this purpose |
| Audit log (write rows) | `reposix-confluence` crate | — | Confluence calls go directly to Atlassian; no server middleware exists in this path |
| Rate-limit gate | `reposix-confluence` crate | — | Already implemented via `rate_limit_gate: Arc<Mutex<Option<Instant>>>` — shared across read+write |
| HTTP transport (SG-01 allowlist) | `reposix-core::http::HttpClient` | — | All backends reuse the sealed client factory; Confluence adds no bypass |
| Tainted/Untainted discipline | `reposix-core::taint` | `reposix-confluence` (calls `sanitize`) | Core owns the type; confluence crate applies sanitize on inbound response |

---

## Technical Approach

### 1. update_issue → PUT /wiki/api/v2/pages/{id}

**Request body:**
```json
{
  "id": "<page_id_string>",
  "status": "current",
  "title": "<issue.title>",
  "version": { "number": <current_version + 1> },
  "body": {
    "representation": "storage",
    "value": "<issue.body converted to XHTML or ADF>"
  }
}
```
[VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-page]

**Version number semantics:** Confluence REST v2 PUT requires `version.number` equal to the **next** version (current + 1), not the current version. If another edit happened concurrently, Confluence returns 409 Conflict. This is the same `"version mismatch"` error the sim already uses.

**`expected_version` mapping:**
- `Some(v)` → send `version.number: v + 1` in the PUT body and validate the pre-flight GET returns `v`.
- `None` → fetch current version via GET, then PUT with `version.number: fetched + 1`.
In both cases the implementation must do a pre-flight read OR use the cached version from the issue store to avoid an extra round-trip.

**Conflict (409) handling:** Same as `SimBackend::update_issue` — map to `Err(Error::Other("version mismatch: ..."))`.

**Body representation choice:** Use `"storage"` (XHTML) for now rather than ADF on the wire — the existing read path already stores XHTML in `issue.body` when `?body-format=storage` is requested. The ADF converter serves the conversion layer above the wire; the wire body is always `"storage"` representation in the PUT/POST until WRITE-04 is verified end-to-end. [ASSUMED: Using storage representation avoids the ADF complexity for the write path; Confluence accepts storage on writes too]

**Returns:** The server's canonical page object (same shape as GET). Translate via `translate()` and return as `Issue`.

### 2. create_issue → POST /wiki/api/v2/pages

**Request body:**
```json
{
  "spaceId": "<resolved_numeric_space_id>",
  "status": "current",
  "title": "<issue.title>",
  "parentId": "<parent_page_id_string or null>",
  "body": {
    "representation": "storage",
    "value": "<issue.body>"
  }
}
```
[VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-page]

**Space key → spaceId:** The `project` argument to `IssueBackend::create_issue` is a space key (e.g. `"REPOSIX"`). The existing `resolve_space_id` method handles this lookup. No API change needed.

**Parent page ID:** `Issue::parent_id` carries the optional parent page ID. The `create_issue` implementation extracts `issue.inner_ref().parent_id.map(|id| id.0.to_string())`.

**Returns 200** with the created page body. Translate via `translate()`.

### 3. delete_or_close → DELETE /wiki/api/v2/pages/{id}

**Behavior:** Confluence moves the page to trash (not permanent deletion). A future "purge" would require `?purge=true`. For this phase, plain `DELETE /wiki/api/v2/pages/{id}` is sufficient — it maps to trash, which maps to `IssueStatus::Done` on the read path anyway.
[VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-page]

**Returns 204 No Content** on success. Any non-204 non-404 is an error. 404 → `"not found: ..."` per the existing error convention.

**`DeleteReason` mapping:** Confluence has no reason field on DELETE; `reason` is ignored (same as `SimBackend` today for non-reason backends).

### 4. ADF ↔ Markdown Converter (WRITE-04)

**Location:** New submodule `crates/reposix-confluence/src/adf.rs`.

**Minimal construct set (required for WRITE-04):**
- Headings H1–H6
- Paragraph
- Fenced code block (with optional language attribute)
- Inline code (code mark on text)
- Bullet list (`bulletList`) + `listItem`
- Ordered list (`orderedList`) + `listItem`
- Plain text

**ADF JSON root structure:**
```json
{
  "version": 1,
  "type": "doc",
  "content": [ /* block nodes */ ]
}
```
[VERIFIED: developer.atlassian.com/cloud/jira/platform/apis/document/structure]

**Markdown → ADF (for POST/PUT bodies):**
Use `pulldown-cmark` (already available via `comrak` is an alternative, but `pulldown-cmark` is the lighter-weight pull parser). Parse the Markdown body into events and emit ADF JSON using `serde_json::Value`. No external ADF crate needed.
[VERIFIED: crates.io pulldown-cmark 0.13.3]

**ADF → Markdown (for GET bodies):**
Walk the ADF JSON tree recursively (`serde_json::Value` traversal) and emit Markdown text. The existing `get_issue` call requests `?body-format=storage` (XHTML). To get ADF on the read side, change the query to `?body-format=atlas_doc_format` and deserialize into `serde_json::Value` rather than a flat string. The converter emits a Markdown string stored in `issue.body`.

**Decision on body format for read path:** Currently `get_issue` stores raw XHTML (`body-format=storage`) in `issue.body`. For WRITE-04 we need Markdown round-trip. The cleanest approach:
1. Change `get_issue` to request `?body-format=atlas_doc_format`
2. The response body is a JSON string; deserialize it and pass through the ADF→Markdown converter
3. Store the result in `issue.body`

This is a single-file change in `get_issue` guarded by a new `#[cfg(feature = "adf")]` or simply unconditional in Phase 16. [ASSUMED: ADF format request works and returns valid JSON for all content types; storage format will be used as fallback if ADF response parse fails]

---

## Design Decisions Resolved

### Q1: Scope — all three methods or just update?

**Decision: Implement all three (create + update + delete).**

Rationale: Each method is 30–50 lines. Stopping at `update` only satisfies WRITE-02 and leaves WRITE-01/WRITE-03 open. The FUSE write path (REQUIREMENTS.md) requires all three for agents to manage pages. The risk of "just update" is that Phase 17 (swarm confluence-direct) would then have a half-working backend. All three share the same infrastructure (rate-limit gate, audit log, ADF converter) so the marginal cost of create + delete is low.

### Q2: ADF round-trip fidelity

**Decision: Minimum set — headings H1-H6, paragraphs, fenced code blocks, inline code, bullet lists, ordered lists.**

Tables, footnotes, panels, expand macros, and other ADF node types are emitted as their raw JSON string (fallback: `[unsupported ADF node type=X]` comment) to avoid silent data loss. The converter module documents the supported set and the fallback behavior. This is sufficient for WRITE-04 as specified in REQUIREMENTS.md.

**Body format strategy:**
- Write path (create/update): accept Markdown body → convert to `storage` XHTML via pulldown-cmark → HTML render. This avoids needing a Markdown-to-ADF converter on the write side; Confluence accepts `storage` representation on writes. [ASSUMED: storage representation write + atlas_doc_format read is a valid round-trip; needs validation against real tenant]
- Read path (get_issue): switch from `?body-format=storage` to `?body-format=atlas_doc_format`; run ADF→Markdown on the result.

Alternative simpler approach (if ADF-on-read proves unreliable): keep `?body-format=storage`, add a lightweight XHTML→Markdown converter instead. This is the fallback plan if ADF read turns out to return empty or malformed for certain page types.

### Q3: Conflict detection — does it mirror sim's If-Match?

**Decision: Yes, implement optimistic locking via `version.number` in the PUT body.**

The sim uses `If-Match: "<version>"` HTTP header. Confluence uses `version.number` in the request body (incremented by 1). The trait already carries `expected_version: Option<u64>` in `update_issue`. Mapping:
- `expected_version = Some(v)` → validate the fetched version matches `v`, then PUT with `version.number: v + 1`.
- `expected_version = None` → fetch current version via GET first, PUT with `fetched_version + 1`.
- 409 from Confluence → `Err(Error::Other("version mismatch: ..."))` — same string as the sim.

The `BackendFeature::StrongVersioning` flag should be added to `supports()` for the new backend, just as it is set on `SimBackend`.

### Q4: Rate-limit handling (429 + Retry-After)

**Decision: Reuse the existing `rate_limit_gate` and `await_rate_limit_gate` pattern that already works on the read path.**

The existing `ingest_rate_limit` + `await_rate_limit_gate` methods already handle `x-ratelimit-remaining: 0` and `429 + Retry-After`. Write methods call `await_rate_limit_gate()` before the HTTP call and `ingest_rate_limit(&resp)` after — identical to the read path. No new logic is needed.

### Q5: Space key vs page ID in create_issue — how through IssueBackend trait?

**Decision: Use the `project: &str` argument as the space key (current convention) and `issue.inner_ref().parent_id` for the optional parent page ID.**

The trait signature is `create_issue(&self, project: &str, issue: Untainted<Issue>) -> Result<Issue>`. The `project` string is already used as the space key in `list_issues` (via `resolve_space_id`). For `create_issue`, the same convention applies: `project = space_key`, resolved to a numeric `spaceId` via one GET call. The `parent_id: Option<IssueId>` field on `Issue` carries the optional parent page ID.

No trait signature change is needed. This is consistent with how `list_issues` uses `project`.

---

## Codebase Patterns

### Struct rename

`ConfluenceReadOnlyBackend` → `ConfluenceBackend`. The module-level doc comment and the `name()` method return value also update. Public API: only `reposix-confluence` re-exports these publicly; a type alias `pub type ConfluenceReadOnlyBackend = ConfluenceBackend;` is optional for semver but not needed since v0.5.0 is pre-1.0.

### Standard headers for write requests

Add `Content-Type: application/json` to `standard_headers()` or create a write-specific `write_headers()` helper, matching the `SimBackend` pattern of `json_headers()`:

```rust
// [VERIFIED: crates/reposix-core/src/backend/sim.rs]
fn write_headers(&self) -> Vec<(&'static str, String)> {
    let mut h = self.standard_headers();
    h.push(("Content-Type", "application/json".to_owned()));
    h
}
```

### Audit log pattern for Confluence

The sim's audit log is written by the server-side axum middleware. Confluence has no such server; the `ConfluenceBackend` must write its own audit row. The backend needs an `Arc<Mutex<rusqlite::Connection>>` optional field (or a separate `AuditWriter` type). Pattern mirrors `reposix-sim/src/state.rs::AppState`.

**Concrete design:** Add `audit: Option<Arc<parking_lot::Mutex<rusqlite::Connection>>>` to `ConfluenceBackend`. If `None`, audit is silently skipped (for tests and cases where no audit path is configured). CLI callers that want audit rows pass an `open_audit_db(path)` connection at construction. The `ConfluenceBackend::builder()` pattern (or a new constructor) accepts the audit connection.

**Audit row format:** Reuse the existing `audit_events` table schema (`ts`, `agent_id`, `method`, `path`, `status`, `request_body`, `response_summary`). For write calls:
- `method`: `"POST"` / `"PUT"` / `"DELETE"`
- `path`: the Confluence REST path, e.g. `/wiki/api/v2/pages/12345`
- `agent_id`: `"reposix-confluence-<pid>"`
- `request_body`: title + first 256 chars of body (truncated, never full page content)
- `response_summary`: `"<status>:<sha256_hex_prefix_16>"`

**Audit is best-effort:** If the insert fails, log-and-swallow (same pattern as the sim's audit middleware). The write to Confluence has already happened; don't mask the success by surfacing an audit failure.

### Error handling patterns

```rust
// [VERIFIED: crates/reposix-confluence/src/lib.rs]
if status == StatusCode::NOT_FOUND {
    return Err(Error::Other(format!("not found: {url}")));
}
if status == StatusCode::CONFLICT {
    let bytes = resp.bytes().await?;
    return Err(Error::Other(format!(
        "version mismatch: {}",
        String::from_utf8_lossy(&bytes)
    )));
}
if !status.is_success() {
    return Err(Error::Other(format!(
        "confluence returned {status} for {method} {url}: {}",
        String::from_utf8_lossy(&bytes)
    )));
}
```

### Tainted/Untainted discipline for write path

```rust
// Pattern from crates/reposix-fuse/src/fs.rs and crates/reposix-core/src/taint.rs
// For inbound server response (after PUT/POST):
let page: ConfPage = serde_json::from_slice(&bytes)?;
let tainted = Tainted::new(page);
translate(tainted.into_inner())
```

The `Untainted<Issue>` argument means server-controlled fields have already been stripped by `sanitize()` before the method is called. The body to send is `issue.inner_ref().body` (Markdown, convert to storage).

### `supports()` matrix update

```rust
// [VERIFIED: crates/reposix-confluence/src/lib.rs line 586]
fn supports(&self, feature: BackendFeature) -> bool {
    matches!(
        feature,
        BackendFeature::Hierarchy
            | BackendFeature::Delete        // new: real DELETE (moves to trash)
            | BackendFeature::StrongVersioning  // new: version.number OCC
    )
}
```

---

## Wave Breakdown

### Wave A — ADF converter module

**Goal:** Ship `crates/reposix-confluence/src/adf.rs` with:
- `markdown_to_storage(md: &str) -> String` — Markdown → XHTML storage (using pulldown-cmark HTML renderer as a starting point, then wrapping in `<p>` where needed)
- `adf_to_markdown(adf_json: &serde_json::Value) -> String` — ADF JSON tree → Markdown
- Unit tests for all supported constructs (headings, paragraphs, code blocks, lists, inline code)

**No network calls.** Pure transformation functions. Fully testable in isolation.

**Dependency to add:** `pulldown-cmark = "0.13"` to `crates/reposix-confluence/Cargo.toml` (dev dep only if only used for conversion utilities; workspace dep if used at runtime).

### Wave B — Backend write methods

**Goal:** Replace the three `Err(Error::Other("not supported: ..."))` stubs with real implementations:
1. `update_issue` — PUT with version +1, 409 → version mismatch
2. `create_issue` — POST with resolve_space_id, parent_id from issue
3. `delete_or_close` — DELETE → 204

**All three covered by wiremock tests** (no real network needed).

**Struct rename:** `ConfluenceReadOnlyBackend` → `ConfluenceBackend` in this wave.

**`supports()` update:** Add `Delete` and `StrongVersioning`.

### Wave C — Audit log

**Goal:** Add `audit: Option<Arc<parking_lot::Mutex<rusqlite::Connection>>>` to `ConfluenceBackend`. Add `with_audit` builder method. Wire up pre/post-write audit inserts in all three write methods. Test that audit rows are inserted (using in-memory SQLite with `open_audit_db`).

**Note:** The audit schema is already defined in `reposix-core/fixtures/audit.sql`. The `open_audit_db` function creates the table + triggers. Confluence just needs to be given a connection.

### Wave D — Read path ADF switch + WRITE-04 integration

**Goal:** Change `get_issue` to request `?body-format=atlas_doc_format` and run the ADF→Markdown converter on the result. Add an integration test that does a full round-trip: create page → read it back → body matches. Version bump to 0.6.0 and CHANGELOG.

---

## Test Strategy

### ADF Converter Unit Tests (Wave A)

Location: `crates/reposix-confluence/src/adf.rs` (inline `#[cfg(test)] mod tests`)

| Test | What it checks |
|------|----------------|
| `markdown_heading_h1_to_storage` | `# Heading` → `<h1>Heading</h1>` |
| `markdown_fenced_code_rust_to_storage` | ` ```rust\nfoo\n``` ` → `<ac:structured-macro>` or `<pre>` |
| `markdown_bullet_list_to_storage` | `- a\n- b` → `<ul><li>a</li><li>b</li></ul>` |
| `markdown_inline_code_to_storage` | `` `x` `` → `<code>x</code>` |
| `adf_paragraph_to_markdown` | `{type:"paragraph",content:[{type:"text",text:"hello"}]}` → `"hello"` |
| `adf_heading_h2_to_markdown` | `{type:"heading",attrs:{level:2},...}` → `"## Title"` |
| `adf_code_block_to_markdown` | `{type:"codeBlock",attrs:{language:"rust"},...}` → ` ```rust\ncode\n``` ` |
| `adf_bullet_list_to_markdown` | two-item bulletList → `"- item1\n- item2"` |
| `adf_inline_code_mark_to_markdown` | text node with `{marks:[{type:"code"}]}` → backtick-wrapped |
| `adf_unknown_node_type_fallback` | `{type:"panel",...}` → contains fallback marker, no panic |
| `roundtrip_heading_paragraph_list` | markdown → (storage) → (parse XHTML) → markdown approximately matches |

### Backend Write Method Wiremock Tests (Wave B)

Location: `crates/reposix-confluence/src/lib.rs` (existing `#[cfg(test)] mod tests` block)

| Test | Wiremock mock | What it asserts |
|------|---------------|-----------------|
| `update_issue_sends_put_with_version` | `PUT /wiki/api/v2/pages/99` → 200 + page_json | body contains `version.number = current+1`, returns translated Issue |
| `update_issue_409_maps_to_version_mismatch` | `PUT /wiki/api/v2/pages/99` → 409 | returns `Err` with "version mismatch" |
| `update_issue_none_version_fetches_then_puts` | `GET + PUT` combo | with `expected_version = None`, does a pre-flight GET |
| `create_issue_posts_to_pages` | `GET /spaces + POST /pages` → 200 + page_json | body has `spaceId`, `title`, response translated |
| `create_issue_with_parent_id` | POST with `parentId` | `parent_id: Some(IssueId(42))` → `parentId: "42"` in body |
| `delete_or_close_sends_delete` | `DELETE /wiki/api/v2/pages/99` → 204 | returns `Ok(())` |
| `delete_or_close_404_maps_to_not_found` | `DELETE /wiki/api/v2/pages/99` → 404 | returns `Err("not found: ...")` |
| `write_methods_send_content_type_json` | custom wiremock matcher | all three methods carry `Content-Type: application/json` |
| `write_methods_send_basic_auth` | `BasicAuthMatches` reused | auth header present on write calls |
| `rate_limit_gate_shared_with_writes` | 429 on GET then PUT succeeds | gate armed by read, write respects it |

### Audit Log Tests (Wave C)

Location: `crates/reposix-confluence/src/lib.rs`

| Test | What it checks |
|------|----------------|
| `update_issue_writes_audit_row` | After wiremock PUT call with audit conn, `SELECT COUNT(*) FROM audit_events` = 1 |
| `create_issue_writes_audit_row` | Same for POST |
| `delete_or_close_writes_audit_row` | Same for DELETE |
| `audit_row_has_correct_method_and_path` | Row has `method="PUT"`, `path="/wiki/api/v2/pages/99"` |
| `audit_insert_failure_does_not_mask_write_result` | DB locked (separate connection with WAL) → write still returns Ok |

---

## Risk Areas

### Risk 1: storage vs ADF representation round-trip fidelity

**What goes wrong:** Confluence's storage format uses Atlassian-specific XHTML macros (`<ac:structured-macro>`) for code blocks, tables, panels etc. The standard pulldown-cmark HTML output does not produce these macros. A page written as `<h1>` may not round-trip back as `# Heading` if Confluence re-normalizes the storage XML differently.

**Mitigation:** Test against a real Confluence tenant (contract test). For v0.6.0, document the limitation: bodies written via reposix will survive text edits but not macro-rich pages. ADF is the higher-fidelity path but also more complex.

**Warning sign:** `get_issue` returns body that differs structurally from what was PUT.

### Risk 2: version.number semantics (current vs next)

**What goes wrong:** The Confluence API requires `version.number` to be the **next** version (current + 1) in a PUT body. If we send the current version, Confluence will either 400 or 409. If we send current - 1, it will silently succeed but the page is rolled back.

**Mitigation:** Double-check with a real Confluence tenant in the contract test. Wiremock tests validate the body contains `version.number = expected_version + 1`.

**Warning sign:** A 400 "bad request" from Confluence with no other explanation.

### Risk 3: struct rename breaks downstream crate references

**What goes wrong:** `ConfluenceReadOnlyBackend` is used by `reposix-cli` and potentially `reposix-swarm` when Phase 17 ships. A rename without auditing all usages will break compilation.

**Mitigation:** `cargo check --workspace` after rename in Wave B. A type alias in the old name is an option but not needed for a pre-1.0 release.
[VERIFIED: grep for `ConfluenceReadOnlyBackend` in workspace before rename]

### Risk 4: audit connection lifetime in async context

**What goes wrong:** `rusqlite::Connection` is `!Send`. The `ConfluenceBackend` is `Send + Sync` (required by `IssueBackend`). An `Arc<Mutex<Connection>>` with `parking_lot::Mutex` is `Sync` but inserting from an async context via `block_on` or a sync scope must be done carefully.

**Mitigation:** Audit inserts are sync-scoped (same pattern as `reposix-sim/src/middleware/audit.rs` step 9: "Insert, sync-scoped, no .await held across the lock"). The insert is done in a short sync block, the lock released before any `.await`. Since `ConfluenceBackend::update_issue` is already `async`, the audit insert happens in a sync block mid-function, not across an await point.

### Risk 5: create_issue parent page ID vs folder/whiteboard IDs

**What goes wrong:** Confluence parentId can point to non-page entities (folders, whiteboards). If an agent passes a whiteboard ID as `parent_id`, the POST may return an error or create the page under the root.

**Mitigation:** Document that `parent_id` must be a page ID. The Confluence API returns a clear error if the parent type is wrong. No extra validation needed in the adapter — surface the Confluence error as `Error::Other(...)`.

### Risk 6: ADF body-format request incompatible with older page types

**What goes wrong:** Very old Confluence pages created before ADF was introduced may not have an ADF representation. Requesting `?body-format=atlas_doc_format` for such pages returns an empty or missing `body`.

**Mitigation:** In Wave D's read-path switch, add a fallback: if ADF body is empty/null, retry with `?body-format=storage` and store the raw XHTML. Wiremock can test the fallback path.

---

## Code Examples

### ADF root doc structure (from Atlassian docs)
```rust
// Source: developer.atlassian.com/cloud/jira/platform/apis/document/structure
// [VERIFIED]
let adf_doc = serde_json::json!({
    "version": 1,
    "type": "doc",
    "content": [
        {
            "type": "heading",
            "attrs": { "level": 2 },
            "content": [{ "type": "text", "text": "My heading" }]
        },
        {
            "type": "paragraph",
            "content": [
                { "type": "text", "text": "Normal text and " },
                {
                    "type": "text",
                    "text": "inline code",
                    "marks": [{ "type": "code" }]
                }
            ]
        },
        {
            "type": "codeBlock",
            "attrs": { "language": "rust" },
            "content": [{ "type": "text", "text": "fn main() {}" }]
        },
        {
            "type": "bulletList",
            "content": [
                {
                    "type": "listItem",
                    "content": [{
                        "type": "paragraph",
                        "content": [{ "type": "text", "text": "item one" }]
                    }]
                }
            ]
        }
    ]
});
```

### PUT body for update_issue
```rust
// [ASSUMED: version.number = current+1 is the Confluence convention]
let put_body = serde_json::json!({
    "id": id.0.to_string(),
    "status": "current",
    "title": issue.title,
    "version": { "number": current_version + 1 },
    "body": {
        "representation": "storage",
        "value": issue.body   // or convert via markdown_to_storage(&issue.body)
    }
});
```

### POST body for create_issue
```rust
// [VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-page]
let post_body = serde_json::json!({
    "spaceId": space_id,          // numeric string from resolve_space_id
    "status": "current",
    "title": issue.title,
    "parentId": issue.parent_id.map(|id| id.0.to_string()),
    "body": {
        "representation": "storage",
        "value": issue.body
    }
});
```

### Audit insert (sync-scoped, no await held)
```rust
// Pattern: crates/reposix-sim/src/middleware/audit.rs step 9
// [VERIFIED]
if let Some(ref audit) = self.audit {
    let conn = audit.lock();
    let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let _ = conn.execute(
        "INSERT INTO audit_events \
         (ts, agent_id, method, path, status, request_body, response_summary) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            ts,
            format!("reposix-confluence-{}", std::process::id()),
            "PUT",
            format!("/wiki/api/v2/pages/{}", id.0),
            i64::from(status.as_u16()),
            truncated_title,    // first 256 chars of title — not body
            format!("{}:{}", status.as_u16(), &sha_hex[..16]),
        ],
    ).inspect_err(|e| tracing::error!(error = %e, "audit insert failed"));
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ConfluenceReadOnlyBackend` returns `NotSupported` for all writes | `ConfluenceBackend` implements all three write methods | Phase 16 | Agents can create/update/delete pages via FUSE |
| Body stored as raw XHTML from `?body-format=storage` | Body stored as Markdown via ADF→Markdown converter | Phase 16 Wave D | `cat mount/pages/XX.md` returns human-readable Markdown |
| `supports()` returns `Hierarchy` only | `supports()` returns `Hierarchy + Delete + StrongVersioning` | Phase 16 | FUSE and callers know write capabilities |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `storage` representation is accepted on PUT/POST writes to Confluence REST v2 | Technical Approach §1 §2 | Must use ADF on wire instead; more complex body serialization |
| A2 | `version.number` in PUT must be current + 1 (next version), not current | Design Decisions Q3 | PUT returns 400 or 409; must send current version number instead |
| A3 | ADF body-format request works for all Confluence page types | Technical Approach §4 | Some pages return empty ADF; fallback to storage required |
| A4 | `storage` → Markdown conversion via pulldown-cmark HTML renderer gives acceptable fidelity for basic constructs | Technical Approach §4 | HTML output doesn't match what Confluence accepts for storage; need Atlassian-specific XHTML macros |
| A5 | ADF→Markdown conversion handles all page bodies an agent would encounter | Technical Approach §4 | Unknown ADF node types silently drop content; fallback marker not sufficient |

---

## Open Questions

1. **version.number: current or current+1?**
   - What we know: ADR community forum says "use current+1"; older Confluence documentation says "pass the current version"
   - What's unclear: REST v2 behavior differs from v1; needs empirical testing
   - Recommendation: Contract test against real tenant before Wave B merge; wiremock test should verify whichever is correct

2. **Storage XHTML round-trip fidelity for code blocks**
   - What we know: Confluence stores code blocks as `<ac:structured-macro ac:name="code">` in storage format
   - What's unclear: Does writing raw `<pre><code>` as storage value get accepted and then re-emitted as the same?
   - Recommendation: Contract test with a simple fenced code block page; if Confluence normalizes it to macros on read, the ADF path is necessary for code block fidelity

3. **Audit log architecture: embedded vs separate tool**
   - What we know: The sim uses server-side middleware; the confluence backend has no server
   - What's unclear: Should the audit log be optional (None = skip) or required? If required, how is the path configured without changing the CLI assembly?
   - Recommendation: Make it optional (`Option<Arc<Mutex<Connection>>>`); CLI passes a connection when `--audit-db` is set

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `pulldown-cmark` | ADF converter (Wave A) | ✗ (not in Cargo.toml) | — | Add as dep; `comrak` is alternative |
| `rusqlite` bundled | Audit log (Wave C) | ✓ (workspace dep) | 0.32 | — |
| `wiremock` | Write method tests | ✓ (dev-dep in confluence crate) | 0.6 | — |
| Confluence Cloud tenant | Contract tests | ✓ (reuben-john.atlassian.net, per Phase 11) | — | wiremock for unit tests |
| `parking_lot::Mutex` | Audit connection | ✓ (workspace dep) | 0.12 | — |

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (tokio::test for async) |
| Config file | none — workspace default |
| Quick run command | `cargo test -p reposix-confluence` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WRITE-01 | `create_issue` POSTs to `/wiki/api/v2/pages` with correct spaceId/title | unit (wiremock) | `cargo test -p reposix-confluence create_issue` | ❌ Wave B |
| WRITE-02 | `update_issue` PUTs to `/wiki/api/v2/pages/{id}` with version +1 | unit (wiremock) | `cargo test -p reposix-confluence update_issue` | ❌ Wave B |
| WRITE-03 | `delete_or_close` sends DELETE, returns Ok on 204 | unit (wiremock) | `cargo test -p reposix-confluence delete_or_close` | ❌ Wave B |
| WRITE-04 | ADF → Markdown round-trips headings/paras/code/lists | unit (pure) | `cargo test -p reposix-confluence adf` | ❌ Wave A |
| LD-16-02 | `sanitize()` strips server fields before write body | existing test | `cargo test -p reposix-core sanitize` | ✅ |
| LD-16-03 | Every write call inserts an audit row | unit (wiremock + in-memory sqlite) | `cargo test -p reposix-confluence audit` | ❌ Wave C |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-confluence`
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/reposix-confluence/src/adf.rs` — covers WRITE-04 (Wave A)
- [ ] Write method wiremock tests — covers WRITE-01/02/03 (Wave B)
- [ ] Audit log tests with in-memory SQLite — covers LD-16-03 (Wave C)
- [ ] Dependency: add `pulldown-cmark = "0.13"` to `crates/reposix-confluence/Cargo.toml`

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | Basic auth (email:api_token) — existing pattern, no change |
| V3 Session Management | no | Stateless REST calls |
| V4 Access Control | no | Controlled by Confluence space permissions |
| V5 Input Validation | yes | `sanitize()` strips server fields; page ID validated as numeric (existing WR-02 pattern) |
| V6 Cryptography | no | TLS via reqwest/rustls; no key management in this phase |

### Known Threat Patterns for Confluence write path

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Tainted body bytes echo'd in write (SG-03) | Tampering | `Untainted<Issue>` parameter type; `sanitize()` already called before reaching backend |
| Path traversal via page ID injection | Tampering | Page ID is `IssueId(u64)` — numeric, not user-supplied string |
| SSRF via Confluence cursor injection | Tampering | Existing relative-path prepend construction (SG-01 already in read path); write paths don't follow cursors |
| Audit log bypass | Repudiation | Best-effort insert; log-and-swallow ensures write is never blocked by audit failure; WAL append-only trigger prevents row manipulation |
| Credential leak in debug output | Information Disclosure | Existing `Debug` redaction on `ConfluenceCreds` and `ConfluenceReadOnlyBackend` — carries forward to renamed struct |
| Write amplification via agent loop | DoS | Rate-limit gate (`rate_limit_gate`) shared across read+write; `MAX_RATE_LIMIT_SLEEP = 60s` cap |

---

## Sources

### Primary (HIGH confidence)
- `crates/reposix-confluence/src/lib.rs` — existing patterns for headers, rate-limit, error handling, taint discipline
- `crates/reposix-core/src/backend.rs` — `IssueBackend` trait signatures (verified in session)
- `crates/reposix-core/src/taint.rs` — `Tainted`/`Untainted`/`sanitize` discipline (verified in session)
- `crates/reposix-sim/src/middleware/audit.rs` — audit insert pattern to replicate (verified in session)
- [developer.atlassian.com/cloud/confluence/rest/v2/api-group-page](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/) — POST/PUT/DELETE API contracts
- [developer.atlassian.com/cloud/jira/platform/apis/document/structure](https://developer.atlassian.com/cloud/jira/platform/apis/document/structure/) — ADF JSON structure
- `crates/reposix-core/src/backend/sim.rs` — `update_issue`/`create_issue`/`delete_or_close` patterns (verified in session)

### Secondary (MEDIUM confidence)
- crates.io `pulldown-cmark 0.13.3` — Markdown parser; appropriate for Markdown→storage direction
- community.developer.atlassian.com discussion threads — version.number semantics (current+1 convention)

### Tertiary (LOW confidence)
- A2: `version.number` must be current+1 — multiple community reports suggest this but needs empirical verification against REST v2

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — existing crate patterns well-established
- Architecture: HIGH — trait signatures, taint discipline, audit pattern all verified in codebase
- ADF wire format: HIGH — official Atlassian docs verified
- version.number convention: MEDIUM — community-sourced; needs contract test to confirm
- ADF round-trip fidelity: LOW — not tested against real tenant; fallback documented

**Research date:** 2026-04-14
**Valid until:** 2026-06-01 (Confluence REST v2 is stable; ADF schema is versioned at `version: 1`)

---

## RESEARCH COMPLETE
