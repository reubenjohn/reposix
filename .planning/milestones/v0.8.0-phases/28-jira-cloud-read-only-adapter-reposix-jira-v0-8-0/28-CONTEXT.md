# Phase 28: JIRA Cloud read-only adapter (`reposix-jira`) — Context

**Gathered:** 2026-04-16
**Status:** Ready for planning

<domain>
## Phase Boundary

Ship a `reposix-jira` crate implementing `BackendConnector` (read-only) against JIRA Cloud REST v3. Covers list/get with cursor pagination, status mapping, ADF stripping, `Issue.extensions` population, wiremock test suite, contract test, CLI dispatch (`list --backend jira`, `mount --backend jira`), ADR-005, and `docs/reference/jira.md`. Write ops are stubbed to return a "not supported" error (Phase 29). No real credentials required in CI — wiremock covers the full path.

</domain>

<decisions>
## Implementation Decisions

### API Endpoints
- **D-01:** `list_issues` → `POST /rest/api/3/search/jql` with JQL `project = "{KEY}" ORDER BY id ASC`. The old `GET /search` was retired Aug 2025; cursor-based pagination via `nextPageToken` + `isLast: true` as terminator; `total` field absent; max 100 issues/page.
- **D-02:** `get_issue` → `GET /rest/api/3/issue/{id}` with numeric id accepted directly.
- **D-03:** `fields` request list pinned to: `id,key,summary,description,status,resolution,assignee,labels,created,updated,parent,issuetype,priority`.

### Status Mapping
- **D-04:** Two-field mapping: primary on `fields.status.statusCategory.key`:
  - `"new"` → `Open`
  - `"indeterminate"` → `InProgress`, UNLESS `status.name` contains "review" → `InReview`
  - `"done"` → `Done`
  - `WontFix` override when `fields.resolution.name` contains "won't"/"wont"/"not a bug"/"duplicate"/"cannot reproduce"
  - Unknown category → `Open` (safe fallback)

### ADF Description Stripping
- **D-05:** `fields.description` is nested ADF JSON (may be null). Walk `content[]` tree; handle `paragraph`, `text`, `hardBreak`, `codeBlock` nodes. Unknown nodes: emit their text children and continue (don't fail). Plain text output — no Markdown conversion (that's a Phase 29+ concern).

### Issue.extensions Keys
- **D-06:** Four keys always populated when present: `jira_key` (string "PROJ-42"), `issue_type` (string "Story"), `priority` (string "Medium" or omit when null), `status_name` (raw status name before mapping), `hierarchy_level` (i64 from `issuetype.hierarchyLevel`).
- **D-07:** `Issue.version` synthesized from `fields.updated` as Unix-millis `u64`. JIRA has no ETag. `StrongVersioning: false`.

### Hierarchy
- **D-08:** `fields.parent.id` (numeric) → `Issue.parent_id`. Subtask (-1 hierarchyLevel) + epic parent (1).

### Security Contract (non-negotiable)
- **D-09:** HTTP client ONLY via `reposix_core::http::client()`. Direct `reqwest::Client::new()` is denied by workspace clippy lint.
- **D-10:** `https://{tenant}.atlassian.net` registered with allowlist on backend construction (same pattern as ConfluenceBackend).
- **D-11:** Tenant validation: non-empty, alphanumeric+hyphen only, no leading/trailing hyphen, length ≤ 63 — blocks SSRF + DNS-label escape.
- **D-12:** All JIRA response bodies wrapped in `Tainted<T>` at HTTP seam; `translate()` converts to `Issue` but does NOT produce `Untainted<Issue>` (that requires `sanitize()` and belongs in Phase 29 write path).
- **D-13:** Audit log rows for BOTH reads (list, get). Response body hashed (SHA-256 prefix), never raw-logged. `Authorization` header NEVER logged. `Debug` impls for `JiraCreds` and `JiraBackend` redact the token.

### Rate Limiting
- **D-14:** Honor `Retry-After` (seconds) on 429. Exponential backoff with jitter if header absent (max 4 attempts, base 1s).

### CLI and Env Vars
- **D-15:** Env vars: `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE`. `read_jira_env()` helper in `reposix-cli` validates all three upfront and lists ALL missing vars in one error message (never echoes values) — same pattern as `read_confluence_env_from`.
- **D-16:** CLI: `list --backend jira`, `mount --backend jira --project <KEY>`. `--no-truncate` support (parity with Confluence `list_issues_strict`). Default 500-issue safety cap.

### Write Stubs
- **D-17:** `create_issue`, `update_issue`, `delete_or_close` return `Err("not supported: read-only backend — see Phase 29")`. `BackendFeature::Hierarchy` → true; `Delete`/`Transitions`/`StrongVersioning` → false.

### Test Matrix (all 12 required)
- **D-18:** Wiremock tests:
  1. `list_single_page` — 10 issues, `isLast: true`, no cursor
  2. `list_pagination_cursor` — 2-page traversal; `nextPageToken` correct; termination on `isLast`
  3. `get_by_numeric_id` — 200 response, canonical JSON
  4. `get_404_maps_to_not_found` — JIRA 404 body `{"errorMessages":["Issue Does Not Exist"]}` → `Error::Other("not found: ...")`
  5. `status_mapping_matrix` — parameterized: new→Open; indeterminate+"In Progress"→InProgress; indeterminate+"In Review"→InReview; done+no-resolution→Done; done+"Won't Fix"→WontFix; done+"Duplicate"→WontFix; unknown→Open
  6. `adf_description_strips_to_markdown` — nested paragraphs + code block + null description
  7. `parent_hierarchy` — subtask + epic parent + no parent
  8. `rate_limit_429_honors_retry_after` — 429 with `Retry-After: 2` delays; 429 without header triggers exponential backoff
  9. `tenant_validation_rejects_ssrf` — `new("a.evil.com")`, `new("-bad")`, `new("")`, `new("a".repeat(64))` all `Err`
  10. `supports_reports_hierarchy_only` — BackendFeature::Hierarchy true; Delete/Transitions/StrongVersioning false
  11. `extensions_omitted_when_empty` — frontmatter roundtrip proves empty extensions doesn't serialize
  12. `write_ops_return_not_supported` — all three write ops return the documented error string

### Contract Test
- **D-19:** `tests/contract.rs` parameterized identically to GitHub/Confluence: `contract_sim` (always) + `contract_jira_wiremock` (always) + `contract_jira_live` (`#[ignore]`, opt-in).

### Documentation
- **D-20:** `docs/reference/jira.md` — user guide + env vars + `--no-truncate` semantics.
- **D-21:** `docs/decisions/005-jira-issue-mapping.md` — ADR-005: ID vs key, status+resolution mapping table, version synthesis, ADF stripping, attachments/comments deferred.
- **D-22:** CHANGELOG entry under `[Unreleased]` promoted to `[v0.8.0]` in ship wave.

### Claude's Discretion
- Internal module layout within `reposix-jira` (e.g., split ADF into a sub-module, or keep inline as in GitHub backend)
- Error message wording for validation failures (must be clear, must not echo credential values)
- Exact ADF text-extraction whitespace handling for multi-paragraph bodies

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Backend Implementations (primary patterns)
- `crates/reposix-confluence/src/lib.rs` — Full read/write Atlassian backend; ADF module, creds struct with redacted Debug, tenant validation, audit log writes, rate limit gate. PRIMARY reference for JIRA.
- `crates/reposix-confluence/src/adf.rs` — ADF-to-markdown conversion (may be re-used or adapted for ADF stripping in JIRA)
- `crates/reposix-github/src/lib.rs` — Read-only backend; rate limit gate, pagination via Link header, contract test pattern
- `crates/reposix-github/tests/contract.rs` — Contract test template to replicate for JIRA

### Core Types
- `crates/reposix-core/src/backend.rs` (or equivalent) — `BackendConnector` trait, `BackendFeature` enum, `DeleteReason`
- `crates/reposix-core/src/lib.rs` — `Issue`, `IssueId`, `IssueStatus`, `Tainted`, `Untainted`, `Error`, `Result`
- `crates/reposix-core/src/http.rs` (or equivalent) — `client()`, `ClientOpts`, `HttpClient`

### CLI Integration
- `crates/reposix-cli/src/list.rs` — `read_confluence_env_from` pattern; `ListBackend` enum; how to add `Jira` variant
- `crates/reposix-cli/src/refresh.rs` — Backend dispatch for mount

### ROADMAP Spec (authoritative for this phase)
- `.planning/ROADMAP.md` §"Phase 28: JIRA Cloud read-only adapter" — full spec (lines ~706–731)

### REQUIREMENTS
- `.planning/PROJECT.md` §"Active — JIRA integration" — JIRA-01 through JIRA-05 requirements

### Prior ADR
- `docs/decisions/004-backend-connector-rename.md` — ADR-004 (BackendConnector naming rationale)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ConfluenceBackend::new` / `new_with_base_url` pattern: direct template for `JiraBackend::new` and `JiraBackend::new_with_base_url`
- `read_confluence_env_from` in `reposix-cli/src/list.rs`: copy-adapt to `read_jira_env_from` (different env var names, same collect-all-missing-vars pattern)
- `crates/reposix-confluence/src/adf.rs`: ADF walker already exists — JIRA uses the same ADF format; may reuse `adf_to_markdown` logic or extract a shared `adf_to_plain_text` helper
- `GithubReadOnlyBackend` rate-limit gate: exponential backoff pattern for 429 handling (JIRA needs similar but also honors `Retry-After`)
- Wiremock test setup in `reposix-github/tests/contract.rs` + `reposix-confluence/tests/contract.rs`: copy the fixture boilerplate

### Established Patterns
- `#[forbid(unsafe_code)]` + `#[warn(clippy::pedantic)]` in every crate
- `async_trait` + `#[async_trait]` on `BackendConnector` impls
- `parking_lot::Mutex` for rate-limit gate state
- Struct-level `Debug` redaction via manual `impl fmt::Debug` for creds structs
- `Arc<HttpClient>` shared across methods
- Test helpers: `wiremock::MockServer::start().await` + `MockServer::uri()` as `base_url`

### Integration Points
- `reposix-cli/src/list.rs` — add `Jira` arm to `ListBackend` enum; add `list --backend jira` dispatch
- `reposix-cli/src/refresh.rs` — add `Jira` arm to mount backend dispatch
- `Cargo.toml` workspace — add `reposix-jira` member
- CI workflow (`.github/workflows/ci.yml`) — `reposix-jira` picked up automatically by `--workspace` flags

</code_context>

<specifics>
## Specific Ideas

- The JIRA search endpoint changed (Aug 2025): use `POST /rest/api/3/search/jql`, not the old `GET /search`. This is a known migration gotcha.
- `nextPageToken` + `isLast: true` is the pagination terminator — `total` is absent from the new endpoint. Don't attempt offset math.
- Numeric issue IDs (not key strings like "PROJ-42") are the canonical `IssueId` in reposix. The JIRA key goes into `extensions["jira_key"]`.
- `Issue.version` synthesized from `fields.updated` as Unix-millis u64 — JIRA has no server-side ETag.
- ADF null description must not panic — treat as empty body.

</specifics>

<deferred>
## Deferred Ideas

- ADF-to-Markdown (rich format) for JIRA descriptions — Phase 29+ concern; Phase 28 only needs plain text extraction.
- Attachments and comments in JIRA — deferred per ADR-005.
- Write path (`create_issue`, `update_issue`, `delete_or_close` via Transitions API) — Phase 29.
- `BackendFeature::Delete` and `BackendFeature::Transitions` enabled — Phase 29.
- Shared ADF library crate (extract from confluence + jira) — post-Phase 29 refactor if desired.

</deferred>

---

*Phase: 28-jira-cloud-read-only-adapter-reposix-jira-v0-8-0*
*Context gathered: 2026-04-16*
