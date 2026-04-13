---
phase: 02-simulator-audit-log
plan: 02
type: execute
wave: 2
depends_on: ["02-01"]
files_modified:
  - crates/reposix-sim/Cargo.toml
  - crates/reposix-sim/src/lib.rs
  - crates/reposix-sim/src/main.rs
  - crates/reposix-sim/src/middleware/mod.rs
  - crates/reposix-sim/src/middleware/audit.rs
  - crates/reposix-sim/src/middleware/rate_limit.rs
  - crates/reposix-sim/tests/api.rs
autonomous: true
requirements: [FC-06, SG-06]
est_minutes: 75
risks:
  - "axum::middleware::from_fn consumes the request body; must read-to-bytes once and rebuild Request for downstream. Done wrong, handlers see an empty body and every POST/PATCH test fails opaquely."
  - "http::request::Parts is NOT Clone in axum 0.7. Capture method/uri/headers into locals BEFORE calling Request::from_parts(parts, ...); the parts value is consumed by from_parts."
  - "governor::RateLimiter is sync; wrap in tower::Layer via a Clone-able key extractor + DashMap<String, Arc<RateLimiter>>."
  - "Audit middleware must be outermost so even 429s and 409s are audited. Axum .layer() ordering is 'last call is outermost' — code reads: build_router().layer(rate_limit).layer(audit)."
  - "Append-only trigger raises via SQLITE_CONSTRAINT_TRIGGER with literal message 'audit_events is append-only' (see crates/reposix-core/fixtures/audit.sql). Test must assert error string contains that literal substring — no disjunctions."
  - "parking_lot::Mutex<Connection> held across .await in the middleware would deadlock; scope the lock to just the INSERT."

must_haves:
  truths:
    - "Every HTTP request (including 404s, 429s, 409s) writes exactly one row to audit_events."
    - "Audit row columns match crates/reposix-core/fixtures/audit.sql EXACTLY: (id, ts, agent_id, method, path, status, request_body, response_summary). No 'timestamp' column, no 'request_body_hash' column."
    - "ts is UTC RFC-3339; agent_id comes from X-Reposix-Agent header (default 'anonymous'); request_body stores the first 256 chars of the request body verbatim."
    - "response_summary stores '<resp_status>:<sha256_hex_prefix_16>' where sha256_hex_prefix_16 is the first 16 hex chars of SHA-256(request_body_bytes). This is the v0.1 pragmatic encoding — it gives forensic integrity (full-body hash prefix) and response context (status) in one TEXT column without adding a column."
    - "The 101st request in a one-second window from the same agent (default 100 rps) returns 429 with Retry-After header."
    - "A request with no X-Reposix-Agent header is bucketed under 'anonymous' and still rate-limited."
    - "A 429 response is still audited: forcing saturation (rps=1, burst=1, 2 back-to-back requests) yields exactly one audit row with status=429."
    - "UPDATE on audit_events fails with a rusqlite error whose Display string contains the literal substring 'append-only' (matches the RAISE text in fixtures/audit.sql)."
    - "Integration test boots the sim on 127.0.0.1:0, exercises list/get/patch-409/delete via a real HTTP client, asserts audit COUNT grew by >= N, and asserts the UPDATE trigger fires."
  artifacts:
    - path: "crates/reposix-sim/src/middleware/audit.rs"
      provides: "audit middleware + body-buffering helper + INSERT into audit_events(ts, agent_id, method, path, status, request_body, response_summary)"
    - path: "crates/reposix-sim/src/middleware/rate_limit.rs"
      provides: "per-agent governor-based rate-limit layer returning 429 + Retry-After"
    - path: "crates/reposix-sim/tests/api.rs"
      provides: "end-to-end integration test proving ROADMAP success criteria 4 and 5, including the 429-is-audited invariant"
      min_lines: 100
  key_links:
    - from: "crates/reposix-sim/src/lib.rs::build_router"
      to: "crates/reposix-sim/src/middleware/audit.rs"
      via: ".layer(middleware::audit::layer(state.clone()))"
      pattern: "audit::layer"
    - from: "crates/reposix-sim/src/middleware/audit.rs"
      to: "audit_events table (via state.db)"
      via: "INSERT INTO audit_events (ts, agent_id, method, path, status, request_body, response_summary) VALUES (?1,?2,?3,?4,?5,?6,?7)"
      pattern: "INSERT INTO audit_events"
    - from: "crates/reposix-sim/tests/api.rs"
      to: "reposix_core::http::client"
      via: "ClientOpts::default() — required by the workspace clippy lint"
      pattern: "reposix_core::http::client"
---

<objective>
Layer the two Phase-2 guardrails on top of plan 02-01's router:
1. **Audit middleware** — outermost layer, writes one row per request to
   `audit_events` (append-only, enforced by Phase-1 triggers). Columns match the
   fixture exactly: `ts`, `agent_id`, `method`, `path`, `status`, `request_body`,
   `response_summary`. `ts` is UTC RFC-3339; `agent_id` comes from
   `X-Reposix-Agent` (default `"anonymous"`); `request_body` is the first 256
   chars; `response_summary` is `"<resp_status>:<sha256_hex_prefix_16>"`.
2. **Rate-limit layer** — per-agent `governor` bucket at `--rate-limit` rps
   (default 100), returns 429 with `Retry-After: 1` on overflow. Lives *inside*
   the audit layer so even rate-limited requests get audited.

Then an integration test at `crates/reposix-sim/tests/api.rs` that boots the sim on
an ephemeral port, drives list/get/patch/delete via a real HTTP client, and asserts
the audit table's invariants — including that a 429 response writes an audit row —
closing ROADMAP success criteria 4 and 5.

Purpose: make the sim trustworthy. The audit log is the non-repudiable record of
every action an agent took; the rate limit is the first guardrail an adversarial
swarm hits.

Output: Phase 2 Bash assertions 1-5 all pass; `cargo test -p reposix-sim` green.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/02-simulator-audit-log/02-CONTEXT.md
@.planning/phases/02-simulator-audit-log/02-01-SUMMARY.md
@.planning/research/simulator-design.md
@crates/reposix-core/src/audit.rs
@crates/reposix-core/fixtures/audit.sql
@crates/reposix-core/src/http.rs
@crates/reposix-sim/src/lib.rs
@crates/reposix-sim/src/state.rs
@crates/reposix-sim/src/routes/issues.rs
@clippy.toml

<interfaces>
<!-- Extracted from what plan 02-01 produced + the audit fixture. No exploration needed. -->

From `reposix-sim/src/state.rs`:
```rust
pub struct AppState {
    pub db: Arc<parking_lot::Mutex<rusqlite::Connection>>,
    pub config: Arc<SimConfig>,
}
// Clone via Arc clones.
```

From `reposix-sim/src/lib.rs` (plan 02-01 version, changed by this plan):
```rust
pub fn build_router(state: AppState) -> Router;       // no layers yet
pub async fn run(cfg: SimConfig) -> Result<()>;
```

From `reposix-core::http`:
```rust
pub fn client(opts: ClientOpts) -> Result<reqwest::Client>;
pub struct ClientOpts { /* ... */ }
impl Default for ClientOpts { /* ... */ }
```
**Integration tests MUST use this.** Direct `reqwest::Client::new()` is a
clippy-denied method workspace-wide (see `clippy.toml`).

Audit table (from `crates/reposix-core/fixtures/audit.sql` — SOURCE OF TRUTH):
```sql
CREATE TABLE IF NOT EXISTS audit_events (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    ts               TEXT    NOT NULL,
    agent_id         TEXT,
    method           TEXT    NOT NULL,
    path             TEXT    NOT NULL,
    status           INTEGER,
    request_body     TEXT,
    response_summary TEXT
);
-- BEFORE UPDATE trigger: RAISE(ABORT, 'audit_events is append-only');
-- BEFORE DELETE trigger: RAISE(ABORT, 'audit_events is append-only');
```

INSERT shape for this plan (bind exactly these 7 columns, in this order):
```sql
INSERT INTO audit_events (ts, agent_id, method, path, status, request_body, response_summary)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
```

From `http::request::Parts` (axum 0.7 / hyper 1.x):
```rust
pub struct Parts {
    pub method: http::Method,     // Method: Clone
    pub uri: http::Uri,           // Uri: Clone
    pub version: http::Version,   // Clone
    pub headers: http::HeaderMap, // Clone
    pub extensions: http::Extensions,
    // #[non_exhaustive]
}
// Parts itself is NOT Clone. Individual fields (method, uri, headers) ARE Clone.
// Pattern: destructure req, clone the fields you need, then pass parts to
// Request::from_parts which MOVES it.
```
</interfaces>

<constraints>
- **Layer ordering.** Axum's `.layer()` wraps; the last `.layer()` call becomes
  the outermost. Required order (outermost first): audit → rate_limit → handlers.
  Code: `build_router(state, rps).layer(rate_limit::layer(rps)).layer(audit::layer(state))`.
- **Body capture.** Axum streams bodies; middleware must
  `axum::body::to_bytes(body, 1_048_576)` (1 MiB limit, 413 on overflow),
  hash/truncate, then rebuild `Request::from_parts(parts, Body::from(bytes))` for
  downstream. This is the #1 footgun — document it in a comment.
- **`parts` is move-only.** `http::request::Parts` is NOT `Clone` in axum 0.7.
  The middleware MUST capture `method`, `uri`, and the `X-Reposix-Agent` header
  into locals (via per-field `.clone()`) BEFORE calling
  `Request::from_parts(parts, Body::from(bytes))` — that call moves `parts`.
  Attempting `parts.clone()` is a compile error and blocks the whole plan.
- **Audit column names — match the fixture EXACTLY.** `ts` (not `timestamp`),
  `response_summary` (not `request_body_hash`). No additional columns. No ALTER
  TABLE. If you think you need another column, stop and document in the SUMMARY.
- **`ts`.** `chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)` —
  RFC-3339, trailing `Z`, second precision.
- **`request_body`.** First 256 chars of the body (UTF-8 lossy conversion). For
  non-UTF-8 bodies, store the lossy string anyway; forensics prefers partial
  truth over nothing.
- **`response_summary`.** Format: `"<resp_status>:<sha256_hex_prefix_16>"` where
  `sha256_hex_prefix_16 = &hex::encode(Sha256::digest(&bytes))[..16]`. Example:
  `"200:9f86d081884c7d65"`. Empty-body requests: hash the empty byte slice
  (hex `e3b0c44298fc1c14`, so `response_summary = "200:e3b0c44298fc1c14"`). This
  single-column encoding keeps schema v0.1 stable while still giving forensic
  integrity (full-body hash prefix) and response context (status).
- **Rate limiting.** `governor::Quota::per_second(NonZeroU32::new(rps).unwrap())`.
  Key = `X-Reposix-Agent` header, else `"anonymous"`. 429 body:
  `{"error":"rate_limited","retry_after_secs":1}` + header `Retry-After: 1`.
- **Lock discipline.** Do not hold `state.db.lock()` across `.await`. Pattern:
  `{ let conn = state.db.lock(); conn.execute(...)?; }` then continue.
- **Integration test client.** Use
  `reposix_core::http::client(ClientOpts::default())?`. This doubles as a smoke
  test that Phase 1's http module allows 127.0.0.1.
- **No Tainted<T> wrapping yet.** Add TODO comment in audit.rs:
  `// TODO(phase-3): wrap captured request_body in Tainted<String> before any future egress use.`
  Phase 3 is the threat surface; Phase 2 establishes the precedent.
</constraints>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Audit middleware with body capture + DB write</name>
  <files>
    crates/reposix-sim/Cargo.toml,
    crates/reposix-sim/src/middleware/mod.rs,
    crates/reposix-sim/src/middleware/audit.rs,
    crates/reposix-sim/src/lib.rs
  </files>
  <behavior>
    - `audit::layer(state: AppState)` returns an axum `from_fn_with_state` layer.
    - On every request: capture method, path, `X-Reposix-Agent` (default
      `"anonymous"`), buffer body (<=1 MiB; 413 JSON error on overflow), truncate to
      256 chars for storage, compute SHA-256 hex of full bytes, call downstream,
      capture response status, INSERT one row into `audit_events` with columns
      `(ts, agent_id, method, path, status, request_body, response_summary)`.
    - `ts` = `chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)`.
    - `response_summary` = `format!("{resp_status}:{}", &sha256_hex[..16])`.
    - Unit test `audit_row_shape`: tiny router with a `POST /echo` handler wrapped in
      `audit::layer(state)`. Send POST body `"hello"`, assert response 200, then
      `SELECT COUNT(*), ts, agent_id, method, path, status, request_body, response_summary FROM audit_events`:
      count=1, method="POST", path="/echo", status=200, request_body="hello",
      agent_id="anonymous", response_summary starts with "200:" and has exactly
      17 chars after the status prefix format "<u16>:<16-hex>".
    - Unit test `agent_id_header`: same but with `X-Reposix-Agent: alpha` →
      agent_id="alpha".
    - Unit test `trigger_blocks_update`:
      `state.db.lock().execute("UPDATE audit_events SET path='x' WHERE id=1", [])`
      returns `Err`; the `e.to_string()` MUST contain the literal substring
      `"append-only"` (matches the RAISE message in `fixtures/audit.sql`). No
      disjunctions — assert exactly one substring.
  </behavior>
  <action>
    1. Confirm — by reading `crates/reposix-core/fixtures/audit.sql` — that the
       columns are exactly `(id, ts, agent_id, method, path, status, request_body,
       response_summary)` and the RAISE text is `'audit_events is append-only'`.
       Do NOT introduce a `timestamp` column or a `request_body_hash` column.
    2. Add to `reposix-sim/Cargo.toml` `[dependencies]`: `sha2 = "0.10"` and
       `hex = "0.4"` (sha2 outputs bytes; we need hex for TEXT storage).
       Confirm `chrono` is already a workspace dep (it is, via reposix-core).
    3. Create `middleware/mod.rs` with `pub mod audit; pub mod rate_limit;`.
    4. Create `middleware/audit.rs`:
       - `async fn buffer_body(body: Body, limit: usize) -> Result<(Bytes, String), (StatusCode, Json<Value>)>`
         uses `axum::body::to_bytes(body, limit)`, returns bytes + utf-8 lossy
         string. On error: 413 with `{"error":"body_too_large","limit":<limit>}`.
       - `pub async fn middleware(State(state): State<AppState>, req: Request, next: Next) -> Response`:
         1) `let (parts, body) = req.into_parts();`
         2) **Capture locals BEFORE moving `parts`** (Parts is not Clone):
            ```rust
            let method = parts.method.clone();         // Method: Clone
            let uri = parts.uri.clone();               // Uri: Clone
            let agent_id = parts.headers
                .get("X-Reposix-Agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("anonymous")
                .to_owned();
            ```
         3) Buffer body → `(bytes, body_string_lossy)`.
         4) `let sha256_hex = hex::encode(Sha256::digest(&bytes));`
         5) `let new_req = Request::from_parts(parts, Body::from(bytes.clone()));`
            (moves `parts`; we've already captured method/uri/agent_id above).
         6) `let response = next.run(new_req).await;`
         7) `let status_u16 = response.status().as_u16();`
         8) Compose fields:
            ```rust
            let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let path = uri.path().to_owned();
            let method_str = method.as_str().to_owned();
            let truncated = {
                let s = body_string_lossy.as_str();
                // Byte-safe truncation at char boundary up to 256 chars.
                s.chars().take(256).collect::<String>()
            };
            let response_summary = format!("{status_u16}:{}", &sha256_hex[..16]);
            ```
         9) Scope the lock tightly — no `.await` inside:
            ```rust
            {
                let conn = state.db.lock();
                let _ = conn.execute(
                    "INSERT INTO audit_events \
                     (ts, agent_id, method, path, status, request_body, response_summary) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![ts, agent_id, method_str, path, status_u16, truncated, response_summary],
                ).map_err(|e| tracing::error!(?e, "audit insert failed"));
            }
            ```
            — log-and-swallow on Err so the downstream response is never masked.
         10) Return `response`.
       - `pub fn layer(state: AppState)` returns
         `axum::middleware::from_fn_with_state(state, middleware)`.
       - Add comment: `// TODO(phase-3): wrap captured request_body in Tainted<String> before any future egress use.`
    5. Update `lib.rs::build_router(state)` to attach only the audit layer:
       `Router::new().route("/healthz", get(healthz)).merge(routes::router(state.clone())).layer(middleware::audit::layer(state))`.
       Rate-limit layer is wired in Task 2.
    6. Unit tests in `audit.rs` using `tower::ServiceExt::oneshot`. Use an in-memory
       seeded DB via `db::open_db(":memory:", true)`. For the router under test,
       construct a minimal `AppState` and a 2-route router (`POST /echo` echoes
       body, `GET /z` returns 204) wrapped via `audit::layer`.
    7. Trigger test `trigger_blocks_update`: after at least one row exists, run
       `conn.execute("UPDATE audit_events SET path='x' WHERE id=1", [])` and
       `assert!(err.to_string().contains("append-only"));`. Only that literal
       substring — no "trigger" fallback, no disjunction.
  </action>
  <verify>
    <automated>cargo test -p reposix-sim --lib middleware::audit -- --nocapture 2>&amp;1 | tail -60 &amp;&amp; cargo clippy -p reposix-sim --all-targets -- -D warnings</automated>
  </verify>
  <done>
    Audit middleware writes one row per request using the fixture's exact column
    names (`ts`, `response_summary`); UPDATE trigger blocks mutations and the test
    asserts on the literal `"append-only"` substring; clippy clean; `build_router`
    emits audit rows for all routes including `/healthz`.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Rate-limit layer + wire build_router + integration test</name>
  <files>
    crates/reposix-sim/Cargo.toml,
    crates/reposix-sim/src/middleware/rate_limit.rs,
    crates/reposix-sim/src/lib.rs,
    crates/reposix-sim/src/main.rs,
    crates/reposix-sim/tests/api.rs
  </files>
  <behavior>
    - `rate_limit::layer(rps: u32)` returns an axum `from_fn` layer holding
      `Arc<DashMap<String, Arc<DefaultDirectRateLimiter>>>`. Per-agent quota =
      `Quota::per_second(NonZeroU32::new(rps.max(1)).unwrap())`.
    - Allowed → `next.run(req).await`. Denied → 429 with JSON
      `{"error":"rate_limited","retry_after_secs":1}` and header `Retry-After: 1`.
    - `build_router(state, rps) -> Router` applies layers in documented order:
      handlers, then `.layer(rate_limit::layer(rps))`, then
      `.layer(audit::layer(state))`.
    - Integration test `crates/reposix-sim/tests/api.rs`:
      1. Bind `tokio::net::TcpListener::bind("127.0.0.1:0")`, read `.local_addr()?`.
      2. Use `tempfile::NamedTempFile` for `db_path`; seed from
         `crates/reposix-sim/fixtures/seed.json`.
      3. Spawn sim via `tokio::spawn(async move { run_with_listener(listener, cfg).await })`.
      4. Build client via `reposix_core::http::client(ClientOpts::default())?`.
      5. GET `/projects/demo/issues` → assert array length >= 3.
      6. GET `/projects/demo/issues/1` → assert JSON `version` is u64 >= 1.
      7. PATCH `/projects/demo/issues/1` body `{"status":"done"}` header
         `If-Match: "bogus"` → assert 409, body
         `{error:"version_mismatch", current:1, sent:"bogus"}`.
      8. DELETE `/projects/demo/issues/2` → assert 204.
      9. Open a second `rusqlite::Connection` to the tempfile, assert
         `SELECT COUNT(*) FROM audit_events WHERE method IN ('GET','PATCH','DELETE') AND path LIKE '/projects/demo/%'` >= 4.
      10. Assert trigger fires:
          `conn.execute("UPDATE audit_events SET path='x' WHERE id=1", []).unwrap_err()`
          error string contains the literal `"append-only"`.
    - Second `#[tokio::test]` `rate_limit_returns_429_on_overflow`: build a sim
      with `rate_limit_rps = 2`, fire 10 sequential requests with the same
      `X-Reposix-Agent: hammer` header, assert >=1 response has status 429 and a
      `Retry-After` header.
    - Third `#[tokio::test]` `rate_limited_request_is_audited`: build a sim with
      `rate_limit_rps = 1` (smallest `Quota::per_second` bucket, burst=1), fire
      two back-to-back requests with the same `X-Reposix-Agent: saturate` header
      (no sleep between them). Assert: the second response is 429 AND a second
      `rusqlite::Connection` to the tempfile satisfies
      `SELECT COUNT(*) FROM audit_events WHERE status = 429 AND agent_id = 'saturate'`
      equals 1. This proves the audit layer wraps rate_limit (outermost-first
      ordering holds).
  </behavior>
  <action>
    1. Add to `reposix-sim/Cargo.toml` `[dev-dependencies]`: `tempfile = "3"`.
    2. Create `middleware/rate_limit.rs`:
       ```rust
       use governor::{Quota, RateLimiter, clock::DefaultClock, middleware::NoOpMiddleware, state::{InMemoryState, NotKeyed}};
       type Limiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;
       ```
       `pub fn layer(rps: u32)`:
       - Build `quota = Quota::per_second(NonZeroU32::new(rps.max(1)).unwrap())`.
       - Build `Arc<DashMap<String, Arc<Limiter>>>`.
       - Return `axum::middleware::from_fn(move |req: Request, next: Next| async move { ... })`
         — clone the map/quota Arcs into the closure.
       - Inside: read `X-Reposix-Agent` header (default "anonymous"). `let limiter = map.entry(agent).or_insert_with(|| Arc::new(RateLimiter::direct(quota))).clone();`
       - `match limiter.check() { Ok(_) => next.run(req).await, Err(_) => rate_limited_response() }`.
       - `rate_limited_response()` builds `Response::builder().status(429).header("Retry-After","1").body(Body::from(serde_json::to_vec(&json!({"error":"rate_limited","retry_after_secs":1}))?))`.
    3. Refactor `lib.rs`:
       - Extend `SimConfig` with `pub rate_limit_rps: u32`.
       - `build_router(state, rps)` signature; layer order per constraints.
       - Add `pub async fn run_with_listener(listener: tokio::net::TcpListener, cfg: SimConfig) -> Result<()>` that opens db + loads seed, constructs `AppState`, and calls `axum::serve(listener, build_router(state, cfg.rate_limit_rps)).await?`.
       - `run(cfg)` becomes `let l = TcpListener::bind(cfg.bind).await?; run_with_listener(l, cfg).await`.
    4. Update `main.rs` so `--rate-limit` populates `SimConfig.rate_limit_rps`;
       default 100.
    5. Write `tests/api.rs` per behavior block (three `#[tokio::test]` functions:
       `full_crud_flow_with_audit`, `rate_limit_returns_429_on_overflow`,
       `rate_limited_request_is_audited`). Use
       `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`. Path to seed
       fixture via `env!("CARGO_MANIFEST_DIR")` + `/fixtures/seed.json`.
    6. Keep a comment at the top of `tests/api.rs` enumerating ROADMAP Phase-2
       success criteria 1-5 and pointing to `.planning/ROADMAP.md` lines 145-165.
  </action>
  <verify>
    <automated>cargo test -p reposix-sim --test api -- --nocapture 2>&amp;1 | tail -80 &amp;&amp; cargo test -p reposix-sim --lib -- --nocapture 2>&amp;1 | tail -40 &amp;&amp; cargo clippy -p reposix-sim --all-targets -- -D warnings</automated>
  </verify>
  <done>
    `cargo test -p reposix-sim` fully green (unit + integration, including the
    `rate_limited_request_is_audited` test that proves 429 responses write audit
    rows). Clippy clean. ROADMAP Phase-2 success criteria 1-5 all pass when run
    against a freshly-booted sim.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| external client → audit middleware | Every byte of inbound body is captured before handler dispatch. |
| audit middleware → audit_events table | Append-only contract enforced by Phase-1 triggers, not app code. |
| agent identity (X-Reposix-Agent header) → rate-limit bucket | Untrusted client-supplied string partitions the limiter. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-02-07 | Repudiation | any handler | mitigate | Audit middleware writes one row per request (including 429/409/500 responses). `rate_limited_request_is_audited` test directly proves the 429 case. Trigger-enforced append-only means post-facto deletion is impossible without deliberate schema tampering. |
| T-02-08 | Tampering | audit_events table | mitigate | Phase-1 `audit_no_update` / `audit_no_delete` triggers; integration test asserts a direct UPDATE fails with error string containing literal `"append-only"`. |
| T-02-09 | Info disclosure | audit row body storage | accept (v0.1) | First 256 chars of request body stored verbatim in `request_body`. Full-body SHA-256 prefix lives in `response_summary` alongside response status. V0.2 hardening: redact/hash-only. Documented in CLAUDE.md as v0.2 item. |
| T-02-10 | DoS | unbounded request body | mitigate | `axum::body::to_bytes(body, 1_048_576)` 1 MiB cap in audit middleware; overflow returns 413 and still writes an audit row recording the attempt (status=413). |
| T-02-11 | DoS | per-agent request flood | mitigate | `governor` rate limiter, default 100 rps per agent, configurable via `--rate-limit`. 429 + Retry-After. Integration test proves the denial path and that the denial itself is audited. |
| T-02-12 | Spoofing | agent identity via header | accept (v0.1) | `X-Reposix-Agent` is self-declared; no auth in v0.1 (documented in PROJECT.md out-of-scope). An adversary can rotate headers to dodge rate limits; audit log still captures each distinct agent_id for forensics. V0.2 adds bearer-token auth (per simulator-design.md §1.3). |
| T-02-13 | Info disclosure | 429 response leaks bucket state | accept | Retry-After: 1 is a coarse, non-sensitive hint. GitHub's public API does the same. |
| T-02-14 | DoS | governor DashMap grows unbounded across agent_ids | accept (v0.1) | Memory footprint per agent is small (a few hundred bytes); for v0.1 the simulator is single-tenant local-only. V0.2 adds an LRU eviction. |
</threat_model>

<verification>
Phase-level checks this plan closes:
- `cargo test -p reposix-sim` green (unit + integration).
- `cargo clippy -p reposix-sim --all-targets -- -D warnings` clean.
- ROADMAP Phase-2 Bash assertion 4: `sqlite3 <db> 'SELECT COUNT(*) FROM
  audit_events WHERE method IN ("GET","PATCH");'` >= 2 after curls; `UPDATE` fails
  with trigger error containing literal `"append-only"`.
- ROADMAP Phase-2 Bash assertion 5: integration test boots ephemeral port, issues a
  GET, asserts audit row was written.

Goal-backward Bash assertion (run after execution of this plan):
```bash
# Assertion: If we run the full Phase-2 success-criteria harness, all 5 pass.
cd /home/reuben/workspace/reposix
cargo build -p reposix-sim --release
DB=$(mktemp -u)
./target/release/reposix-sim --bind 127.0.0.1:17878 --db "$DB" \
  --seed-file crates/reposix-sim/fixtures/seed.json &
PID=$!; sleep 1

# SC1: list returns >=3
test "$(curl -sf http://127.0.0.1:17878/projects/demo/issues | python3 -c 'import sys,json;print(len(json.load(sys.stdin)))')" -ge 3 \
  || { kill $PID; echo "SC1 FAIL"; exit 1; }

# SC2: GET /projects/demo/issues/1 returns 200
test "$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:17878/projects/demo/issues/1)" = "200" \
  || { kill $PID; echo "SC2 FAIL"; exit 1; }

# SC3: PATCH /projects/demo/issues/1 with bogus If-Match returns 409
test "$(curl -s -o /dev/null -w '%{http_code}' -X PATCH \
  -H 'If-Match: "bogus"' -H 'Content-Type: application/json' \
  -d '{"status":"done"}' http://127.0.0.1:17878/projects/demo/issues/1)" = "409" \
  || { kill $PID; echo "SC3 FAIL"; exit 1; }

# SC4: audit rows written; UPDATE trigger fires with literal "append-only"
sleep 0.2
test "$(sqlite3 "$DB" "SELECT COUNT(*) FROM audit_events WHERE method IN ('GET','PATCH');")" -ge 2 \
  || { kill $PID; echo "SC4 count FAIL"; exit 1; }
sqlite3 "$DB" "UPDATE audit_events SET path='x' WHERE id=1;" 2>&1 \
  | grep -q "append-only" \
  || { kill $PID; echo "SC4 trigger FAIL"; exit 1; }

# SC5: integration tests pass
kill $PID; wait $PID 2>/dev/null
cargo test -p reposix-sim --test api -- --nocapture \
  || { echo "SC5 FAIL"; exit 1; }

echo "ALL FIVE SUCCESS CRITERIA PASS"
```
</verification>

<success_criteria>
- Audit middleware is the outermost layer; rate-limit wraps handlers.
- Audit rows use the fixture's exact columns: `ts`, `agent_id`, `method`, `path`,
  `status`, `request_body`, `response_summary`. No `timestamp`. No
  `request_body_hash`.
- Every request produces exactly one `audit_events` row (verified by integration
  test counting rows before/after a fixed number of calls).
- Rate limit denies overflow with 429 + Retry-After (verified by the overflow
  test) AND the denial itself is audited (verified by
  `rate_limited_request_is_audited`).
- Trigger UPDATE failure string contains the literal `"append-only"`.
- No `reqwest::Client::{new,builder}` call anywhere in the sim or its tests
  (clippy enforces; the test uses `reposix_core::http::client`).
- No `.await` inside a held `state.db.lock()` critical section.
- `parts.clone()` does not appear anywhere in `middleware/audit.rs` (Parts is not
  Clone in axum 0.7).
</success_criteria>

<output>
After completion, create
`.planning/phases/02-simulator-audit-log/02-02-SUMMARY.md` documenting: the
resolved audit table schema (the actual `ts`/`response_summary` columns from the
fixture), the exact RAISE text the trigger emits (`audit_events is append-only`),
the `response_summary` encoding (`"<status>:<sha256_hex_prefix_16>"`), the chosen
`governor` + `DashMap` wiring, any Cargo.toml additions (`sha2`, `hex`,
`tempfile`), and the Bash assertion harness used for goal-backward verification.
Plan 03-* reads this SUMMARY.
</output>
</content>
