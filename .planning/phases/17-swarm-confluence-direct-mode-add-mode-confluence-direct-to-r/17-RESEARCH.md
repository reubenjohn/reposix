# Phase 17: Swarm Confluence-Direct Mode — Research

**Researched:** 2026-04-14
**Domain:** reposix-swarm workload extension, ConfluenceBackend, wiremock CI testing
**Confidence:** HIGH — all findings verified directly against codebase

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
1. **Rate-limit strategy:** Back off via existing `rate_limit_gate` (reuse `ConfluenceBackend`'s built-in handling). No custom retry logic in the workload.
2. **CI test:** 3 clients × 5s against wiremock; real-tenant test under `--ignored`.
3. **Write ops:** Read-only for Phase 17 — `list_issues` + `get_issue` only. Write-contention in Phase 21 (OP-7).
4. **Metrics format:** Match `SimDirectWorkload` summary format exactly.

### Claude's Discretion
- Internal workload structure (how to store page IDs, RNG usage).
- Whether `--space` or `--project` arg name is used for the space key.
- Whether rate-limited requests get a dedicated `record_error(RateLimited)` call vs just being delayed transparently.

### Deferred Ideas (OUT OF SCOPE)
- Write operations (`create_issue`, `update_issue`, `delete_or_close`) — Phase 21.
- 50-client × 30s real-tenant runs — too expensive for CI.
- Contention testing — Phase 21 (HARD-01).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SWARM-01 | `reposix-swarm --mode confluence-direct` exercises `ConfluenceBackend` directly (no FUSE overhead), mirroring `SimDirectWorkload` pattern | `ConfluenceBackend` is `Clone + Send + Sync`; mirrors `SimBackend` usage in `SimDirectWorkload`. New `confluence_direct.rs` module + `Mode::ConfluenceDirect` variant in `main.rs`. |
| SWARM-02 | Swarm run produces summary metrics + audit-log rows, matching sim-direct output format | `MetricsAccumulator` is already shared; `render_markdown` format is fixed. Audit: `ConfluenceBackend` writes audit rows on writes; read-only workload has NO writes, so audit row count assertion must be 0 or elided. |
</phase_requirements>

---

## Summary

Phase 17 adds `ConfluenceDirectWorkload` to `reposix-swarm` by mirroring the existing `SimDirectWorkload` pattern almost exactly. The primary structural difference is that `ConfluenceBackend` requires credentials + a base URL (not just an origin string), and it handles rate limiting internally via its `rate_limit_gate` / `await_rate_limit_gate` mechanism — the workload does not need to implement any retry logic.

The workload is read-only (locked decision): each step calls `list_issues` (project = space key, e.g. `"REPOSIX"`) then up to 3 × `get_issue` on random page IDs from the cached list. No patch/update step. The `MetricsAccumulator` only records `OpKind::List` and `OpKind::Get` (no `Patch`), which is fine — the histogram for `Patch` stays at zero and renders as `| patch |         0 | - | - | - | - |`.

**SWARM-02 audit caveat:** `ConfluenceBackend` only writes audit rows on write operations. Since Phase 17 is read-only, the audit row count after a swarm run will be **0**. The mini_e2e CI test should assert `audit_rows == 0` (or omit the audit assertion entirely) to avoid a false test. This differs from the sim-direct test where `SimBackend`'s simulator logs every request.

**Primary recommendation:** Implement `confluence_direct.rs` as a close structural copy of `sim_direct.rs` with credentials injected from env vars (or CLI args for `--email` / `--token` / `--tenant`), omit the patch step, and use `ConfluenceBackend::new_with_base_url` for wiremock tests and `ConfluenceBackend::new` for real-tenant tests.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Workload loop (list + get) | reposix-swarm binary | — | Swarm driver owns concurrency; workload owns op mix |
| HTTP to Confluence | reposix-confluence (ConfluenceBackend) | reposix-core (HttpClient + allowlist) | Backend owns protocol; core enforces SG-01 egress |
| Rate-limit backpressure | reposix-confluence (rate_limit_gate) | — | Already implemented; workload just calls `list/get` normally |
| Metrics recording | reposix-swarm (MetricsAccumulator) | — | Shared accumulator, unchanged from sim-direct |
| CI mocking | wiremock MockServer | — | In-process, no external dependency |
| Credential handling | CLI args / env vars | — | Real-tenant test only; CI wiremock uses dummy creds |

---

## Standard Stack

### Core (already in workspace, no new deps needed)
| Crate | Version | Purpose |
|-------|---------|---------|
| `reposix-confluence` | workspace | `ConfluenceBackend` + `ConfluenceCreds` |
| `reposix-core` | workspace | `IssueBackend`, `IssueId`, `Error` |
| `async-trait` | workspace | `Workload` trait impl |
| `parking_lot` | workspace | `Mutex<StdRng>` + `Mutex<Vec<IssueId>>` |
| `rand` | 0.8 | `StdRng::seed_from_u64` for determinism |
| `anyhow` | workspace | factory + step error type |

### Dev-dependencies to add to reposix-swarm/Cargo.toml
| Crate | Version | Purpose |
|-------|---------|---------|
| `reposix-confluence` | path | `ConfluenceBackend` in integration test |
| `wiremock` | 0.6 | Mock Confluence HTTP server for CI test |

`wiremock = "0.6"` is already in `reposix-confluence/Cargo.toml` as a dev-dep — same version must be used in `reposix-swarm` dev-deps for consistency.

**Cargo.toml changes:**
```toml
# crates/reposix-swarm/Cargo.toml — [dependencies]
reposix-confluence = { path = "../reposix-confluence" }

# [dev-dependencies]  (add)
reposix-confluence = { path = "../reposix-confluence" }
wiremock = "0.6"
```

Note: `reposix-confluence` must be added as a **runtime** dependency too (not just dev) because `ConfluenceDirectWorkload` lives in `src/`, not `tests/`.

---

## Architecture Patterns

### Codebase structure after Phase 17
```
crates/reposix-swarm/src/
├── lib.rs               # add `pub mod confluence_direct;`
├── main.rs              # add Mode::ConfluenceDirect + dispatch arm
├── workload.rs          # unchanged (Workload trait)
├── driver.rs            # unchanged
├── metrics.rs           # unchanged
├── sim_direct.rs        # unchanged (template)
├── fuse_mode.rs         # unchanged
└── confluence_direct.rs # NEW — ConfluenceDirectWorkload

crates/reposix-swarm/tests/
├── mini_e2e.rs          # add confluence_direct_3_clients_5s test
└── confluence_real_tenant.rs  # NEW — #[ignore] real tenant test
```

### Pattern 1: ConfluenceDirectWorkload struct (mirror of SimDirectWorkload)

`SimDirectWorkload` holds:
- `backend: SimBackend` — cheaply cloneable (`Arc` inside)
- `project: String`
- `rng: Mutex<StdRng>`
- `ids: Mutex<Vec<IssueId>>`

`ConfluenceDirectWorkload` holds the same shape:
- `backend: ConfluenceBackend` — cheaply cloneable (`Arc<HttpClient>` + `Arc<Mutex<Option<Instant>>>` inside; documented in ConfluenceBackend doc comment: "Clone is cheap")
- `space: String` — the Confluence space key (maps to `project` param in `list_issues`)
- `rng: Mutex<StdRng>`
- `ids: Mutex<Vec<IssueId>>`

```rust
// Source: crates/reposix-swarm/src/confluence_direct.rs (to be created)
pub struct ConfluenceDirectWorkload {
    backend: ConfluenceBackend,
    space: String,
    rng: Mutex<StdRng>,
    ids: Mutex<Vec<IssueId>>,
}

impl ConfluenceDirectWorkload {
    /// Build a new instance.
    ///
    /// `base_url` — e.g. `"https://tenant.atlassian.net"` for production,
    /// or the wiremock server URI for tests.
    ///
    /// # Errors
    /// Propagates [`ConfluenceBackend::new_with_base_url`] failures.
    pub fn new(
        base_url: String,
        creds: ConfluenceCreds,
        space: String,
        seed: u64,
    ) -> anyhow::Result<Self> {
        let backend = ConfluenceBackend::new_with_base_url(creds, base_url)?;
        Ok(Self {
            backend,
            space,
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
            ids: Mutex::new(Vec::new()),
        })
    }
}
```

### Pattern 2: Workload::step — read-only (no patch)

```rust
#[async_trait]
impl Workload for ConfluenceDirectWorkload {
    async fn step(&self, metrics: &Arc<MetricsAccumulator>) -> anyhow::Result<()> {
        // 1. list (also warms the id cache)
        let start = Instant::now();
        match self.backend.list_issues(&self.space).await {
            Ok(issues) => {
                metrics.record(OpKind::List, elapsed_us(start));
                let mut g = self.ids.lock();
                g.clear();
                g.extend(issues.iter().map(|i| i.id));
            }
            Err(err) => {
                metrics.record(OpKind::List, elapsed_us(start));
                metrics.record_error(ErrorKind::classify(&err));
            }
        }

        // 2. 3 × get
        for _ in 0..3 {
            let Some(id) = self.random_id() else { break; };
            let start = Instant::now();
            match self.backend.get_issue(&self.space, id).await {
                Ok(_) => metrics.record(OpKind::Get, elapsed_us(start)),
                Err(err) => {
                    metrics.record(OpKind::Get, elapsed_us(start));
                    metrics.record_error(ErrorKind::classify(&err));
                }
            }
        }
        Ok(())
    }
}
```

Rate limiting is **transparent**: `ConfluenceBackend::list_issues` and `get_issue` both call `self.await_rate_limit_gate()` before issuing requests. If a 429 is received, `ingest_rate_limit` arms the gate, and the next call to `await_rate_limit_gate` sleeps. The workload records a `RateLimited` error if Confluence returns an error; if the backend absorbs the 429 by sleeping and retrying, no error is recorded (the latency just gets longer). This is the correct behavior per the locked decision.

### Pattern 3: main.rs dispatch — add Mode::ConfluenceDirect

```rust
// Add to Mode enum:
/// HTTP to ConfluenceBackend directly.
ConfluenceDirect,

// Add to Mode::as_str():
Self::ConfluenceDirect => "confluence-direct",

// Add to Args:
/// Atlassian account email (for `confluence-direct`).
#[arg(long)]
email: Option<String>,
/// Atlassian API token (for `confluence-direct`). Reads from env
/// ATLASSIAN_API_KEY if not set explicitly.
#[arg(long, env = "ATLASSIAN_API_KEY")]
api_token: Option<String>,

// Add dispatch arm:
Mode::ConfluenceDirect => {
    let email = args.email
        .ok_or_else(|| anyhow::anyhow!("--email required for confluence-direct"))?;
    let token = args.api_token
        .ok_or_else(|| anyhow::anyhow!("--api-token / ATLASSIAN_API_KEY required"))?;
    let creds = ConfluenceCreds { email, api_token: token };
    let base = args.target.clone();
    let space = args.project.clone();
    run_swarm(cfg, |i| {
        ConfluenceDirectWorkload::new(base.clone(), creds.clone(), space.clone(),
                                      u64::try_from(i).unwrap_or(0))
    }).await?
}
```

`--target` doubles as the base URL for `confluence-direct` (consistent with `sim-direct` reuse of `--target` as the sim URL). `--project` doubles as the space key (consistent: sim uses `--project` as the project slug).

### Anti-Patterns to Avoid
- **Custom retry loop in the workload:** `ConfluenceBackend` already handles `Retry-After` via `rate_limit_gate`. Adding a second retry layer in the workload creates double-sleep and metric double-counting.
- **Creating a new `ConfluenceBackend` per step:** `Clone` is cheap (Arc-shared internals), but per-step construction re-builds the `HttpClient` and loses the shared `rate_limit_gate`. Construct once in `new()`, clone if the driver needs `Arc<dyn Workload>` wrapping.
- **Using `ConfluenceBackend::new(creds, tenant)` in wiremock tests:** `new()` builds the production URL from the tenant subdomain. Use `new_with_base_url(creds, server.uri())` in tests.
- **Asserting audit rows > 0 in CI test:** Read-only workload produces 0 audit writes. The assertion from `mini_e2e.rs` for sim (rows >= 5) does NOT apply here. Either assert `rows == 0` or skip the audit assertion.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Rate-limit backoff | Custom sleep + retry in workload | `ConfluenceBackend::await_rate_limit_gate` (already ships) |
| Mock HTTP server | Custom axum handler for CI | `wiremock 0.6` (already in confluence dev-deps) |
| Credential redaction in logs | Manual string scrubbing | `ConfluenceCreds`'s manual `Debug` impl already redacts |
| Concurrent access to id cache | `std::sync::Mutex` | `parking_lot::Mutex` (already in swarm deps, lower overhead) |

---

## Wave Breakdown

### Wave A — Core workload implementation
**Files:** `src/confluence_direct.rs` (new), `src/lib.rs` (add module), `src/main.rs` (Mode variant + dispatch + credential args)

Tasks:
1. Create `crates/reposix-swarm/src/confluence_direct.rs` — `ConfluenceDirectWorkload` struct + `Workload` impl (list + 3×get, no patch).
2. Expose `pub mod confluence_direct;` in `src/lib.rs`.
3. Add `Mode::ConfluenceDirect` + `as_str` arm + `--email`/`--api-token` args + dispatch arm in `src/main.rs`.
4. Add `reposix-confluence` to `[dependencies]` in `Cargo.toml`.
5. `cargo check --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` green.

### Wave B — Tests + docs + release prep
**Files:** `tests/mini_e2e.rs` (add confluence test), `tests/confluence_real_tenant.rs` (new, `#[ignore]`), `Cargo.toml` dev-deps, CHANGELOG

Tasks:
1. Add `reposix-confluence` + `wiremock = "0.6"` to `[dev-dependencies]` in swarm `Cargo.toml`.
2. Add `confluence_direct_3_clients_5s` test in `tests/mini_e2e.rs` (wiremock, 3 clients, 5s). Stub: `GET /wiki/api/v2/spaces`, `GET /wiki/api/v2/spaces/{id}/pages`, `GET /wiki/api/v2/pages/{id}?body-format=atlas_doc_format`.
3. Create `tests/confluence_real_tenant.rs` with `#[ignore]` test — uses `skip_if_no_env!` for `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`.
4. `cargo test --workspace` green (including new wiremock test).
5. CHANGELOG + `17-SUMMARY.md`.

---

## Test Strategy

### CI Test (Wave B, task 2)
**File:** `crates/reposix-swarm/tests/mini_e2e.rs` — new test function `confluence_direct_3_clients_5s`

**Wiremock stubs needed** (all on `MockServer::start().await`):

| Stub | Method + Path | Response |
|------|---------------|----------|
| Space resolver | `GET /wiki/api/v2/spaces?keys=TESTSPACE` | `{"results":[{"id":"9001"}]}` |
| Page list | `GET /wiki/api/v2/spaces/9001/pages?limit=100` | `{"results":[{id,title,status,createdAt,version,body:{}},...], "_links":{}}` (5 pages, no next cursor) |
| Page get × N | `GET /wiki/api/v2/pages/{id}?body-format=atlas_doc_format` | minimal ADF page JSON |

**Invariants to assert** (mirrors `swarm_mini_e2e_sim_5_clients_1_5s`):
1. `markdown.contains("Clients: 3")`
2. `markdown.contains("| list ")` — list op row present
3. `total_ops >= 3` (conservative; 3 clients × at least 1 completed step)
4. No `| Other` in errors section (transport errors indicate a real bug)
5. **Do NOT assert** audit rows > 0 — read-only workload writes 0 audit rows

**Test setup pattern** (from `roundtrip.rs`):
```rust
let server = MockServer::start().await;
// Each stub needs `Mock::given(method("GET")).and(path(...)).respond_with(...).mount(&server).await;`
// Allowlist: set REPOSIX_ALLOWED_ORIGINS to server.uri() before constructing backend
std::env::set_var("REPOSIX_ALLOWED_ORIGINS", server.uri());
let creds = ConfluenceCreds { email: "t@e.com".into(), api_token: "tok".into() };
```

**Important:** `REPOSIX_ALLOWED_ORIGINS` must include the wiremock server URI, since the default allowlist is `http://127.0.0.1:*` only. Wiremock binds to `127.0.0.1:0` so the URI is `http://127.0.0.1:{port}`, which IS covered by the default glob. Verify at test time — if wiremock ever binds `0.0.0.0` this becomes a problem.

### Real-tenant test (Wave B, task 3)
**File:** `crates/reposix-swarm/tests/confluence_real_tenant.rs`
- Gated on `#[ignore]` — only runs with `cargo test -- --ignored`
- Uses `skip_if_no_env!` macro from `reposix-core` or inline check for `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`
- 3 clients × 10s (not 50 × 30s, per locked decision)
- Asserts: no `Other`-class errors, total_ops > 0

---

## Risk Areas

### Risk 1: `rate_limit_gate` is shared across `ConfluenceBackend` clones
**Detail:** `rate_limit_gate: Arc<Mutex<Option<Instant>>>` is cloned by `Arc::clone` when `ConfluenceBackend` is cloned. This means all N swarm clients share one gate if they clone from the same backend. This is actually correct behavior for real-tenant runs (one gate governs the whole connection), but means a single 429 serializes ALL clients until the gate expires.

**Mitigation:** The workload constructs a fresh `ConfluenceBackend` per client (in the factory closure, `|i| ConfluenceDirectWorkload::new(...)` — each call to `new_with_base_url` creates a new `Arc<Mutex<None>>` gate). So each swarm client has an independent gate. This matches `SimDirectWorkload` where each client gets its own `SimBackend::with_agent_suffix`.

**Verify:** Confirm `ConfluenceBackend::new_with_base_url` always creates `Arc::new(Mutex::new(None))` — confirmed at line ~480 of lib.rs.

### Risk 2: Wiremock stub exhaustion / call count mismatch
**Detail:** Wiremock stubs in Phase 16 tests use `.expect(N)` matchers. If the swarm makes more calls than expected, wiremock panics. For a duration-based test (5s), call count is non-deterministic.

**Mitigation:** Register stubs WITHOUT `.expect(N)` — let them respond unlimited times. Use `wiremock::Mock::given(...).respond_with(...).mount(&server)` without `.expect(N_calls)`. The mini_e2e sim test does NOT use expect matchers — follow the same pattern.

### Risk 3: Page list wiremock stub returning no `_links.next`
**Detail:** `list_issues` loops on `parse_next_cursor`. If the stub's `_links` is missing or malformed, the loop exits after page 1 (correct). But if `_links.next` is a relative path that starts with `/`, `list_issues` prepends `self.base()` — producing a valid URL that wiremock will NOT have a stub for, causing `Error::Other`.

**Mitigation:** Return `"_links": {}` (no `next` key) in the stub response to ensure single-page pagination. This is the simplest correct fixture.

### Risk 4: `list_issues` calls `resolve_space_id` on every step
**Detail:** Each `step()` call re-resolves the space ID via `GET /wiki/api/v2/spaces?keys={space}`. With 3 clients × ~1 step/cycle × 5s ≈ ~15+ space-resolve calls against wiremock. The stub must handle repeated calls (no `.expect(1)` on the space stub).

**Performance note:** For real-tenant runs this doubles the request count. Not a Phase 17 concern but worth documenting for Phase 21 optimization.

---

## Common Pitfalls

### Pitfall 1: `REPOSIX_ALLOWED_ORIGINS` not set for wiremock test
**What goes wrong:** `HttpClient::new` reads `REPOSIX_ALLOWED_ORIGINS` at construction time. Default allows `http://127.0.0.1:*`. Wiremock typically binds to `127.0.0.1:PORT` so it matches. If the test constructs the backend BEFORE the env var is set, the allowlist is built from the old value.

**How to avoid:** Set `REPOSIX_ALLOWED_ORIGINS` (or leave default for 127.0.0.1) before calling `ConfluenceDirectWorkload::new`. Since default already covers 127.0.0.1, this is usually a non-issue for CI — but document the dependency.

### Pitfall 2: `get_issue` requires `?body-format=atlas_doc_format`
**What goes wrong:** `get_issue` requests ADF body format. The wiremock stub for the page GET must include the query param `body-format=atlas_doc_format` in the matcher, or wiremock won't match the request.

**How to avoid:** Use `wiremock::matchers::query_param("body-format", "atlas_doc_format")` in the `GET /wiki/api/v2/pages/{id}` stub. The roundtrip test in Phase 16 already shows this pattern.

### Pitfall 3: `translate()` fails on minimal wiremock fixture
**What goes wrong:** `translate()` requires `createdAt`, `version.number`, `version.createdAt` fields. A too-minimal fixture (just `id` + `title`) produces a deserialization error → `Error::Other` → `ErrorKind::Other` in metrics → test assertion `!err_section.contains("| Other")` fails.

**How to avoid:** Wiremock page fixture must include at minimum:
```json
{
  "id": "10001",
  "status": "current",
  "title": "Test Page",
  "createdAt": "2026-01-01T00:00:00Z",
  "version": { "number": 1, "createdAt": "2026-01-01T00:00:00Z" },
  "body": {}
}
```

---

## Code Examples

### Wiremock space + page list stubs (minimal correct fixture)
```rust
// Source: crates/reposix-confluence/tests/roundtrip.rs (adapted)
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use serde_json::json;

let server = MockServer::start().await;

// Space resolver
Mock::given(method("GET"))
    .and(path("/wiki/api/v2/spaces"))
    .and(query_param("keys", "TESTSPACE"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "results": [{"id": "9001", "key": "TESTSPACE"}]
    })))
    .mount(&server).await;

// Page list (no _links.next → single page)
Mock::given(method("GET"))
    .and(path("/wiki/api/v2/spaces/9001/pages"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "results": [
            {"id":"10001","status":"current","title":"Page 1",
             "createdAt":"2026-01-01T00:00:00Z",
             "version":{"number":1,"createdAt":"2026-01-01T00:00:00Z"},
             "body":{}},
            {"id":"10002","status":"current","title":"Page 2",
             "createdAt":"2026-01-01T00:00:00Z",
             "version":{"number":1,"createdAt":"2026-01-01T00:00:00Z"},
             "body":{}},
        ],
        "_links": {}
    })))
    .mount(&server).await;

// Page get (wildcard path + required query param)
Mock::given(method("GET"))
    .and(wiremock::matchers::path_regex(r"^/wiki/api/v2/pages/\d+$"))
    .and(query_param("body-format", "atlas_doc_format"))
    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "id":"10001","status":"current","title":"Page 1",
        "createdAt":"2026-01-01T00:00:00Z",
        "version":{"number":1,"createdAt":"2026-01-01T00:00:00Z"},
        "body":{"atlas_doc_format":{"value":{"type":"doc","version":1,"content":[]}}}
    })))
    .mount(&server).await;
```

### ConfluenceDirectWorkload constructor
```rust
// Source: pattern derived from sim_direct.rs + confluence lib.rs
use reposix_confluence::{ConfluenceBackend, ConfluenceCreds};

pub fn new(base_url: String, creds: ConfluenceCreds, space: String, seed: u64)
    -> anyhow::Result<Self>
{
    let backend = ConfluenceBackend::new_with_base_url(creds, base_url)
        .map_err(|e| anyhow::anyhow!("ConfluenceBackend init: {e}"))?;
    Ok(Self {
        backend,
        space,
        rng: Mutex::new(StdRng::seed_from_u64(seed)),
        ids: Mutex::new(Vec::new()),
    })
}
```

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[tokio::test]` |
| Config file | none (workspace-level `rust-toolchain.toml`) |
| Quick run command | `cargo test -p reposix-swarm` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| SWARM-01 | `--mode confluence-direct` spawns N clients against ConfluenceBackend | integration | `cargo test -p reposix-swarm confluence_direct_3_clients_5s` | ❌ Wave B |
| SWARM-01 | Mode dispatches to ConfluenceDirectWorkload in main.rs | compile-time | `cargo check -p reposix-swarm` | ❌ Wave A |
| SWARM-02 | Summary markdown matches sim-direct format (Clients:, Total ops:, list row) | integration | `cargo test -p reposix-swarm confluence_direct_3_clients_5s` | ❌ Wave B |
| SWARM-02 | No `Other`-class errors under wiremock | integration | same | ❌ Wave B |

### Sampling Rate
- **Per task commit:** `cargo check --workspace && cargo test -p reposix-swarm`
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/reposix-swarm/src/confluence_direct.rs` — covers SWARM-01 (Wave A creates this)
- [ ] `confluence_direct_3_clients_5s` test in `tests/mini_e2e.rs` — covers SWARM-01 + SWARM-02 (Wave B)
- [ ] `crates/reposix-swarm/tests/confluence_real_tenant.rs` — real-tenant smoke under `--ignored` (Wave B)

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `wiremock 0.6` | CI integration test | ✓ (in confluence dev-deps) | 0.6 | — |
| `ATLASSIAN_API_KEY` env | Real-tenant test | n/a in CI | — | Test gated `#[ignore]` |
| `reposix-confluence` crate | Runtime + test | ✓ (workspace) | 0.6.0 | — |
| Rust 1.82+ | Build | ✓ (`cargo check` passes) | per rust-toolchain.toml | — |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Wiremock binds to 127.0.0.1, covered by default allowlist `http://127.0.0.1:*` | Test Strategy | Test would fail with allowlist rejection; fix: set REPOSIX_ALLOWED_ORIGINS in test setup |
| A2 | `ConfluenceBackend` is `Send + Sync` and can be used in `Arc<dyn Workload>` | Codebase Patterns | Compile error; fix: verify `Arc<HttpClient>` + `Arc<Mutex<...>>` internals are Send+Sync (they are) |

---

## Sources

### Primary (HIGH confidence — verified directly in codebase)
- `crates/reposix-swarm/src/sim_direct.rs` — SimDirectWorkload template (verified full file)
- `crates/reposix-swarm/src/main.rs` — CLI dispatch pattern (verified full file)
- `crates/reposix-swarm/src/workload.rs` — Workload trait (verified full file)
- `crates/reposix-swarm/src/driver.rs` — run_swarm factory pattern (verified full file)
- `crates/reposix-swarm/src/metrics.rs` — MetricsAccumulator + OpKind + ErrorKind (verified full file)
- `crates/reposix-swarm/tests/mini_e2e.rs` — existing CI test pattern (verified full file)
- `crates/reposix-confluence/src/lib.rs` — ConfluenceBackend struct, rate_limit_gate, new_with_base_url, list_issues, get_issue (verified lines 1–780)
- `crates/reposix-confluence/tests/roundtrip.rs` — wiremock test pattern (verified lines 1–80)
- `crates/reposix-swarm/Cargo.toml` — current dependencies (verified full file)
- `crates/reposix-confluence/Cargo.toml` — wiremock version (verified full file)

---

## Metadata

**Confidence breakdown:**
- Codebase patterns: HIGH — verified directly, no assumptions
- Wiremock stubs: HIGH — derived from existing roundtrip.rs patterns
- Rate-limit behavior: HIGH — read `rate_limit_gate`, `ingest_rate_limit`, `await_rate_limit_gate` in full
- Audit row count caveat: HIGH — verified `audit_write` only called in write methods, not list/get

**Research date:** 2026-04-14
**Valid until:** Stable (no external APIs; all findings are in-repo code)

---

## RESEARCH COMPLETE
