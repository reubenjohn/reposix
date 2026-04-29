---
title: Time travel — every sync is a checkout
---

# Time travel

The audit log says *what reposix did*; sync tags say *what reposix observed*. Together they are a fully replayable history of how the backend looked, sync by sync, all the way back to the first `git fetch`.

## What gets tagged

Every successful `Cache::sync` writes one ref of the form:

```text
refs/reposix/sync/<ISO8601-no-colons>
```

For example, `refs/reposix/sync/2026-04-25T01-13-00Z` points at the synthesis commit produced by the sync that ran at 2026-04-25 01:13:00 UTC. Colons are illegal inside git ref names, so we substitute `-`; the format round-trips one-to-one with `chrono::DateTime<Utc>`.

The tag lives inside the **cache's bare repo** at `~/.cache/reposix/<backend>-<project>.git`, not in your working tree. `git tag -l` in the working tree never shows it. The reasons:

- The helper's `list` advertisement only mentions `refs/heads/main`.
- `transfer.hideRefs = refs/reposix/sync/` is set on the cache's bare repo so `git upload-pack --advertise-refs` skips the namespace entirely.

This is private state for the cache. Inspecting it requires going to the cache directly.

## Inspecting one historical sync

```bash
# The cache path is deterministic; reposix doctor prints it:
$ reposix doctor /tmp/repo | grep "cache DB"
OK    cache.db: cache DB present at /home/me/.cache/reposix/sim-demo.git/cache.db

# Check out the sync from a known timestamp:
$ git -C /home/me/.cache/reposix/sim-demo.git checkout refs/reposix/sync/2026-04-25T01-13-00Z
$ git -C /home/me/.cache/reposix/sim-demo.git show HEAD:issues/PROJ-42.md
```

The bare repo doesn't have a working tree, but `git show <ref>:<path>` works without one. For visual diffing across two syncs:

```bash
$ git -C /home/me/.cache/reposix/sim-demo.git diff \
    refs/reposix/sync/2026-04-24T22-30-00Z \
    refs/reposix/sync/2026-04-25T01-13-00Z \
    -- issues/PROJ-42.md
```

That diff is the literal byte-level change reposix observed for `PROJ-42` between those two syncs. No reconstruction, no simulator replay, no rebuilding state from audit rows — just `git diff`.

## CLI surface

Two subcommands surface the tag namespace from the working tree without you having to hand-construct the cache path:

```bash
$ reposix history /tmp/repo
2026-04-25T01-13-00Z   commit 1a2b3c4   delta_sync (3 record(s) in this sync)
2026-04-25T01-08-00Z   commit 0f9e8d7   delta_sync (1 record(s) in this sync)
2026-04-25T01-03-00Z   commit deadbee   tree_sync (47 record(s) in this sync)

3 sync tag(s). Earliest: 2026-04-25T01-03-00Z. Use `git -C /home/me/.cache/reposix/sim-demo.git checkout <tag>` to inspect a historical state.

$ reposix at 2026-04-25T01:00:00Z /tmp/repo
refs/reposix/sync/2026-04-25T00-58-00Z   commit 7c2d4f1
(use: git -C /home/me/.cache/reposix/sim-demo.git checkout refs/reposix/sync/2026-04-25T00-58-00Z)
```

`history` lists most-recent first, capped at 10 entries by default (override with `--limit`). `at <ts>` finds the latest sync tag whose timestamp is ≤ the target — useful for "what did reposix see when I filed this bug?". The synthesis op (`tree_sync` for the seed sync, `delta_sync` for incrementals) and the record count are pulled from `audit_events_cache` on a best-effort basis.

## Audit row pairing

Every tag write also produces one audit row:

| Column | Value |
|---|---|
| `op` | `sync_tag_written` |
| `oid` | synthesis commit OID |
| `reason` | full ref name (`refs/reposix/sync/<slug>`) |

So a forensic query that joins `audit_events_cache.op = 'sync_tag_written'` against the bare repo's ref store gives you (timestamp, commit, ref-name) triples plus everything else the sync row recorded. The row is part of the same append-only audit table as `tree_sync`, `delta_sync`, `materialize`, etc. — same WAL, same triggers, same security guarantees.

## Cost

Each tag is one git ref — 41 bytes on disk in loose form, less when packed. A repo synced hourly for a year accumulates ~360 KB of refs. A cleanup pass (`reposix gc`, planned for v0.12.0) will be able to prune old sync tags by TTL, but for normal use this is below the noise floor.

## Why this is interesting

Most issue-tracker integrations expose the *current* state and leave history as a database query against the backend. Sync tags expose history as the same primitive your version-control already speaks: refs and commits. An agent that knows `git checkout` and `git diff` can reconstruct what changed without ever learning a reposix-specific API.

The pattern is generalisable beyond reposix — any partial-clone promisor remote could write per-sync refs and turn its observation history into a checkable artefact. To our knowledge, reposix is the first to ship it. Design intent and prior-art search are recorded in `.planning/research/v0.11.0/vision-and-innovations.md` §3b.

## Next

Sync tags are part of the [filesystem layer ←](filesystem-layer.md). The push round-trip lives in the [git layer →](git-layer.md).
