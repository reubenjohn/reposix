# Example 05 -- Blob-limit recovery (the dark-factory teaching mechanism)

The helper's blob-limit guardrail prints a self-teaching error message when an agent's `command=fetch` RPC requests more blobs than `REPOSIX_BLOB_LIMIT` allows. The error message names the recovery move (`git sparse-checkout set <pathspec>`) verbatim, so a stderr-reading agent can recover without prompt engineering.

```text
error: refusing to fetch <N> blobs (limit: <M>). Narrow your scope with `git sparse-checkout set <pathspec>` and retry.
```

That literal substring is the contract: the dark-factory regression test (`scripts/dark-factory-test.sh`) asserts it is committed in `crates/reposix-remote/src/stateless_connect.rs`. The teaching string is the API.

## What this example demonstrates

The script in this directory:

1. Echoes the literal `BLOB_LIMIT_EXCEEDED_FMT` template from the helper source so you can see the exact bytes an agent would read on stderr.
2. Bootstraps a working tree with `REPOSIX_BLOB_LIMIT=3`.
3. Runs the dark-factory recovery: applies `git sparse-checkout set` to narrow scope BEFORE materialising the full tree, then checks out the subset. This is the recovery move, performed pre-emptively.

> **v0.9.0 note.** Today's helper takes the `import` (fast-import) path on the canonical clone+fetch flow, which bundles all blobs in one packet -- the per-RPC `command=fetch` blob-limit check therefore does not fire in this script's runtime. The teaching string and the recovery move are still the contract, regression-protected by `scripts/dark-factory-test.sh` and `crates/reposix-cli/tests/agent_flow.rs::dark_factory_blob_limit_teaching_string_present`. When the helper migrates to a pure stateless-connect read path (planned for v0.10), this script's pre-emptive recovery becomes the literal recovery from a runtime stderr.

## Prerequisites

- Binaries built: `cargo build -p reposix-cli -p reposix-sim -p reposix-remote`.
- Simulator running on `127.0.0.1:7878` with the demo seed.

## Run

```bash
./run.sh
```

## What success looks like

See [`expected-output.md`](expected-output.md) for the captured stdout and the resulting working-tree state.
