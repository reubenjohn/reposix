[index](./index.md)

# 5. Architecture: What Changes

## Architecture Diagram

```mermaid
graph TB
    subgraph AGENT["Agent (pure git — no reposix awareness)"]
        direction LR
        A1["git clone<br/>(or reposix init)"]
        A2["cat / ls / grep"]
        A3["git fetch"]
        A4["git commit + push"]
    end

    subgraph GIT["Git Client + Working Tree"]
        G1["Working Tree<br/>real files on disk"]
        G2[".git/objects<br/>(partial clone — blobs lazy)"]
        G3["sparse-checkout config<br/>(controls which blobs materialize)"]
    end

    subgraph HELPER["git-remote-reposix (helper binary — two capabilities)"]
        direction TB
        H_READ["stateless-connect<br/>clone, fetch, lazy blob reads<br/>protocol: git v2 stdin/stdout"]
        H_WRITE["export<br/>push via fast-export stream<br/>parses commits → REST calls"]
        H_GUARD["Blob Limit Guard<br/>counts want lines per request<br/>refuses if > REPOSIX_BLOB_LIMIT<br/>stderr teaches agent to narrow scope"]
        H_CONFLICT["Conflict Detector<br/>(inside export handler)<br/>fetches current backend state<br/>compares with push base<br/>rejects with standard git error"]
        H_AUDIT["Audit Log<br/>every fetch + push logged"]
    end

    subgraph CACHE["Backing Cache (reposix-cache crate)"]
        C1["Local bare git repo<br/>(objects built from REST)"]
        C2["cache.db SQLite<br/>last_fetched_at timestamp"]
    end

    subgraph BACKENDS["REST Backends (existing — unchanged)"]
        direction LR
        B1["GitHub API"]
        B2["Confluence API"]
        B3["Jira API"]
        B4["Simulator"]
    end

    A1 -->|"--filter=blob:none"| GIT
    A2 -->|"read file"| G1
    A3 -->|"check changes"| GIT
    A4 -->|"write back"| GIT

    G1 ---|"backed by"| G2
    G2 -->|"missing blob"| H_READ
    G3 ---|"scopes"| G1

    GIT -->|"git fetch (v2)"| H_READ
    GIT -->|"git push (fast-export)"| H_WRITE

    H_READ -->|"proxies v2"| C1
    H_WRITE -->|"parse stream"| H_CONFLICT
    H_CONFLICT -->|"check + write"| BACKENDS
    H_GUARD -.->|"refuses large batches"| GIT
    H_READ --- H_GUARD

    C1 -->|"cache miss"| BACKENDS
    C2 -->|"since timestamp"| C1
    H_AUDIT --> C2
```

Also available as rendered PNG: `architecture-pivot-diagram.png` in this directory.

## Delete

- **`crates/reposix-fuse/`** -- the entire crate, including all FUSE callbacks, mount/unmount lifecycle, and the `fuse-mount-tests` feature gate.
- **`fuser` dependency** -- removes the `pkg-config` / `libfuse-dev` build requirement.
- **All FUSE-related runtime concerns:** `/dev/fuse` permissions, `fusermount3` requirement, WSL2 kernel module configuration, stale mount cleanup.
- **FUSE integration tests** -- no longer needed; replaced by git-level integration tests.

## Add

- **`stateless-connect` capability in `git-remote-reposix`** -- approximately 200 lines of Rust, tunnelling protocol-v2 traffic to a backing bare-repo cache. Implementation follows the same pattern as the Python POC (`poc/git-remote-poc.py`).
- **`reposix-cache` crate** -- a new crate that materializes REST API responses into a local bare git repo. This is where sync logic lives: tree construction from issue listings, blob creation from issue content, delta sync via `since` queries, and cache eviction.
- **`list_changed_since()` on `BackendConnector` trait** -- enables delta sync by querying the backend for items modified after a given timestamp.
- **Blob limit enforcement** -- the helper counts `want` lines per `command=fetch` request and refuses if the count exceeds `REPOSIX_BLOB_LIMIT`.

## Change

- **CLI flow:** `reposix mount <path>` becomes `reposix init <backend>::<project> <path>`. The new command runs `git init`, configures `extensions.partialClone`, sets `remote.origin.url=reposix::<backend>/<project>`, and runs `git fetch --filter=blob:none origin` to bootstrap.
- **Helper capability advertisement:** adds `stateless-connect` alongside existing `export`. The `import` capability becomes redundant once `stateless-connect` handles all fetch paths; it should be kept for one release cycle, then deprecated.

## Keep

- **`BackendConnector` trait and all backend implementations** (`SimBackend`, `GithubBackend`, `ConfluenceBackend`, etc.) -- consumed by `reposix-cache` instead of by FUSE.
- **`export` capability for push path** -- confirmed working alongside `stateless-connect`; the existing fast-import parsing and REST write logic in `crates/reposix-remote` is preserved.
- **Audit log** -- the helper writes audit rows for every protocol-level fetch and push, same as today.
- **Threat model** -- tainted-by-default policy, allowed-origins egress allowlist, frontmatter field allowlist all remain. The push-through-export flow needs a threat model update (see Risks).
- **Simulator as default/testing backend** -- unchanged.
