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

## Chapters

- [2. Fetch Model — Lightweight Delta Sync](./ch-2-fetch-model.md)
- [3. Push Model — Conflict Detection at Write Time](./ch-3-push-model.md)
- [4. Blob Limit Guardrail](./ch-4-blob-limit.md)
- [5. Tree Sync Has No Limit](./ch-5-tree-sync.md)
- [6. Agent Workflow Examples](./ch-6-agent-workflows.md)
- [7. Comparison with Current FUSE Design](./ch-7-fuse-comparison.md)
- [8. Backend Requirements](./ch-8-backend-requirements.md)
- [9. Open Design Questions](./ch-9-open-questions.md)
