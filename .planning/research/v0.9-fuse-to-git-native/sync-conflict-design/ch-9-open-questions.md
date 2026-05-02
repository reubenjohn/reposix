# 9. Open Design Questions

← [back to index](./index.md)

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
