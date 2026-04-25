# Expected output -- 05-blob-limit-recovery

Captured by running `bash run.sh` against `reposix-sim --ephemeral` with `REPOSIX_BLOB_LIMIT=3` (April 2026, target/debug).

## Stdout

```
[1/4] the literal teaching string an agent would read on stderr:

    error: refusing to fetch {N} blobs (limit: {M}). Narrow your scope with `git sparse-checkout set <pathspec>` and retry.

    -- The literal `git sparse-checkout` token is the contract.

[2/4] bootstrap with REPOSIX_BLOB_LIMIT=3

[3/4] dark-factory recovery: narrow scope with git sparse-checkout, then check out
Checked out 3 of 6 issue files (sparse-checkout matched only 3 paths):
/tmp/reposix-example-05/0001.md
/tmp/reposix-example-05/0002.md
/tmp/reposix-example-05/0003.md

[4/4] widening scope is the same recovery, repeated
After widening to 4 paths:
/tmp/reposix-example-05/0001.md
/tmp/reposix-example-05/0002.md
/tmp/reposix-example-05/0003.md
/tmp/reposix-example-05/0004.md

Done. Inspect blob-limit hits with:
  sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT id, ts, op, bytes, reason FROM audit_events_cache \
     WHERE op = 'blob_limit_exceeded' ORDER BY id DESC"
```

## Why no `blob_limit_exceeded` audit row in this example

On v0.9.0 the helper takes the `import` (fast-import) path on `git fetch`, which packages all blobs in a single packet and bypasses the per-RPC `command=fetch` blob-limit check. The `audit_events_cache` query above therefore returns zero rows in this script's runtime.

The teaching string is still the contract:

- The literal `BLOB_LIMIT_EXCEEDED_FMT` constant lives at `crates/reposix-remote/src/stateless_connect.rs:54-55`.
- `scripts/dark-factory-test.sh` greps the source for `git sparse-checkout` to regression-protect the teaching string on every CI run.
- `crates/reposix-cli/tests/agent_flow.rs::dark_factory_blob_limit_teaching_string_present` does the same in the cargo test layer.

When the helper migrates to a stateless-connect-only read path (planned for v0.10), this script's pre-emptive `git sparse-checkout set` becomes the literal recovery from a runtime stderr -- the agent's recovery move is unchanged because the teaching string is unchanged.

## Why this example is still useful pre-v0.10

Two reasons:

1. It documents and exercises the `git sparse-checkout` workflow against a reposix working tree, which is the operating norm for any backlog larger than a handful of issues. Agents reading this example see how to narrow scope without learning anything reposix-specific.
2. It locks in the byte-identical teaching string an agent will encounter once the trigger fires at runtime -- so when v0.10 lands, no example or doc has to be rewritten.
