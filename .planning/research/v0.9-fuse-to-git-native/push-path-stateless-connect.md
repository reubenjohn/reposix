# Research: How does `git push` translate to REST writes through a stateless-connect helper?

## Context

We've confirmed that `git-remote-reposix` can serve as a promisor remote via `stateless-connect` for reads (see `partial-clone-remote-helper-findings.md`). The read path is solved: the helper proxies protocol-v2 `command=fetch` traffic to a backing bare repo cache.

The write path is the open question. Today, `crates/reposix-remote` uses the `export` capability: git fast-exports commits, the helper parses the stream, extracts issue changes, and translates them to REST API calls (POST/PUT/DELETE). In the new `stateless-connect` world, `git push` would go through `git receive-pack` — a fundamentally different flow.

## Questions to answer

### Q1: What happens when `git push` goes through a `stateless-connect` helper?

When the helper advertises `stateless-connect`, does `git push` route through the same capability? Specifically:
- Does `transport-helper.c` dispatch push through `stateless-connect` the same way it dispatches fetch?
- Or does push use a separate `connect git-receive-pack` path?
- Can the helper intercept and inspect the pushed packfile/commits before "accepting" them?

Sources to check:
- `transport-helper.c` — push routing logic (look for `push_refs`, `process_connect`)
- `Documentation/gitremote-helpers.adoc` — `connect` vs `stateless-connect` for push
- `send-pack.c` / `receive-pack.c` — what the helper would need to proxy

### Q2: Can the helper reject a push with a meaningful error?

We need push-time conflict detection: when the agent pushes, the helper fetches the current backend state, compares, and rejects if there are conflicts — mimicking a "fast-forward rejected" error.

- Can the helper return a rejection that git surfaces as a normal push failure?
- Can the error message be customised (e.g. "rejected: issue-2444.md was modified on Confluence since your last fetch")?
- Does git retry or prompt the user after rejection, or does it just fail?

### Q3: Can we intercept pushed commits and extract per-file diffs?

The helper needs to:
1. Receive the pushed commits
2. Diff each commit against the previous state to find changed issue files
3. Parse the changed `.md` files to extract issue field updates
4. Translate to REST API calls (POST for new issues, PUT for updates, DELETE for removals)
5. Either accept (update refs) or reject (conflict) the push

Can this be done inside the `stateless-connect` / `receive-pack` flow? Or do we need to keep the `export` capability specifically for writes?

### Q4: Is a hybrid approach viable?

Could we use:
- `stateless-connect` for reads (fetch, partial clone, lazy blobs) — confirmed working
- `export` for writes (push) — already implemented today

This avoids solving the push-through-receive-pack problem entirely. The helper advertises both capabilities. Git uses `stateless-connect` for fetch and `export` for push.

Is this a valid combination? Does git allow a helper to advertise both? Does it correctly route fetch vs push to different capabilities?

Sources to check:
- Can a helper advertise both `stateless-connect` and `export`?
- `transport-helper.c` — does push path check `stateless-connect` first, or does it check `export`/`push` independently?

### Q5: Push-time conflict detection — where does the backend comparison happen?

Regardless of which capability handles push, we need:
1. Helper receives the agent's changes
2. Helper queries the backend for current state of affected issues (`GET /issues/<id>`)
3. Helper compares: if the backend version differs from what the agent's commit was based on, reject
4. If no conflict, helper sends the REST writes and accepts the push

Where in the flow is the right interception point? Before `receive-pack` processes the pack? After? In a `pre-receive` hook on the backing bare repo?

## What "success" looks like

- **Best case:** Hybrid approach (Q4) works — we keep `export` for push, add `stateless-connect` for fetch. Minimal new code for the write path.
- **Good case:** Push through `stateless-connect`/`receive-pack` works and the helper can intercept commits, but we need to rewrite the push path.
- **Acceptable case:** Push needs `connect` (not `stateless-connect`) for `receive-pack`. Helper advertises `stateless-connect` for fetch and `connect` for push.
- **Bad case:** `stateless-connect` and `export` can't coexist; we have to choose one and rewrite the other path.

## POC scope (if building one)

If the answers to Q1-Q4 suggest the hybrid approach works, build a minimal POC extending the existing `git-remote-poc.py`:
- Add `export` capability alongside `stateless-connect`
- Push a commit that creates a new file
- Verify git routes fetch through `stateless-connect` and push through `export`
- Log which capability handles which operation

If hybrid doesn't work, build the smallest POC that demonstrates push-through-receive-pack with commit inspection.

## Current implementation reference

- `crates/reposix-remote/` — current helper binary, uses `import`/`export`
- `.planning/research/git-remote-poc.py` — read-path POC using `stateless-connect`
- `.planning/research/partial-clone-remote-helper-findings.md` — confirmed findings for the read path

## Deliverable

A findings document answering Q1-Q5, with a recommendation for which push architecture to use, and ideally a POC demonstrating it.
