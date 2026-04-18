# Phase 28 — Pattern Map

> Analog files and code excerpts for the gsd-planner.
> Generated: 2026-04-16

## Files to Create / Modify

### New Files (Phase 28)

| File | Role | Analog |
|------|------|--------|
| `crates/reposix-jira/Cargo.toml` | Crate manifest | `crates/reposix-confluence/Cargo.toml` |
| `crates/reposix-jira/src/lib.rs` | `JiraBackend` + `BackendConnector` impl | `crates/reposix-confluence/src/lib.rs` |
| `crates/reposix-jira/src/adf.rs` | `adf_to_plain_text` walker | `crates/reposix-confluence/src/adf.rs` |
| `crates/reposix-jira/tests/contract.rs` | Contract test (5 invariants) | `crates/reposix-confluence/tests/contract.rs` |
| `docs/reference/jira.md` | User guide + env vars | `docs/reference/confluence.md` (if exists) |
| `docs/decisions/005-jira-issue-mapping.md` | ADR-005 | `docs/decisions/004-*.md` |

### Modified Files (Phase 28)

| File | Change | Analog |
|------|--------|--------|
| `Cargo.toml` (workspace) | Add `crates/reposix-jira` to `members` | Existing workspace Cargo.toml members list |
| `crates/reposix-cli/src/list.rs` | Add `ListBackend::Jira` variant + `read_jira_env_from` | Existing `ListBackend::Confluence` + `read_confluence_env_from` |

---

## Key Code Patterns

### 1. Crate manifest (Cargo.toml)

Exact analog: `crates/reposix-confluence/Cargo.toml`. Differences:
- Remove `pulldown-cmark` (ADF→plain text doesn't need a markdown renderer)
- Change description to JIRA
- No `serde_yaml` needed in deps (only in dev)

```toml
[package]
name = "reposix-jira"
version.workspace = true
edition.workspace = true
# ... (same workspace = true pattern)

[dependencies]
reposix-core.workspace = true
reqwest = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
base64 = { workspace = true }
async-trait = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }
parking_lot = { workspace = true }
rusqlite = { workspace = true }
sha2 = { workspace = true }
url = { workspace = true }

[dev-dependencies]
wiremock = "0.6"
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
reposix-sim = { path = "../reposix-sim" }
tempfile = "3"
```

### 2. Crate header (#![...] attributes)

Exact match: `crates/reposix-confluence/src/lib.rs` lines 65-67

```rust
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]
```

### 3. Creds type with Debug redaction

Exact analog: `ConfluenceCreds` in `crates/reposix-confluence/src/lib.rs` lines 115-130

```rust
#[derive(Clone)]
pub struct JiraCreds {
    pub email: String,
    pub api_token: String,
}

impl std::fmt::Debug for JiraCreds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JiraCreds")
            .field("email", &self.email)
            .field("api_token", &"<redacted>")
            .finish()
    }
}
```

### 4. Backend struct + Debug redaction

Exact analog: `ConfluenceBackend` in `crates/reposix-confluence/src/lib.rs` lines 148-191

```rust
#[derive(Clone)]
pub struct JiraBackend {
    http: Arc<HttpClient>,
    creds: JiraCreds,
    base_url: String,
    rate_limit_gate: Arc<Mutex<Option<Instant>>>,
    audit: Option<Arc<Mutex<Connection>>>,
}

impl std::fmt::Debug for JiraBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JiraBackend")
            .field("base_url", &self.base_url)
            .field("creds", &self.creds)
            .field("rate_limit_gate", &"<gate>")
            .field("audit", if self.audit.is_some() { &"<present>" } else { &"<none>" })
            .finish_non_exhaustive()
    }
}
```

### 5. Tenant validation

Exact analog: `ConfluenceBackend::new` validates tenant. The regex is identical for JIRA (both are `*.atlassian.net` subdomains):

```rust
fn validate_tenant(tenant: &str) -> Result<()> {
    // Non-empty, 1..=63 chars, [a-z0-9][a-z0-9-]*, no trailing hyphen
    // Error: Error::Other(format!("invalid jira tenant subdomain: {:?} (...)", tenant))
}
```

### 6. HTTP client construction

Exact analog: all backends. MUST use `reposix_core::http::client()`, NEVER `reqwest::Client::new()`:

```rust
use reposix_core::http::{client, ClientOpts, HttpClient};
let http = client(ClientOpts::default())?;
```

### 7. Audit write pattern (read variant for Phase 28)

Analog: `ConfluenceBackend::audit_write` in `crates/reposix-confluence/src/lib.rs` lines 1079-1114

For JIRA Phase 28: audit covers READ operations (list + get), not just writes. Adapt the method name to `audit_event`:

```rust
fn audit_event(&self, method: &'static str, path: &str, status: u16, summary: &str, response_bytes: &[u8]) {
    // Same SHA-256 prefix pattern, same INSERT INTO audit_events schema
    // agent_id: format!("reposix-jira-{}", std::process::id())
}
```

### 8. Basic auth header

Exact analog: `basic_auth_header` in `crates/reposix-confluence/src/lib.rs` lines 542-556

```rust
pub fn basic_auth_header(email: &str, token: &str) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    format!("Basic {}", STANDARD.encode(format!("{email}:{token}")))
}
```

### 9. BackendConnector impl for read-only backend

Analog: `impl BackendConnector for ConfluenceBackend` lines 1476+

```rust
impl BackendConnector for JiraBackend {
    fn name(&self) -> &'static str { "jira" }

    fn supports(&self, feature: BackendFeature) -> bool {
        matches!(feature, BackendFeature::Hierarchy)  // true; Delete/Transitions/StrongVersioning false
    }

    fn root_collection_name(&self) -> &'static str { "issues" }

    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>> { ... }
    async fn get_issue(&self, _project: &str, id: IssueId) -> Result<Issue> { ... }

    // Read-only stubs:
    async fn create_issue(&self, _project: &str, _issue: Untainted<Issue>) -> Result<Issue> {
        Err(Error::Other("not supported: read-only backend — see Phase 29".into()))
    }
    async fn update_issue(...) -> Result<Issue> { Err(Error::Other("not supported: ...".into())) }
    async fn delete_or_close(...) -> Result<()> { Err(Error::Other("not supported: ...".into())) }
}
```

### 10. Rate limit gate pattern

Analog: `ConfluenceBackend` rate_limit_gate field + usage pattern:

```rust
// Field: rate_limit_gate: Arc<Mutex<Option<Instant>>>
// On 429:
//   check Retry-After header → sleep(secs)
//   no header → exponential backoff: 1s, 2s, 4s, 8s (max 4 attempts, jitter)
// Max wait: 60s (MAX_RATE_LIMIT_SLEEP from Confluence pattern)
```

### 11. ListBackend extension in reposix-cli

Exact analog: existing `ListBackend` enum in `crates/reposix-cli/src/list.rs`

```rust
pub enum ListBackend {
    Sim,
    Github,
    Confluence,
    Jira,  // NEW
}
```

`read_jira_env_from` follows the exact same pure-fn pattern as `read_confluence_env_from`:
- Reads `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE`
- Collects ALL missing into one error (never separate errors)
- Does NOT echo values in errors

### 12. Contract test structure

Analog: `crates/reposix-confluence/tests/contract.rs` — 5 invariants, 3 arms:
- `contract_sim` (always)
- `contract_jira_wiremock` (always, wiremock)
- `contract_jira_live` (`#[ignore]`, `skip_if_no_env!("JIRA_EMAIL", "JIRA_API_TOKEN", "REPOSIX_JIRA_INSTANCE")`)

---

## Constants Pattern

From Confluence: `MAX_ISSUES_PER_LIST = 500`, `PAGE_SIZE = 100`. JIRA uses same defaults.

---

## ADF Walker

`crates/reposix-confluence/src/adf.rs` has `adf_to_markdown`. Phase 28 needs `adf_to_plain_text` — simpler (no heading/list/bold rendering, just extract text nodes). Share the recursive tree-walking structure but emit plain text instead of Markdown.
