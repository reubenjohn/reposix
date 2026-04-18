---
phase: 02-simulator-audit-log
plan: 02
subsystem: simulator-middleware
status: complete
tasks: 2/2
commits:
  - 0eb6eb4  # feat(02-02): audit middleware + body capture + DB write
  - 171c775  # feat(02-02): rate-limit layer + run_with_listener + integration tests
requirements:
  - FC-06
  - SG-06
roadmap_success_criteria:
  SC1: PASS
  SC2: PASS
  SC3: PASS
  SC4: PASS
  SC5: PASS
tests:
  unit_sim: 26   # +4 audit, +2 rate_limit over plan 02-01's 20
  integration_sim: 3
---

# Phase 2 Plan 02: Audit Middleware + Rate Limit Summary

One-liner: The simulator now writes one append-only audit row per HTTP
request (including 429s and 413s), rate-limits per-agent via `governor`, and
ships an integration test suite that closes all five ROADMAP Phase-2 SC.

## Shipped

### Audit schema (matches fixture EXACTLY)

Columns (from `crates/reposix-core/fixtures/audit.sql`):

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
```

INSERT shape used by the middleware (7 columns, in order):

```sql
INSERT INTO audit_events (ts, agent_id, method, path, status, request_body, response_summary)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
```

**No `timestamp` column. No `request_body_hash` column.** The fixture is
the source of truth.

### RAISE message (literal the test asserts on)

```
audit_events is append-only
```

The integration test assertion:

```rust
let msg = err.to_string();
assert!(msg.contains("append-only"),
        "trigger error must contain literal `append-only`; got {msg:?}");
```

### `response_summary` encoding

```
"<status>:<sha256_hex_prefix_16>"
```

Where:
- `status` = response status code as u16 decimal (`"200"`, `"204"`, `"413"`, `"429"`, …).
- `sha256_hex_prefix_16` = `&hex::encode(Sha256::digest(&request_body_bytes))[..16]`.

Examples:
- 200 with body `"hello"` → `"200:2cf24dba5fb0a30e"`.
- 204 with empty body → `"204:e3b0c44298fc1c14"`.
- 429 with empty body → `"429:e3b0c44298fc1c14"`.

Empty-body hash (sha256("")) starts with `e3b0c44298fc1c14`.

### `request_body` encoding

First 256 **chars** (not bytes) of the body via
`String::from_utf8_lossy(&bytes).chars().take(256).collect()`. Non-UTF-8
bodies survive as the lossy replacement.

### `agent_id` encoding

`X-Reposix-Agent` request header value, else `"anonymous"`. Header is
client-self-declared (no auth in v0.1 — T-02-12 accepted).

### `ts` encoding

`chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)` →
`"2026-04-13T12:34:56Z"`.

### Governor wiring

- Quota: `Quota::per_second(NonZeroU32::new(rps.max(1)).unwrap())` — burst
  is `rps`.
- Map: `Arc<DashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState,
  DefaultClock, NoOpMiddleware>>>>` keyed by `X-Reposix-Agent` header
  (default `"anonymous"`).
- Denial: 429 with JSON
  `{"error":"rate_limited","retry_after_secs":1}` + header `Retry-After: 1`.
- Lives INSIDE the audit layer, so even 429 responses get audited (the
  `rate_limited_request_is_audited` integration test proves this).

### Layer ordering

```rust
let handlers = Router::new()
    .route("/healthz", get(healthz))
    .merge(routes::router(state.clone()));
let with_rate_limit = middleware::rate_limit::attach(handlers, rps);
middleware::audit::attach(with_rate_limit, state)
```

Axum `.layer()` wraps: last `.layer()` is outermost → **audit → rate_limit →
handlers**. Every request (including 429s) goes through the audit layer.

### Cargo.toml additions

| Crate          | Section           | Added                  |
|----------------|-------------------|------------------------|
| reposix-sim    | `[dependencies]`  | `sha2 = "0.10"`, `hex = "0.4"` |
| reposix-sim    | `[dev-dependencies]` | `tempfile = "3"`   |
| reposix-core   | (method only)     | `HttpClient::request_with_headers_and_body` |

### Goal-backward harness

`scripts/phase2_goal_backward.sh` — replays the full ROADMAP Phase-2
SC1–SC5 harness against a freshly-built sim. Output:

```
SC1 PASS (list length=3)
SC2 PASS (GET 200)
SC3 PASS (PATCH bogus If-Match -> 409)
SC4 count PASS (4 audit rows for GET/PATCH)
SC4 trigger PASS (UPDATE blocked with 'append-only')
SC5 PASS (3 integration tests green)
ALL FIVE SUCCESS CRITERIA PASS
```

## Deviations from plan

- **Rule 2 — added `request_with_headers_and_body` in reposix-core.** The
  plan's integration test spec needs PATCH with BOTH a custom header
  (If-Match) AND a request body. The existing `HttpClient` API only had
  `request_with_headers` (no body) and `post`/`patch` (no headers). The
  smallest correctness fix was a new method that takes both. Allowlist
  gate runs BEFORE body serialization, preserving SG-01. Documented with
  a `# Errors` block pointing at the same error set as the sibling method.
- **Rule 2 — added rate-limit sanity unit tests.** Plan's `<behavior>`
  specified three integration tests but no unit tests for the layer
  itself. Added two fast unit tests (deny-on-second-call, per-agent
  buckets) so regressions show up in milliseconds, not after a 5s sim
  boot.
- **`attach(router, state)` instead of `layer(state)`.** The layer type
  from `from_fn_with_state` is unnameable in stable Rust without TAIT;
  `attach` hides the type behind a generic `Router -> Router` transform.
  Call sites remain ergonomic.
- **`scripts/phase2_goal_backward.sh` promoted.** The plan's
  `<verification>` block contained a large inline Bash harness that would
  trip the ad-hoc-bash hook. Promoted to a committed script per CLAUDE.md
  §4.

## Tests

| Suite                              | Count |
|------------------------------------|-------|
| sim lib unit tests                 | 26    |
| sim integration tests (tests/api.rs) | 3   |
| **Phase 2 total**                  | 29    |

All green. `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Known stubs

None. Plan 02-02 closes all its own TODOs. A single comment remains in
`middleware/audit.rs` noting that Phase 3 will introduce the `Tainted<T>`
wrapping around the captured `request_body` before egress paths use it —
that's per-design, not a regression.

## Threat flags

None — all new surface is accounted for in plan's `<threat_model>`
(T-02-07 through T-02-14).

## Self-Check: PASSED

- Files exist: middleware/{audit,rate_limit,mod}.rs, tests/api.rs,
  scripts/phase2_goal_backward.sh — all present.
- Commits exist: 0eb6eb4 (task 1), 171c775 (task 2).
- 29 tests pass.
- Goal-backward harness output: "ALL FIVE SUCCESS CRITERIA PASS".
