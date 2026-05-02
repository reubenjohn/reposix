# Endpoint surface for v0.1

← [back to index](./index.md)

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
