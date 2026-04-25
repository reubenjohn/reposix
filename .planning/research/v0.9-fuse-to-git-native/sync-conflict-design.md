# Design: Sync and Conflict Resolution Model

**Date:** 2026-04-24
**Status:** Draft
**Depends on:** `partial-clone-remote-helper-findings.md`, `push-path-stateless-connect-findings.md`

This document describes the sync and conflict resolution model that sits ON TOP of the partial-clone transport layer. The transport is confirmed viable (stateless-connect for fetch, export for push). This document is about the higher-level semantics: when data moves, how conflicts are detected, and how agents recover from them using only standard git commands.

---

## 1. Design Principles

**P1: Agent uses ONLY standard git commands.**
The agent never runs `reposix` CLI subcommands during normal operation. `git clone`, `git fetch`, `git pull`, `git push`, `git sparse-checkout`, `cat`, `grep`, `sed` -- these are the entire interface. The `reposix` CLI exists only for one-time setup (`reposix init`), not for ongoing work.

**P2: Agent learns from error messages, not documentation.**
When something goes wrong (conflict, blob limit exceeded, egress denied), the helper writes a self-contained error message to stderr or via `error <ref> <message>`. The message tells the agent exactly what happened and what to do next. No prompt engineering, no tool schemas, no pre-loaded instructions required. An agent that has never heard of reposix can use a reposix-backed repo.

**P3: Push is the only sync point that matters.**
Reads are local (from the git object store). Writes are optimistic -- the agent edits freely, commits locally, and conflict detection happens at `git push` time. There is no lock-acquire step, no checkout-for-edit ceremony. This is optimistic concurrency: assume no conflict, detect at commit time, recover via standard git rebase.

**P4: Tree sync is always cheap; blob materialization is the expensive operation.**
A tree is metadata: filenames, sizes, content hashes. Even for 10,000 Confluence pages, the tree is ~1 MB. Fetching the tree is always a single API call and always completes in seconds. Fetching blob content (the actual issue/page bodies) is the expensive part -- it hits the REST API per-blob (or per-batch). The architecture exploits this asymmetry: tree is always fully synced, blobs are fetched lazily and only for the paths the agent actually reads.

---

## 2. Fetch Model -- Lightweight Delta Sync

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

---

## 3. Push Model -- Conflict Detection at Write Time

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

---

## 4. Blob Limit Guardrail

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

---

## 5. Tree Sync Has No Limit

Tree metadata is structurally small:

| Scale | Tree Size | Fetch Time |
|-------|-----------|------------|
| 100 issues (typical GitHub project) | ~10 KB | <100ms |
| 1,000 issues (active Jira project) | ~100 KB | <500ms |
| 10,000 pages (large Confluence space) | ~1 MB | <2s |
| 50,000 issues (enterprise Jira) | ~5 MB | <5s |

A tree entry is approximately 100 bytes: mode (6) + space (1) + filename (60 avg) + null (1) + SHA-1 (20) + overhead (12). Even at enterprise scale, the tree fits in a single git packfile transfer that takes seconds.

No limit is applied to tree sync. The tree is always fully synced on every fetch. This gives the agent full awareness of every item in the project via:

```
$ ls issues/                     # see all issue filenames
$ git diff --name-only origin/main   # see what changed since last fetch
$ wc -l issues/*                 # count items without fetching content
```

The agent can make decisions about what to read based on filenames, paths, and directory structure -- all without downloading a single blob.

---

## 6. Agent Workflow Examples

### 6.1 Read Workflow -- Browsing Confluence Pages

```bash
# One-time setup (done by reposix init, not by the agent)
git clone --filter=blob:none --no-checkout reposix://confluence/ENGINEERING ~/eng-docs
cd ~/eng-docs

# Agent scopes to Q4 pages
git sparse-checkout set pages/2024-Q4/

# Agent reads a specific page -- one blob fetched on demand
cat pages/2024-Q4/quarterly-review.md

# Agent searches within scoped pages -- blobs already cached locally
grep -r "OKR" pages/2024-Q4/

# Agent discovers a page outside scope -- fetches just that blob
git checkout origin/main -- pages/2024-Q3/retrospective.md
cat pages/2024-Q3/retrospective.md
```

### 6.2 Write Workflow -- Updating a GitHub Issue

```bash
cd ~/project-issues

# Agent edits an issue
cat issues/fix-login-timeout.md
# ... reads current content ...

vim issues/fix-login-timeout.md
# ... makes changes ...

git add issues/fix-login-timeout.md
git commit -m "update fix-login-timeout: add reproduction steps"

# Push triggers conflict check + REST write
git push origin main
# To reposix://github/acme/webapp
#    a1b2c3d..e4f5g6h  main -> main
```

### 6.3 Conflict Workflow -- Concurrent Modification

```bash
# Agent pushes, but someone else edited the issue on GitHub
git push origin main
# ! [remote rejected] main -> main (reposix: fix-login-timeout.md was modified on backend since last fetch)
# error: failed to push some refs to 'reposix://github/acme/webapp'

# Standard git recovery
git pull --rebase origin main
# helper fetches ONLY the conflicting file's new content
# rebase auto-merges or drops into conflict resolution

# If auto-merged:
git push origin main
# success

# If conflict markers present:
vim issues/fix-login-timeout.md    # resolve <<<<<<< markers
git add issues/fix-login-timeout.md
git rebase --continue
git push origin main
```

### 6.4 Discovery Workflow -- Monitoring for Changes

```bash
# Agent periodically checks for backend changes
git fetch origin
# helper calls GET /issues?since=<last_fetched_at>
# tree updated, no blobs transferred

# See what changed
git diff --name-only origin/main
# issues/new-feature-request.md
# issues/fix-login-timeout.md

# Fetch only the interesting one
git checkout origin/main -- issues/new-feature-request.md
cat issues/new-feature-request.md
```

### 6.5 Bulk Creation Workflow -- Agent Creates Multiple Issues

```bash
# Agent creates several new issue files
cat > issues/improve-caching.md << 'EOF'
---
title: "Improve Redis caching layer"
state: open
labels: [enhancement, performance]
---

The current caching layer has no TTL management...
EOF

cat > issues/fix-memory-leak.md << 'EOF'
---
title: "Fix memory leak in worker pool"
state: open
labels: [bug, critical]
---

Workers are not releasing connections...
EOF

git add issues/improve-caching.md issues/fix-memory-leak.md
git commit -m "create two new issues"

# Single push, helper creates both via REST
git push origin main
# helper: POST /repos/acme/webapp/issues (improve-caching)
# helper: POST /repos/acme/webapp/issues (fix-memory-leak)
# To reposix://github/acme/webapp
#    h7i8j9k..l0m1n2o  main -> main
```

---

## 7. Comparison with Current FUSE Design

| Aspect | FUSE (current) | Git-native (proposed) |
|--------|---------------|----------------------|
| **First read** | Live API call (~500ms) | Blob fetch + cache (once, ~500ms) |
| **Subsequent reads** | Live API call (~500ms) | Local file read (~1ms) |
| **Directory listing** | Live API call (~200ms) | Local tree (~1ms) |
| **Write** | FUSE write handler -> immediate API call | `git push` -> batch REST writes |
| **Write batching** | None (one API call per write syscall) | Natural (all changes in one commit = one push) |
| **Conflict detection** | None (last write wins) | Push-time version comparison |
| **Offline capability** | None (all ops require network) | Full (read/write/commit offline, push when online) |
| **Agent learning** | None needed | None needed (P2: learns from errors) |
| **Change tracking** | None (no diff capability) | `git diff` / `git log` show full history |
| **Dependencies** | fuser crate, fusermount3, /dev/fuse, Linux only | git >= 2.27 (everywhere git runs) |
| **Platform support** | Linux only (WSL2 quirky, macOS via macFUSE) | Linux, macOS, Windows, WSL2 |
| **Concurrent access** | Race conditions on overlapping writes | Git merge semantics (well-understood) |
| **Rollback** | None | `git revert`, `git reset` |

The git-native model is strictly superior for the agentic use case. The FUSE model's only advantage -- transparent filesystem integration without any git awareness -- is irrelevant when the consumer is an LLM agent that already knows git.

---

## 8. Backend Requirements

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

---

## 9. Open Design Questions

### Q1: Cache Eviction in the Backing Bare Repo

The backing bare repo grows as blobs are fetched. For a large Confluence space, this could reach gigabytes. Options:

- **LRU eviction:** track access times per blob; prune least-recently-used blobs when cache exceeds a size threshold. Re-fetch on next access (the promisor mechanism handles this transparently).
- **TTL eviction:** blobs older than N days are pruned. Simple but may evict frequently-accessed items.
- **No eviction:** rely on git's packfile compression (which is excellent for text). A 10,000-page Confluence space with average 5 KB per page is ~50 MB packed -- acceptable for most systems.

**Leaning toward:** no eviction initially, with `git gc` on the bare repo as part of periodic maintenance. Add LRU later if real-world usage shows cache bloat.

### Q2: Webhook Integration (Optional Push-Based Sync)

Instead of polling on `git fetch`, backends could push changes to reposix via webhooks. This would make `git fetch` instant (the backing bare repo is already up-to-date).

- **GitHub:** `issues` webhook event.
- **Jira:** Jira webhooks or Atlassian Connect `issue_updated` event.
- **Confluence:** Confluence webhooks for `page_updated`.

This is a pure optimization and does not change the sync model. The fetch path still works identically -- it just finds that `last_fetched_at` is already current. Defer to a future milestone.

### Q3: Bulk Operations (Agent Creates 50 Issues in One Commit)

A single `git push` with 50 new files means 50 REST POST calls. Concerns:

- **Rate limiting:** GitHub allows 5,000 requests/hour for authenticated users. 50 POSTs in one push is fine. Jira and Confluence have similar limits.
- **Atomicity:** if POST #37 fails, we have 36 created issues and 14 uncreated. The compensating-operation logic (section 3, "Transactional Semantics") handles this, but rolling back 36 created issues is ugly.
- **Mitigation:** for bulk creates, consider a two-phase approach: (1) create all via REST, collecting IDs; (2) update the backing bare repo only after all succeed. On failure, delete the successfully-created items. Log everything to audit.
- **Alternative:** accept partial success and surface it clearly: `warning: 36 of 50 items created; 14 failed (see audit log)`. The agent can retry the failed subset.

### Q4: Binary Attachments

Issue attachments (images, PDFs) are binary blobs. Options:

- **Inline blobs:** store in the git repo as regular files. Simple but bloats the repo.
- **Git LFS:** store pointers in the repo, actual content in LFS storage backed by the REST API. Clean but adds an LFS dependency.
- **Sidecar directory:** `issues/fix-bug.md` has attachments in `issues/.attachments/fix-bug/screenshot.png`. Still inline blobs but organized.

**Leaning toward:** inline blobs in a sidecar directory for v1. Binary content is rare in issue trackers (mostly screenshots). Git handles moderate binary content fine. Evaluate LFS if attachment volume becomes a problem.

### Q5: Merge vs Rebase on Pull

When `git pull` resolves a conflict, should reposix recommend `--rebase` or `--merge`?

- **Rebase** produces a linear history that maps cleanly to "sequence of REST writes." Each rebased commit becomes one REST update.
- **Merge** produces a merge commit that has no obvious REST analog (what does "merge" mean for a Jira issue?).

**Decision:** reposix should default to rebase. The helper's error message should suggest `git pull --rebase`. The `reposix init` setup should configure `pull.rebase=true` for the repo.

### Q6: Multi-Agent Collaboration and Commit History

Multiple independent agents can work against the same backend (e.g. five agents editing Confluence pages simultaneously). Each agent has its own local git repo with its own commit history. **Agents do NOT share git history** — they share backend state via the REST API.

This means:
- Agent A's `origin/main` has commit `abc123`; Agent B's has commit `def456` — even if they represent the same backend state. This is expected.
- Conflict detection works against the **backend version**, not against other agents' commit hashes. When Agent B pushes and Agent A already modified the same page, the helper checks the backend and rejects.
- `git log` in each agent's repo shows only that agent's local history. The unified timeline lives in the backend's audit trail (Confluence page history, GitHub issue edit history, etc.) and in reposix's own audit log.
- This is identical to how multiple developers work against any git remote — each has their own clone with independent commit hashes, and the server is the shared state.

**No action needed** — this is the natural consequence of the architecture. But it should be explained in user-facing docs to avoid confusion ("why doesn't my git log show the other agent's changes?").

### Q7: Blob Limit Gap — Lazy-Fetch Fan-Out

The blob limit guard (section 4) counts `want` lines per helper invocation. This catches **batched** requests (e.g. `git checkout` sends multiple wants in one RPC). However, some git operations trigger **one helper process per blob**:

- `git cat-file -p <oid>` — 1 helper, 1 want
- `git log -p` — 1 helper per diff blob
- `git grep HEAD` — 1 helper per searched blob
- `git blame <file>` — 1 helper per historical blob

Each invocation has exactly 1 want, so the blob limit (which checks per-request) never triggers. An agent doing `git grep HEAD` across 10,000 files spawns 10,000 helper processes, each fetching 1 blob — completely bypassing the guard.

**Primary mitigation: sparse-checkout.** Default `git grep` (no ref argument) searches only the working tree, which only contains sparse-checkout files. No fan-out. The problem only surfaces with object-layer commands (`git grep HEAD`, `git log -p`, `git blame`) that bypass the working tree.

**Options to close the remaining gap:**

1. **Shared counter:** helper increments a counter in `cache.db` on each blob fetch. If the running total in a time window exceeds the limit, refuse. Cross-process coordination via SQLite (already exists).
2. **Rate limit:** refuse if blob fetches-per-minute exceed a threshold. Simpler but blunter — may interfere with legitimate batched-by-git-but-serial-by-process patterns.
3. **Accept:** sparse-checkout prevents the common case. Fan-out only hits object-layer commands, which are uncommon in typical agent workflows (agents use `cat` and `grep`, not `git grep HEAD`).

**Decision:** option 1 (shared counter in `cache.db`). Implementation: add a `blob_fetch_log` table to `cache.db` with `(timestamp, blob_oid)` rows. Each helper invocation inserts a row and checks `SELECT COUNT(*) FROM blob_fetch_log WHERE timestamp > datetime('now', '-5 minutes')`. If the count exceeds `REPOSIX_BLOB_LIMIT`, refuse with the same error message as the per-request guard. ~20 lines of SQL, uses existing SQLite infrastructure, catches both batched and fan-out patterns uniformly. Sparse-checkout remains the primary scope guard; the shared counter is the safety net for object-layer commands (`git grep HEAD`, `git log -p`, `git blame`) that bypass the working tree. This should be a task in one of the v0.9.0 implementation phases.

### Q8: Multi-Branch Workflows

The current design assumes a single `main` branch. Should agents be able to create feature branches?

- Feature branches could represent "draft changes" that aren't pushed to the backend until merged to main.
- This is a natural git workflow but adds complexity: the helper needs to know which branch maps to the backend.
- **Decision:** defer. Single-branch (`main`) for v1. The architecture does not preclude multi-branch later -- the export stream includes the target ref, so the helper can gate on `refs/heads/main` and reject pushes to other branches with a clear error message.
