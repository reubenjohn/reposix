# Research: Can git-remote-helpers act as promisor remotes for partial clone?

> **Status (2026-04-24): ANSWERED — yes, via the `stateless-connect` capability.** See `partial-clone-remote-helper-findings.md` in this directory for the full write-up, source citations, and a working proof-of-concept (`git-remote-poc.py` + `run-poc.sh` + `poc-helper-trace.log`). The recommendation is to migrate `crates/reposix-remote` from `import`/`export` to `stateless-connect`, delete `crates/reposix-fuse` entirely, and adopt partial-clone + sparse-checkout as the access-control mechanism.

## Context

reposix currently uses FUSE to lazy-load issue/page content from REST backends (GitHub, Confluence, Jira). This works but is slow — every `cat` triggers a live API call. We're exploring replacing FUSE entirely with git's built-in lazy-loading (partial clone + sparse checkout), keeping `git-remote-reposix` as the transport layer.

The viability of this design hinges on whether the git remote helper protocol can serve as a promisor remote — i.e., can git lazy-fetch individual blobs through a `git-remote-<name>` helper binary the same way it does through HTTP/SSH transports?

## Questions to answer

### Q1: Promisor remote support in helper protocol

Can a `git-remote-<transport>` helper (using the `connect`, `stateless-connect`, `fetch`, or `import` capabilities) act as a promisor remote?

Specifically:
- When git has a partial clone and needs a missing blob, does it invoke the remote helper to fetch it?
- Or does lazy blob fetching only work through git's native HTTP/SSH smart protocol, bypassing the helper entirely?

Sources to check:
- `Documentation/gitremote-helpers.txt` in the git source
- `Documentation/technical/partial-clone.txt`
- `Documentation/technical/pack-protocol.txt`
- git source: `promisor-remote.c`, `fetch-object.c`
- The `stateless-connect` capability (added for protocol v2 support in helpers)

### Q2: If helpers CAN be promisor remotes, what capabilities are required?

- Does the helper need `connect` or `stateless-connect` to expose the pack protocol?
- Or is the `fetch` capability (which our helper already implements) sufficient?
- Does the helper need to advertise `filter` or `partial-clone` capabilities?

### Q3: If helpers CANNOT be promisor remotes, what are the alternatives?

- **Local HTTP server:** Could `reposix` run a thin localhost HTTP server that speaks git's smart HTTP protocol and acts as the promisor remote? The helper binary becomes a server process.
- **FUSE with aggressive caching:** Fall back to FUSE but add an SQLite blob cache so only the first read hits the API. Less elegant but known to work.
- **Hybrid:** Use the helper for push/pull (tree sync), but a local HTTP promisor for lazy blob fetches.

### Q4: Can a remote helper limit or refuse blob fetch requests?

If the helper does serve blobs:
- Can it see how many blobs git is requesting in a single fetch batch?
- Can it return a meaningful error message that git surfaces to the user (stderr)?
- Or does git treat helper errors as opaque failures?

This matters because we want to guide agents: if an agent does `git grep` across 10,000 files, the helper should refuse with a message like "narrow your search with sparse-checkout" rather than silently fetching everything.

### Q5: How does sparse-checkout interact with partial clone through a helper?

- Does `git sparse-checkout set <path>` trigger blob fetches through the remote helper?
- Or does sparse-checkout only control which files appear in the working tree, independent of whether blobs are present in the object store?
- Can sparse-checkout be the mechanism that controls *which* blobs get fetched, or is it purely a working-tree filter?

## What "success" looks like

- **Best case:** Helper protocol supports promisor remotes natively. Our existing `git-remote-reposix` binary can serve lazy blob fetches. We delete `reposix-fuse` entirely.
- **Good case:** Helpers can't be promisor remotes directly, but `stateless-connect` lets us tunnel the pack protocol through the helper. Some protocol work needed but architecturally clean.
- **Acceptable case:** We need a thin local HTTP server for blob serving. More moving parts but the agent UX is identical.
- **Bad case:** None of the above work without forking git. FUSE stays, we add caching.

## Current implementation reference

- `crates/reposix-remote/` — current git-remote-reposix helper binary (Rust)
- `crates/reposix-fuse/` — FUSE daemon (candidate for deletion)
- `crates/reposix-core/src/backend.rs` — `BackendConnector` trait (all backends implement this)

## Deliverable

A findings document answering Q1-Q5 with citations to git source/docs, and a recommendation for which architecture path to pursue.
