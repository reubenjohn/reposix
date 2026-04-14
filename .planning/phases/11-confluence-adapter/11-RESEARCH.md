# Phase 11: Confluence Adapter — Research

**Researched:** 2026-04-13
**Domain:** Atlassian Confluence Cloud REST v2, Rust IssueBackend adapter pattern
**Confidence:** HIGH (protocol) / HIGH (codebase pattern) / MEDIUM (body.storage schema — official docs show empty `{}` examples; cross-verified via community thread)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Crate name:** `reposix-confluence`, sibling of `reposix-github` under `crates/`.
- **Backend struct:** `ConfluenceReadOnlyBackend` implementing `reposix_core::backend::IssueBackend`.
- **Constructor:** `new(creds: ConfluenceCreds, tenant: &str)` + `new_with_base_url(creds, base_url)` for wiremock tests.
- **HTTP client:** `reposix_core::http::client(ClientOpts::default())` — sealed. No direct reqwest.
- **Thread-safety:** `Arc<HttpClient>`, all methods `&self`.
- **Protocol mapping:** Option A flattening — every page becomes a flat `Issue`. No `parent_id` on `Issue`. Loss documented in ADR-002.
- **Auth:** Basic auth `Authorization: Basic base64(email:token)`. Env vars: `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`.
- **Page-to-Issue mapping:** `id = parse_u64(page.id)`, `title = page.title`, `body = page.body.storage.value`, `created_at = page.createdAt`, `updated_at = page.version.createdAt`, `version = page.version.number`, `assignee = page.ownerId`.
- **Status mapping:** `current|draft` → `Open`; `archived|trashed|deleted` → `Done`.
- **Pagination:** cursor via `_links.next` (relative path). Follow until exhausted or 500-page cap.
- **Rate limiting:** `Retry-After` seconds + `Arc<Mutex<Option<Instant>>>` gate (same as GitHub pattern).
- **Security:** SG-01 via HttpClient; SG-05 `Tainted::new` on ingress; `#![forbid(unsafe_code)]`.
- **CLI dispatch:** Add `Confluence` variant to `ListBackend` enum in `list.rs` and `mount.rs`. Fail fast if any of the three env vars missing.
- **`--project`** is space key (e.g. `REPOSIX`), not space numeric ID. Adapter resolves key→id via `GET /wiki/api/v2/spaces?keys=KEY`.
- **Tests:** ≥5 wiremock unit tests in `lib.rs`; contract test at `tests/contract.rs` parameterized; live half `#[ignore]`-gated.
- **Demos:** `scripts/demos/parity-confluence.sh` (Tier 3B) + `scripts/demos/06-mount-real-confluence.sh` (Tier 5); both exit 0 when `ATLASSIAN_API_KEY` unset.
- **ADR-002** follows ADR-001 structure.
- **`.env.example`:** rename `TEAMWORK_GRAPH_API` → `ATLASSIAN_API_KEY`; add `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`.
- **CHANGELOG** + **v0.3.0 tag**.
- **Write path:** `create_issue`, `update_issue`, `delete_or_close` all return `Err(Error::Other("not supported: ..."))`.

### Claude's Discretion
- Exact module layout within the crate (prefer flat `lib.rs` like reposix-github unless it grows big).
- Exact cursor-parsing helper (probably `fn parse_next_cursor(body: &Value) -> Option<String>`).
- Whether to unit-test the space-key→space-id resolver separately.
- Where to document the "lost metadata" tradeoff (ADR-002 primary, module doc comment secondary).

### Deferred Ideas (OUT OF SCOPE)
- PageBackend trait (Option B). v0.4+.
- `atlas_doc_format` → Markdown renderer. v0.4.
- Write path on Confluence. v0.4.
- Confluence labels → `Issue.labels`. v0.4.
- Jira adapter. Phase 12.
- `git-remote-reposix` rewire through `IssueBackend`. v0.4.
- Writing page comments / attachments. Not a v0.3 goal.
</user_constraints>

---

## Summary

Five key decisions the planner needs:

1. **Pagination is `_links.next` relative path, not Link header.** Confluence v2 returns `_links.next: "/wiki/api/v2/spaces/{id}/pages?cursor=<opaque>&limit=100"` in the JSON body, not in HTTP headers (unlike GitHub's `Link: rel="next"` header). The implementation must parse the JSON body for `_links.next`, then prepend the tenant base URL to get the absolute URL for the next request. [VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/intro]

2. **`body.storage.value` is the correct field for raw HTML content.** The v2 page response body object has shape `{ "storage": { "value": "<HTML>", "representation": "storage" }, "atlas_doc_format": { ... } }`. Request with `?body-format=storage` on the `get_issue` path (single-page fetch) but omit body on list (too expensive). [VERIFIED: community.developer.atlassian.com; MEDIUM confidence on exact nested field name]

3. **Space-key→ID resolution requires a dedicated resolver call.** `GET /wiki/api/v2/spaces?keys=KEY` returns a `results[]` array; take `results[0].id`. This is an extra round-trip before the first `list_issues` call, and the space ID is a **string** (numeric string like `"12345"`), not an int in Rust terms. [VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-space]

4. **Rate limit headers are `X-RateLimit-Remaining` + `X-RateLimit-Reset` (ISO 8601) + `Retry-After` (seconds).** This is different from GitHub's `x-ratelimit-reset` (unix epoch integer). The adapter must parse `Retry-After` as seconds integer, not a timestamp. [VERIFIED: developer.atlassian.com/cloud/confluence/rate-limiting]

5. **All page IDs in v2 are string-typed JSON fields**, even though they are numeric values. The Rust deserializer must handle them as `String` then `parse::<u64>()` — NOT as JSON integers. The official schema specifies `"id": "string"`. [VERIFIED: official API schema at developer.atlassian.com/cloud/confluence/rest/v2/api-group-page]

**Primary recommendation:** Copy `crates/reposix-github/src/lib.rs` structure exactly, substituting cursor-from-body for Link-header parsing and adding the space-key resolver. The overall struct/trait/wiremock shape is identical.

---

## Confluence REST v2 Endpoint Reference

### 1. Resolve Space Key → Space ID

```
GET https://{tenant}.atlassian.net/wiki/api/v2/spaces?keys={SPACE_KEY}
```

Required headers:
```
Authorization: Basic <base64(email:token)>
Accept: application/json
```

Response shape (200 OK):
```json
{
  "results": [
    {
      "id": "12345",
      "key": "MYSPACE",
      "name": "My Space",
      "type": "global",
      "status": "current",
      "authorId": "557058:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "createdAt": "2024-01-15T10:30:00.000Z",
      "homepageId": "98765",
      "_links": {
        "webui": "/wiki/spaces/MYSPACE"
      }
    }
  ],
  "_links": {
    "base": "https://mycompany.atlassian.net/wiki",
    "next": "/wiki/api/v2/spaces?keys=MYSPACE&cursor=eyJsaW1pdCI6MjV9"
  }
}
```

Field types used by the adapter:
- `results[0].id` — `String` (numeric string); must `parse::<u64>()` for internal use
- `results[0].key` — `String`
- `results[0].name` — `String`

Error responses:
- 401: invalid credentials
- 403: no permission
- 404: space does not exist (results array is empty, not a 404 status — check `results.is_empty()`)

[VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-space]

---

### 2. List Pages in a Space

```
GET https://{tenant}.atlassian.net/wiki/api/v2/spaces/{space_id}/pages?limit=100
```

Required headers:
```
Authorization: Basic <base64(email:token)>
Accept: application/json
```

Note: do NOT include `?body-format=storage` on this endpoint — body is expensive and not needed for `list_issues`. The `body` field in list responses contains empty sub-objects when not requested.

Response shape (200 OK):
```json
{
  "results": [
    {
      "id": "98765",
      "status": "current",
      "title": "My Page Title",
      "spaceId": "12345",
      "parentId": "11111",
      "parentType": "page",
      "position": 1,
      "authorId": "557058:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "ownerId": "557058:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
      "lastOwnerId": null,
      "createdAt": "2024-01-15T10:30:00.000Z",
      "version": {
        "createdAt": "2024-02-20T14:00:00.000Z",
        "message": "",
        "number": 3,
        "minorEdit": false,
        "authorId": "557058:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
      },
      "body": {},
      "_links": {
        "webui": "/wiki/spaces/MYSPACE/pages/98765",
        "editui": "/wiki/spaces/MYSPACE/pages/edit-v2/98765",
        "tinyui": "/wiki/x/abc123"
      }
    }
  ],
  "_links": {
    "base": "https://mycompany.atlassian.net/wiki",
    "next": "/wiki/api/v2/spaces/12345/pages?cursor=eyJsaW1pdCI6MTAwLCJzdGFydCI6MTAwfQ%3D%3D&limit=100"
  }
}
```

When there is no next page, `_links.next` is absent (key not present, not null).

Field types:
- `id` — `String` (numeric string)
- `status` — `String`: one of `"current"`, `"draft"`, `"archived"`, `"trashed"`, `"deleted"`
- `authorId` / `ownerId` — `String` (Atlassian account ID format)
- `createdAt` — ISO 8601 string: `"2024-01-15T10:30:00.000Z"` — parse with `chrono::DateTime<Utc>`
- `version.createdAt` — same format
- `version.number` — integer (JSON number, fits in u64)
- `parentId` — `String` or null (deferred; not mapped in v0.3)

[VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-page]

---

### 3. Get Single Page with Body

```
GET https://{tenant}.atlassian.net/wiki/api/v2/pages/{page_id}?body-format=storage
```

Required headers:
```
Authorization: Basic <base64(email:token)>
Accept: application/json
```

Response shape (200 OK) — extends the list-page shape:
```json
{
  "id": "98765",
  "status": "current",
  "title": "My Page Title",
  "spaceId": "12345",
  "parentId": "11111",
  "parentType": "page",
  "position": 1,
  "authorId": "557058:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "ownerId": "557058:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "lastOwnerId": null,
  "createdAt": "2024-01-15T10:30:00.000Z",
  "version": {
    "createdAt": "2024-02-20T14:00:00.000Z",
    "message": "minor edit",
    "number": 3,
    "minorEdit": false,
    "authorId": "557058:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
  },
  "body": {
    "storage": {
      "value": "<p>This is the <strong>page content</strong> in Confluence storage format.</p>",
      "representation": "storage"
    },
    "atlas_doc_format": {
      "value": "{\"version\":1,\"type\":\"doc\",\"content\":[...]}",
      "representation": "atlas_doc_format"
    }
  },
  "_links": {
    "base": "https://mycompany.atlassian.net/wiki"
  }
}
```

The adapter reads `body.storage.value` for `Issue.body`. `body.atlas_doc_format` is ignored in v0.3.

Error responses:
- 401: `{"statusCode":401,"message":"Unauthorized"}`
- 403: `{"statusCode":403,"message":"Forbidden","data":{"..."}}`
- 404: `{"statusCode":404,"message":"Not found","data":{"..."}}`
- 429: see Rate-limit section

[VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-page; MEDIUM — body.storage.value field name cross-verified via community.developer.atlassian.com/t/get-body-of-a-page-through-api-v2]

---

## Pagination Contract

**Mechanism:** Cursor-based. The next-page URL is returned as a **relative path** under `_links.next` in the JSON response body (not as an HTTP `Link:` header — that is the GitHub pattern).

**JSON key path:**
```
response_body._links.next
```

Example value:
```
"/wiki/api/v2/spaces/12345/pages?cursor=eyJsaW1pdCI6MTAwLCJzdGFydCI6MTAwfQ%3D%3D&limit=100"
```

The cursor is URL-safe Base64 of an opaque internal token. Do not parse it.

**Algorithm to follow pages:**

```rust
fn parse_next_cursor(body: &serde_json::Value) -> Option<String> {
    body.get("_links")
        .and_then(|l| l.get("next"))
        .and_then(|n| n.as_str())
        .map(str::to_owned)
}

// In list_issues:
// next_url = Some(format!("{base_url}{relative_path}"))
// where base_url = "https://{tenant}.atlassian.net"
```

**Key difference from GitHub:** GitHub puts the absolute next URL in the `Link:` HTTP response header. Confluence puts a relative path in `_links.next` in the JSON body. The adapter must:
1. Consume the full response bytes first (for serde).
2. Extract `_links.next` from the parsed JSON.
3. Prepend the tenant base URL (NOT the wiki-relative base from `_links.base`).

**No more pages:** `_links.next` key is absent. This is NOT a `null` value; the key is missing. Use `.and_then()` not `.map()` to handle absence cleanly.

**Pagination cap:** Same constant as GitHub: `MAX_PAGES_PER_LIST = 5` pages at `PAGE_SIZE = 100` = 500 issues. Log WARN at cap hit.

[VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/intro; community.developer.atlassian.com/t/what-is-the-correct-way-to-handle-pagination-in-the-confluence-rest-api-v2]

---

## Auth Contract

**Scheme:** HTTP Basic Authentication.

```
Authorization: Basic <base64(email:api_token)>
```

Where:
- `email` = the exact Atlassian account email address under which the token was issued (NOT just any email; see `00-CREDENTIAL-STATUS.md` for the common mismatch failure mode).
- `api_token` = the API token from `id.atlassian.com/manage-profile/security/api-tokens`.
- Separator is `:` (colon). No spaces.
- Standard Base64 encoding (`standard` alphabet, NOT URL-safe, WITH padding). In Rust: `base64::engine::general_purpose::STANDARD.encode(format!("{email}:{token}"))`.

Rust construction:
```rust
fn basic_auth_header(email: &str, token: &str) -> String {
    use base64::Engine;
    let raw = format!("{email}:{token}");
    let encoded = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
    format!("Basic {encoded}")
}
```

Required additional headers (beyond Authorization):
```
Accept: application/json
```

User-Agent is set automatically by `HttpClient` via `ClientOpts::default()`. No additional Atlassian-specific headers needed.

**Bearer auth does NOT work** for user API tokens. Bearer requires OAuth 2.0 3LO. Document this in ADR-002. The CONTEXT.md decision locks Basic auth.

**Failure modes:**
- 401 with `x-failure-category: FAILURE_CLIENT_AUTH_MISMATCH` — email and token belong to different Atlassian accounts.
- 401 without that header — token invalid or expired.
- 403 — valid auth, but no permission to the space.

**SG-01 implication:** The tenant base URL `https://{tenant}.atlassian.net` must appear in `REPOSIX_ALLOWED_ORIGINS` at runtime. The default allowlist is loopback-only; the user must set the env var. Fail fast (same pattern as GitHub) if the env var does not include the tenant origin.

[VERIFIED: developer.atlassian.com/cloud/confluence/basic-auth-for-rest-apis]

---

## Rate-limit Contract

**Headers returned with every response** (not just 429):
```
X-RateLimit-Limit: 65000
X-RateLimit-Remaining: 64500
X-RateLimit-Reset: 2025-10-08T15:00:00Z   ← ISO 8601, NOT unix epoch
X-RateLimit-NearLimit: false
```

**On 429 Too Many Requests:**
```
HTTP/1.1 429 Too Many Requests
Retry-After: 1847              ← seconds until retry (integer)
X-RateLimit-Limit: 40000
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 2025-10-08T15:00:00Z
RateLimit-Reason: confluence-quota-global-based
```

**Key difference from GitHub:** GitHub uses `x-ratelimit-reset` as a **unix epoch integer**. Atlassian uses `X-RateLimit-Reset` as an **ISO 8601 string** AND provides `Retry-After` in seconds. The adapter should use `Retry-After` (simpler) rather than parsing ISO 8601 from `X-RateLimit-Reset`.

**Adapter implementation** (mirrors GitHub's gate exactly, but reads `Retry-After` not `x-ratelimit-reset`):
```rust
fn ingest_rate_limit(&self, resp: &reqwest::Response) {
    let remaining = resp.headers()
        .get("x-ratelimit-remaining")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    let retry_after = resp.headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    match remaining {
        Some(0) => {
            let wait = retry_after.unwrap_or(60).min(MAX_RATE_LIMIT_SLEEP.as_secs());
            if wait > 0 {
                let gate = Instant::now() + Duration::from_secs(wait);
                *self.rate_limit_gate.lock() = Some(gate);
                tracing::warn!(wait_secs = wait, "Confluence rate limit — backing off");
            }
        }
        Some(n) if n < 10 => {
            tracing::warn!(remaining = n, "Confluence rate limit approaching");
        }
        _ => {}
    }
}
```

**Note on log header case:** `reqwest` normalizes headers to lowercase. Use `"retry-after"` and `"x-ratelimit-remaining"` (lowercase) when calling `resp.headers().get(...)`.

[VERIFIED: developer.atlassian.com/cloud/confluence/rate-limiting]

---

## Status Mapping

Confluence pages have a `status` field with these documented values:

| Confluence `status` | Meaning | Maps to `IssueStatus` |
|---------------------|---------|----------------------|
| `current` | Published, live | `Open` |
| `draft` | Unpublished draft | `Open` (treat as open work) |
| `archived` | Archived (visible, not active) | `Done` |
| `trashed` | In trash (soft-deleted) | `Done` |
| `deleted` | Permanently deleted | `Done` (rarely seen via API) |
| *(unknown)* | Forward-compat | `Open` (pessimistic fallback — don't surface `WontFix` for unknown statuses) |

`InProgress` and `InReview` are not representable from Confluence page metadata alone in v0.3 — deferred (no label endpoint mapped yet).

Rust translation:
```rust
fn status_from_confluence(s: &str) -> IssueStatus {
    match s {
        "current" | "draft" => IssueStatus::Open,
        "archived" | "trashed" | "deleted" => IssueStatus::Done,
        _ => IssueStatus::Open, // pessimistic forward-compat
    }
}
```

[VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-page — status field enum values]

---

## Body Format Choice

**Recommendation for v0.3: `storage` (Confluence HTML/XML).**

| Format | Content | Rust complexity | Stability |
|--------|---------|-----------------|-----------|
| `storage` | XHTML-based markup with Confluence macros | `String` — no transform needed | Stable, well-documented |
| `atlas_doc_format` | JSON-encoded ADF (Atlassian Document Format) | Requires ADF→Markdown renderer | Stable but verbose |
| `view` | Already-rendered HTML | `String` — but includes full CSS/JS references | Less stable across upgrades |

**Rationale for v0.3:** `storage` is a raw `String` that fits directly into `Issue.body` with zero transform code. The content is ugly XHTML (e.g. `<p>Hello <strong>world</strong></p>`) but it is correct and stable. An agent using `cat page.md` gets the raw XML, which is not pretty but is unambiguous.

The `atlas_doc_format` value is a JSON string embedded inside the JSON response — it requires an extra `serde_json::from_str` step plus a recursive ADF→Markdown walk. That is v0.4 scope per the locked decisions.

**Query parameter to add on `get_issue` call only:**
```
?body-format=storage
```

Do NOT add `?body-format=storage` to `list_issues` calls — body content is expensive to return for 100 pages and is not needed for a listing. The `body` field in list responses will be an empty object (`{}`), which deserializes to `body.storage = None`.

[VERIFIED: developer.atlassian.com/cloud/confluence/rest/v2/api-group-page; community.developer.atlassian.com/t/get-body-of-a-page-through-api-v2]

---

## Pattern Delta vs reposix-github

The `ConfluenceReadOnlyBackend` is structurally nearly identical to `GithubReadOnlyBackend`. The differences are:

| Concern | GitHub pattern | Confluence pattern | Delta |
|---------|---------------|--------------------|-------|
| **Pagination signal** | `Link:` HTTP response header | `_links.next` in JSON body | Must parse body before reading next-page signal |
| **Next URL format** | Absolute URL | Relative path (`/wiki/api/v2/...`) | Must prepend `https://{tenant}.atlassian.net` |
| **Pagination parser fn** | `parse_next_link(header: &str)` | `parse_next_cursor(body: &Value)` | Different input type |
| **Space resolution** | Not needed (project = `owner/repo` path) | `GET /wiki/api/v2/spaces?keys=KEY` → `space_id` | Extra resolver method + extra HTTP round-trip before first `list_issues` |
| **Auth header** | `Authorization: Bearer {token}` | `Authorization: Basic base64(email:token)` | Two credentials required, not one |
| **Constructor creds** | `token: Option<String>` | `creds: ConfluenceCreds { email, api_token }` + `tenant: &str` | Different credential struct |
| **Rate-limit header** | `x-ratelimit-reset` (unix epoch u64) | `Retry-After` (seconds u64) | Different header name and format |
| **ID type on wire** | JSON integer (`number`) | JSON string (`"id": "12345"`) | Deserialize as `String`, then `parse::<u64>()` |
| **`updated_at` source** | `updated_at` top-level field | `version.createdAt` nested field | Different JSON path |
| **Body on list** | `body` top-level field | Must call separate `get_issue` with `?body-format=storage` | List returns empty body; body only populated on single-page fetch |
| **Status encoding** | `state` (open/closed) + `state_reason` | `status` (current/draft/archived/trashed) | Different values, different mapping logic |
| **Assignee field** | `assignee.login` (nested object) | `ownerId` (top-level string — Atlassian account ID) | Simpler extraction; no nested object |

**Struct layout** (one crate, flat `lib.rs`):
```rust
pub struct ConfluenceReadOnlyBackend {
    http: Arc<HttpClient>,
    creds: ConfluenceCreds,
    base_url: String,  // "https://{tenant}.atlassian.net"
    rate_limit_gate: Arc<Mutex<Option<Instant>>>,
}

pub struct ConfluenceCreds {
    pub email: String,
    pub api_token: String,
}
```

The **space-key resolver** is a private method, not part of `IssueBackend`:
```rust
async fn resolve_space_id(&self, space_key: &str) -> Result<String>
```

This is called at the top of `list_issues` on every call (no in-memory cache needed for v0.3 — the round-trip is cheap and cacheable behavior is a v0.4 concern).

**Allowlist guard in `mount.rs`** (mirrors GitHub guard):
```rust
if backend == ListBackend::Confluence {
    let raw = std::env::var("REPOSIX_ALLOWED_ORIGINS").unwrap_or_default();
    let tenant = std::env::var("REPOSIX_CONFLUENCE_TENANT").unwrap_or_default();
    if !raw.contains(&format!("{tenant}.atlassian.net")) {
        bail!("REPOSIX_ALLOWED_ORIGINS must include https://{tenant}.atlassian.net for --backend confluence");
    }
}
```

---

## Validation Architecture

`nyquist_validation: true` in `.planning/config.json` — this section is required.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) + tokio-test + wiremock 0.6 |
| Config file | none — inline `#[tokio::test]` attributes |
| Quick run command | `cargo test -p reposix-confluence` |
| Full suite command | `cargo test --workspace --locked` |

### Coverage Map — Public Functions × Wire Shapes

Every `pub` function on `ConfluenceReadOnlyBackend` must be covered. Minimum 5 unit tests; recommended 8 to cover the delta from GitHub.

| Test name (in `lib.rs` `mod tests`) | Covers | Type | Wire shape exercised |
|--------------------------------------|--------|------|----------------------|
| `list_resolves_space_key_and_fetches_pages` | `list_issues` | unit (wiremock) | spaces?keys= → 200, spaces/{id}/pages → 200 |
| `list_paginates_via_links_next` | `list_issues` cursor loop | unit (wiremock) | page 1 has `_links.next`; page 2 has none |
| `get_issue_returns_body_storage_value` | `get_issue` | unit (wiremock) | pages/{id}?body-format=storage → 200 with nested body |
| `get_404_maps_to_not_found` | `get_issue` error path | unit (wiremock) | pages/{id} → 404 |
| `status_current_maps_to_open` | status translation | unit (wiremock) | page.status = "current" |
| `status_trashed_maps_to_done` | status translation | unit (wiremock) | page.status = "trashed" |
| `auth_header_is_basic_not_bearer` | SG-01 auth | unit (wiremock custom Match) | No `Bearer` in Authorization header |
| `rate_limit_retry_after_arms_gate` | rate-limit gate | unit (wiremock) | 429 with Retry-After: 2 header |
| `write_methods_return_not_supported` | read-only guard | unit (no wiremock needed) | no HTTP |
| `supports_returns_no_features` | capability matrix | unit (no wiremock needed) | no HTTP |
| `contract_sim` | IssueBackend contract | contract (tests/contract.rs) | sim backend |
| `contract_confluence_wiremock` | IssueBackend contract | contract (tests/contract.rs) | wiremock-backed ConfluenceReadOnlyBackend |
| `contract_confluence_live` | end-to-end live | contract (`#[ignore]`) | real Atlassian tenant |

### Custom `Match` impl for auth header test

The HANDOFF §8 gotcha: wiremock permissive matchers always pass. To prove `Authorization: Bearer ...` is ABSENT, write:

```rust
struct NoBearer;
impl wiremock::Match for NoBearer {
    fn matches(&self, request: &wiremock::Request) -> bool {
        request
            .headers
            .get("authorization")
            .map(|v| {
                let s = v.to_str().unwrap_or("");
                s.starts_with("Basic ") && !s.starts_with("Bearer ")
            })
            .unwrap_or(false)
    }
}
```

Mount with `.and(NoBearer)`.

### Wave 0 Gaps (must exist before Wave 1)

- [ ] `crates/reposix-confluence/Cargo.toml` — new crate
- [ ] `crates/reposix-confluence/src/lib.rs` — stub with `ConfluenceReadOnlyBackend` and `#![forbid(unsafe_code)]`
- [ ] `crates/reposix-confluence/tests/contract.rs` — initially just the sim contract test
- [ ] Add `reposix-confluence` to workspace `Cargo.toml` members
- [ ] Add `reposix-confluence.workspace = true` as a dependency of `reposix-cli`

### Sampling Rate

- Per task commit: `cargo test -p reposix-confluence`
- Per wave merge: `cargo test --workspace --locked`
- Phase gate: full suite green + `bash scripts/demos/smoke.sh` 4/4 before `/gsd-verify-work`

---

## Open Questions / Risks

### OQ-1: `base64` crate availability
**What we know:** The workspace `Cargo.toml` does not currently include `base64` as a workspace dependency.
**Gap:** `ConfluenceReadOnlyBackend` needs `base64::engine::general_purpose::STANDARD.encode(...)` to construct the Basic auth header. The alternative is `base64 = "0.22"` added to `Cargo.toml`.
**Recommendation:** Add `base64 = "0.22"` to workspace dependencies and to `reposix-confluence`'s `[dependencies]`. [ASSUMED — version 0.22 is current as of training; verify with `cargo search base64` during execution]

### OQ-2: `body.storage` deserialization when body is absent
**What we know:** List-page responses return `"body": {}` (empty object) when `?body-format` is not requested.
**Gap:** The serde struct for the page body must handle both the populated case (`"body": {"storage": {"value": "...", "representation": "storage"}}`) and the empty case (`"body": {}`).
**Recommendation:**
```rust
#[derive(Debug, Deserialize)]
struct ConfPageBody {
    #[serde(default)]
    storage: Option<ConfBodyStorage>,
}

#[derive(Debug, Deserialize)]
struct ConfBodyStorage {
    value: String,
    #[allow(dead_code)]
    representation: String,
}
```
Then `Issue.body = page.body.storage.as_ref().map(|s| s.value.clone()).unwrap_or_default()`.

### OQ-3: Page ID is a string — parse_u64 failure
**What we know:** Confluence page IDs are numeric strings like `"98765"`.
**Gap:** If Atlassian ever returns a non-numeric page ID (e.g. for special system pages), `parse::<u64>()` will fail.
**Recommendation:** Map parse failure to `Error::Other(format!("confluence page id is not a u64: {id}"))` rather than panicking. Log a WARN.

### OQ-4: Space not found vs. empty results
**What we know:** `GET /wiki/api/v2/spaces?keys=BADKEY` returns HTTP 200 with `{"results": [], "_links": {...}}`, not HTTP 404.
**Gap:** The adapter must check `results.is_empty()` and return `Err(Error::Other("not found: space key ..."))` explicitly, not propagate a 200 as success.
**Recommendation:** Add this check in `resolve_space_id` explicitly and include a wiremock test for it.

### OQ-5: 5-second HttpClient timeout vs. large pages
**What we know:** `ClientOpts::default()` sets a 5-second total timeout (SG-07).
**Gap:** A very large Confluence page (e.g. one with embedded tables and 50KB of storage HTML) may take longer than 5s to download + deserialize on a slow connection.
**Recommendation:** This is a known SG-07 limitation consistent with the GitHub adapter. Keep the 5s default. Document in ADR-002 and crate module doc. Do not increase the timeout without a separate config mechanism (v0.4 concern).

### OQ-6: Integration CI job secret
**What we know:** The user must add `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` as GitHub Actions repo secrets.
**Gap:** The integration CI YAML job must gate on all three: `if: ${{ secrets.ATLASSIAN_API_KEY != '' && secrets.ATLASSIAN_EMAIL != '' && secrets.REPOSIX_CONFLUENCE_TENANT != '' }}`.
**Recommendation:** Document the `gh secret set` commands in `MORNING-BRIEF-v0.3.md` (Phase 11-F).

---

## Project Constraints (from CLAUDE.md)

All of the following MUST be satisfied by every file produced in this phase:

| Constraint | Source |
|------------|--------|
| `#![forbid(unsafe_code)]` at crate root | CLAUDE.md |
| `#![warn(clippy::pedantic, missing_docs)]` at crate root | CONTEXT.md |
| All public items documented; `# Errors` on every `Result`-returning fn | CLAUDE.md |
| All HTTP via `reposix_core::http::HttpClient` — clippy `disallowed-methods` lint enforces | CLAUDE.md |
| `Tainted::new(...)` on every ingress from the backend | CLAUDE.md (SG-05) |
| No direct `reqwest::Client` construction — use `client(ClientOpts::default())` | CLAUDE.md |
| `REPOSIX_ALLOWED_ORIGINS` must include tenant origin at runtime | CLAUDE.md (SG-01) |
| Frontmatter uses `serde_yaml` 0.9 + Markdown body; never JSON on disk | CLAUDE.md |
| `chrono::DateTime<Utc>` for times; no `SystemTime` in serialized form | CLAUDE.md |
| `cargo test --workspace --locked` ≥180 tests, 0 failures | CONTEXT.md |
| `cargo clippy --workspace --all-targets -- -D warnings` clean | CONTEXT.md |
| `cargo fmt --all --check` clean | CONTEXT.md |
| `bash scripts/demos/smoke.sh` 4/4 green | CONTEXT.md |
| No secrets committed; `.env` gitignored | CLAUDE.md |
| Each commit atomic with `feat(11-X-N):` / `test(...):` / `docs(...):` / `fix(...):` prefix | CONTEXT.md |
| Demos exit 0 on missing env (SKIP path) | CONTEXT.md |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `base64 = "0.22"` is the current crate version | Open Questions OQ-1 | Use wrong version; fails at compile; fix with `cargo search base64` |
| A2 | `body.storage.value` contains the raw HTML string (not a nested object) | Body Format / Endpoint Reference | Deserialization panic; would need to adjust struct; low risk — cross-verified via community thread |
| A3 | Confluence page IDs will always parse as u64 for real spaces | Open Questions OQ-3 | Hard error on any non-numeric ID (system pages, etc.); handled by OQ-3 mitigation |
| A4 | `body` field is fully absent from list responses (not `null`) | Validation Architecture OQ-2 | Serde error if `null` body unexpectedly appears; mitigated by `#[serde(default)]` |

**All claims in Standard Stack and Endpoint Reference sections are VERIFIED from official Atlassian docs except A1–A4 above.**

---

## Sources

### Primary (HIGH confidence)
- [developer.atlassian.com/cloud/confluence/rest/v2/api-group-page](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/) — page endpoints, response schema, field names and types
- [developer.atlassian.com/cloud/confluence/rest/v2/api-group-space](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-space/) — space endpoints, `keys` param
- [developer.atlassian.com/cloud/confluence/rest/v2/intro](https://developer.atlassian.com/cloud/confluence/rest/v2/intro/) — cursor pagination mechanism, `_links.next`
- [developer.atlassian.com/cloud/confluence/rate-limiting](https://developer.atlassian.com/cloud/confluence/rate-limiting/) — exact header names, 429 body example
- [developer.atlassian.com/cloud/confluence/basic-auth-for-rest-apis](https://developer.atlassian.com/cloud/confluence/basic-auth-for-rest-apis/) — Basic auth format
- `crates/reposix-github/src/lib.rs` — pattern template (read directly from codebase)
- `crates/reposix-github/tests/contract.rs` — contract test pattern (read directly)
- `crates/reposix-core/src/http.rs` — sealed HttpClient API (read directly)
- `crates/reposix-core/src/backend.rs` — IssueBackend trait surface (read directly)

### Secondary (MEDIUM confidence)
- [community.developer.atlassian.com/t/what-is-the-correct-way-to-handle-pagination-in-the-confluence-rest-api-v2/86716](https://community.developer.atlassian.com/t/what-is-the-correct-way-to-handle-pagination-in-the-confluence-rest-api-v2/86716) — practical pagination cursor extraction pattern
- [community.developer.atlassian.com/t/get-body-of-a-page-through-api-v2/67966](https://community.developer.atlassian.com/t/get-body-of-a-page-through-api-v2/67966) — body.storage.value field existence confirmed

---

## Metadata

**Confidence breakdown:**
- Protocol (endpoint URLs, query params, status codes): HIGH — verified against official Atlassian developer docs
- Field names and types: HIGH for top-level fields; MEDIUM for `body.storage` nested structure (official schema shows `{}` placeholder; confirmed via community thread)
- Pagination mechanism: HIGH — cursor-in-body confirmed via two sources
- Auth format: HIGH — official Basic auth docs
- Rate-limit headers: HIGH — exact header names listed in official rate-limiting docs

**Research date:** 2026-04-13
**Valid until:** 2026-07-13 (Atlassian APIs are stable; REST v2 docs have been consistent; cursor pagination confirmed as the intended mechanism)

---

## RESEARCH COMPLETE
