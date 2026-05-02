# 2. Fetch Model -- Lightweight Delta Sync

← [back to index](./index.md)

### Flow

1. Agent runs `git fetch origin`.
2. Git invokes `git-remote-reposix` with `stateless-connect git-upload-pack`.
3. Helper reads `last_fetched_at` from the cache metadata table (see `crates/reposix-cli/src/cache_db.rs`, `refresh_meta.last_fetched_at`).
4. Helper calls the backend's "changed since" API with that timestamp.
5. For each changed item, helper updates the backing bare repo's tree: creates or updates the blob object (issue body as Markdown with YAML frontmatter), updates the tree entry.
6. Helper commits the new tree to the backing bare repo.
7. Helper proxies the protocol-v2 `ls-refs` and `fetch` responses from the backing bare repo back to git, filtered with `blob:none`.
8. Git receives the new tree objects but NO blob content (partial clone filter).
9. Helper updates `last_fetched_at` in the cache metadata table.
10. Agent sees changes via `git diff --name-only origin/main`.

### Backend API Calls

Each backend has a native "changed since" endpoint:

| Backend | API Call | Notes |
|---------|----------|-------|
| GitHub | `GET /repos/{owner}/{repo}/issues?since={timestamp}&state=all&per_page=100` | `since` filters by `updated_at`. Pagination via `Link` header. |
| Jira | `GET /rest/api/3/search?jql=updated>="{timestamp}"&maxResults=100` | JQL `updated` field. Pagination via `startAt`. |
| Confluence | `GET /wiki/api/v2/pages?space-id={id}&sort=-modified-date` with CQL `lastModified > "{timestamp}"` | V2 API. Pagination via cursor. |
| Simulator | `GET /api/v1/projects/{id}/issues?since={timestamp}` | Reposix-sim native endpoint. |

### Cost

- **First fetch (cold):** one paginated API call to list all items + tree construction. No blobs transferred to the working tree.
- **Subsequent fetches (warm):** one API call returning only items modified since `last_fetched_at`. Typically returns 0-10 items. Tree diff is tiny.
- **Blob cost:** zero at fetch time. Blobs are fetched on-demand when the agent reads a file (via `cat`, `git show`, `git checkout`, etc.).

### BackendConnector Trait Extension

```rust
/// Return IDs of all items modified since `since`.
/// Used by the fetch path to build a delta tree.
///
/// # Errors
/// Returns `ConnectorError` on network failure or auth issues.
fn list_changed_since(
    &self,
    project: &ProjectId,
    since: DateTime<Utc>,
) -> Result<Vec<ItemSummary>, ConnectorError>;
```

Where `ItemSummary` carries enough metadata to build a tree entry (id, title slug for filename, content hash) without requiring the full body. The full body is written to the backing bare repo as a blob, but is filtered out by `blob:none` during the protocol-v2 fetch to the working tree.
