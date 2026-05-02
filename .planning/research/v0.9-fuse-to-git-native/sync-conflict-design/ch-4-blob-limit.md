# 4. Blob Limit Guardrail

← [back to index](./index.md)

### Problem

An agent that runs `git checkout` on a large sparse-checkout set, or does `git grep` across the entire tree, will trigger blob fetches for every missing blob. Each fetch is a REST API call. An unchecked agent could exhaust API rate limits or download gigabytes of content it does not need.

### Mechanism

The helper intercepts each `command=fetch` protocol-v2 request (confirmed inspectable per findings Q4). It counts `want` lines in the request.

```
REPOSIX_BLOB_LIMIT=200    # env var, default
```

When a single fetch request contains more `want` lines than the limit:

1. Helper writes to stderr:
   ```
   error: refusing to fetch 1,247 blobs (limit: 200). Narrow your working set with:
     git sparse-checkout set <path>
   ```
2. Helper exits non-zero.
3. Git surfaces the stderr to the agent/user.

### Why This Works for Agents

The agent does not need to know about `REPOSIX_BLOB_LIMIT` in advance. It tries an operation, gets an error, reads the error message, and adapts. The error message contains the exact command to run. This is P2 (learn from error messages) in action.

Typical agent recovery:

```
$ git checkout main
error: refusing to fetch 1,247 blobs (limit: 200). Narrow your working set with:
  git sparse-checkout set <path>

$ git sparse-checkout set pages/2024-Q4/
$ git checkout main
# succeeds -- only 23 blobs in pages/2024-Q4/
```

### Configuration

| Env Var | Default | Effect |
|---------|---------|--------|
| `REPOSIX_BLOB_LIMIT` | `200` | Max `want` lines per fetch request |
| `REPOSIX_BLOB_LIMIT=0` | -- | Disable limit (unlimited fetches) |

The limit applies per-request, not cumulatively. An agent that fetches 100 blobs, then later fetches another 100, is fine. The guard catches bulk operations (large `git checkout`, `git grep` on unfiltered tree).
