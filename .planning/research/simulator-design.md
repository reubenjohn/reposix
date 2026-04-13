# Simulator Design — `reposix-sim`

**Audience:** the agent(s) who will implement `crates/reposix-sim` tonight.
**Mode:** ecosystem + feasibility, code-heavy. Confidence: HIGH for axum / governor / rusqlite shapes, MEDIUM for Jira workflow semantics (modeled conservatively from public docs), HIGH for GitHub Issues semantics (validated against `docs.github.com/en/rest/issues/issues`, API version `2026-03-10`).
**North star:** the StrongDM dark-factory pattern from `AgenticEngineeringReference.md` §1. A swarm of agent-shaped clients hammers `/projects/{slug}/issues/...` overnight; the simulator must be **fast, free, deterministic, and faithful enough that bugs caught here would also occur in production.**

The defining tension is *fidelity vs. velocity*. Every behavior in §2 is non-negotiable because each one corresponds to a class of bug that would otherwise only surface against a real backend (where we have no credentials, no quota, no time). Everything else — pagination edge cases, custom field types, rich text rendering — is explicitly out of v0.1.

---

## 0. TL;DR for the impatient implementer

```
crates/reposix-sim/
├── Cargo.toml             # axum 0.7, tower-governor, rusqlite (bundled), serde, etc.
├── src/
│   ├── main.rs            # binary entrypoint: `reposix-sim --db sim.db --port 7878`
│   ├── lib.rs             # `pub fn build_router(state: AppState) -> Router`
│   ├── state.rs           # AppState { db: Arc<Mutex<Connection>>, limiters, config }
│   ├── routes/
│   │   ├── projects.rs    # GET /projects, GET /projects/{slug}
│   │   ├── issues.rs      # GET/POST/PATCH/DELETE /projects/{slug}/issues[/id]
│   │   ├── transitions.rs # GET /projects/{slug}/issues/{id}/transitions, POST .../transition
│   │   ├── perms.rs       # GET /projects/{slug}/permissions
│   │   └── dashboard.rs   # GET / -> embedded HTML, GET /_audit -> JSON for the UI
│   ├── middleware/
│   │   ├── audit.rs       # tower::Layer that writes one row per request
│   │   ├── rate_limit.rs  # GovernorLayer wrapper keyed on `X-Agent-Token`
│   │   ├── etag.rs        # If-Match / If-None-Match handling
│   │   ├── chaos.rs       # latency injection, fault injection, controlled by config
│   │   └── auth.rs        # bearer-token -> agent_id + role
│   ├── db/
│   │   ├── schema.sql     # embedded via include_str!
│   │   ├── seed.rs        # deterministic seeding from a u64 RNG seed
│   │   └── audit.rs       # append-only audit-log writer
│   ├── domain/
│   │   ├── issue.rs       # Issue, IssueState, IssuePatch
│   │   ├── workflow.rs    # transition table, validate(from, to) -> Result
│   │   └── rbac.rs        # Role, Permission, can(role, action)
│   └── ui/
│       └── index.html     # vibe-coded dashboard, ~200 lines, no build step
└── tests/
    ├── contract.rs        # property-style tests vs. real GitHub public API
    └── workflow.rs        # state-machine tests for transitions
```

Build target: a single `reposix-sim` binary with `--db`, `--port`, `--seed`, `--chaos`, `--rate-limit` flags. Boots in under 100 ms. SQLite file is the only on-disk state.

---

## 1. Endpoint surface for v0.1

The naming compromise: **GitHub-shaped paths and bodies** (because they are simpler and the agent population already knows them from pre-training), **Jira-shaped workflow semantics** (because that is where the interesting bug surface lives). Multi-project is first-class: every path is rooted at `/projects/{slug}`.

### 1.1 Endpoint table

| Method   | Path                                                     | Purpose                              | Auth?  | Idempotent? | Notes                                      |
|----------|----------------------------------------------------------|--------------------------------------|--------|-------------|--------------------------------------------|
| `GET`    | `/projects`                                              | List projects                        | yes    | yes         | paginated                                  |
| `GET`    | `/projects/{slug}`                                       | Get project                          | yes    | yes         | returns project metadata + workflow        |
| `GET`    | `/projects/{slug}/issues`                                | List issues                          | yes    | yes         | filters: `state`, `assignee`, `label`, `since`, `per_page`, `page` |
| `POST`   | `/projects/{slug}/issues`                                | Create issue                         | yes    | with `Idempotency-Key` | returns 201 + `Location`                   |
| `GET`    | `/projects/{slug}/issues/{number}`                       | Get issue                            | yes    | yes         | sets `ETag`, honors `If-None-Match` -> 304 |
| `PATCH`  | `/projects/{slug}/issues/{number}`                       | Update issue (non-state fields)      | yes    | yes (with If-Match) | requires `If-Match`; 409 on stale          |
| `DELETE` | `/projects/{slug}/issues/{number}`                       | Soft-delete (state=deleted)          | yes    | yes         | admins only                                |
| `GET`    | `/projects/{slug}/issues/{number}/transitions`           | List legal next states               | yes    | yes         | Jira-style                                 |
| `POST`   | `/projects/{slug}/issues/{number}/transitions`           | Apply a workflow transition          | yes    | with `Idempotency-Key` | 409 if illegal transition or stale ETag    |
| `GET`    | `/projects/{slug}/permissions`                           | Effective permissions for caller     | yes    | yes         | returns role + boolean grid                |
| `GET`    | `/_health`                                               | Liveness                             | no     | yes         | returns `{ "ok": true, "build": "..." }`   |
| `GET`    | `/_audit?since=...&limit=...`                            | Recent requests, JSON                | yes    | yes         | for the dashboard; admin-only              |
| `GET`    | `/`                                                      | Embedded dashboard HTML              | no     | yes         | UI in §4                                   |

`Authorization: Bearer <token>` for everything except `/_health` and `/`. The token is just a UUID stored in a small `agents` table; no JWT, no expiry. The whole point is to be a substrate, not a security boundary.

### 1.2 JSON shapes

The objective is for an `Issue` to round-trip cleanly into the YAML-frontmatter `.md` representation that the FUSE layer expects. Field names are lowercase snake_case, matching GitHub.

#### `Project`
```json
{
  "slug": "reposix",
  "name": "reposix",
  "description": "git-backed FUSE for issue trackers",
  "created_at": "2026-04-13T00:30:00Z",
  "workflow": {
    "states": ["open", "in_progress", "in_review", "done", "closed"],
    "transitions": [
      { "id": "start",    "from": "open",        "to": "in_progress" },
      { "id": "review",   "from": "in_progress", "to": "in_review"   },
      { "id": "complete", "from": "in_review",   "to": "done"        },
      { "id": "close",    "from": "done",        "to": "closed"      },
      { "id": "reopen",   "from": "closed",      "to": "open"        },
      { "id": "drop",     "from": "in_progress", "to": "open"        }
    ],
    "default_state": "open"
  }
}
```

#### `Issue` (response)
```json
{
  "id": 1042,
  "number": 17,
  "project": "reposix",
  "title": "FUSE getattr returns ENOENT for newly-created files",
  "body": "Steps to reproduce...\n",
  "state": "in_progress",
  "state_reason": null,
  "labels": ["bug", "fuse"],
  "assignees": ["agent-alice"],
  "author": "agent-bob",
  "version": 4,
  "created_at": "2026-04-13T01:02:03Z",
  "updated_at": "2026-04-13T03:14:15Z",
  "closed_at": null,
  "url": "http://127.0.0.1:7878/projects/reposix/issues/17",
  "etag": "W/\"reposix:17:4\""
}
```

`version` is a monotonically-increasing integer per issue. The `ETag` HTTP header carries `W/"{slug}:{number}:{version}"`; the body field `etag` mirrors it for clients (like the FUSE layer) that don't bother parsing headers. Mirroring removes a class of "I forgot to read the header" bugs.

#### `CreateIssueRequest` (POST body)
```json
{
  "title": "string, required, 1..256 chars",
  "body": "string, optional, default empty",
  "labels": ["bug", "ui"],
  "assignees": ["agent-alice"],
  "initial_state": "open"
}
```
- Returns `201 Created`, `Location: /projects/{slug}/issues/{number}`, body = full Issue.
- 422 if title missing/too long. 400 if `initial_state` not in workflow. 403 if RBAC denies.

#### `PatchIssueRequest` (PATCH body)
```json
{
  "title": "optional string",
  "body": "optional string",
  "labels": ["optional", "array"],
  "assignees": ["optional", "array"],
  "state_reason": "optional, e.g. 'duplicate'"
}
```
- **State changes via PATCH are forbidden.** State only changes via `/transitions` so the workflow rules in §2 cannot be bypassed. PATCH that includes `"state"` returns 400 with `{ "error": "use POST /transitions to change state" }`. This is intentionally stricter than GitHub and matches Jira; agents will discover the transitions endpoint via the 400 error message and the embedded `/projects/{slug}` workflow metadata.

#### `Transition list` (GET `.../transitions`)
```json
{
  "current_state": "in_progress",
  "available": [
    { "id": "review", "to": "in_review", "name": "Send for review" },
    { "id": "drop",   "to": "open",      "name": "Drop back to backlog" }
  ]
}
```
Note `available` is computed against the workflow definition AND the caller's RBAC. A read-only agent gets `available: []` even when transitions exist. This is the Jira-shaped endpoint that `git-remote-reposix` will translate to a "valid moves" hint in conflict messages.

#### `Apply transition` (POST `.../transitions`)
```json
{ "transition": "review", "comment": "ready for the swarm" }
```
- 200 + full Issue on success, with bumped `version`.
- 409 + `{ "error": "illegal_transition", "from": "in_progress", "to": "done", "via": "review" }` if the transition is not legal from the current state.
- 409 + `{ "error": "stale_etag", "expected": "W/\"reposix:17:4\"", "actual": "W/\"reposix:17:5\"" }` if `If-Match` doesn't equal the current version.

#### `Permissions` (GET `.../permissions`)
```json
{
  "agent": "agent-alice",
  "role": "contributor",
  "can": {
    "issues.read": true,
    "issues.create": true,
    "issues.update": true,
    "issues.delete": false,
    "issues.transition.start": true,
    "issues.transition.review": true,
    "issues.transition.complete": false
  }
}
```

### 1.3 Headers and conventions

These conventions are mandatory because they are what the FUSE+git-remote layers will rely on.

| Header                    | On request? | On response? | Behavior                                                                 |
|---------------------------|-------------|--------------|---------------------------------------------------------------------------|
| `Authorization`           | yes         | —            | `Bearer <agent-token>`. Required everywhere except `/_health` and `/`.    |
| `X-RateLimit-Limit`       | —           | yes          | Configured limit, e.g. `5000`                                             |
| `X-RateLimit-Remaining`   | —           | yes          | Tokens left in the bucket for this caller                                 |
| `X-RateLimit-Reset`       | —           | yes          | UNIX seconds when the bucket is fully replenished                         |
| `Retry-After`             | —           | on 429       | Seconds the caller should wait                                            |
| `ETag`                    | —           | on issues    | `W/"{slug}:{number}:{version}"`                                           |
| `If-None-Match`           | yes         | —            | On GET; returns 304 if equal (does not consume a token, see §2.1)         |
| `If-Match`                | required    | —            | On PATCH and POST `/transitions`; 412 if missing, 409 if stale            |
| `Idempotency-Key`         | optional    | —            | UUID. Same key + same body within 24h returns the cached response         |
| `X-Request-Id`            | optional    | yes (always) | Echoed if provided, generated otherwise. Joins request to audit row.      |
| `X-Sim-Chaos`             | —           | optional     | `latency=120ms`, `injected=429`. Lets the swarm harness assert on chaos.  |

GitHub's behavior for 304 returning no rate-limit cost is documented at `docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api`; we copy it because it incentivizes the FUSE cache to use ETags. That cache hygiene is the *entire reason* the simulator can survive a 10k-agent swarm.

---

## 2. Behavior fidelity (the dark-factory non-negotiables)

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

---

## 3. State persistence — SQLite + WAL + audit log

**Why SQLite:** single file, zero ops, `rusqlite` with `bundled` feature avoids the `libsqlite3-dev` dependency we don't have apt access to install (per `PROJECT.md` §Constraints). WAL mode means readers don't block the writer, which matters under swarm load even though SQLite remains a single-writer database. WAL semantics are documented at `sqlite.org/wal.html`.

### 3.1 Connection setup

```rust
use rusqlite::{Connection, OpenFlags};
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn open_db(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
    )?;
    // PRAGMAs that matter for swarm load:
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "synchronous", &"NORMAL")?; // WAL + NORMAL is durable enough
    conn.pragma_update(None, "foreign_keys", &true)?;
    conn.pragma_update(None, "busy_timeout", &5000_i64)?; // ms
    conn.pragma_update(None, "temp_store", &"MEMORY")?;
    conn.execute_batch(include_str!("schema.sql"))?;
    Ok(conn)
}

pub type Db = Arc<Mutex<Connection>>;
```

We use `tokio::sync::Mutex`, not `std::sync::Mutex`, because handlers `await` between the lock acquire and DB ops. Per the axum FAQ (`docs.rs/axum`), holding `std::sync::Mutex` across `.await` produces `!Send` futures which axum rejects.

For v0.1 a single connection behind a mutex is fine — SQLite is single-writer regardless. If contention shows up in the swarm trace, the upgrade path is `r2d2_sqlite` for a small read pool plus one writer connection. Don't pre-optimize.

### 3.2 Schema (excerpt)

```sql
CREATE TABLE IF NOT EXISTS projects (
    slug         TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    description  TEXT NOT NULL DEFAULT '',
    workflow     TEXT NOT NULL,            -- JSON
    created_at   INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS agents (
    id          TEXT PRIMARY KEY,          -- 'agent-alice'
    token       TEXT UNIQUE NOT NULL,      -- bearer token
    role        TEXT NOT NULL CHECK (role IN ('viewer','contributor','admin')),
    created_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS issues (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    project_slug  TEXT NOT NULL REFERENCES projects(slug),
    number        INTEGER NOT NULL,
    title         TEXT NOT NULL,
    body          TEXT NOT NULL DEFAULT '',
    state         TEXT NOT NULL,
    state_reason  TEXT,
    labels        TEXT NOT NULL DEFAULT '[]',   -- JSON array
    assignees     TEXT NOT NULL DEFAULT '[]',   -- JSON array
    author        TEXT NOT NULL REFERENCES agents(id),
    version       INTEGER NOT NULL DEFAULT 1,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    closed_at     INTEGER,
    UNIQUE (project_slug, number)
);
CREATE INDEX idx_issues_project_state ON issues(project_slug, state);

CREATE TABLE IF NOT EXISTS audit_log (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    ts              INTEGER NOT NULL,           -- unix millis
    request_id      TEXT NOT NULL,
    agent_id        TEXT,                       -- null if anonymous
    method          TEXT NOT NULL,
    path            TEXT NOT NULL,
    query           TEXT,
    request_body    TEXT,                       -- truncated to 64 KB
    response_status INTEGER NOT NULL,
    response_body   TEXT,                       -- truncated to 64 KB
    duration_ms     INTEGER NOT NULL,
    rate_limit_remaining INTEGER,
    chaos_applied   TEXT                        -- JSON or null
);
CREATE INDEX idx_audit_ts ON audit_log(ts);
CREATE INDEX idx_audit_agent ON audit_log(agent_id, ts);
CREATE INDEX idx_audit_status ON audit_log(response_status, ts);
```

### 3.3 The audit log is the observability layer

`PROJECT.md` Active requirement: *"Audit log of every network-touching action. SQLite, queryable. Non-optional per OP #6 (ground truth)."*

Implementation: an `axum::middleware::from_fn` wrapper that runs **outermost** in the layer stack. It does *not* block the request on the DB write — it `tokio::spawn`s a write task with the captured fields. The downside (a crashing process loses the last few rows) is acceptable; the upside (handlers stay fast) is essential.

```rust
pub async fn audit_layer(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let request_id = req.headers().get("x-request-id")
        .and_then(|h| h.to_str().ok()).map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let method = req.method().clone();
    let uri = req.uri().clone();
    let agent_id = req.extensions().get::<AgentContext>().map(|c| c.id.clone());

    // Buffer body so we can record it AND pass it on. 64 KB cap.
    let (parts, body) = req.into_parts();
    let bytes = axum::body::to_bytes(body, 64 * 1024).await.unwrap_or_default();
    let req_body = String::from_utf8_lossy(&bytes).into_owned();
    let req = Request::from_parts(parts, axum::body::Body::from(bytes));

    let mut response = next.run(req).await;
    response.headers_mut().insert("x-request-id", request_id.parse().unwrap());

    let status = response.status().as_u16() as i64;
    let duration_ms = start.elapsed().as_millis() as i64;

    let db = state.db.clone();
    tokio::spawn(async move {
        let conn = db.lock().await;
        let _ = conn.execute(
            "INSERT INTO audit_log
                (ts, request_id, agent_id, method, path, query,
                 request_body, response_status, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                chrono::Utc::now().timestamp_millis(),
                request_id, agent_id, method.as_str(),
                uri.path(), uri.query(), req_body, status, duration_ms,
            ],
        );
    });
    response
}
```

Capturing the response body cleanly requires consuming and rebuilding it (similar pattern to the request body). For v0.1 we omit response-body capture in the hot path and reconstruct it from request + status when investigating. If it turns out we need full bidirectional capture for prompt-injection forensics, it's a one-evening upgrade.

---

## 4. Observability dashboard — vibe-coded, working

Following Simon Willison's "tiny Go binary with vibe-coded UI" pattern (`AgenticEngineeringReference.md` §1). Goals:

- Single embedded HTML file, no build step, no node_modules.
- Polls `/_audit?since={cursor}` every 2s.
- Shows: live request stream, conflict frequency, rate-limit hits, chaos events.
- Useful enough that a human glancing at it during the demo *immediately* sees the swarm working.

`src/ui/index.html` is roughly 200 lines: a single `<table>` for the request log, two Chart.js sparklines (loaded from CDN — fine for a localhost dev tool), and a controls bar to toggle chaos and reset the rate limiter. Embedded via `include_str!` and served at `/`. The widget shape:

```
+---------------------------------------------------------------+
| reposix-sim   [agents: 8]  [rps: 47]  [conflict-rate: 4.2%]   |
| chaos: latency [0..50ms] errors [0%/0%]   [reset]  [pause]    |
+---------------------------------------------------------------+
| sparkline: req/s last 5min     | sparkline: 409s last 5min    |
+---------------------------------------------------------------+
| time     agent          method path                  status   |
| 03:14:15 agent-alice    PATCH  /projects/reposix/... 409 (CONFLICT)
| 03:14:15 agent-bob      POST   /projects/reposix/... 201
| 03:14:14 agent-cara     GET    /projects/reposix/... 304
+---------------------------------------------------------------+
```

The dashboard is read-only over the network from outside the host — there is no auth on `/`, but the `/_audit` endpoint requires admin auth. That keeps a casual `curl 127.0.0.1:7878/` safe while gating the actual data. For a swarm demo we run the UI behind the same `--bind 127.0.0.1` as the API.

`tokio::sync::broadcast` channel is the optional upgrade for "live tail" — the audit middleware sends `AuditRow` over the channel; the dashboard subscribes via `GET /_audit/stream` (SSE). Skip for v0.1 unless polling proves laggy. Polling every 2s is fine for a dev tool.

---

## 5. Seed data — deterministic and realistic

The contract: `reposix-sim --seed 0xC0FFEE --db sim.db init` produces the exact same database every time. This is non-negotiable for reproducible tests and demo scripts.

### 5.1 Seeder shape

```rust
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct SeedConfig {
    pub seed: u64,
    pub n_projects: usize,
    pub n_agents_per_project: usize,
    pub n_issues_per_project: usize,
}

pub fn seed(conn: &mut Connection, cfg: SeedConfig) -> rusqlite::Result<()> {
    let mut rng = ChaCha8Rng::seed_from_u64(cfg.seed);
    let tx = conn.transaction()?;
    for p in 0..cfg.n_projects {
        let slug = format!("proj-{p:02}");
        // ... insert project ...
        for a in 0..cfg.n_agents_per_project {
            let role = match a {
                0 => "admin",
                1 | 2 => "viewer",
                _ => "contributor",
            };
            // ... insert agent with deterministic UUID derived from rng ...
        }
        for i in 1..=cfg.n_issues_per_project {
            // pick title from a fixed corpus of 200 phrases, indexed by rng
            // pick state weighted: 50% open, 25% in_progress, 15% in_review, 8% done, 2% closed
            // pick 0..3 labels from a fixed 10-label vocabulary
            // pick 0..2 assignees
            // assign created_at deterministically: base_ts + (i * 60s) + jitter
        }
    }
    tx.commit()
}
```

### 5.2 The default seed

`reposix-sim init` with no flags creates one project `reposix` with 50 issues, 8 agents (1 admin, 2 viewer, 5 contributor), and 5 labels. Just enough to make `ls /mnt/reposix/issues/` look interesting in the demo without being so much that scrolling becomes annoying.

### 5.3 Why `ChaCha8Rng` and not `StdRng`

`StdRng` is documented as not stable across rust releases. `ChaCha8Rng` from `rand_chacha` is. Test seeds need to outlive the crate version of the day they were filed.

---

## 6. Multi-project / multi-tenant

Already baked into the URL shape: `/projects/{slug}/...`. Multi-tenancy is achieved by:

1. **Foreign keys** — every issue has `project_slug`. The handler extracts `slug` from path, validates the project exists (404 otherwise), and scopes every query by `project_slug = ?`. There is no cross-project endpoint in v0.1; agents know they live in one project.
2. **Per-project workflow** — stored as JSON on the `projects` row. Agents `GET /projects/{slug}` to discover the rules. The FUSE layer caches this and the rules are advertised in `getxattr`.
3. **Per-project rate limiter buckets are NOT the design** — buckets are per-agent-token, not per-project. An agent that talks to two projects shares one bucket, mirroring real-world API behavior where the bucket is per credential.
4. **Per-project agent overrides** — table `agent_project_roles(agent_id, project_slug, role)` overrides the global role when present. Lets us model "alice is admin in proj-a, viewer in proj-b" without inventing a new role.

`/projects` listing returns only projects the caller has at least `viewer` access to. Slug collisions are forbidden by primary key; the seeder uses `proj-NN` to avoid them.

---

## 7. Concrete axum skeleton

Below is the spine. Implementer should expand handler bodies; types and wiring are the load-bearing parts.

### 7.1 `Cargo.toml` (relevant excerpts)

```toml
[dependencies]
axum = { version = "0.7", features = ["macros", "tokio", "json"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.5", features = ["trace", "cors"] }
tower-governor = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled", "chrono", "serde_json"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
rand = "0.8"
rand_chacha = "0.3"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4", features = ["derive"] }
sha2 = "0.10"

[dev-dependencies]
proptest = "1"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

`reqwest` with `rustls-tls` (no openssl-sys) is required by `PROJECT.md` §Dependencies.

### 7.2 `main.rs`

```rust
use clap::Parser;
use reposix_sim::{build_router, AppState, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let state = AppState::new(&cli)?;

    let app = build_router(state.clone());
    let addr = format!("{}:{}", cli.bind, cli.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "reposix-sim listening");
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await?;
    Ok(())
}
```

`into_make_service_with_connect_info` is required by tower-governor's IP-based extractors even though we use a custom one — keeping the connect info available means fallback to IP works for unauth endpoints.

### 7.3 `state.rs`

```rust
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use rusqlite::Connection;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub chaos: Arc<RwLock<crate::middleware::chaos::ChaosConfig>>,
    pub config: Arc<Config>,
}

pub struct Config {
    pub rate_limit_per_hour: u32,
    pub burst_size: u32,
    pub bind: String,
    pub port: u16,
}

impl AppState {
    pub fn new(cli: &Cli) -> anyhow::Result<Self> {
        let conn = crate::db::open_db(&cli.db)?;
        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            chaos: Arc::new(RwLock::new(Default::default())),
            config: Arc::new(Config {
                rate_limit_per_hour: cli.rate_limit,
                burst_size: cli.burst,
                bind: cli.bind.clone(),
                port: cli.port,
            }),
        })
    }
}
```

### 7.4 `lib.rs` — the router wiring

```rust
use axum::{Router, routing::{get, post, patch, delete}};
use tower_governor::GovernorLayer;
use tower::ServiceBuilder;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/projects", get(routes::projects::list))
        .route("/projects/:slug", get(routes::projects::get_one))
        .route("/projects/:slug/issues",
               get(routes::issues::list).post(routes::issues::create))
        .route("/projects/:slug/issues/:number",
               get(routes::issues::get_one)
                   .patch(routes::issues::patch)
                   .delete(routes::issues::delete))
        .route("/projects/:slug/issues/:number/transitions",
               get(routes::transitions::list).post(routes::transitions::apply))
        .route("/projects/:slug/permissions", get(routes::perms::effective))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::auth::auth_layer))
        .layer(GovernorLayer::new(governor_config(&state)))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::chaos::chaos_layer));

    let admin = Router::new()
        .route("/_audit", get(routes::dashboard::audit_json))
        .route("/_chaos", post(routes::dashboard::set_chaos))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::auth::admin_only));

    Router::new()
        .route("/", get(routes::dashboard::index_html))
        .route("/_health", get(|| async { axum::Json(serde_json::json!({"ok": true})) }))
        .merge(api)
        .merge(admin)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::audit::audit_layer))
        .with_state(state)
}
```

Layer ordering recap (axum applies outer-first on requests, inner-first on responses): audit > chaos > governor > auth > handler. Audit must wrap everything so even rejected requests are logged; chaos before governor so injected delays don't burn tokens; governor before auth so unauthenticated floods are throttled; auth last so handlers always have `AgentContext`.

### 7.5 A handler in full — `issues::patch`

```rust
pub async fn patch(
    State(state): State<AppState>,
    Path((slug, number)): Path<(String, i64)>,
    Extension(ctx): Extension<AgentContext>,
    headers: HeaderMap,
    Json(payload): Json<PatchIssueRequest>,
) -> Result<Response, ApiError> {
    ctx.require("issues.update")?;
    if payload.has_state_field() {
        return Err(ApiError::bad_request(
            "use POST /transitions to change state"));
    }

    let if_match = headers.get(IF_MATCH)
        .ok_or(ApiError::precondition_required("If-Match required for PATCH"))?
        .to_str().map_err(|_| ApiError::bad_request("If-Match not utf8"))?
        .to_owned();

    let mut conn = state.db.lock().await;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current = db::issues::fetch(&tx, &slug, number)?
        .ok_or(ApiError::not_found("issue"))?;
    let actual_etag = current.etag();
    if if_match != actual_etag {
        return Err(ApiError::conflict(json!({
            "error": "stale_etag",
            "expected": if_match,
            "actual": actual_etag,
        })));
    }
    let updated = db::issues::apply_patch(&tx, &current, &payload, &ctx.id)?;
    tx.commit()?;

    let mut resp = Json(&updated).into_response();
    resp.headers_mut().insert(ETAG, updated.etag().parse().unwrap());
    Ok(resp)
}
```

`ApiError` is a `thiserror`-derived enum that implements `IntoResponse`, mapping each variant to the JSON envelope `{ "error": "...", ... }`. Centralizing it keeps every handler under 30 lines.

---

## 8. Validating fidelity — contract harness vs. real GitHub

**The point:** if our simulator drifts from the real API, the swarm catches phantom bugs. We need a small, cheap, network-tolerant harness that asserts our simulator matches GitHub on the **shape** of the GET-issue path. We cannot match full semantics (we deliberately differ on workflow), but the data shapes for read endpoints should round-trip.

### 8.1 Strategy

For each test:
1. Pick a public, stable issue on a well-known repo. Suggestion: `octocat/Hello-World` issue #1, or one of the never-closed `rust-lang/rust` historical issues. They've existed for years and won't disappear before the demo.
2. `GET https://api.github.com/repos/octocat/Hello-World/issues/1` (no auth — public, stays under unauthenticated 60/hr limit).
3. `GET http://127.0.0.1:7878/projects/octocat-hello-world/issues/1` against a sim seeded with one issue copied from the GitHub fixture.
4. Assert: same set of top-level keys, same value types, same enum domains for `state`. Allow our extra keys (`etag`, `version`, `project`) and missing GitHub-only keys (`node_id`, `repository_url`, etc.) — we don't pretend to be wire-compatible, only schema-shaped.

### 8.2 Test code

```rust
// tests/contract.rs
use serde_json::Value;

const GITHUB_KEYS: &[&str] = &[
    "id", "number", "title", "body", "state", "labels",
    "assignees", "user", "created_at", "updated_at",
];
const SIM_RENAMES: &[(&str, &str)] = &[
    ("user", "author"),  // we call it author
];

#[tokio::test]
async fn github_issue_shape_matches_simulator() {
    // 1. fetch from real GitHub (skip if offline)
    let gh: Value = match reqwest::get(
        "https://api.github.com/repos/octocat/Hello-World/issues/1"
    ).await {
        Ok(r) if r.status().is_success() => r.json().await.unwrap(),
        _ => { eprintln!("skipping: GitHub unreachable"); return; }
    };

    // 2. spin up sim with that issue seeded
    let state = test_state_with_seed_from(&gh).await;
    let app = reposix_sim::build_router(state);
    let server = axum_test::TestServer::new(app).unwrap();

    let sim: Value = server
        .get("/projects/octocat-hello-world/issues/1")
        .add_header("authorization", "Bearer test-admin-token")
        .await
        .json();

    // 3. shape assertions
    for key in GITHUB_KEYS {
        let sim_key = SIM_RENAMES.iter()
            .find(|(g,_)| g == key).map(|(_,s)| *s).unwrap_or(key);
        let gh_val = &gh[key];
        let sim_val = &sim[sim_key];
        assert!(
            same_kind(gh_val, sim_val),
            "key {key}: github={gh_val:?} sim={sim_val:?}"
        );
    }
    // state enum
    assert!(matches!(
        sim["state"].as_str().unwrap(),
        "open" | "closed" | "in_progress" | "in_review" | "done"
    ));
    assert!(matches!(gh["state"].as_str().unwrap(), "open" | "closed"));
}

fn same_kind(a: &Value, b: &Value) -> bool {
    use Value::*;
    matches!((a, b),
        (Null, Null) | (Bool(_), Bool(_)) | (Number(_), Number(_)) |
        (String(_), String(_)) | (Array(_), Array(_)) | (Object(_), Object(_))
    )
}
```

### 8.3 Property-test fold-in

For workflow safety:

```rust
// tests/workflow.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn no_random_transition_sequence_reaches_done_without_in_review(
        seq in proptest::collection::vec(
            "(start|review|complete|close|reopen|drop)", 0..40
        )
    ) {
        let mut state = "open".to_string();
        let mut visited_in_review = false;
        for tid in &seq {
            if let Ok(t) = WORKFLOW.validate(&state, tid) {
                state = t.to.clone();
                if state == "in_review" { visited_in_review = true; }
            }
        }
        if state == "done" {
            prop_assert!(visited_in_review,
                "reached done without going through in_review: {seq:?}");
        }
    }
}
```

This is the kind of test that earns its keep — it asserts an invariant that humans will *believe* but might not test, and it generates inputs faster than humans can think them up. Cite Luke Palmieri's testing material at `lpalmieri.com` for the broader pattern of using property tests to lock down API invariants.

### 8.4 What this harness explicitly does NOT cover

- Authentication (we don't have GitHub credentials in autonomous mode).
- Pagination (next-link headers are GitHub-specific; we keep it simple).
- Comments, reactions, sub-issues — out of scope for v0.1.
- Markdown rendering — `body` is opaque text in both APIs.

If the implementer has spare cycles, the next contract test to add is "POST → GET round-trip preserves field types" against the simulator alone, which doesn't need network and runs in CI.

---

## 9. Sources

Authoritative (HIGH confidence):
- axum docs and discussions: `docs.rs/axum/latest/axum/`, GitHub discussion #964 on shared SQLite state, discussion #1758 on multi-field state, `docs.rs/axum/latest/axum/extract/struct.State.html`.
- tower-governor: `docs.rs/tower_governor/latest/tower_governor/`, `github.com/benwis/tower-governor`, `lib.rs/crates/tower_governor`.
- wiremock-rs (design pattern reference, not a dependency): `github.com/LukeMathWalker/wiremock-rs`, `docs.rs/wiremock/`, `lpalmieri.com/posts/2020-04-13-wiremock-async-http-mocking-for-rust-applications/`.
- GitHub Issues REST API: `docs.github.com/en/rest/issues/issues` (API version `2026-03-10` per page header). Rate-limit and conditional-request semantics: `docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api`.
- SQLite WAL: `sqlite.org/wal.html`, `sqlite.org/walformat.html`.

MEDIUM confidence (pattern based on training data + cross-checked with public docs, not deeply re-fetched here):
- Jira workflow transitions endpoint shape (`/rest/api/3/issue/{key}/transitions`). Atlassian developer portal renders JS-heavy; the transitions list/apply shape is well-known and modeled conservatively. If the implementer wants belt-and-braces, fetch the page through playwright before hardening the sim's transitions schema.
- Stripe-style idempotency-key semantics (return cached on hit, 422 on key reuse with different body). This is a widely-replicated pattern; cite Stripe's own docs if/when adding to README.

LOW confidence flags:
- "5000/hr matches GitHub authenticated" — true today (2026-04), but real product limits drift. The simulator exposes the limit as a `--rate-limit` flag rather than baking it in.
- "tower-governor 0.6 line is the right pin" — train-data shows `tower_governor` versions in flux. The implementer should check `lib.rs/crates/tower_governor` at build time and pin to whatever current minor version supports the `KeyExtractor` trait shape used in §2.1. If the API has shifted, the migration is small and the conceptual design doesn't change.

---

## 10. Implications for the roadmap

If the orchestrator is using this report to shape phases:

1. **Phase: simulator core** — schema, AppState, /projects + /issues GET/POST/PATCH/DELETE, audit middleware, rate-limit middleware, auth middleware. ~half of the remaining build budget.
2. **Phase: workflow + RBAC + idempotency** — `/transitions`, `/permissions`, idempotency table, the conflict 409 path. The dark-factory-defining behaviors. Should land before the FUSE layer is wired so FUSE has something real to talk to.
3. **Phase: dashboard + chaos + seeder** — vibe-coded UI, chaos config endpoint, deterministic seed. Demo-facing polish.
4. **Phase: contract harness** — §8 tests. Should run in CI, including a `--ignored` real-network test that the demo runs once during walkthrough.

Phases 1 and 2 are the load-bearing ones. Phase 3 is what makes the demo *legible*; Phase 4 is what lets us claim "faithful enough."

The biggest risk to overnight delivery: getting fancy with §3.3's response-body capture or §4's SSE streaming. Both are clearly-marked "skip for v0.1" — if the implementer is tempted to do them, that's the time to stop and ship.
