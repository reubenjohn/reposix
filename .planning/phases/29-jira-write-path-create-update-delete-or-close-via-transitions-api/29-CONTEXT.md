# Phase 29: JIRA write path — `create_issue`, `update_issue`, `delete_or_close` via Transitions API — Context

**Gathered:** 2026-04-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete the JIRA write path on `JiraBackend`:
- `create_issue` → `POST /rest/api/3/issue` with ADF body
- `update_issue` → `PUT /rest/api/3/issue/{id}` (fields only — status changes via transitions only)
- `delete_or_close` → two-step transitions discovery + POST; fallback `DELETE` if no transitions

Enable `BackendFeature::Delete` and `BackendFeature::Transitions`. Remove "not supported" stubs.
Audit log rows for all mutations. Wiremock write-path tests ≥3. Contract test write extension.

Out of scope for Phase 29:
- Full Markdown-to-ADF conversion (Phase 30+ if desired)
- Attachments, comments, sub-task creation
- Status transitions that go to non-"done" categories

</domain>

<decisions>
## Implementation Decisions

### ADF Body Encoding for Writes
- **D-01:** Minimal single-paragraph ADF wrapper for `description`:
  ```json
  {"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"<plain body>"}]}]}
  ```
  No Markdown-to-ADF conversion needed — plain text body wrapped in a single paragraph. This matches what JIRA renders correctly for issue bodies written via the API and round-trips with the existing `adf_to_plain_text` reader.

### ADF Read Path Upgrade (opportunistic)
- **D-02:** Upgrade `adf_to_plain_text` in `reposix-jira/src/adf.rs` to full Markdown output using the same recursive visitor pattern as `reposix-confluence/src/adf.rs::adf_to_markdown`. Do NOT add a crate dependency from `reposix-jira` on `reposix-confluence` — copy-adapt the logic into a new `adf_to_markdown` function in `reposix-jira/src/adf.rs`. Rename the existing `adf_to_plain_text` → internal use only, expose `adf_to_markdown` as the public API. This enables round-trip verification in write tests.

### `create_issue` API Flow
- **D-03:** `POST /rest/api/3/issue` with body:
  ```json
  {
    "fields": {
      "project": {"key": "<project>"},
      "summary": "<issue.title>",
      "issuetype": {"name": "<discovered_type>"},
      "description": <adf_paragraph_wrapper(issue.body)>
    }
  }
  ```
- **D-04:** Issue type discovery: `GET /rest/api/3/issuetype?projectKeys=<KEY>` on first `create_issue` call. Cache in `OnceLock<Vec<String>>` on `JiraBackend` — "cache per session" per ROADMAP spec. Prefer type named "Task"; fallback to first available type.
- **D-05:** Response: JIRA returns `{"id": "...", "key": "...", "self": "..."}`. Follow with a `get_issue(id)` call to hydrate the full `Issue` struct for the return value (same pattern as Confluence `create_issue`).

### `update_issue` API Flow
- **D-06:** `PUT /rest/api/3/issue/{id}` with body:
  ```json
  {
    "fields": {
      "summary": "<issue.title>",
      "description": <adf_paragraph_wrapper(issue.body)>,
      "labels": <issue.labels or []>,
      "assignee": null
    }
  }
  ```
- **D-07:** `expected_version` is silently ignored — JIRA has no ETag. `BackendFeature::StrongVersioning` remains false.
- **D-08:** Status changes are NOT allowed via PUT. If `issue.status` differs from current, log a warning and proceed with the PUT (update fields only). Status changes require transitions — out of scope for Phase 29's `update_issue`.
- **D-09:** Successful PUT returns 204 No Content. Follow with `get_issue(id)` to return the updated `Issue`.

### `delete_or_close` Transition Discovery Flow
- **D-10:** Two-step flow:
  1. `GET /rest/api/3/issue/{id}/transitions` → parse `transitions[]` array
  2. Filter to transitions where `to.statusCategory.key == "done"`
  3. For `DeleteReason::WontFix`: prefer transition whose `name` (lowercased) contains "won't", "reject", "not planned", "invalid", or "duplicate"
  4. For all other reasons (`Completed`, `NotPlanned`, `Abandoned`, `Duplicate`): pick first "done" transition
  5. `POST /rest/api/3/issue/{id}/transitions` with `{"transition": {"id": "<id>"}}` — if `fields.resolution` is required by the screen, include `{"fields": {"resolution": {"name": "Done"}}}` on first attempt; catch 400 and retry without it
- **D-11:** Fallback to `DELETE /rest/api/3/issue/{id}` ONLY if the GET transitions returns empty `transitions[]` AND the first attempt at DELETE returns 204. Log `WARN` before fallback.
- **D-12:** Return an empty-body success marker after close (no hydrate call needed — the issue is gone or closed).

### BackendFeature Changes
- **D-13:** `supports()` change:
  - `BackendFeature::Delete` → `true` (DELETE fallback path exists)
  - `BackendFeature::Transitions` → `true` (two-step close via transitions)
  - `BackendFeature::StrongVersioning` → `false` (no ETag on JIRA — unchanged)
  - `BackendFeature::Hierarchy` → `true` (unchanged from Phase 28)

### Audit Log
- **D-14:** Audit log rows for all three write ops using the existing `audit_write` method pattern from `ConfluenceBackend`. Store method + path + status + summary (issue title, max 256 chars) — never body content (same privacy constraint as D-13 in Phase 28).

### Wiremock Test Matrix (≥3 required)
- **D-15:** Write-path wiremock tests:
  1. `create_issue_posts_to_rest_api` — POST /issue → 201 with `{"id":"10001","key":"P-1","self":"..."}` → GET /issue/10001 → full fixture → returns hydrated Issue
  2. `update_issue_puts_fields` — PUT /issue/{id} → 204 → GET /issue/{id} → updated fixture → returns updated Issue
  3. `delete_or_close_via_transitions` — GET /issue/{id}/transitions → list with "Done" transition → POST /issue/{id}/transitions → 204 → success
  4. `delete_or_close_wontfix_picks_reject` — transitions list includes "Done" + "Won't Fix" → WontFix reason → picks "Won't Fix" transition
  5. `delete_or_close_fallback_delete` — GET transitions → empty array → DELETE /issue/{id} → 204 → success
  6. `create_issue_discovers_issuetype` — GET /issuetype?projectKeys=P → ["Story","Task","Bug"] → prefers "Task"

### Contract Test Extension
- **D-16:** Extend `tests/contract.rs` with write assertions:
  - `create → get → assert title matches`
  - `update → get → assert title updated`
  - `delete → get → assert not found`
  - Gated on `contract_jira_wiremock` (always runs) and `contract_jira_live` (`#[ignore]`)

### No New Env Vars / CLI Changes
- **D-17:** No new env vars — `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE` unchanged from Phase 28.
- **D-18:** No new CLI commands — write ops are triggered via FUSE mount (write file → `update_issue`, create file → `create_issue`, unlink → `delete_or_close`), not via direct CLI flags.

### Test for Removed "not supported" Stubs
- **D-19:** The Phase 28 test `write_ops_return_not_supported` (test 12 in lib.rs) MUST be deleted and replaced by the new write-path tests. Do not leave the old test asserting the opposite of the new behavior.

### Claude's Discretion
- Internal struct for issuetype cache entry (`IssueTypeInfo` vs plain `String`)
- Error message wording for transition-discovery failures (must be clear, not echo credential values)
- Whether to add `labels` field to `create_issue` body (if `issue.labels` is non-empty) — reasonable to include
- Exact resolution name string in `{"resolution": {"name": "Done"}}` retry — "Done" is the JIRA standard

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary Pattern References
- `crates/reposix-confluence/src/lib.rs` §`create_issue`, `update_issue`, `delete_or_close` — full write path pattern; audit_write, write_headers, version handling. Primary reference.
- `crates/reposix-jira/src/lib.rs` — Phase 28 read-only JIRA backend; stub write ops (lines ~491–516) to be replaced; `supports()` (line ~478) to be updated; test 12 `write_ops_return_not_supported` to be deleted
- `crates/reposix-jira/src/adf.rs` — existing `adf_to_plain_text`; upgrade to `adf_to_markdown` in this phase
- `crates/reposix-confluence/src/adf.rs` §`adf_to_markdown` — ADF→Markdown conversion logic to copy-adapt

### Core Types
- `crates/reposix-core/src/backend.rs` — `BackendFeature` enum, `DeleteReason` enum, `BackendConnector` trait
- `crates/reposix-core/src/taint.rs` — `Untainted<T>`, `sanitize()` — write path takes `Untainted<Issue>`

### Test Patterns
- `crates/reposix-confluence/tests/contract.rs` or `crates/reposix-jira/tests/` — contract test structure for write extension
- `crates/reposix-confluence/src/lib.rs` §`make_untainted` helper (~line 2546) — pattern for building `Untainted<Issue>` in tests

### ROADMAP Spec (authoritative)
- `.planning/ROADMAP.md` §"Phase 29: JIRA write path" (lines ~736–744) — spec for all three write ops
- `.planning/ROADMAP.md` §"Phase 28" §"Security contract" — non-negotiable security constraints inherited

### Prior Phase Context
- `.planning/phases/28-jira-cloud-read-only-adapter-reposix-jira-v0-8-0/28-CONTEXT.md` — D-09 through D-14 (security contract, audit log, rate limit) all inherited

### JIRA REST API v3 Endpoints Used
- `POST /rest/api/3/issue` — create
- `PUT /rest/api/3/issue/{id}` — update fields
- `GET /rest/api/3/issue/{id}/transitions` — discover transitions
- `POST /rest/api/3/issue/{id}/transitions` — close via transition
- `DELETE /rest/api/3/issue/{id}` — fallback delete
- `GET /rest/api/3/issuetype?projectKeys=<KEY>` — issue type discovery

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `JiraBackend::audit_write` (established in Phase 28 for reads) — extend to cover write mutations using same row structure
- `JiraBackend::standard_headers()` — GET headers; need a `write_headers()` equivalent (Content-Type: application/json) following Confluence's `write_headers` pattern
- `ConfluenceBackend::create_issue` (~line 1582) — full template: build JSON body, POST, parse 201/201 Created response, hydrate via get_issue
- `ConfluenceBackend::fetch_current_version` — pattern for post-write hydration (JIRA uses `get_issue` directly after write)
- `adf_to_plain_text` in `reposix-jira/src/adf.rs` — reader to upgrade

### Established Patterns
- `OnceLock<T>` for per-instance initialization cache (use for issuetype discovery)
- `serde_json::json!()` macro for building request bodies inline
- `audit_write(method, path, status, summary, bytes)` signature — match exactly
- `#[forbid(unsafe_code)]` + `#[warn(clippy::pedantic)]` remain in effect
- Test helper `make_untainted(title, body, parent_id)` pattern for write tests

### Integration Points
- `JiraBackend::supports()` — change `Delete` and `Transitions` arms from false to true
- `reposix-jira/src/lib.rs` test module — delete test 12, add new tests D-15
- `tests/contract.rs` in `reposix-jira` — add write extension block (D-16)

</code_context>

<specifics>
## Specific Ideas

- The ADF read path upgrade (D-02) is opportunistic but strongly recommended: write tests will want to verify round-trips (write body → read back → same content), which only works cleanly if both paths use the same representation. Adding `adf_to_markdown` to the JIRA ADF module is a ~50-line addition reusing the exact same recursive visitor logic from the Confluence module.
- The issuetype cache should be `OnceLock<Vec<String>>` (just the type names, "Task" preferred) — no need for a full struct. Keep it simple.
- For `update_issue`, the `labels` field in the PUT body should be included when `issue.labels` is non-empty to maintain label round-trips.
- `delete_or_close` fallback to DELETE should log at `WARN` level: `warn!("JIRA: no done transitions found for issue {id}, falling back to DELETE")` — this is an admin-permission path that should be visible in logs.

</specifics>

<deferred>
## Deferred Ideas

- Full Markdown-to-ADF conversion for writes (structured headings, lists, code blocks in issue descriptions) — post-Phase 29 if desired
- `BackendFeature::Workflows` — named workflow transitions beyond done/not-done — deferred
- `BackendFeature::BulkEdit` — JIRA bulk edit API — deferred
- Sub-task creation (parent_id on create_issue maps to JIRA parent link) — deferred
- Label management via JIRA label API — Phase 29 writes labels in the PUT body but does not create new labels

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 29-jira-write-path-create-update-delete-or-close-via-transitions-api*
*Context gathered: 2026-04-16*
