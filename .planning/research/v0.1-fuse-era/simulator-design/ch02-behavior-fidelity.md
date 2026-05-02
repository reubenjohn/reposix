# Behavior fidelity (the dark-factory non-negotiables)

← [back to index](./index.md)

Each subsection below names a class of production bug that the simulator MUST be able to reproduce. If the implementation cannot trip these failure modes, the swarm proves nothing.

### 2.1 Token-bucket rate limiting — `tower-governor`

**Bug class caught:** caller burns budget on `grep -r` against the FUSE mount, `getattr` storms the backend, swarm dies at hour 1.

`tower-governor` (v0.6 line) is the right pick: GCRA-based, integrates with axum via `GovernorLayer`, supports custom key extractors. The peer-IP default is wrong for us; we want **per-agent-token** buckets so two agents on `127.0.0.1` are billed separately.

```rust
use tower_governor::{
    governor::{GovernorConfigBuilder},
    key_extractor::KeyExtractor,
    GovernorError, GovernorLayer,
};
use axum::http::Request;
use std::net::IpAddr;

#[derive(Clone)]
pub struct AgentTokenKeyExtractor;

impl KeyExtractor for AgentTokenKeyExtractor {
    type Key = String;

    fn extract<B>(&self, req: &Request<B>) -> Result<Self::Key, GovernorError> {
        req.headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_owned())
            .ok_or(GovernorError::UnableToExtractKey)
    }
}

// 5000/hr like GitHub. burst_size = 50 keeps short bursts feeling instant.
// per_millisecond(720) ≈ 5000 tokens per 3_600_000 ms.
let governor_conf = GovernorConfigBuilder::default()
    .key_extractor(AgentTokenKeyExtractor)
    .per_millisecond(720)
    .burst_size(50)
    .finish()
    .expect("rate-limit config");

let app = Router::new()
    .merge(routes())
    .layer(GovernorLayer::new(governor_conf));
```

Two essentials beyond the basic example from `docs.rs/tower_governor`:

1. **Override the response**: `tower-governor` returns 429 with a plain body. We need `Retry-After` and a JSON envelope. Wrap `GovernorLayer` in a small `map_response` that, on 429, injects the headers and serializes `{ "error": "rate_limited", "retry_after_secs": N }`. The headers must also be added on **2xx** responses (so callers can tune their own pacing). Adding them on 2xx requires a tiny extra middleware that reads the limiter state — `governor::RateLimiter::check_key` returns the snapshot needed.

2. **304s are free.** When the etag middleware (§2.2) returns 304, do it *before* GovernorLayer in the layer stack so the bucket is not decremented. In axum, layer order is "outermost first": apply the etag layer with `.layer()` after `GovernorLayer`. (Cite `docs.rs/axum/latest/axum/middleware/index.html` on layer ordering.)

The configurability matters: the swarm harness sets `--rate-limit 100` to deliberately starve agents and observe their backoff behavior. This is "chaos-as-config" rather than chaos-as-incident.

### 2.2 409 Conflict on stale ETag — must trip git merge conflicts

**Bug class caught:** two agents `git push` overlapping edits to `PROJ-17.md`; the second push silently overwrites instead of producing a conflict.

```rust
// Pseudocode in the PATCH handler
let if_match = headers.get("if-match")
    .ok_or(StatusCode::PRECONDITION_REQUIRED)?;
let current = db.fetch_issue(slug, number).await?;
let actual_etag = format!("W/\"{}:{}:{}\"", slug, number, current.version);
if if_match.as_bytes() != actual_etag.as_bytes() {
    return Err(Conflict {
        expected: if_match.to_str()?.into(),
        actual: actual_etag,
    }.into_response());
}
let updated = db.apply_patch(slug, number, patch, current.version + 1).await?;
```

The DB write must be atomic with the version check. With SQLite + WAL, wrap both in a single `IMMEDIATE` transaction:

```rust
let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
let row_version: i64 = tx.query_row(
    "SELECT version FROM issues WHERE project=?1 AND number=?2",
    params![slug, number], |r| r.get(0))?;
if row_version != expected { return Err(Conflict { .. }); }
tx.execute("UPDATE issues SET ..., version=?1 WHERE id=?2",
    params![row_version + 1, id])?;
tx.commit()?;
```

The 409 body shape (`{error, expected, actual}`) is what `git-remote-reposix` parses to synthesize the `<<<<<<<` / `=======` / `>>>>>>>` conflict block. That parser will be in `crates/reposix-remote`; the simulator's contract is that the JSON shape is stable.

### 2.3 Workflow rules — Jira-style state machine

**Bug class caught:** an agent jumps `Open` directly to `Done`, skipping QA, and the FUSE layer happily writes `state: done` to frontmatter.

The workflow lives in `domain/workflow.rs` as a typed table per project, loaded from a JSON column on `projects`:

```rust
pub struct Workflow {
    pub states: Vec<String>,
    pub transitions: Vec<Transition>, // {id, from, to}
    pub default_state: String,
}

impl Workflow {
    pub fn legal(&self, from: &str) -> Vec<&Transition> {
        self.transitions.iter().filter(|t| t.from == from).collect()
    }
    pub fn validate(&self, from: &str, transition_id: &str)
        -> Result<&Transition, WorkflowError>
    {
        self.transitions.iter()
            .find(|t| t.id == transition_id && t.from == from)
            .ok_or(WorkflowError::Illegal { from: from.into(), via: transition_id.into() })
    }
}
```

The default workflow (§1.2 `Project`) deliberately requires `open -> in_progress -> in_review -> done`. There is no shortcut. The state-machine tests in `tests/workflow.rs` use `proptest` to assert: for any random sequence of transitions starting at `default_state`, the final state is reachable via the transitions' graph.

### 2.4 RBAC — some agents are read-only

**Bug class caught:** the audit-only agent successfully `git push`es a state change because nothing checked permissions.

Three roles are enough for v0.1. Bigger taxonomies are a YAGNI trap.

| Role          | issues.read | issues.create | issues.update | issues.delete | transitions      |
|---------------|-------------|---------------|---------------|---------------|------------------|
| `viewer`      | yes         | no            | no            | no            | none             |
| `contributor` | yes         | yes           | yes           | no            | all except `close`/`reopen` |
| `admin`       | yes         | yes           | yes           | yes           | all              |

Roles are stored on `agents`. The `auth` middleware resolves `Authorization` to an `AgentContext { id, role, project_overrides }` and inserts it into request extensions. Each handler does `ctx.require("issues.update")?` at the top. A central `permissions::check(role, action)` keeps the matrix in one place.

The `/permissions` endpoint exists primarily so the FUSE layer can translate roles into POSIX modes when computing `getattr` — `viewer` -> 0444, `contributor` -> 0644, `admin` -> 0664. That translation lives in `reposix-fuse`; the simulator's job is to expose the truth.

### 2.5 Latency injection / chaos modes

**Bug class caught:** the FUSE layer assumes API calls return in <10 ms and uses unbounded channels; under realistic latency, queues balloon and OOM the daemon.

A single `ChaosConfig` lives in `AppState`, hot-reloadable via `POST /_chaos` (admin-only):

```rust
#[derive(Clone, Default, Deserialize)]
pub struct ChaosConfig {
    pub latency_ms_min: u64,           // default 0
    pub latency_ms_max: u64,           // default 0
    pub error_rate_5xx: f32,           // 0.0..1.0; coin-flip 503
    pub error_rate_429: f32,           // independent forced rate-limit
    pub jitter_seed: Option<u64>,      // deterministic chaos when set
}
```

A tower middleware reads it on each request. Latency is a `tokio::time::sleep` between `min` and `max` chosen from a `SmallRng` keyed by `jitter_seed.unwrap_or(rand::random())`. Setting `jitter_seed` makes overnight runs reproducible: same seed + same request sequence -> same chaos timeline. This is the property that lets us file bugs against the swarm and re-run them.

Important: **chaos applies after auth and before the route handler**, so authentic 401s are never overridden by injected 503s. Otherwise debugging gets confusing.

### 2.6 Idempotency keys

**Bug class caught:** a network blip causes the agent to retry POST `/issues`; we end up with two issues titled "Fix the bug".

```sql
CREATE TABLE idempotency (
    key          TEXT NOT NULL,
    agent_id     TEXT NOT NULL,
    method       TEXT NOT NULL,
    path         TEXT NOT NULL,
    body_sha256  BLOB NOT NULL,
    response_status   INTEGER NOT NULL,
    response_body     TEXT NOT NULL,
    response_headers  TEXT NOT NULL,    -- JSON
    created_at   INTEGER NOT NULL,      -- unix seconds
    PRIMARY KEY (key, agent_id)
);
CREATE INDEX idx_idempotency_created ON idempotency(created_at);
```

Logic for any mutation that accepts `Idempotency-Key`:

1. If header missing -> proceed normally (best-effort idempotency, like Stripe).
2. Compute `body_sha256`.
3. `SELECT ... FROM idempotency WHERE key=? AND agent_id=?`.
   - Hit + same body_sha256 -> return cached `(status, headers, body)`.
   - Hit + different body_sha256 -> 422 `{ "error": "idempotency_key_reused_with_different_body" }`. This is what real APIs do; replicating it catches a real bug class.
   - Miss -> execute, then INSERT the result.
4. A daily checkpoint (or on startup) deletes rows older than 24h.

The `idempotency` table sits in the same SQLite DB; insertion is part of the same transaction as the mutation so a crash mid-write cannot leave a "cached response for an action that didn't happen."
