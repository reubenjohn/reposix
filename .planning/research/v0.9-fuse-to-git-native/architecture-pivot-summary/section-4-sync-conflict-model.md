[index](./index.md)

# 4. Sync and Conflict Model

This section captures design decisions made in the exploration session that are not recorded in either findings document. This is new content.

## Push-time conflict detection

The agent works against a local git checkout that may be stale (another agent or a human may have modified the same issue on the backend). At push time:

1. The helper receives the agent's commits via the `export` fast-import stream.
2. For each changed file (issue), the helper fetches the current backend state via REST.
3. If the backend version differs from what the agent's commit was based on, the helper rejects the push with a standard git error.
4. The agent sees `! [remote rejected]` and does the normal `git pull --rebase` + `git push` cycle.

No new concepts for the agent to learn. This is how git works with any remote.

## Tree sync is always full; blob limit is the only guardrail

- **Tree sync (directory structure) is cheap.** `git fetch` updates the entire tree -- all filenames, directory structure, blob OIDs. This maps to a single paginated list-API call regardless of repo size. Tree metadata is small (a project with 10,000 issues produces maybe 500KB of tree objects).
- **Blob materialization is where cost lives.** Each blob is a REST API call to fetch the actual issue content. The helper's blob limit (see below) is the guardrail against unbounded API usage.
- **No limit needed on tree sync** because the cost is fixed and small.

## Delta sync via `since` queries

All three target backends support incremental queries:

| Backend | Mechanism |
|---|---|
| GitHub Issues | `GET /repos/:owner/:repo/issues?since=<ISO8601>` |
| Jira | `JQL: updated >= "<datetime>"` |
| Confluence | `CQL: lastModified > "<datetime>"` |

The existing `cache_db.rs` (in `crates/reposix-cli/src/`) already stores a `last_fetched_at` timestamp in a single-row SQLite table (`refresh_meta`). This timestamp is the `since` parameter for delta sync.

**Fetch flow:**

1. `git fetch origin` invokes the helper.
2. Helper reads `last_fetched_at` from the cache DB.
3. Helper calls the backend with a `since` query, receiving only items changed since the last fetch.
4. Helper updates the backing bare-repo cache with the changed items.
5. Helper serves the updated tree/blobs to git via protocol v2.
6. Helper updates `last_fetched_at` to now.

**Agent sees changes via pure git:**

```bash
git fetch origin
git diff --name-only origin/main   # shows which issues changed since last fetch
```

No custom tools, no reposix CLI awareness.

**Trait addition needed:** `BackendConnector` needs a `list_changed_since(timestamp) -> Vec<IssueId>` method (or equivalent) to support delta queries. The existing `list_issues()` method fetches everything; the new method fetches only the delta.

## Agent UX: pure git, zero in-context learning

The entire agent interaction model is standard git:

| Agent action | Git command | What happens behind the scenes |
|---|---|---|
| Get the repo | `git clone reposix://github/org/repo` | Partial clone; tree downloaded, blobs lazy |
| Read an issue | `cat issues/2444.md` | Blob lazy-fetched via helper on first read |
| See what changed | `git fetch && git diff --name-only origin/main` | Delta sync via `since` query |
| Edit and push | `git add . && git commit && git push` | Helper parses export stream, translates to REST |
| Handle conflict | `git pull --rebase && git push` | Standard git rebase flow |
| Narrow scope | `git sparse-checkout set issues/PROJ-24*` | Controls which blobs get materialized |

The agent needs zero reposix-specific knowledge. Every operation is a git command the agent already knows.

## Blob limit as teaching mechanism

The helper enforces a configurable blob limit to prevent unbounded API usage:

- **Configuration:** `REPOSIX_BLOB_LIMIT` environment variable (default: 200).
- **Enforcement:** when the helper receives a `command=fetch` request with more `want` lines than the limit, it refuses with a stderr error message.
- **Error message:** `"error: refusing to fetch N blobs (limit: M). Narrow your scope with sparse-checkout."`
- **Agent learning:** the agent reads the error message and learns to use `git sparse-checkout` to narrow its scope. This is the same way agents learn from any tool error -- no prompt engineering or system prompt instructions needed.

Example scenario: an agent runs `git grep "TODO"` across 10,000 files. Git tries to lazy-fetch all 10,000 blobs. The helper refuses. The agent reads the error, runs `git sparse-checkout set issues/PROJ-24*`, and retries with a narrower scope.
