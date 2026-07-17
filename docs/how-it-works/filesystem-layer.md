---
title: Filesystem layer — the cache and the working tree
---

# Filesystem layer

**Plain-English summary.** A reposix working tree is a real git checkout.
The `git checkout` you run right after `reposix init` is what pulls blob
*contents* down — lazily and on demand, for exactly the files being
checked out. Once a file is checked out, `cat` is a plain local read: no
network, ever. This page explains how that lazy-fetch trick works, why
it's a real git checkout (so `git diff` and `git stash` Just Work), and
where the bytes actually live on your machine.

---

The first key from [Mental model in 60 seconds](../concepts/mental-model-in-60-seconds.md) is *clone IS a git working tree*. This page explains why that statement is literally true: there is no virtual filesystem, no daemon between you and the bytes — just a real `.git/` directory backed by a local cache that pulls blobs from the backend on demand.

## How blob contents get materialized (at `git checkout`, not `cat`)

```mermaid
flowchart TD
    A["agent: git checkout origin/main — (the network trigger)"]
    O[".git/objects — partial-clone — blobs lazy"]
    H["git-remote-reposix — (helper binary)"]
    C["reposix-cache — (bare git repo + cache.db)"]
    R["Backend REST API — GitHub · Confluence · JIRA · sim"]
    G["Working tree — (real files on disk)"]
    K["agent: cat issues/PROJ-42.md — (local read, no network)"]

    A -->|"materialize blobs"| O
    O -->|"blob present?"| O
    O -.->|"miss → lazy fetch"| H
    H -->|"protocol v2 tunnel"| C
    C -->|"already materialized"| H
    C -.->|"first time only"| R
    R -.->|"GET /issues/PROJ-42"| C
    H -->|"packfile"| O
    O -->|"bytes → working tree"| G
    G -->|"already on disk"| K
```

A bare POSIX `cat` never triggers a network call — it reads whatever bytes are already on disk. The REST call happens earlier, at `git checkout`/`git fetch` time: the `git checkout -B main refs/reposix/origin/main` you run right after `reposix init` is what materializes a blob's *contents*, lazily and on demand, for exactly the files being checked out. After that checkout, every `cat` is a local read — `6 ms` against the simulator, [measured](../benchmarks/latency.md). The tree (filenames, directory structure, blob OIDs) is fetched once at `init` and is essentially free thereafter; only blob *contents* are lazy, and they arrive at `git checkout`/`git fetch` time.

## Why partial clone, not a virtual filesystem

The v0.1 architecture mounted a virtual filesystem so `ls` and `cat` would fan out to live REST calls. That made every read pay a network round-trip — `cat issues/2444.md` blocked on HTTP, and `ls` over 10 000 Confluence pages meant 10 000 calls just to render a directory. The v0.9.0 design (see [`architecture-pivot-summary.md`](https://github.com/reubenjohn/reposix/tree/main/.planning/research/v0.9-fuse-to-git-native)) **superseded** that virtual filesystem with git's own partial-clone mechanism. The `crates/reposix-fuse/` crate was deleted in the same milestone; the `fuser` dependency, the `/dev/fuse` permission song-and-dance, and the WSL2 kernel-module quirks all went with it.

[Partial clone](../reference/glossary.md#partial-clone) (a [git feature](https://git-scm.com/docs/git-clone#Documentation/git-clone.txt---filterltfilter-specgt) that fetches the tree up front but materializes blob *contents* lazily at `git checkout`/`git fetch`) is built into git ≥ 2.27 and stable in practice since 2019. The `--filter=blob:none` flag asks the remote for the tree without blobs; the helper then lazy-fetches blobs on demand the same way `git-remote-http` would. To git, our remote is just another remote — the agent never has to learn that it's talking to a REST API.

The other thing this buys: the working tree is **real**. `git status`, `git diff`, `git stash`, `git restore` all work the way they do on any other repo. Hooks fire. `.gitignore` applies. Editors track changes. Nothing about the working tree is synthetic.

## What lives where

The layer has two pieces:

- **`crates/reposix-cache/`** — a real on-disk [bare git repo](../reference/glossary.md#bare-repo) (a [git repository without a working tree](https://git-scm.com/docs/git-init#Documentation/git-init.txt---bare); built with [`gix`](https://github.com/Byron/gitoxide)) plus `cache.db` (SQLite, [WAL mode](https://sqlite.org/wal.html) — write-ahead logging so readers don't block writers). The bare repo holds the tree and any materialized blobs; `cache.db` holds the [audit log](../reference/glossary.md#audit-log) and the `last_fetched_at` timestamp used for delta sync.
- **The working tree** — created by `reposix init`, which runs `git init`, sets [`extensions.partialClone=origin`](../reference/glossary.md#extensionspartialclone) (a [git config flag](https://git-scm.com/docs/git-config#Documentation/git-config.txt-extensionspartialClone) telling git this remote is a [promisor](../reference/glossary.md#promisor-remote) — it'll deliver missing blobs on request), points `remote.origin.url` at the helper, and runs `git fetch --filter=blob:none`. After that command, the working tree is yours; reposix does not touch it again unless you `git fetch` or `git push`.

Wire-level details (cache schema, audit columns, helper invocation flags) live in [the simulator reference](../reference/simulator.md) and [testing targets](../reference/testing-targets.md). This page intentionally stays at user-experience altitude.

## Failure modes

- **Network down at checkout.** If the backend is unreachable when git tries to materialize a blob — during the `git checkout`/`git fetch` that pulls contents down — the helper surfaces its stderr and *that checkout* fails. A `cat` **after** a successful checkout cannot fail for network reasons: the fetch already happened at checkout time, so reads of already-materialized blobs continue to work offline — the cache is a real local git store. (`v0.1` had no offline story at all; every read was live.)
- **Blob limit hit.** A bulk operation like `git grep` over a never-checked-out tree can ask for thousands of blobs in one shot. The helper refuses past `REPOSIX_BLOB_LIMIT` (default 200) and emits a stderr message that names `git sparse-checkout` as the recovery move. The detail of how this is wired lives in the [git layer](git-layer.md#blob-limit-guardrail).
- **OID drift.** A backend write that bypasses reposix (someone using the REST API directly) changes an issue between your `git fetch` and your read. The cache will lazy-fetch the new content the next time the helper sees a `want` for that OID; the audit log shows a fresh `materialize` row. If you've already committed against the stale base and try to push, the push-time conflict detector rejects you with the standard git "fetch first" error — that flow is the subject of the [git layer](git-layer.md).

## Next

The blobs got into the working tree somehow; the edits get back to the backend somehow. Both halves of that round-trip are git protocol, and they live in [the git layer →](git-layer.md).

Every sync also writes a private tag (`refs/reposix/sync/<ts>`) in the cache's bare repo, so you can `git checkout` an earlier observation. See [time travel →](time-travel.md).
