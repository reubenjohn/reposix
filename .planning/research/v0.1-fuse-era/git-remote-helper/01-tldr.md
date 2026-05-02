← [back to index](./index.md)

# 1. TL;DR — Recommendation for reposix

| Decision | Choice | One-liner |
|----------|--------|-----------|
| Capability set | `import` + `export` (NOT `fetch`/`push`) | Stream fast-import; we never reconstruct packfiles. |
| Auxiliary capabilities | `refspec`, `*export-marks`, `*import-marks`, `option` | Required for `import`/`export`; marks give us O(1) incremental sync. |
| State diffing | Maintain a `last-pushed.tree` SHA per ref in `$GIT_DIR/reposix/<remote>/state` | On `export`, walk new tree vs. last tree → field-level deltas → REST verbs. |
| Auth | Env vars `REPOSIX_TOKEN`, fallback `git config remote.<name>.reposixToken`, namespaced per-remote via `argv[1]` (the alias) | Helper receives `(alias, url)` as argv. |
| Error surface | Print to **stderr** (not stdout — stdout is reserved for protocol); also `error <ref> <msg>` on the protocol channel for per-ref failures | Both: human-readable on stderr, machine-readable on stdout per spec. |
| Conflict mode | On `import`, fetch authoritative remote state and emit a fast-import commit on `refs/reposix/<remote>/<ref>`; let git's three-way merge produce textual conflict markers in the agent's working tree | Native git semantics; no JSON conflict synthesis. |
| Async-from-sync | `tokio::runtime::Builder::new_current_thread().enable_all().build()` once at startup; every command handler is a sync function that calls `runtime.block_on(async { ... })` | Same bridge pattern as `fuser` callbacks. |

**Why NOT `fetch`/`push`:** those require us to materialize git packfiles ourselves (delta compression, OFS_DELTA, idx generation). For a REST-backed remote where there is *no upstream pack store*, that is gratuitous work. `import`/`export` lets us speak fast-import — a textual line protocol that fits naturally on top of HTTP. This matches the design of `git-remote-hg`, `git-remote-bzr`, and most non-git-native helpers.

**Why NOT `connect`:** `connect` is for remotes that *already speak git's native pack protocol* (we'd just proxy bytes through SSH or TLS). REST APIs do not.
