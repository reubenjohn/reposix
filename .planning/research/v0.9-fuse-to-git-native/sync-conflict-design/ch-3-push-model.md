# 3. Push Model -- Conflict Detection at Write Time

← [back to index](./index.md)

### Flow

1. Agent runs `git push origin main`.
2. Git invokes `git-remote-reposix` with `list` then `export` capability (confirmed: push always routes through `export`, never `stateless-connect` -- see `push-path-stateless-connect-findings.md` Q1).
3. Helper receives the fast-export stream containing all commits since the last push.
4. Helper parses the stream (existing `fast_import.rs` parser) to extract per-file changes: `M` (modify), `D` (delete), `R` (rename).
5. For each changed file that maps to a backend item:
   a. Helper fetches the current backend version of that item (`GET /issues/{id}`).
   b. Helper compares the backend's `updated_at` (or ETag, or version number) against the base version recorded when the agent last fetched that item.
6. **No conflict:** Helper sends REST writes (POST for new files, PUT/PATCH for modifications, DELETE or state-change for deletions), updates the backing bare repo, responds `ok refs/heads/main`.
7. **Conflict detected:** Helper drains the remaining export stream (required -- cannot leave the pipe half-read), responds `error refs/heads/main reposix: <path> was modified on backend since last fetch`. Does NOT touch the backing bare repo or the REST backend.
8. Git surfaces the error: `! [remote rejected] main -> main (reposix: quarterly-review.md was modified on backend since last fetch)`.

### Version Tracking

Each backend provides a different mechanism for detecting concurrent modification:

| Backend | Version Signal | Storage |
|---------|---------------|---------|
| GitHub | `updated_at` timestamp on issue | Stored in YAML frontmatter: `reposix_version: "2026-04-24T10:30:00Z"` |
| Jira | Issue `changelog` version ID | Stored in frontmatter: `reposix_version: "10042"` |
| Confluence | Page `version.number` (integer) | Stored in frontmatter: `reposix_version: "14"` |
| Simulator | `version` field (auto-incrementing integer) | Stored in frontmatter: `reposix_version: "7"` |

The `reposix_version` frontmatter field is a server-controlled field (per CLAUDE.md: "Server-controlled fields cannot be overridden by client writes; they are stripped on the inbound path before serialization"). If an agent edits this field, the helper ignores the edit and uses the value from the last fetch as the comparison base.

### Conflict Recovery

The agent recovers using standard git commands:

```
$ git push origin main
 ! [remote rejected] main -> main (reposix: quarterly-review.md was modified on backend since last fetch)
error: failed to push some refs to 'reposix://confluence/ENGINEERING'

$ git pull --rebase origin main
```

`git pull --rebase` triggers a fetch (which updates the tree and fetches ONLY the blobs for conflicting files), then rebases the agent's local commits on top. If the rebase has textual conflicts, git drops the agent into standard conflict resolution (`<<<<<<< HEAD` markers). If not, the rebase auto-completes and the agent can push again.

### Transactional Semantics

The push path must be atomic: either all REST writes succeed, or none do.

1. Helper collects all changes from the export stream into an in-memory plan.
2. Helper validates ALL changes against the backend (version checks) before writing ANY of them.
3. If all checks pass, helper executes REST writes in order.
4. If a REST write fails mid-batch:
   a. Helper attempts compensating operations (DELETE newly created items, PATCH reverted updates) for any writes that already succeeded.
   b. Helper logs the partial failure to the audit table with full detail.
   c. Helper responds `error refs/heads/main reposix: partial write failure, see audit log`.
   d. The backing bare repo is NOT updated (it still reflects pre-push state).
5. Only after ALL REST writes succeed does the helper update the backing bare repo and respond `ok`.
