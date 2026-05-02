# 8. Backend Requirements

← [back to index](./index.md)

Each backend must implement the following operations to support the full sync model:

### Required Operations

| Operation | Purpose | Existing? |
|-----------|---------|-----------|
| `list_changed_since(project, timestamp)` | Delta fetch | **New** -- needs trait extension |
| `get_item(project, id)` | Blob materialization on demand | Exists (`get_issue`) |
| `create_item(project, content)` | Push: new file | Exists (`create_issue`) |
| `update_item(project, id, content)` | Push: modified file | Exists (`update_issue`) |
| `close_item(project, id)` | Push: deleted file | Exists (`delete_or_close`) |

### Version/ETag Strategy Per Backend

| Backend | Conflict Detection Field | How to Obtain | How to Compare |
|---------|-------------------------|---------------|----------------|
| **GitHub** | `updated_at` (ISO 8601 timestamp) | Returned in every `GET /issues/{n}` response | String comparison; if backend `updated_at` > local `reposix_version`, conflict |
| **Jira** | `changelog` max version ID (integer) | `GET /rest/api/3/issue/{key}?expand=changelog` | Integer comparison; if backend version > local, conflict |
| **Confluence** | `version.number` (integer) | `GET /wiki/api/v2/pages/{id}` response body | Integer comparison; if backend version > local, conflict |
| **Simulator** | `version` (auto-incrementing integer) | `GET /api/v1/issues/{id}` response body | Integer comparison |

### Pagination Strategy

All backends paginate list results. The `list_changed_since` implementation must handle pagination transparently:

- **GitHub:** follow `Link: <url>; rel="next"` headers. Max 100 per page.
- **Jira:** increment `startAt` by `maxResults` until `total` is reached. Max 100 per page.
- **Confluence:** follow `_links.next` cursor. Max 250 per page.
- **Simulator:** single response (no pagination needed at simulator scale).
