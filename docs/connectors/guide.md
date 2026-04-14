# Building your own connector

## Why this document

reposix-core exposes [`IssueBackend`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-core/src/backend.rs)
as a public trait. Any crate that implements it can slot into a reposix fork's
`reposix list` / `reposix mount` / `reposix-fuse` dispatch with a handful of
lines of glue. Until the subprocess/JSON-RPC connector ABI lands in Phase 12
(see [ROADMAP.md §Phase 12](https://github.com/reubenjohn/reposix/blob/main/.planning/ROADMAP.md)
and the forward-look paragraph at the bottom of this document), publishing
`reposix-adapter-<name>` on crates.io and wiring it via a fork is the fastest
supported path for third-party backends. `reposix-github` (v0.2) and
`reposix-confluence` (v0.3) are the two in-tree worked examples — this guide
walks a third-party author through the same pattern.

## The trait contract

The seam is a single trait in
[`crates/reposix-core/src/backend.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-core/src/backend.rs).
The file itself is the source of truth and carries full rustdoc; this section
is a summary.

### Methods (five)

| Method                                                                                       | Purpose                                                                                                   | Read-only behaviour                                                      |
| -------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `fn name(&self) -> &'static str`                                                             | Stable human-readable backend name (e.g. `"github"`, `"confluence"`). Used in log lines + parity demos.  | Always implement.                                                        |
| `fn supports(&self, feature: BackendFeature) -> bool`                                        | Capability query. Cheap, synchronous, no network.                                                         | Return `false` for every variant in read-only backends.                  |
| `async fn list_issues(&self, project: &str) -> Result<Vec<Issue>>`                           | List all issues in `project`. Empty vec on empty-but-valid project — NOT an error.                        | Required.                                                                |
| `async fn get_issue(&self, project: &str, id: IssueId) -> Result<Issue>`                     | Fetch one issue. 404 → `Err(Error::Other("not found: ..."))`.                                            | Required. `u64::MAX` and similar sentinels still map to "not found".     |
| `async fn create_issue` / `update_issue` / `delete_or_close`                                 | Write path.                                                                                               | Read-only backends return `Err(Error::Other("not supported: ..."))`.     |

### `BackendFeature`

Closed enum, new variants = API break. Current members:

- `Delete` — real `DELETE`, not close-to-delete remap.
- `Transitions` — honors `DeleteReason` variants (e.g. GitHub's `completed` /
  `not_planned`).
- `StrongVersioning` — optimistic concurrency via version / etag
  (sim: `If-Match`; GitHub v0.2: `If-Unmodified-Since`).
- `BulkEdit` — single-request bulk edits.
- `Workflows` — named transitions beyond the 5-valued `IssueStatus`.

Read-only adapters hard-code `supports(_) => false`. A write-capable backend
advertises `Transitions` if it honours `DeleteReason`, `StrongVersioning` if
it has a version/etag mechanism, and so on.

### `DeleteReason`

`Completed` / `NotPlanned` / `Duplicate` / `Abandoned`. Backends that support
real delete may ignore the reason; close-with-reason backends (GitHub) map
variants onto their wire shape.

### Error model

- Not-found surfaces as `Err(Error::Other("not found: ..."))`.
- Not-supported surfaces as `Err(Error::Other("not supported: ..."))`.
- Transport errors propagate as `Error::Http` / `Error::Json` / etc.
- Typed `NotFound` / `NotSupported` enum variants are a future cleanup; for
  now the `Error::Other(msg)` shape lets callers add variants without
  breaking downstream matchers.

Do NOT read the above and copy it into your adapter's docs — link to
`crates/reposix-core/src/backend.rs` as the single source of truth.

## Step-by-step: writing your own adapter

Use `reposix-github` and `reposix-confluence` as twin worked examples
throughout — they were built one after the other, so the diffs between them
show which parts are per-backend and which parts are shared pattern.

### Step 1. Cargo skeleton

```bash
cargo new --lib reposix-adapter-foo
cd reposix-adapter-foo
```

Copy the dependency list from
[`reposix-confluence/Cargo.toml`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-confluence/Cargo.toml)
as a starting point. The minimum set is:

```toml
[dependencies]
reposix-core = "..."               # match the version of reposix you're targeting
reqwest      = { workspace = true } # only for types; construction goes through reposix-core
tokio        = { workspace = true }
serde        = { workspace = true }
serde_json   = { workspace = true }
async-trait  = { workspace = true }
tracing      = { workspace = true }
thiserror    = { workspace = true }
chrono       = { workspace = true }
parking_lot  = { workspace = true }

[dev-dependencies]
wiremock = "0.6"
tokio    = { workspace = true, features = ["macros", "rt-multi-thread"] }
```

At the crate root of `src/lib.rs`:

```rust
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs)]
```

Both in-tree adapters do this. Blanket-allowing `clippy::pedantic` is not
acceptable — allow-list specific lints with a one-line rationale.

### Step 2. Implement `IssueBackend`

Structural reference:
[`reposix-github/src/lib.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-github/src/lib.rs)
(852 lines) and
[`reposix-confluence/src/lib.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-confluence/src/lib.rs)
(1106 lines). The shape each adapter converges on:

```rust
use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use parking_lot::Mutex;

use reposix_core::backend::{BackendFeature, DeleteReason, IssueBackend};
use reposix_core::http::{client, ClientOpts, HttpClient};
use reposix_core::{Error, Issue, IssueId, Result, Tainted, Untainted};

#[derive(Clone)]
pub struct FooBackend {
    http: Arc<HttpClient>,
    creds: FooCreds,
    base_url: String,
    rate_limit_gate: Arc<Mutex<Option<Instant>>>,
}

#[async_trait]
impl IssueBackend for FooBackend {
    fn name(&self) -> &'static str { "foo" }
    fn supports(&self, _feature: BackendFeature) -> bool { false }

    async fn list_issues(&self, project: &str) -> Result<Vec<Issue>> {
        // 1. await_rate_limit_gate()
        // 2. http.request_with_headers(Method::GET, url, headers)
        // 3. ingest_rate_limit(&resp)
        // 4. deserialize -> Tainted::new(raw) -> translate -> Issue
        // ...
    }

    async fn get_issue(&self, project: &str, id: IssueId) -> Result<Issue> {
        // ... 404 -> Err(Error::Other(format!("not found: {project}/{id}"))) ...
    }

    async fn create_issue(&self, _project: &str, _issue: Untainted<Issue>) -> Result<Issue> {
        Err(Error::Other("not supported: foo is read-only".into()))
    }
    async fn update_issue(&self, _project: &str, _id: IssueId, _patch: Untainted<Issue>, _expected: Option<u64>) -> Result<Issue> {
        Err(Error::Other("not supported: foo is read-only".into()))
    }
    async fn delete_or_close(&self, _project: &str, _id: IssueId, _reason: DeleteReason) -> Result<()> {
        Err(Error::Other("not supported: foo is read-only".into()))
    }
}
```

The constructor pattern used by both in-tree adapters:

- `pub fn new(creds: FooCreds, ...) -> Result<Self>` for production (validates
  its inputs, calls `new_with_base_url` with the production URL).
- `pub fn new_with_base_url(creds: FooCreds, base_url: String) -> Result<Self>`
  for tests (wiremock points at its own `MockServer::uri()`).

Both `new` paths call `reposix_core::http::client(ClientOpts::default())?` to
get an `HttpClient`. **Never construct a `reqwest::Client` directly** — see
Security rule 1 below.

### Step 3. Wiremock tests

Both in-tree adapters ship ≥5 unit tests against
[`wiremock::MockServer`](https://docs.rs/wiremock/latest/wiremock/). Minimum
coverage for a new adapter:

1. **List returns ≥1 issue on a happy path.** Seed the mock with a
   canonical response, call `list_issues`, assert length + first row
   matches.
2. **`get_issue` 404 path.** Mock returns 404; assert the error is an
   `Error::Other` whose message starts with `"not found: "`.
3. **Auth header is exactly what the backend advertises.** Use a custom
   `wiremock::Match` impl (both in-tree adapters have one). For Basic auth
   the assertion is byte-exact on `Basic <base64>`; for Bearer it's the
   raw token prefix.
4. **Pagination cursor is followed correctly.** Seed two pages; assert the
   adapter's second request URL is what the first response's next-cursor
   said it should be (and NOT the server-supplied `_links.base`).
5. **Rate-limit gate arms on 429 / exhausted-remaining.** Mock returns a
   429 with `Retry-After: 2` (or equivalent); assert
   `backend.rate_limit_gate.lock().is_some()` afterwards.

Look at
[`reposix-confluence/src/lib.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-confluence/src/lib.rs)'s
`#[cfg(test)]` module for the full 17-test reference.

Additionally, publish a contract test in `tests/contract.rs` that runs the
same invariant set against both `SimBackend` (always, as a control) and your
backend via wiremock (always) plus a live variant behind `#[ignore]`.
`reposix-github/tests/contract.rs` and `reposix-confluence/tests/contract.rs`
are the templates.

### Step 4. Publish on crates.io

```bash
cargo publish --dry-run    # sanity check
cargo publish
```

Pick a name that is obvious — the convention is `reposix-adapter-<name>`.
Semver your crate independently of reposix; note the minimum compatible
`reposix-core` version in README.

### Step 5. Consume it from a reposix fork

Until Phase 12's subprocess ABI ships, consuming your adapter requires a
fork of `reubenjohn/reposix`. The diff is small — three files:

1. `crates/reposix-cli/Cargo.toml` — add your crate as a dependency.
2. `crates/reposix-cli/src/list.rs` — add a `Foo` variant to the
   `--backend` enum and a dispatch arm to `Box::new(FooBackend::new(...)?)`.
3. `crates/reposix-cli/src/mount.rs` + `crates/reposix-fuse/src/main.rs` —
   the same two-line change so `reposix mount --backend foo` also works.

The Phase 11 diff for Confluence is the canonical worked example — see the
[`11-B-cli-dispatch.md` SUMMARY](https://github.com/reubenjohn/reposix/blob/main/.planning/phases/11-confluence-adapter/11-B-SUMMARY.md)
and the commits it cites. The total change is ~60 lines of CLI plumbing.

## Five non-negotiable security rules for adapter authors

reposix is a textbook lethal-trifecta machine (private data + untrusted
ticket text + `git push` exfiltration). Every rule below is mechanically
enforced for the in-tree adapters and WILL be checked by code review before
your adapter lands in any fork that claims the reposix name.

### 1. Every HTTP call MUST go through `reposix_core::http::HttpClient` (SG-01)

```rust
use reposix_core::http::{client, ClientOpts, HttpClient};

let http = client(ClientOpts::default())?;    // honours REPOSIX_ALLOWED_ORIGINS
```

Do NOT `use reqwest::Client` and call `Client::new()`. The workspace has a
clippy `disallowed-methods` lint on `reqwest::Client::new` and
`Client::builder` that rejects this at compile time. Rationale: SG-01's
egress allowlist is the single choke-point that prevents a malicious
`parent_url` or injected redirect from smuggling private data to an
attacker-controlled origin. A direct `reqwest::Client` bypasses the
allowlist.

### 2. Every parsed response wraps in `Tainted<T>` at the crate boundary (SG-05)

```rust
use reposix_core::Tainted;

let raw: FooWireShape = resp.json().await?;
let tainted = Tainted::new(raw);              // SG-05: everything from the network is tainted
let issue = translate_foo(tainted)?;          // your internal translator unwraps
```

Rationale: any byte that came from the network is attacker-influenced.
`Tainted<T>` makes this typed — your translator has to explicitly unwrap
before it can route the value into any sink (audit log, FUSE write buffer,
`git push`). The `Untainted<T>` counterpart is what the core produces
AFTER `sanitize()` has stripped server-authoritative fields.

### 3. Credentials: manual `Debug` that redacts secrets

```rust
#[derive(Clone)]
pub struct FooCreds {
    pub username: String,
    pub api_token: String,
}

impl std::fmt::Debug for FooCreds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FooCreds")
            .field("username", &self.username)
            .field("api_token", &"<redacted>")
            .finish()
    }
}
```

Do NOT `#[derive(Debug)]` on a credential struct. The derive would print
the token into every `tracing` span, every error message, every
`anyhow::Error::downcast`. The
[`ConfluenceCreds`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-confluence/src/lib.rs)
impl is the reference — apply the same pattern to your credential type AND
to any struct that embeds it (e.g. the backend struct itself).

### 4. Tenant / host strings validated to defeat SSRF

If your backend takes a user-supplied tenant or hostname fragment (e.g.
`REPOSIX_FOO_TENANT=mycompany`), validate it against DNS-label rules
**before** interpolating it into a URL. The reference is
`ConfluenceReadOnlyBackend::validate_tenant` in `reposix-confluence`:

```rust
fn validate_tenant(tenant: &str) -> Result<()> {
    if tenant.is_empty() || tenant.len() > 63 {
        return Err(Error::Other(format!("invalid tenant: {tenant:?}")));
    }
    if tenant.starts_with('-') || tenant.ends_with('-') {
        return Err(Error::Other(format!("invalid tenant: {tenant:?}")));
    }
    if !tenant.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(Error::Other(format!("invalid tenant: {tenant:?}")));
    }
    Ok(())
}
```

Rationale: without this check, `REPOSIX_FOO_TENANT=a.evil.com` interpolated
into `https://{tenant}.yourbackend.net/...` could smuggle the request to
`a.evil.com.yourbackend.net` or (worse, depending on DNS rebinding) to
`evil.com` directly. This is T-11-02 in the Phase 11 threat model and the
reference implementation of the mitigation.

**Also do not trust `_links.base` / `next` / redirect URLs returned in
response bodies** — Confluence's cursor pagination returns a relative path
which the adapter prepends to its own pre-validated base URL; it ignores
the server's `_links.base` exactly because a compromised tenant could
redirect cursor-following calls to an arbitrary origin.

### 5. Rate limits: implement a shared-state gate

```rust
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

pub struct FooBackend {
    // ...
    rate_limit_gate: Arc<Mutex<Option<Instant>>>,
}
```

The gate is `Arc`-shared so `Clone`ing the backend does not reset it —
otherwise a single throttled token could be bypassed by cloning. Before
every outbound request call `await_rate_limit_gate()` which sleeps until
the gate elapses. After every response call `ingest_rate_limit(&resp)`
which arms the gate if the response tells you you're throttled.

- GitHub's pattern: `x-ratelimit-remaining == 0` + `x-ratelimit-reset`
  (unix epoch).
- Atlassian's pattern: `429` + `Retry-After` (seconds from now), with
  `x-ratelimit-remaining` as an early-warning signal.

Whatever your backend emits, implement this gate; do NOT skip it on the
theory that "we won't hit rate limits in normal usage." You will, and
without the gate the symptom is either a 429 loop or a silent ban.

## What's coming in Phase 12 (subprocess ABI)

The crates.io + fork model documented above works but has three real
limitations: it requires a fork, it requires Rust, and adding or removing
an adapter requires a recompile. Phase 12 solves all three with a
subprocess / JSON-RPC connector ABI.

**The plan:** connectors become standalone binaries named
`reposix-connector-<name>` on `PATH`, mirroring how `git-remote-reposix`
already plugs into git. `reposix list --backend $PROTOCOL/foo` spawns
`reposix-connector-foo` and speaks a documented JSON-RPC protocol over
stdio. The reposix daemon proxies outbound HTTP on the connector's behalf
(re-enforcing SG-01 at the daemon boundary, so a malicious connector
cannot bypass the allowlist) and surfaces the trait's five methods as
JSON-RPC methods `list_issues`, `get_issue`, etc.

**Why it's better:**

- **Polyglot.** Python / Go / Node / anything-that-can-parse-JSON-over-stdin
  can write a connector. The Rust-only requirement disappears.
- **No recompile.** Adding a connector is `cp
  reposix-connector-foo ~/.local/bin/`. Removing it is `rm`.
- **Sandbox per plugin.** Each connector is a separate process — OS-level
  resource limits, seccomp filters, and namespace isolation become
  practical.
- **Stable ABI.** The JSON-RPC schema is versioned; `reposix-core`'s Rust
  trait can evolve without breaking connectors.

Phase 12 is tracked in
[ROADMAP.md §Phase 12](https://github.com/reubenjohn/reposix/blob/main/.planning/ROADMAP.md).
It is **not** shipping tonight — the skeleton exists, full planning and
execution are queued for the next milestone. Until then, this guide's
crates-model is the supported path; when Phase 12 lands, a new ADR-003
will document the migration path and both models will coexist for at
least one release cycle.

## FAQ

**Q: Can I write a connector in Python?**
Not yet. The v0.3 crates-model is Rust-only because the trait lives in
Rust and is consumed via `cargo`. Phase 12's subprocess ABI (above) fixes
this — Python connectors will be first-class citizens once it lands.

**Q: Do I need to fork reposix?**
Yes for v0.3 — the `--backend` enum and dispatch arms live in
`crates/reposix-cli` and `crates/reposix-fuse`, and there is no
dynamic-discovery mechanism yet. No for v0.4+ once Phase 12 ships —
dropping a connector binary onto `PATH` will be enough.

**Q: How do I test against a mock API?**
Use the [`wiremock`](https://docs.rs/wiremock/latest/wiremock/) crate. Both
in-tree adapters have extensive `#[cfg(test)]` modules against
`wiremock::MockServer` —
[`reposix-github/src/lib.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-github/src/lib.rs)
and
[`reposix-confluence/src/lib.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-confluence/src/lib.rs).
The pattern: use `new_with_base_url` to point the backend at
`server.uri()`, seed responses with `Mock::given(method("GET")).and(path(...)).respond_with(...)`,
and assert against observed requests via
`mock.received_requests().await`.

**Q: My backend doesn't fit the `Issue` shape (e.g. it has a page
hierarchy).**
Either flatten it (Confluence did; see
[ADR-002](../decisions/002-confluence-page-mapping.md)) and document the
lost metadata in an ADR under `docs/decisions/`, or wait for Phase 12's
protocol to land and model your domain natively on the JSON-RPC side. Do
NOT extend `reposix-core`'s `Issue` struct with backend-specific fields;
that leaks semantics across backend boundaries.

**Q: My backend returns labels / comments / attachments. How do I expose
them?**
For v0.3, they go into the `body` field of the rendered frontmatter —
agents read them as text on the FUSE mount. `Issue.labels` exists for
GitHub-style string labels but is intentionally minimal. First-class
comment / attachment support is a v0.5+ item that depends on Phase 12 or
an `Issue.extensions` field landing in `reposix-core`.

**Q: What license should my adapter use?**
The reposix workspace is dual-licensed MIT / Apache-2.0. Aligning with
that license choice makes vendoring your adapter into a reposix fork
trivial; a more restrictive license means downstream users cannot ship
pre-built reposix binaries that include it.

## See also

- [`crates/reposix-core/src/backend.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-core/src/backend.rs)
  — `IssueBackend` trait, `BackendFeature`, `DeleteReason`.
- [`crates/reposix-github/src/lib.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-github/src/lib.rs)
  — worked example (v0.2).
- [`crates/reposix-confluence/src/lib.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-confluence/src/lib.rs)
  — worked example (v0.3).
- [ADR-001 GitHub state mapping](../decisions/001-github-state-mapping.md)
  — the pattern for documenting a backend's mapping decisions.
- [ADR-002 Confluence page mapping](../decisions/002-confluence-page-mapping.md)
  — the pattern for documenting lost-metadata tradeoffs.
- [Confluence backend reference](../reference/confluence.md) — the pattern
  for user-facing backend docs.
- [ROADMAP.md §Phase 12](https://github.com/reubenjohn/reposix/blob/main/.planning/ROADMAP.md)
  — where the subprocess connector ABI is tracked.
