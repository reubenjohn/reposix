← [back to index](./index.md)

# Milestone v0.8.0 — JIRA Cloud Integration

**Goal:** Add JIRA Cloud as a first-class backend. Two prerequisite foundation changes land first: rename the misnamed `IssueBackend` trait to `BackendConnector` (Confluence pages are not "issues"; the trait name should be neutral), and add an `extensions` field to `Issue` for backend-specific metadata that doesn't fit the canonical 5-field schema (JIRA needs `jira_key`, `issue_type`, `priority`, plus structured data like `hierarchyLevel: 0`). Then ship the `reposix-jira` crate with read-only Cloud REST v3 support, cursor-based JQL pagination, status-category mapping, and subtask/epic hierarchy. Write path lands as a stretch phase.

**Breaking release.** This milestone bumps the workspace from `0.7.x` → `0.8.0` (SemVer breaking: trait rename + new `Issue.extensions` field in the public `reposix-core` API).

**Target features:**
- Rename `IssueBackend` → `BackendConnector` across all crates + ADR-004 documenting naming alternatives considered
- Add `extensions: BTreeMap<String, serde_yaml::Value>` to `Issue` for typed backend metadata (preserves JSON-ish structure: strings, ints, booleans, nested maps) in frontmatter
- `reposix-jira` crate: `BackendConnector` impl against JIRA Cloud REST v3 (`https://{instance}.atlassian.net/rest/api/3`)
- JQL listing via `POST /rest/api/3/search/jql` with **cursor pagination** (`nextPageToken` + `isLast`; the old `GET /search` + `startAt` was retired Aug 2025)
- Status-category-key mapping → 5-valued `IssueStatus` (with `InReview` heuristic on status `name` and `WontFix` heuristic on `resolution.name`)
- Subtask + epic hierarchy: `fields.parent.id` → `Issue.parent_id`
- JIRA extensions in frontmatter: `jira_key` ("PROJ-42"), `issue_type` ("Story"), `priority` ("Medium" or null), `status_name` (raw pre-mapping), `hierarchy_level` (i64)
- Env vars: `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE` — fail-closed if any missing
- HTTP egress routed through `reposix_core::http::client()` (enforces `REPOSIX_ALLOWED_ORIGINS`); all JIRA responses wrapped in `Tainted<T>` at the seam
- Audit log rows for **both reads and writes** (parity with ConfluenceBackend audit policy)
- Wiremock unit tests (enumerated per phase) + contract test (always-on wiremock + `#[ignore]`-gated live)
- CLI: `list --backend jira`, `mount --backend jira --project <PROJECT_KEY>`
- `docs/reference/jira.md` + **two ADRs**: `docs/decisions/004-backend-connector-rename.md` (rename rationale + alternatives: `RemoteBackend`, `TrackerBackend`, `WorkItemBackend`) and `docs/decisions/005-jira-issue-mapping.md` (ID vs key, status mapping table, ADF stripping, version synthesis, attachments/comments deferred)
- (Stretch) Write path: `create_issue`, `update_issue`, `delete_or_close` via Transitions API

**Deferred to follow-up phases (explicitly out of v0.8.0 scope):**
- **JIRA attachments** as `issues/<id>.attachments/<name>` (parity with planned Confluence attachments, Phase 24)
- **JIRA comments** as `issues/<id>.comments/<comment-id>.md` (parity with Phase 23's Confluence comments)
- **Custom fields** (`customfield_10014`, story points, sprint, etc.) — route through `extensions` when a concrete need appears
- **JIRA Data Center** — different auth + v2 API; separate adapter if ever needed

**Requirements:** RENAME-01, EXT-01, JIRA-01 … JIRA-06

### Phase 27: Foundation — `IssueBackend` → `BackendConnector` rename + `Issue.extensions` field (v0.8.0)

**Goal:** Rename the `IssueBackend` trait to `BackendConnector` in `reposix-core` and update all impls (`SimBackend`, `GithubReadOnlyBackend`, `ConfluenceBackend`) plus every call-site (`reposix-fuse` ≈20 sites in `fs.rs`, `reposix-cli` dispatch in `list.rs`/`mount.rs`/`refresh.rs`, `reposix-remote`, `reposix-swarm`). Simultaneously add `extensions: BTreeMap<String, serde_yaml::Value>` to the `Issue` struct (defaults to empty via `#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]`; flows through `frontmatter::render`/`frontmatter::parse` without schema churn). Bump workspace Cargo.toml `[workspace.package] version = "0.8.0"`. Write ADR-004 (`docs/decisions/004-backend-connector-rename.md`) capturing the rename rationale with alternatives considered (`RemoteBackend`, `TrackerBackend`, `WorkItemBackend`, `Connector`) and why `BackendConnector` won (neutral across issue/page/content domains; aligns with Phase 12's "Connector protocol" vocabulary). Grep-verify zero remaining `IssueBackend` references in source or docs after the rename. Roundtrip test: serialize `Issue { extensions: {"foo": Value::Int(42), "bar": Value::String("x")} }` → parse → assert equality.
**Requirements:** RENAME-01, EXT-01
**Depends on:** none (Phase 26 docs work is orthogonal; if Phase 26 lands first, this phase refreshes any doc lines mentioning `IssueBackend` during the rename sweep)
**Plans:** 3/3 plans executed

Plans:
- [x] 27-01-PLAN.md — Rename IssueBackend → BackendConnector in reposix-core
- [x] 27-02-PLAN.md — Update all external impls and call-sites; workspace tests green
- [x] 27-03-PLAN.md — Issue.extensions field + ADR-004 + v0.8.0 + CHANGELOG

### Phase 28: JIRA Cloud read-only adapter (`reposix-jira`) (v0.8.0)

**Goal:** Ship a `reposix-jira` crate implementing `BackendConnector` against JIRA Cloud REST v3 (`https://{instance}.atlassian.net/rest/api/3`). Basic auth (`email:JIRA_API_TOKEN`, same pattern as `reposix-confluence`). `list_issues(project)` → `POST /rest/api/3/search/jql` (the old `GET /search` was retired Aug 2025) with JQL `project = "{KEY}" ORDER BY id ASC`, cursor-based pagination (`nextPageToken` + `isLast: true` as terminator — `total` is absent in the new endpoint, max 100 issues/page, `fields` request list pinned to: `id,key,summary,description,status,resolution,assignee,labels,created,updated,parent,issuetype,priority`). `get_issue(project, id)` → `GET /rest/api/3/issue/{id}` (numeric id accepted directly). Two-field status mapping: primary on `fields.status.statusCategory.key` (`"new"` → `Open`, `"indeterminate"` → `InProgress` unless status `name` contains "review" → `InReview`, `"done"` → `Done`), with `WontFix` override when `fields.resolution.name` contains "won't"/"wont"/"not a bug"/"duplicate"/"cannot reproduce". `Issue.version` synthesized from `fields.updated` as Unix-millis `u64` (JIRA has no ETag/optimistic-lock; `StrongVersioning: false`). ADF description (`fields.description` is nested JSON, may be null) stripped to plain text by walking the `content[]` tree (paragraph/text/hardBreak/codeBlock nodes; unknown nodes emit their text children and continue). Subtask/epic hierarchy: `fields.parent.id` (numeric) → `Issue.parent_id`. `Issue.extensions` keys: `jira_key` (string "PROJ-42"), `issue_type` (string "Story"), `priority` (string "Medium" or absent when null), `status_name` (raw status name before mapping), `hierarchy_level` (i64 from `issuetype.hierarchyLevel`).

**Security contract (non-negotiable, per CLAUDE.md threat model):** HTTP client constructed ONLY via `reposix_core::http::client()` (enforces `REPOSIX_ALLOWED_ORIGINS`); direct `reqwest::Client::new()` is denied by the workspace clippy lint. `https://{tenant}.atlassian.net` registered with the allowlist on backend construction. All JIRA response bodies wrapped in `Tainted<T>` at the HTTP seam; `translate()` converts to plain `Issue` but does NOT cross the `Untainted<T>` boundary (Phase 29 writes take `Untainted<Issue>` which has gone through `sanitize()`). Tenant validation: non-empty, alphanumeric+hyphen only, no leading/trailing hyphen, length ≤ 63 (blocks SSRF + DNS-label escape). Audit log rows for BOTH reads (list, get) and writes (parity with `ConfluenceBackend`'s policy); response body hashed (SHA-256 prefix), never raw-logged; `Authorization` header NEVER logged; `Debug` impls for `JiraCreds` and `JiraBackend` redact the token.

**Operational details:** `root_collection_name()` → `"issues"`. `create_issue`/`update_issue`/`delete_or_close` return `Err("not supported: read-only backend — see Phase 29")` in the read-only phase. Rate limit: honor `Retry-After` (seconds) on 429; exponential backoff with jitter if header absent (max 4 attempts, base 1s). Env vars: `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE` — `read_jira_env()` helper in `reposix-cli` validates all three upfront and lists ALL missing vars in one error message (never echoes values). CLI: `list --backend jira`, `mount --backend jira --project <KEY>`. `--no-truncate` support (parity with Confluence's `list_issues_strict`) for projects larger than the default 500-issue safety cap.

**Test matrix (wiremock; all 12 are required):**
1. `list_single_page` — 10 issues, `isLast: true`, no cursor.
2. `list_pagination_cursor` — 2-page traversal; `nextPageToken` plumbed correctly; termination on `isLast`.
3. `get_by_numeric_id` — 200 response, canonical JSON.
4. `get_404_maps_to_not_found` — JIRA 404 body `{"errorMessages":["Issue Does Not Exist"]}` → `Error::Other("not found: ...")`.
5. `status_mapping_matrix` — parameterized: `new`→Open; `indeterminate`+"In Progress"→InProgress; `indeterminate`+"In Review"→InReview; `done`+no-resolution→Done; `done`+"Won't Fix"→WontFix; `done`+"Duplicate"→WontFix; unknown category→Open.
6. `adf_description_strips_to_markdown` — nested paragraphs + code block + null description.
7. `parent_hierarchy` — subtask (hierarchyLevel -1) + epic parent (hierarchyLevel 1) + no parent.
8. `rate_limit_429_honors_retry_after` — 429 with `Retry-After: 2` delays next call; 429 without header triggers exponential backoff.
9. `tenant_validation_rejects_ssrf` — `new("a.evil.com")`, `new("-bad")`, `new("")`, `new("a".repeat(64))` all `Err`.
10. `supports_reports_hierarchy_only` — `BackendFeature::Hierarchy` true; `Delete`/`Transitions`/`StrongVersioning` false until Phase 29.
11. `extensions_omitted_when_empty` — frontmatter roundtrip proves empty `extensions` doesn't serialize.
12. `write_ops_return_not_supported` — `create_issue`, `update_issue`, `delete_or_close` each return the documented error string.

Contract test (`tests/contract.rs`) parameterized identically to GitHub/Confluence: `contract_sim` (always) + `contract_jira_wiremock` (always) + `contract_jira_live` (`#[ignore]`, opt-in). `docs/reference/jira.md` (user guide + env vars + `--no-truncate` semantics). `docs/decisions/005-jira-issue-mapping.md` (ADR-005 — ID vs key, status+resolution mapping table, version synthesis, ADF stripping, attachments/comments deferred). CHANGELOG + CI green on `ubuntu-latest`.
**Requirements:** JIRA-01, JIRA-02, JIRA-03, JIRA-04, JIRA-05
**Depends on:** Phase 27 (BackendConnector rename + extensions field)
**Plans:** 3 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 28 to break down)

### Phase 29: JIRA write path — `create_issue`, `update_issue`, `delete_or_close` via Transitions API (stretch) (v0.8.0)

**Goal:** Complete the JIRA write path on `JiraBackend`. `create_issue` → `POST /rest/api/3/issue` (`fields.project.key`, `fields.summary`, `fields.issuetype.name` — discover valid types via `GET /rest/api/3/issuetype`, cache per session; body encoded as ADF `{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"..."}]}]}`). `update_issue` → `PUT /rest/api/3/issue/{id}` with `{fields:{summary,description(ADF),labels,assignee}}` — status changes are NOT allowed via PUT, only via transitions; `expected_version` silently ignored (no ETag). `delete_or_close` → two-step: (1) `GET /rest/api/3/issue/{id}/transitions` to discover available transitions, (2) `POST /rest/api/3/issue/{id}/transitions` with `{transition:{id}}` — select transition by matching `DeleteReason` to target `statusCategory.key == "done"`, prefer a transition name containing "won't"/"reject"/"not planned" for `WontFix`; if `fields.resolution` is required by the screen, include `{fields:{resolution:{name:"Done"}}}` and catch 400 to retry without it. Fallback to `DELETE /rest/api/3/issue/{id}` if transitions unavailable and DELETE returns 204 (requires admin permission). Enable `supports(BackendFeature::Delete)` and `supports(BackendFeature::Transitions)`. Audit log rows for all mutations. Wiremock write-path tests ≥3. Contract test write extension.
**Requirements:** JIRA-06
**Depends on:** Phase 28 (JIRA read-only adapter)
**Plans:** 3/3 plans complete

Plans:
- [x] 29-01-PLAN.md — ADF helpers + create_issue (SHIPPED)
- [x] 29-02-PLAN.md — update_issue + audit rows (SHIPPED)
- [x] 29-03-PLAN.md — delete_or_close transitions + contract test (SHIPPED)

## Backlog

### Phase 999.1: Follow-up — missing SUMMARY.md files from prior phases (BACKLOG)

**Goal:** Resolve plans that ran without producing summaries during earlier phase executions
**Deferred at:** 2026-04-16 during /gsd-next advancement to /gsd-verify-work (Phase 29 → milestone completion)
**Plans:**
- [ ] Phase 16: 16-D-docs-and-release (ran, no SUMMARY.md)
- [ ] Phase 17: 17-A-workload-and-cli (ran, no SUMMARY.md)
- [ ] Phase 17: 17-B-tests-and-docs (ran, no SUMMARY.md)
- [ ] Phase 18: 18-02 (ran, no SUMMARY.md)
- [ ] Phase 21: 21-A-audit (ran, no SUMMARY.md)
- [ ] Phase 21: 21-B-contention (ran, no SUMMARY.md)
- [ ] Phase 21: 21-C-truncation (ran, no SUMMARY.md)
- [ ] Phase 21: 21-D-chaos (ran, no SUMMARY.md)
- [ ] Phase 21: 21-E-macos (ran, no SUMMARY.md)
- [ ] Phase 22: 22-A-bench-upgrade (ran, no SUMMARY.md)
- [ ] Phase 22: 22-B-fixtures-and-table (ran, no SUMMARY.md)
- [ ] Phase 22: 22-C-wire-docs-ship (ran, no SUMMARY.md)
- [ ] Phase 25: 25-02 (ran, no SUMMARY.md)
- [ ] Phase 27: 27-02 (ran, no SUMMARY.md)
