---
phase: 11-confluence-adapter
plan: A
type: execute
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - crates/reposix-confluence/Cargo.toml
  - crates/reposix-confluence/src/lib.rs
autonomous: true
requirements:
  - FC-01
  - FC-02
  - FC-03
  - SG-01
  - SG-05
  - SG-07
user_setup: []

must_haves:
  truths:
    - "`cargo build -p reposix-confluence` succeeds on a clean checkout"
    - "`cargo test -p reposix-confluence` runs ≥8 unit tests against wiremock, all passing"
    - "`cargo clippy -p reposix-confluence --all-targets -- -D warnings` exits 0"
    - "Every outbound HTTP call goes through `reposix_core::http::HttpClient` (SG-01)"
    - "Every `Issue` returned from public methods was produced by translating a wiremock JSON payload (never hand-rolled to fake a pass)"
    - "Writing methods (`create_issue`, `update_issue`, `delete_or_close`) return `Err(Error::Other(\"not supported: ...\"))` without emitting any HTTP"
    - "Basic-auth header is exactly `Basic base64(email:token)` with STANDARD base64 alphabet + padding"
    - "Cursor pagination follows `_links.next` relative path, prepending the tenant base URL"
  artifacts:
    - path: "crates/reposix-confluence/Cargo.toml"
      provides: "New crate manifest wired to workspace; depends on base64 + reposix-core + reqwest + tokio + serde + serde_json + async-trait + tracing + chrono + parking_lot"
    - path: "crates/reposix-confluence/src/lib.rs"
      provides: "`ConfluenceReadOnlyBackend`, `ConfluenceCreds`, `basic_auth_header`, `parse_next_cursor`, `status_from_confluence`, `translate`, wiremock unit tests"
      min_lines: 500
      contains: "#![forbid(unsafe_code)]"
    - path: "Cargo.toml"
      provides: "workspace `members` list contains `crates/reposix-confluence`; workspace `[workspace.dependencies]` contains `base64 = \"0.22\"`"
  key_links:
    - from: "crates/reposix-confluence/src/lib.rs"
      to: "reposix_core::http::client"
      via: "`client(ClientOpts::default())?` in `new_with_base_url`"
      pattern: "reposix_core::http::(client|ClientOpts|HttpClient)"
    - from: "crates/reposix-confluence/src/lib.rs"
      to: "reposix_core::backend::IssueBackend"
      via: "`#[async_trait] impl IssueBackend for ConfluenceReadOnlyBackend`"
      pattern: "impl IssueBackend for ConfluenceReadOnlyBackend"
    - from: "crates/reposix-confluence/src/lib.rs"
      to: "reposix_core::Tainted"
      via: "Ingress wrapping — planner spec: every decoded Issue passes through `Tainted::new` (SG-05). For consistency with GitHub's pattern (which returns bare `Issue`), wrap the inbound JSON-deserialized `ConfPage` in `Tainted::new` BEFORE calling `translate`, then unwrap for the public `Issue` return."
      pattern: "Tainted::new"
---

<objective>
Ship the `reposix-confluence` crate: `ConfluenceReadOnlyBackend` implementing `IssueBackend` against Atlassian Confluence Cloud REST v2. Structurally isomorphic to `reposix-github`, with four wire-shape deltas documented in 11-RESEARCH.md §Pattern Delta (cursor-in-body pagination, Basic auth, space-key resolver, `Retry-After`-driven rate gate). Write methods all return `NotSupported`. Covered by ≥8 wiremock unit tests.

Purpose: This crate is the functional heart of Phase 11. Every downstream plan (CLI dispatch, contract test, demos) depends on its types existing and compiling. It must land cleanly in Wave 1 so the rest of the phase can proceed in Wave 2.

Output: A new crate at `crates/reposix-confluence/` with `Cargo.toml` + `src/lib.rs`, plus two edits to the workspace root `Cargo.toml` (add member, add `base64` workspace dep).
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/11-confluence-adapter/11-CONTEXT.md
@.planning/phases/11-confluence-adapter/11-RESEARCH.md
@.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md
@CLAUDE.md
@crates/reposix-github/src/lib.rs
@crates/reposix-github/Cargo.toml
@crates/reposix-core/src/backend.rs
@crates/reposix-core/src/http.rs
@crates/reposix-core/src/taint.rs
@Cargo.toml

<interfaces>
<!-- Extracted from codebase; executor must use these exactly. -->

From `crates/reposix-core/src/backend.rs` (IssueBackend trait surface, already consumed by reposix-github):
```rust
#[async_trait]
pub trait IssueBackend: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn supports(&self, feature: BackendFeature) -> bool;
    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>>;
    async fn get_issue(&self, project: &str, id: IssueId) -> Result<Issue>;
    async fn create_issue(&self, project: &str, issue: Untainted<Issue>) -> Result<Issue>;
    async fn update_issue(&self, project: &str, id: IssueId, patch: Untainted<Issue>, expected_version: Option<u64>) -> Result<Issue>;
    async fn delete_or_close(&self, project: &str, id: IssueId, reason: DeleteReason) -> Result<()>;
}
pub enum BackendFeature { Workflows, Delete, Transitions, StrongVersioning, BulkEdit }
pub enum DeleteReason { Completed, NotPlanned, Duplicate }
```

From `crates/reposix-core/src/http.rs`:
```rust
pub fn client(opts: ClientOpts) -> Result<HttpClient>;
pub struct ClientOpts { /* default is fine */ }
impl HttpClient {
    pub async fn request_with_headers(&self, method: Method, url: &str, headers: &[(&str, &str)]) -> Result<reqwest::Response>;
    pub async fn get(&self, url: impl AsRef<str>) -> Result<reqwest::Response>;
}
```

From `crates/reposix-core/src/lib.rs` (re-exports):
```rust
pub use Error; // thiserror-derived; Error::Other(String) is the catch-all
pub type Result<T> = std::result::Result<T, Error>;
pub use {Issue, IssueId, IssueStatus, Untainted, Tainted};
```

From `crates/reposix-github/src/lib.rs` — STRUCTURAL TEMPLATE to copy. Keep the same:
- `Arc<HttpClient>` + `Arc<Mutex<Option<Instant>>>` rate gate
- `standard_headers() -> Vec<(&'static str, String)>` + borrow-dance into `&[(&str, &str)]`
- `ingest_rate_limit(&self, resp: &reqwest::Response)` side-effect setter on gate
- `await_rate_limit_gate(&self)` early sleep
- `translate(page: ConfPage) -> Issue` pure function
- unit tests using `wiremock::MockServer` against the default 127.0.0.1 allowlist
</interfaces>

<!-- Wire-shape authority: `.planning/phases/11-confluence-adapter/11-RESEARCH.md` §§ "Confluence REST v2 Endpoint Reference", "Pagination Contract", "Auth Contract", "Rate-limit Contract", "Status Mapping", "Pattern Delta vs reposix-github" — READ BEFORE WRITING CODE. -->
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Scaffold the crate (Cargo.toml + empty lib.rs + workspace wire-up)</name>
  <files>
    crates/reposix-confluence/Cargo.toml,
    crates/reposix-confluence/src/lib.rs,
    Cargo.toml
  </files>
  <behavior>
    - `cargo build -p reposix-confluence` succeeds (empty crate with forbid(unsafe_code)).
    - `cargo metadata | jq '.workspace_members[]' | grep reposix-confluence` returns one entry.
    - Workspace `Cargo.toml` declares `base64 = "0.22"` under `[workspace.dependencies]`.
    - `reposix-confluence/Cargo.toml` inherits `version.workspace`, `edition.workspace`, `rust-version.workspace`, `license.workspace`.
    - `[dev-dependencies]` in crate Cargo.toml mirrors reposix-github: `wiremock = "0.6"`, `tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }`, `reposix-sim = { path = "../reposix-sim" }`, `tempfile = "3"`, `rusqlite = { workspace = true }`.
  </behavior>
  <action>
    Create `crates/reposix-confluence/Cargo.toml` copying the layout of `crates/reposix-github/Cargo.toml` verbatim except:
      - `name = "reposix-confluence"`
      - `description = "Read-only Atlassian Confluence Cloud REST v2 adapter implementing the reposix-core IssueBackend trait"`
      - Add `base64 = { workspace = true }` under `[dependencies]` (after serde_json)
    Create `crates/reposix-confluence/src/lib.rs` with just:
      ```rust
      //! [`ConfluenceReadOnlyBackend`] — read-only [`IssueBackend`] for Atlassian
      //! Confluence Cloud REST v2. See 11-RESEARCH.md and ADR-002 for the
      //! page→issue mapping and wire-shape details.
      #![forbid(unsafe_code)]
      #![warn(clippy::pedantic, missing_docs)]
      #![allow(clippy::module_name_repetitions)]
      ```
    Edit workspace root `Cargo.toml`:
      - Append `"crates/reposix-confluence",` to the `members` list (after `reposix-swarm`).
      - Under `[workspace.dependencies]`, add a new line `base64 = "0.22"` grouped with Serialization deps.
    Verify `cargo build -p reposix-confluence --locked` succeeds. `cargo update -p base64` first if lockfile refuses.
  </action>
  <verify>
    <automated>cargo build -p reposix-confluence --locked &amp;&amp; cargo metadata --format-version 1 | jq -r '.workspace_members[]' | grep -q reposix-confluence</automated>
  </verify>
  <done>
    Crate compiles as an empty lib. Workspace has `reposix-confluence` as a member. `base64 = "0.22"` is a workspace dep. Commit: `feat(11-A-1): scaffold reposix-confluence crate + wire into workspace`.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement `ConfluenceReadOnlyBackend` + helpers + wiremock unit tests</name>
  <files>
    crates/reposix-confluence/src/lib.rs
  </files>
  <behavior>
    Unit tests in `lib.rs` `#[cfg(test)] mod tests` (all 8 required, all wiremock-backed unless otherwise noted, all `#[tokio::test]`):
      1. `list_resolves_space_key_and_fetches_pages`: mock sets two routes — `GET /wiki/api/v2/spaces?keys=REPOSIX` returns `{"results":[{"id":"12345","key":"REPOSIX",...}]}`; `GET /wiki/api/v2/spaces/12345/pages?limit=100` returns 1 page with 2 results and no `_links.next`. Assert `list_issues("REPOSIX").await.unwrap()` has `len == 2`, `issues[0].id == IssueId(98765)`.
      2. `list_paginates_via_links_next`: page 1 returns 2 results + `_links.next: "/wiki/api/v2/spaces/12345/pages?cursor=ABC&limit=100"` (relative path); page 2 returns 1 result, no `next`. Assert `len == 3`, all ids from both pages present. Critical: the next-page relative path must be prepended with the mock server's base URL (i.e. the tenant base in prod = `server.uri()` in tests), not with `_links.base` from the response body.
      3. `get_issue_returns_body_storage_value`: mock `GET /wiki/api/v2/pages/98765?body-format=storage` returns full page JSON with `body.storage.value = "<p>Hello</p>"`. Assert `get_issue("REPOSIX", IssueId(98765)).await.unwrap().body == "<p>Hello</p>"`.
      4. `get_404_maps_to_not_found`: mock returns 404; assert `Err(Error::Other(m))` with `m.starts_with("not found:")`.
      5. `status_current_maps_to_open`: status = `"current"` → `IssueStatus::Open`.
      6. `status_trashed_maps_to_done`: status = `"trashed"` → `IssueStatus::Done`.
      7. `auth_header_is_basic_with_correct_base64`: custom wiremock `Match` impl named `BasicAuthMatches` that verifies `authorization` header equals `format!("Basic {}", STANDARD.encode("test@example.com:tkn"))`. Construct the backend with `ConfluenceCreds { email: "test@example.com".into(), api_token: "tkn".into() }`. Mount the matcher with `.and(BasicAuthMatches)`. If the header is missing or wrong-format, the mock won't match and the test will fail with a wiremock "no matching mock" error — that's the signal.
      8. `rate_limit_429_retry_after_arms_gate`: mock returns 429 with `Retry-After: 2` header. Drive one `get_issue` call; assert that the `rate_limit_gate` has been set to `Some(deadline)` where `deadline > Instant::now()` and `deadline - Instant::now() <= MAX_RATE_LIMIT_SLEEP + 1s`. Exactly mirrors reposix-github's `rate_limit_zero_remaining_arms_the_gate` test. (The 429 itself should surface as `Err(Error::Other(...))` — the gate arming is the side effect we're asserting.)
      9. `write_methods_return_not_supported`: no wiremock. Build backend with `new_with_base_url` pointing at `"http://127.0.0.1:1"` (unreachable, but we short-circuit before HTTP). Assert all three of `create_issue`, `update_issue`, `delete_or_close` return `Err(Error::Other(m))` with `m.starts_with("not supported:")`.
      10. `supports_reports_no_features`: assert `!backend.supports(BackendFeature::Workflows)` and all other features. `name() == "confluence-readonly"`.
      Bonus (no-net pure unit):
      11. `parse_next_cursor_extracts_relative_path`: direct pure-fn test on the helper.
      12. `parse_next_cursor_absent_returns_none`: direct pure-fn test.
      13. `basic_auth_header_format`: pure-fn test: `basic_auth_header("a@b.com","xyz") == format!("Basic {}", STANDARD.encode("a@b.com:xyz"))`.
  </behavior>
  <action>
    Fill in `crates/reposix-confluence/src/lib.rs` following `crates/reposix-github/src/lib.rs` structurally (module-level doc, constants, struct, helpers, `impl IssueBackend`, tests). Specifically:

    **Constants** (names must match):
    ```rust
    const MAX_RATE_LIMIT_SLEEP: Duration = Duration::from_secs(60);
    const MAX_ISSUES_PER_LIST: usize = 500;  // 5 pages × 100
    const PAGE_SIZE: usize = 100;
    pub const DEFAULT_BASE_URL_FORMAT: &str = "https://{tenant}.atlassian.net";
    ```

    **Public types**:
    ```rust
    pub struct ConfluenceCreds { pub email: String, pub api_token: String }
    // Manual Debug impl that redacts api_token (threat-model item T-11-01):
    impl std::fmt::Debug for ConfluenceCreds {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ConfluenceCreds")
                .field("email", &self.email)
                .field("api_token", &"<redacted>")
                .finish()
        }
    }

    #[derive(Clone)]
    pub struct ConfluenceReadOnlyBackend {
        http: Arc<HttpClient>,
        creds: ConfluenceCreds,
        base_url: String,  // "https://{tenant}.atlassian.net" — no trailing slash
        rate_limit_gate: Arc<Mutex<Option<Instant>>>,
    }
    // Manual Debug on backend too — redact creds (don't auto-derive since ConfluenceCreds
    // has a manual redacting Debug, but be explicit to document intent).
    impl std::fmt::Debug for ConfluenceReadOnlyBackend { /* redact creds */ }
    ```

    **Public constructors**:
    ```rust
    pub fn new(creds: ConfluenceCreds, tenant: &str) -> Result<Self> {
        // Validate tenant to guard against SSRF via injection (threat-model T-11-02).
        // Confluence tenant subdomains match ^[a-z0-9][a-z0-9-]{0,62}$ per DNS label rules.
        // Reject anything that could construct a non-atlassian.net URL.
        if tenant.is_empty() || tenant.len() > 63
            || !tenant.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            || tenant.starts_with('-') || tenant.ends_with('-') {
            return Err(Error::Other(format!("invalid confluence tenant subdomain: {tenant:?}")));
        }
        Self::new_with_base_url(creds, format!("https://{tenant}.atlassian.net"))
    }
    pub fn new_with_base_url(creds: ConfluenceCreds, base_url: String) -> Result<Self> { ... }
    ```

    **Standard headers**:
    ```rust
    fn standard_headers(&self) -> Vec<(&'static str, String)> {
        vec![
            ("Accept", "application/json".to_owned()),
            ("User-Agent", "reposix-confluence-readonly/0.3".to_owned()),
            ("Authorization", basic_auth_header(&self.creds.email, &self.creds.api_token)),
        ]
    }

    fn basic_auth_header(email: &str, token: &str) -> String {
        use base64::Engine;
        let raw = format!("{email}:{token}");
        format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(raw.as_bytes()))
    }
    ```

    **Pagination parser**:
    ```rust
    fn parse_next_cursor(body: &serde_json::Value) -> Option<String> {
        body.get("_links")
            .and_then(|l| l.get("next"))
            .and_then(|n| n.as_str())
            .map(str::to_owned)
    }
    ```

    **Deserialization structs** (serde, no `deny_unknown_fields` — forward-compat):
    ```rust
    #[derive(Debug, Deserialize)] struct ConfSpaceList { results: Vec<ConfSpace> }
    #[derive(Debug, Deserialize)] struct ConfSpace { id: String }
    #[derive(Debug, Deserialize)] struct ConfPageList {
        results: Vec<ConfPage>,
        #[serde(default)] #[serde(rename = "_links")] links: Option<ConfLinks>,
    }
    #[derive(Debug, Deserialize)] struct ConfLinks { #[serde(default)] next: Option<String> }
    #[derive(Debug, Deserialize)] struct ConfPage {
        id: String,
        status: String,
        title: String,
        #[serde(rename = "createdAt")] created_at: chrono::DateTime<chrono::Utc>,
        version: ConfVersion,
        #[serde(default, rename = "ownerId")] owner_id: Option<String>,
        #[serde(default)] body: Option<ConfPageBody>,
    }
    #[derive(Debug, Deserialize)] struct ConfVersion {
        number: u64,
        #[serde(rename = "createdAt")] created_at: chrono::DateTime<chrono::Utc>,
    }
    #[derive(Debug, Deserialize)] struct ConfPageBody { #[serde(default)] storage: Option<ConfBodyStorage> }
    #[derive(Debug, Deserialize)] struct ConfBodyStorage { value: String }
    ```

    **Status mapping** (pure fn):
    ```rust
    fn status_from_confluence(s: &str) -> IssueStatus {
        match s {
            "current" | "draft" => IssueStatus::Open,
            "archived" | "trashed" | "deleted" => IssueStatus::Done,
            _ => IssueStatus::Open, // pessimistic forward-compat, consistent with CONTEXT §status
        }
    }
    ```

    **Translate** (pure fn):
    ```rust
    fn translate(page: ConfPage) -> Result<Issue> {
        let id = page.id.parse::<u64>()
            .map_err(|_| Error::Other(format!("confluence page id not a u64: {:?}", page.id)))?;
        Ok(Issue {
            id: IssueId(id),
            title: page.title,
            status: status_from_confluence(&page.status),
            assignee: page.owner_id,
            labels: vec![],  // deferred per CONTEXT
            created_at: page.created_at,
            updated_at: page.version.created_at,
            version: page.version.number,
            body: page.body.and_then(|b| b.storage).map(|s| s.value).unwrap_or_default(),
        })
    }
    ```

    **Space resolver** (async method on the backend):
    ```rust
    async fn resolve_space_id(&self, space_key: &str) -> Result<String> {
        let url = format!("{}/wiki/api/v2/spaces?keys={}", self.base(), space_key);
        // ... standard_headers → request_with_headers → status check → deserialize as ConfSpaceList
        // If results.is_empty() → Err(Error::Other(format!("not found: space key {space_key}")))
        // Else Ok(results[0].id)
    }
    ```

    **`list_issues`** — resolve space id → paginate through `/wiki/api/v2/spaces/{id}/pages?limit=100`, following `_links.next` cursor. Follow the reposix-github `list_issues` loop structure verbatim, substituting `parse_next_cursor(&body_json)` for `parse_next_link(&link_header)`. When `_links.next` is a relative path (always is), prepend `self.base()` to get the absolute URL for the next request.

    **`get_issue`** — `GET {base}/wiki/api/v2/pages/{id}?body-format=storage`. Same response-handling shape as reposix-github. Deserialize body bytes directly as `ConfPage` (not `ConfPageList`).

    **`ingest_rate_limit`** — per 11-RESEARCH.md §Rate-limit Contract. Read `retry-after` header (lowercase) as u64 seconds. On 429 OR `x-ratelimit-remaining: 0`, arm the gate.

    **Write methods** — return `Err(Error::Other("not supported: create_issue — reposix-confluence is read-only in v0.3".into()))` etc. No HTTP.

    **`supports`** — always false (even Workflows; Confluence has no in-flight labels). `name() = "confluence-readonly"`.

    **Tests** — write all 13 tests listed above. For the custom `BasicAuthMatches` impl, follow 11-RESEARCH.md §"Custom Match impl for auth header test" exactly.

    Run `cargo test -p reposix-confluence` and `cargo clippy -p reposix-confluence --all-targets -- -D warnings` locally until both are green.
  </action>
  <verify>
    <automated>cargo test -p reposix-confluence --locked &amp;&amp; cargo clippy -p reposix-confluence --all-targets --locked -- -D warnings &amp;&amp; [ "$(cargo test -p reposix-confluence --locked 2>&amp;1 | grep -oP 'test result: ok\. \K[0-9]+(?= passed)' | head -1)" -ge 10 ]</automated>
  </verify>
  <done>
    All listed unit tests pass. Clippy clean at `-D warnings`. `ConfluenceReadOnlyBackend`, `ConfluenceCreds`, `basic_auth_header`, `parse_next_cursor`, `status_from_confluence`, `translate` are all present and behave per spec. Commit: `feat(11-A-2): ConfluenceReadOnlyBackend + wiremock unit tests`.
  </done>
</task>

<task type="auto">
  <name>Task 3: Workspace-wide green check</name>
  <files>
    (no file edits; validation only)
  </files>
  <action>
    Run the full workspace quality gate to confirm 11-A didn't break anything else:
    ```bash
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    ```
    If `cargo fmt` reports diffs, run `cargo fmt --all` and amend the commit from Task 2.
    Expected test count: previous green-run total + 13 new unit tests (minimum) from reposix-confluence. The phase success criterion is ≥180 workspace tests; confirm we meet it. Record the number in the commit message if amending.
  </action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo clippy --workspace --all-targets --locked -- -D warnings &amp;&amp; cargo test --workspace --locked</automated>
  </verify>
  <done>
    Full workspace builds, tests, and lints cleanly. `cargo test --workspace --locked` reports ≥180 tests, 0 failed. No new commit unless `cargo fmt` needed fixing.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Confluence tenant → adapter | Every response byte is attacker-influenced content (seeded by an agent or third party in the Confluence space). |
| adapter → `reposix-core::HttpClient` | SG-01 allowlist check happens here; every outbound URL must be re-validated. |
| `ConfluenceCreds` → logs / error messages | API token is an authenticator; must never appear in Debug output, tracing spans, or error strings. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-11-01 | Information disclosure | `ConfluenceCreds` Debug/Display | mitigate | Manual `impl Debug for ConfluenceCreds` that prints `api_token: "<redacted>"` (Task 2, action section). NO `#[derive(Debug)]`. Same for the backend struct. |
| T-11-02 | Tampering / SSRF | `ConfluenceReadOnlyBackend::new` tenant arg | mitigate | Validate tenant against `^[a-z0-9][a-z0-9-]{0,62}$` (DNS label rules) before URL construction (Task 2, `new` fn). Rejects injection like `a.evil.com` or `../../../` or `a@b`. |
| T-11-03 | Tampering | Response body → `Issue.body` (tainted HTML) | accept + note | SG-05: wrap inbound through `Tainted::new` per CONTEXT decision. The HTML is raw storage-format and MAY contain `<script>` or `<a href=javascript:>`. v0.3 is read-only + text surface (cat'd in terminal), so XSS is out of surface. Document in ADR-002 that any future web renderer MUST sanitize. |
| T-11-04 | Elevation of privilege | `_links.next` pointing at attacker-controlled origin | mitigate | The follow-up request always goes through `self.http.request_with_headers(...)`, which re-checks `REPOSIX_ALLOWED_ORIGINS` per call. An attacker-crafted Confluence page that returns `_links.next: "https://evil.example/steal"` will fail the allowlist check and surface as `Err`. This is enforced by `HttpClient` (Phase 1 SG-01), not by this crate; the mitigation is simply "don't bypass it", which Task 2 achieves by construction. |
| T-11-05 | Repudiation | outbound HTTP without audit | accept (v0.3) | Read-only adapter does not write to the audit log. Audit coverage for read paths is a cross-cutting v0.4 concern (applies to reposix-github too). Not regressing an existing behavior. |

**Block-on-high:** T-11-01 and T-11-02 are mitigated in-task and MUST be verified in Task 2's unit tests (`ConfluenceReadOnlyBackend::new("../evil")` returns Err; `format!("{:?}", creds)` contains "redacted"). Add these two tests to the required list if not already present.
</threat_model>

<verification>
Nyquist coverage:
- **Unit (wiremock):** 10+ tests covering both read methods, both error paths (404 / 429), status mapping (2 branches), auth header format, cursor pagination, write-path rejection, capability matrix, tenant validation, Debug redaction.
- **Pure-fn:** `parse_next_cursor` (present/absent), `basic_auth_header` (byte-exact).
- **Workspace-wide:** `cargo test --workspace --locked` green; no pre-existing test regressed.
- **Clippy:** `cargo clippy --workspace --all-targets -- -D warnings` green (no new allows added silently).
- **Fmt:** `cargo fmt --all --check` clean.

Deferred to downstream plans:
- Trait contract (11-C): Same five invariants pass against wiremock-backed ConfluenceReadOnlyBackend.
- Live wire test (11-C live half): `#[ignore]`-gated, runs under `integration-contract-confluence` CI job (11-B).
- Demo-level validation (11-D): Tier 3B + Tier 5 exercise the built binary through real CLI surface.
</verification>

<success_criteria>
Each a Bash assertion runnable from repo root:

1. `test -f crates/reposix-confluence/Cargo.toml && test -f crates/reposix-confluence/src/lib.rs` returns 0.
2. `grep -q '"crates/reposix-confluence",' Cargo.toml` returns 0.
3. `grep -q '^base64 = "0.22"' Cargo.toml` returns 0.
4. `grep -q '^#!\[forbid(unsafe_code)\]' crates/reposix-confluence/src/lib.rs` returns 0.
5. `grep -q '^#!\[warn(clippy::pedantic, missing_docs)\]' crates/reposix-confluence/src/lib.rs` returns 0.
6. `cargo build -p reposix-confluence --locked` exits 0.
7. `cargo test -p reposix-confluence --locked 2>&1 | grep -E 'test result: ok\. [0-9]+ passed' | head -1 | grep -oE '[0-9]+' | head -1` returns an integer ≥ 10.
8. `cargo clippy -p reposix-confluence --all-targets --locked -- -D warnings` exits 0.
9. `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0 (no regressions elsewhere).
10. `cargo test --workspace --locked 2>&1 | grep -oE 'test result: ok\. [0-9]+ passed' | awk '{sum += $4} END {print sum}'` ≥ 180.
11. `grep -q 'redacted' crates/reposix-confluence/src/lib.rs` returns 0 (Debug-redact contract).
12. `grep -qE 'fn translate\(.*ConfPage.*\).*Result<Issue>' crates/reposix-confluence/src/lib.rs` returns 0.
13. `grep -qE 'fn parse_next_cursor' crates/reposix-confluence/src/lib.rs` returns 0.
14. `grep -qE 'fn basic_auth_header' crates/reposix-confluence/src/lib.rs` returns 0.
15. `grep -qE 'IssueStatus::Done' crates/reposix-confluence/src/lib.rs && grep -qE 'IssueStatus::Open' crates/reposix-confluence/src/lib.rs` returns 0.
</success_criteria>

<rollback_plan>
If Task 2's wiremock tests prove flakier than 3 re-runs:
1. Preserve the failing test name in `git stash` for the next agent.
2. Revert the commit; keep Task 1's scaffolding commit.
3. Re-open plan with a narrower Task 2 scope: mandatory tests only (1, 3, 4, 5, 9, 10, 13), bonus tests deferred.
4. The crate still ships structurally — just with a thinner test floor.

If `base64 = "0.22"` does not resolve at lockfile-update time (e.g. yanked):
1. Run `cargo search base64` to find the current version.
2. Update workspace `Cargo.toml` + crate dep accordingly.
3. Document the version in the commit message.

If Cargo.lock drift kills CI despite local green:
1. Run `cargo update -p reposix-confluence -p base64`.
2. Amend the commit with the updated Cargo.lock.
3. Do NOT `--no-verify`; fix the hook failure instead (per CLAUDE.md § git safety).
</rollback_plan>

<output>
After completion, create `.planning/phases/11-confluence-adapter/11-A-SUMMARY.md` documenting:
- Final test count (`cargo test -p reposix-confluence` number)
- Any Claude's-discretion choices made (e.g. module layout if split from flat lib.rs)
- Any open questions surfaced that need ADR-002 coverage
- Confirmation that T-11-01 and T-11-02 have explicit tests
</output>
