# Phase 28: JIRA Cloud read-only adapter — Research

## RESEARCH COMPLETE

**Phase:** 28 — JIRA Cloud read-only adapter (`reposix-jira`)
**Date:** 2026-04-16
**Researcher:** Inline (comprehensive ROADMAP spec + codebase scout)

---

## Validation Architecture

The 12 enumerated wiremock tests in ROADMAP.md §"Test matrix" form the complete validation architecture. Each test targets a distinct failure mode:
- Tests 1-2: pagination correctness
- Tests 3-4: happy-path + error path
- Tests 5-6: data-mapping correctness
- Tests 7: hierarchy
- Tests 8: rate limit resilience
- Tests 9: SSRF prevention
- Tests 10-12: capability flags + serialization + write stubs

---

## API Shape

### JIRA Cloud REST v3 endpoints needed

**`POST /rest/api/3/search/jql`** (list issues)
- Request body: `{"jql": "project = \"{KEY}\" ORDER BY id ASC", "fields": ["id","key","summary","description","status","resolution","assignee","labels","created","updated","parent","issuetype","priority"], "maxResults": 100}`
- With cursor: add `"nextPageToken": "{token}"`
- Response: `{"issues": [...], "isLast": true|false, "nextPageToken": "..."}`
- **Critical:** `total` field is absent from this endpoint (retired). Do NOT use `startAt` or offset math.
- Old `GET /rest/api/3/search` was retired Aug 2025 — MUST use POST endpoint.

**`GET /rest/api/3/issue/{issueIdOrKey}`** (get single issue)
- Path param: numeric issue ID (not PROJ-42 key) — JIRA accepts both but we use numeric.
- Response: full issue object with `fields` nested under a `fields` key.

### Authentication
- Basic auth: `Authorization: Basic base64(email:api_token)`
- Same pattern as Confluence — `base64::engine::general_purpose::STANDARD.encode(format!("{email}:{api_token}"))`

---

## Data Mapping

### Status mapping (two-field)
```
fields.status.statusCategory.key:
  "new"           → Open
  "indeterminate" → InProgress  (unless status.name contains "review" → InReview)
  "done"          → Done
  unknown         → Open  (safe fallback)

Override: if fields.resolution.name contains "won't"|"wont"|"not a bug"|"duplicate"|"cannot reproduce"
  → WontFix  (regardless of statusCategory)
```

### Version synthesis
- `fields.updated` is ISO-8601 string (e.g. `"2025-11-15T10:30:00.000+0000"`)
- Parse with `chrono::DateTime<chrono::FixedOffset>::parse_from_rfc3339` (or similar)
- Convert to Unix milliseconds: `dt.timestamp_millis() as u64`
- Store as `Issue.version: u64`

### ADF plain-text extraction
JIRA `fields.description` is ADF JSON or `null`. The existing `adf.rs` in `reposix-confluence` handles ADF→Markdown. For Phase 28 we need ADF→plain text (simpler). Walk the ADF tree:
- `paragraph`, `doc` → join children with newline
- `text` → emit `text` field value
- `hardBreak` → emit `\n`
- `codeBlock` → emit content + `\n`
- Unknown nodes → recurse into `content[]` children, emit their text
- `null` description → empty string

Can share the recursive tree-walking structure from `adf.rs` but produce plain text instead of markdown.

### Issue.extensions keys
```rust
extensions: BTreeMap<String, serde_yaml::Value> = {
  "jira_key"       => Value::String("PROJ-42"),
  "issue_type"     => Value::String("Story"),
  "priority"       => Value::String("Medium"),  // omit if null
  "status_name"    => Value::String("In Progress"),
  "hierarchy_level" => Value::Number(-1_i64 as serde_yaml::Number),
}
```
`serde_yaml::Value::Number` from i64: use `serde_yaml::Value::from(hierarchy_level as i64)`.

### Hierarchy
- `fields.parent.id` (number, not key) → parse as u64 → `Issue.parent_id = Some(IssueId(id))`
- Subtask: `issuetype.hierarchyLevel == -1`
- Epic: `issuetype.hierarchyLevel == 1`
- Standard issue: `issuetype.hierarchyLevel == 0`

---

## Codebase Patterns to Follow

### New crate structure (mirror `reposix-confluence`)
```
crates/reposix-jira/
├── Cargo.toml          — workspace = true, same deps as reposix-confluence minus pulldown-cmark
├── src/
│   ├── lib.rs          — JiraBackend, JiraCreds (redacted Debug), BackendConnector impl
│   └── adf.rs          — adf_to_plain_text (simpler than adf_to_markdown)
└── tests/
    └── contract.rs     — contract_sim + contract_jira_wiremock + contract_jira_live (#[ignore])
```

### Tenant validation (copy from ConfluenceBackend, adapt error messages)
```rust
fn validate_tenant(tenant: &str) -> Result<()> {
    // Non-empty, 1..=63 chars, [a-z0-9-], no leading/trailing hyphen
    // Use same logic as ConfluenceBackend::validate_tenant
    // Error: Error::Other("invalid jira tenant subdomain: {tenant:?} (…)")
}
```
Note: JIRA tenant regex is identical to Confluence — both are `*.atlassian.net` subdomains.

### Rate limit gate (adapt from GithubReadOnlyBackend or ConfluenceBackend)
```rust
// parking_lot::Mutex<Option<Instant>> for retry-after tracking
// On 429 response:
//   if Retry-After header present: sleep(header_seconds)
//   else: exponential backoff (base 1s, max 4 attempts, jitter)
// Max 4 attempts total
```

### HTTP client construction
```rust
use reposix_core::http::{client, ClientOpts};
let http = client(ClientOpts::default())?;
// Register allowlist entry: REPOSIX_ALLOWED_ORIGINS must include https://{tenant}.atlassian.net
// Do NOT hard-code the allowlist — only register the format in docs/CLI
```

### Audit log (read operations — Phase 28 requirement)
Per ROADMAP: "Audit log rows for BOTH reads (list, get)". This differs from ConfluenceBackend which only audits writes. The audit must cover:
- `list_issues` call: log `{action: "jira_list", project, count: N, hash: sha256_prefix(body)}`
- `get_issue` call: log `{action: "jira_get", project, id, hash: sha256_prefix(body)}`
The `audit_write` pattern from ConfluenceBackend can be adapted but renamed `audit_event` since this covers reads too.

### ListBackend enum extension (reposix-cli)
```rust
pub enum ListBackend {
    Sim, Github, Confluence,
    Jira,  // NEW: add variant
}
```
Add `read_jira_env_from` pure-fn helper in `list.rs` (same pattern as `read_confluence_env_from`):
- Reads `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE`
- Collects ALL missing vars into one error message
- Does NOT echo values in error messages

---

## Wiremock Test Fixtures

Key JSON fixture shapes needed:

### Issue JSON (used in tests 1-5, 7)
```json
{
  "id": "10001",
  "key": "PROJ-1",
  "fields": {
    "summary": "Fix login bug",
    "description": {
      "type": "doc", "version": 1,
      "content": [{"type":"paragraph","content":[{"type":"text","text":"Body text"}]}]
    },
    "status": {
      "name": "In Progress",
      "statusCategory": {"key": "indeterminate"}
    },
    "resolution": null,
    "assignee": {"displayName": "Alice"},
    "labels": [],
    "created": "2025-01-01T00:00:00.000+0000",
    "updated": "2025-11-15T10:30:00.000+0000",
    "parent": null,
    "issuetype": {"name": "Story", "hierarchyLevel": 0},
    "priority": {"name": "Medium"}
  }
}
```

### List response wrapper
```json
{
  "issues": [...],
  "isLast": true
}
```
Pagination variant adds `"nextPageToken": "abc123"` and `"isLast": false` on first page.

### 404 body
```json
{"errorMessages": ["Issue Does Not Exist"], "errors": {}}
```

---

## CI Integration

`reposix-jira` is added to the workspace `members` list in `Cargo.toml`. The CI workflow (`cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`) picks it up automatically.

`contract_jira_live` must be `#[ignore]`-gated (requires real JIRA credentials). The wiremock-backed `contract_jira_wiremock` must run without any credentials or network access — suitable for CI.

---

## Documentation

### `docs/reference/jira.md`
Required sections:
1. Setup (env vars: `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE`)
2. Usage (`reposix list --backend jira --project KEY`)
3. `--no-truncate` semantics (error if project > 500 issues)
4. Authentication notes (basic auth, token from id.atlassian.com/manage-profile/security/api-tokens)
5. Allowlist configuration (`REPOSIX_ALLOWED_ORIGINS=https://{instance}.atlassian.net`)

### `docs/decisions/005-jira-issue-mapping.md` (ADR-005)
Required sections:
1. ID vs key — why numeric ID is canonical IssueId, key goes in extensions
2. Status + resolution mapping table (exact mapping rules)
3. Version synthesis (updated → Unix millis, StrongVersioning: false)
4. ADF stripping (plain text extraction, not markdown)
5. Attachments/comments deferred to Phase 29+

---

## Risk Areas

1. **ADF null handling** — `fields.description` can be `null` (not just absent). Must check before traversing.
2. **Resolution name variants** — "Won't Fix", "Won't Do", "Duplicate", "Not a Bug", "Cannot Reproduce" — use `contains()` with lowercase for robustness.
3. **`isLast` vs absent** — The `isLast` field may be absent on first page if only one page. Treat absent as `true` (safe: no next-page loop).
4. **Pagination termination** — Loop condition: while `!is_last` AND `next_page_token.is_some()`. Double-condition prevents infinite loops.
5. **Tenant validation case sensitivity** — JIRA tenants are lowercase; validate with `.is_ascii_lowercase()` (matches Confluence pattern exactly).
