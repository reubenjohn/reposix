---
title: Write your own connector
---

# Write your own connector

reposix talks to four backends out of the box: the in-process [simulator](../reference/simulator.md), GitHub, Confluence, and JIRA. They all share one trait — [`BackendConnector`](../reference/glossary.md#backendconnector) (the Rust trait every adapter implements; see `crates/reposix-core/src/backend.rs`) — and adding a fifth backend is a matter of implementing that trait and dropping a crate into the workspace. This guide walks the trait method-by-method, then sketches a Linear connector as a worked example.

The three reference implementations live in `crates/reposix-{github,confluence,jira}/`. Cite them by file path, not by copy-paste — they are the source of truth for every pattern below.

## Anatomy of a `BackendConnector`

The trait lives in `crates/reposix-core/src/backend.rs`. Every method:

- `fn name(&self) -> &'static str` — stable backend tag (`"github"`, `"sim"`). Used in audit rows and log lines.
- `fn supports(&self, feature: BackendFeature) -> bool` — capability query. Cheap, synchronous, no network. Lets callers branch on `Delete`, `StrongVersioning`, `Hierarchy`, etc. without trying the operation.
- `async fn list_records(&self, project: &str) -> Result<Vec<Record>>` — full project listing. Empty project returns an empty vec, NOT an error.
- `async fn list_changed_since(&self, project: &str, since: DateTime<Utc>) -> Result<Vec<RecordId>>` — incremental query for delta sync. Default impl filters `list_records` in memory; backends with native incremental queries (`?since=` for GitHub, JQL `updated >=` for JIRA, CQL `lastModified >` for Confluence, `?since=` for the sim) MUST override.
- `async fn get_record(&self, project, id) -> Result<Record>` — single fetch. Unknown id returns `Err(Error::Other("not found: ..."))`.
- `async fn create_record(&self, project, issue: Untainted<Record>) -> Result<Record>` — POST. The `Untainted` wrapper (the safe half of the [`Tainted<T>`](../reference/glossary.md#taintedt) newtype pair; see `crates/reposix-core/src/tainted.rs`) proves you stripped server-controlled fields (`id`, `created_at`, `version`).
- `async fn update_record(&self, project, id, patch: Untainted<Record>, expected_version) -> Result<Record>` — PATCH/PUT with optional optimistic-concurrency token.
- `async fn delete_or_close(&self, project, id, reason: DeleteReason) -> Result<()>` — real DELETE on backends with `BackendFeature::Delete`; close-with-reason on the rest.
- `fn root_collection_name(&self) -> &'static str` — defaults to `"issues"`. Override for backends with a domain term (Confluence overrides to `"pages"`).

Read-only adapters return `Err(Error::Other("not supported: ..."))` from the three write methods and `false` from every `supports(...)` query.

## Walkthrough — stub a Linear connector

The Linear API is REST, paginated, with a Bearer token. The shape is close enough to GitHub that we can lift the GitHub adapter's skeleton and rename. Five steps.

### Step 1 — Cargo skeleton

```bash
cargo new --lib crates/reposix-linear
```

Add it to the workspace `Cargo.toml`:

```toml
[workspace]
members = ["crates/reposix-linear", ...]
```

Lift the dependency block from `crates/reposix-github/Cargo.toml` (the closest cousin). Minimum:

```toml
[dependencies]
reposix-core = { path = "../reposix-core" }
async-trait = { workspace = true }
chrono      = { workspace = true }
parking_lot = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
tokio       = { workspace = true }
tracing     = { workspace = true }

[dev-dependencies]
wiremock = "0.6"
tokio    = { workspace = true, features = ["macros", "rt-multi-thread"] }
```

At the top of `src/lib.rs`:

```rust
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs)]
```

Both rules are non-negotiable per project conventions.

### Step 2 — Minimum viable `BackendConnector`

You only need three methods to get past the read-side smoke test: `list_records`, `get_record`, `list_changed_since`. The four write methods can `Err(...)` "not supported" until you wire them.

```rust
use async_trait::async_trait;
use reposix_core::backend::{BackendConnector, BackendFeature, DeleteReason};
use reposix_core::http::{client, ClientOpts, HttpClient};
use reposix_core::{Error, Record, RecordId, Result, Tainted, Untainted};

pub struct LinearBackend {
    http: std::sync::Arc<HttpClient>,
    creds: LinearCreds,
    base_url: String,
}

#[async_trait]
impl BackendConnector for LinearBackend {
    fn name(&self) -> &'static str { "linear" }
    fn supports(&self, _f: BackendFeature) -> bool { false }

    async fn list_records(&self, project: &str) -> Result<Vec<Record>> {
        let url = format!("{}/issues?team={project}", self.base_url);
        let raw: Vec<LinearWireIssue> =
            self.http.get(&url, /* headers */).await?.json().await?;
        let tainted = Tainted::new(raw);
        Ok(translate(tainted))
    }

    async fn get_record(&self, project: &str, id: RecordId) -> Result<Record> {
        // 404 → Err(Error::Other(format!("not found: {project}/{id}")))
        // ...
    }

    async fn create_record(&self, _: &str, _: Untainted<Record>) -> Result<Record> {
        Err(Error::Other("not supported: linear write path TODO".into()))
    }
    async fn update_record(
        &self, _: &str, _: RecordId, _: Untainted<Record>, _: Option<u64>,
    ) -> Result<Record> {
        Err(Error::Other("not supported: linear write path TODO".into()))
    }
    async fn delete_or_close(&self, _: &str, _: RecordId, _: DeleteReason) -> Result<()> {
        Err(Error::Other("not supported: linear write path TODO".into()))
    }
}
```

`reposix-github/src/lib.rs` is the canonical fully-fledged version of this shape (852 lines); `reposix-jira/src/lib.rs` is the JQL-paginated counterpart. Lift whichever is closer to your wire shape.

### Step 3 — Audit log requirements

Every network operation a connector performs is recorded in `audit_events_cache` (the helper-side audit table). The connector itself does not write rows directly — `reposix-cache` does it on the connector's behalf when the cache materializes a blob, applies a push, or runs a delta-sync. What you have to keep clean:

- Always go through `reposix_core::http::client(ClientOpts::default())?`. The audit middleware hooks the client, not the call site.
- Surface backend errors as `Error::Other("not found: ...")` / `Error::Other("not supported: ...")` so the helper logs the right `op` (`helper_fetch_error` vs `helper_push_rejected_conflict`).

The full ops vocabulary lives in [trust model §audit log](../how-it-works/trust-model.md#audit-log). Your connector inherits that vocabulary for free as long as it speaks through the standard client.

### Step 4 — Egress allowlist

```rust
use reposix_core::http::{client, ClientOpts};

let http = client(ClientOpts::default())?;     // honors REPOSIX_ALLOWED_ORIGINS
```

Do **NOT** call `reqwest::Client::new()` or `Client::builder()`. The workspace has a clippy `disallowed-methods` rule that rejects both at compile time. The reason: the egress allowlist is the single choke-point that prevents an attacker-influenced URL from smuggling private data to a non-allowlisted origin (see [trust model](../how-it-works/trust-model.md#concentric-rings-taint-in-audited-bytes-out)). A direct `reqwest::Client` bypasses the check.

If your backend needs a custom timeout, retry, or connection pool, pass options through `ClientOpts` rather than constructing a client by hand. `crates/reposix-confluence/src/rate_limit.rs` is the canonical example of layering rate-limit logic on top of the standard client.

### Step 5 — Tests

Both reference connectors ship ≥ 5 tests against `wiremock::MockServer`. Minimum coverage for a new connector:

1. `list_records` returns ≥ 1 issue on a happy path. Seed the mock, assert length + first row.
2. `get_record` 404 → `Error::Other` whose message starts with `"not found: "`.
3. The auth header is byte-exact (Bearer prefix, Basic + base64 — whatever the backend wants).
4. Pagination cursor is followed correctly. Seed two pages, assert the second request URL is what the first response said.
5. Rate-limit gate arms on 429 / `Retry-After`.

`crates/reposix-github/src/lib.rs` `#[cfg(test)] mod tests` shows all five against GitHub's wire shape. Lift the structure; rename the matchers.

Additionally, every connector publishes a contract test in `tests/contract.rs` that runs a fixed invariant set against both `SimBackend` (control) and the connector via wiremock. `reposix-github/tests/contract.rs` and `reposix-confluence/tests/contract.rs` are the templates.

## Closing — the bar to land in the tree

Submit a PR. The bar is:

- The contract test passes against your connector via wiremock.
- The clippy `disallowed-methods` lint stays green (no direct `reqwest::Client::new`).
- A real-backend smoke fixture lands behind `#[ignore]` so a credentialed run can validate the wire shape end-to-end. See [testing targets](../reference/testing-targets.md) for the env-var conventions.
- Your README spells out cleanup if your backend is mutable in tests (the project owner does not want stale junk on their account).

## See also

- `crates/reposix-core/src/backend.rs` — `BackendConnector`, `BackendFeature`, `DeleteReason` source of truth.
- `crates/reposix-github/src/lib.rs`, `crates/reposix-confluence/src/lib.rs`, `crates/reposix-jira/src/lib.rs` — three worked examples.
- [Trust model](../how-it-works/trust-model.md) — the taint typing rules every connector inherits.
- [Integrate with your agent](integrate-with-your-agent.md) — how the connector is consumed once it is in the tree.
