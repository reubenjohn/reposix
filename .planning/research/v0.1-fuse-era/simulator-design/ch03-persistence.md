# State persistence — SQLite + WAL + audit log

← [back to index](./index.md)

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
