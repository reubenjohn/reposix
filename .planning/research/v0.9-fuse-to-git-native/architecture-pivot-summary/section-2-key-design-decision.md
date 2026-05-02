[index](./index.md)

# 2. Key Design Decision: Delete FUSE, Use Git's Partial Clone

The replacement architecture uses git's own partial clone mechanism (`--filter=blob:none`) to achieve lazy blob loading natively, with `git-remote-reposix` serving as the promisor remote.

## How it works

1. **`git clone --filter=blob:none reposix://github/org/repo`** downloads the full tree structure (directory names, filenames, blob OIDs) but zero file contents. This is a single list-API call to the backend, regardless of repo size.

2. **Blobs are lazy-fetched on demand.** When git needs a blob (during `checkout`, `cat-file`, `show`, etc.), it invokes the remote helper to fetch just that blob. The helper translates the OID into a REST API call (e.g., `GET /issues/2444`), returns the content as a packfile, and git caches it locally in `.git/objects`.

3. **`git-remote-reposix` is the promisor remote.** It advertises the `stateless-connect` capability, proxying protocol-v2 traffic to a local bare-repo cache that reposix builds from REST responses. Git treats it identically to an HTTP or SSH remote.

4. **The agent uses ONLY standard git commands.** `cat`, `grep`, `git diff`, `git push` -- no reposix-specific CLI, no MCP tools, no in-context learning. The mount point is a real git working tree with a real `.git` directory.

5. **Sparse-checkout controls scope.** An agent working on a subset of issues uses `git sparse-checkout set issues/PROJ-24*` to materialize only matching files. The helper sees a single batched fetch request for exactly those blobs.

## Why this is better

| Dimension | FUSE | Partial clone |
|---|---|---|
| Read latency | Every `cat` = 1 API call | First read = 1 API call; subsequent reads = local |
| Directory listing | N API calls for N items | Free (tree already local) |
| Offline support | None (EIO) | Full (all fetched blobs cached) |
| Build dependencies | `fuser`, `libfuse-dev`, `pkg-config` | None beyond git >= 2.27 |
| Platform support | Linux only (FUSE), WSL2 fragile | Everywhere git runs |
| Working tree | Virtual (FUSE callbacks) | Real git checkout |
| Change tracking | Custom diff logic | `git diff` natively |
| Agent UX | Must learn reposix CLI | Already knows git |
