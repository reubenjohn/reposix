# Expected output -- 05-blob-limit-recovery

Captured by running `bash run.sh` against `reposix sim --ephemeral` with `REPOSIX_BLOB_LIMIT=3` (July 2026, target/debug, git 2.50 host / git 2.43 in the ubuntu:24.04 rehearsal container).

This is the REAL observe-then-recover cycle: the no-narrow checkout drives the helper's actual `BLOB_LIMIT_EXCEEDED_FMT` stderr refusal, the script proves it fired, then recovers with `git sparse-checkout set`.

## Stdout

```
[1/4] bootstrap the partial-clone working tree with REPOSIX_BLOB_LIMIT=3

[2/4] check out the full backlog WITHOUT narrowing (drives the real refusal)
    the helper refused on stderr (this is the message an agent reads):
    error: refusing to fetch 6 blobs (limit: 3). Narrow your scope with `git sparse-checkout set <pathspec>` and retry. [RPX-0503] (run `reposix explain RPX-0503` for the full cause + recovery)

[3/4] recover exactly as the stderr taught: narrow scope, then retry
    retry succeeded -- materialized 3 of 6 issue files (sparse-checkout narrowed the scope):
    /tmp/reposix-example-05/issues/1.md
    /tmp/reposix-example-05/issues/2.md
    /tmp/reposix-example-05/issues/3.md

[4/4] widening scope is the same recovery, repeated
    after widening to 4 paths:
    /tmp/reposix-example-05/issues/1.md
    /tmp/reposix-example-05/issues/2.md
    /tmp/reposix-example-05/issues/3.md
    /tmp/reposix-example-05/issues/4.md

Done. The refusal wrote a blob_limit_exceeded audit row -- inspect it with:
  sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT id, ts, op, reason FROM audit_events_cache \
     WHERE op = 'blob_limit_exceeded' ORDER BY id DESC"
```

git prints `Switched to a new branch 'main'` / `Reset branch 'main'` on stderr during the recovery checkouts (interleaved, not shown above).

## Where the error fires (and why the checkout, not the fetch)

The helper enforces the blob limit per protocol-v2 `command=fetch` RPC, counting the `want` lines in that turn (`crates/reposix-remote/src/stateless_connect.rs`, `proxy_one_rpc`). Two paths reach the cache, and only one hits the check:

- `reposix init` runs `git fetch --filter=blob:none origin`, which brings back the tree + commit but **no** blobs (`want_count` stays ~1). It is under the limit and succeeds -- the blobs stay unmaterialized.
- `git checkout refs/reposix/origin/main` **without** narrowing lazy-fetches all 6 issue blobs in ONE `command=fetch` (`want_count=6 > limit=3`). The helper refuses on stderr BEFORE materializing anything. This is the runtime error the agent reads.

The fast-import (`import` capability) path bundles blobs in a single stream and bypasses the per-RPC check, so it is the **checkout's** lazy protocol-v2 fetch -- reachable on modern git (2.34+) -- that fires the refusal. ubuntu:24.04 ships git 2.43, so the container rehearsal drives the real error too.

## The blob_limit_exceeded audit row

Because the refusal fires for real, `proxy_one_rpc` calls `cache.log_blob_limit_exceeded(want_count, limit)`, so the `audit_events_cache` query in the footer now returns a row (`op = 'blob_limit_exceeded'`) -- the dual-table audit contract (OP-3) holds end to end. The teaching string is still the API: `quality/gates/agent-ux/dark-factory.sh` and `crates/reposix-cli/tests/agent_flow.rs::dark_factory_blob_limit_teaching_string_present` regression-protect the literal `git sparse-checkout` token + `[RPX-0503]` code in the source.

## Why this example matters

An agent that has never heard of `git sparse-checkout` hits the limit, reads the stderr, runs the named recovery command verbatim, and succeeds -- no prompt engineering, no reposix-specific knowledge beyond `reposix init`. That is the dark-factory teaching mechanism working end to end: the error message IS the recovery documentation, and this example proves the agent can act on it against the real helper.
