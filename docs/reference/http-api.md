# HTTP API (simulator)

The `reposix-sim` crate exposes a REST API shaped like a hybrid of GitHub Issues and Jira. Any reposix-compatible backend (including real Jira/GitHub in v0.2) must conform to this surface.

## Endpoints

### `GET /healthz`

Liveness probe. Returns `200 ok` as plain text. Not rate-limited.

### `GET /projects/:slug/issues`

List all issues in a project. Returns a JSON array.

```json
[
  {
    "frontmatter": {
      "id": 1,
      "title": "database connection drops under load",
      "status": "open",
      "labels": ["bug", "p1"],
      "created_at": "2026-04-13T00:00:00Z",
      "updated_at": "2026-04-13T00:00:00Z",
      "version": 1
    },
    "body": "Steps to reproduce: ..."
  },
  ...
]
```

### `GET /projects/:slug/issues/:id`

Fetch a single issue. Same shape as one element of the list response.

### `POST /projects/:slug/issues`

Create a new issue. Body: a partial frontmatter (`title` is required; server assigns `id`, `version`, `created_at`, `updated_at` — any client-supplied values are stripped by SG-03 at the client side).

```bash
curl -X POST http://127.0.0.1:7878/projects/demo/issues \
    -H 'Content-Type: application/json' \
    -H 'X-Reposix-Agent: manual-test' \
    -d '{"title":"thing broke","status":"open","body":"details"}'
```

Response: `201 Created` with the server-assigned representation.

### `PATCH /projects/:slug/issues/:id`

Partial update. Honors `If-Match: "<version>"` header per [RFC 7232](https://datatracker.ietf.org/doc/html/rfc7232):

- Absent → wildcard match (accepted).
- Present and matches current `version` → apply, bump `version`, update `updated_at`.
- Present but stale → `409 Conflict` with body:

```json
{
  "error": "version_mismatch",
  "current": 3,
  "sent": 1
}
```

This is the path that becomes a git merge conflict via `git-remote-reposix`.

### `DELETE /projects/:slug/issues/:id`

Remove an issue. Returns `204 No Content`.

### `GET /projects/:slug/issues/:id/transitions`

Jira-style: list currently-legal status transitions. In v0.1 all 5 `IssueStatus` values are always legal (the simulator does not enforce workflow rules); v0.2 will model real transitions.

## Headers

| Header | Purpose |
|--------|---------|
| `X-Reposix-Agent` | Logical agent identity. Required for audit attribution. Both `reposix-fuse` and `git-remote-reposix` set this to `<binary-name>-{pid}`. The simulator uses it as the rate-limit bucket key (see [SG-* deferred to v0.2](../security.md#whats-deferred-to-v02) — spoofing is a known gap). |
| `If-Match` | RFC 7232 quoted ETag. Used on `PATCH` for optimistic concurrency. |
| `Content-Type: application/json` | Required on PATCH/POST. |

## Error envelope

All non-2xx responses carry a JSON body of the form:

```json
{
  "error": "kind_identifier",
  "message": "human readable summary",
  "details": { /* kind-specific */ }
}
```

Kinds: `version_mismatch`, `not_found`, `invalid_body`, `rate_limited`, `payload_too_large` (413), `internal`.

## Rate limiting

Per-agent bucket keyed by `X-Reposix-Agent`. Default 100 req/sec with 100-request burst. Overflow returns `429 Too Many Requests` with `Retry-After: <seconds>`. Rate-limited requests **still** produce an audit row (outermost middleware captures them).

## Audit log

Every request produces a row in the `audit_events` SQLite table. Schema:

```sql
CREATE TABLE audit_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts TEXT NOT NULL,            -- RFC 3339 UTC
    agent_id TEXT,               -- from X-Reposix-Agent header
    method TEXT NOT NULL,
    path TEXT NOT NULL,
    status INTEGER,
    request_body TEXT,           -- first 256 chars
    response_summary TEXT        -- "<status>:<sha256-hex-16>"
);
```

Append-only enforced via `BEFORE UPDATE` / `BEFORE DELETE` triggers. See `crates/reposix-core/fixtures/audit.sql`.

## Optimistic concurrency worked example

```bash
# Agent A reads current version
curl -s http://127.0.0.1:7878/projects/demo/issues/1 \
    -H 'X-Reposix-Agent: agent-A' \
  | jq '.frontmatter.version'
# → 2

# Agent A successfully updates with If-Match: "2"
curl -s -X PATCH http://127.0.0.1:7878/projects/demo/issues/1 \
    -H 'X-Reposix-Agent: agent-A' \
    -H 'Content-Type: application/json' \
    -H 'If-Match: "2"' \
    -d '{"status":"in_progress"}'
# → 200, new version 3

# Agent B tries to update with stale If-Match: "2"
curl -s -X PATCH http://127.0.0.1:7878/projects/demo/issues/1 \
    -H 'X-Reposix-Agent: agent-B' \
    -H 'Content-Type: application/json' \
    -H 'If-Match: "2"' \
    -d '{"status":"done"}'
# → 409 {"error":"version_mismatch","current":3,"sent":2}
```

At the git layer, this 409 becomes a real merge conflict inside `0001.md` with `<<<<<<< HEAD` markers — the agent resolves it via `sed` and retries the push.
