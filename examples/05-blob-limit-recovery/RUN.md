# Example 05 -- Blob-limit recovery (the dark-factory teaching mechanism)

The helper's blob-limit guardrail prints a self-teaching error message when an agent's `command=fetch` RPC requests more blobs than `REPOSIX_BLOB_LIMIT` allows. The error names the recovery move (`git sparse-checkout set <pathspec>`) verbatim and carries the stable `[RPX-0503]` code, so a stderr-reading agent recovers without prompt engineering.

```text
error: refusing to fetch <N> blobs (limit: <M>). Narrow your scope with `git sparse-checkout set <pathspec>` and retry. [RPX-0503] (run `reposix explain RPX-0503` for the full cause + recovery)
```

That literal substring is the contract: the dark-factory regression test (`quality/gates/agent-ux/dark-factory.sh`) asserts it is committed in `crates/reposix-remote/src/stateless_connect.rs`. The teaching string is the API.

## What this example demonstrates

The script drives the REAL observe-then-recover cycle:

1. Bootstraps a partial-clone working tree with `REPOSIX_BLOB_LIMIT=3` (the demo seed has 6 issues). `reposix init`'s `--filter=blob:none` fetch brings back the tree + commit but no blobs, so it stays under the limit.
2. Checks out `refs/reposix/origin/main` **without** narrowing. That lazy-fetches all 6 issue blobs in one protocol-v2 `command=fetch` (`want_count=6 > 3`), so the helper refuses on stderr. The script captures that stderr and fails loud unless it contains the `git sparse-checkout` token and `[RPX-0503]`.
3. Recovers exactly as the stderr taught: `git sparse-checkout set /issues/1.md /issues/2.md /issues/3.md` narrows scope to 3 blobs, the retried checkout completes 0, and the 3 records materialize.
4. Widening scope to 4 paths is the same recovery, repeated.

The error fires through the modern-git (2.34+) stateless-connect protocol-v2 checkout path -- the fast-import path bundles blobs in one stream and bypasses the per-RPC blob-limit check, so it is the checkout's lazy fetch, not the initial filtered fetch, that triggers the refusal. ubuntu:24.04 ships git 2.43, so the container rehearsal (`quality/gates/docs-repro/container-rehearse.sh`) drives the real error too.

## Prerequisites

- Binaries built: `cargo build -p reposix-cli -p reposix-sim -p reposix-remote` (the CLI, the sim it runs in-process, and the `git-remote-reposix` helper git invokes on the `reposix::` URL).
- Simulator running on `127.0.0.1:7878` with the demo seed (`reposix sim --bind 127.0.0.1:7878 --ephemeral`).

## Run

```bash
./run.sh
```

## What success looks like

See [`expected-output.md`](expected-output.md) for the captured stdout, the resulting working-tree state, and the `blob_limit_exceeded` audit row the refusal writes.
