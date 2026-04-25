# Expected output -- 04-conflict-resolve

Captured by running `bash run.sh` against a freshly-seeded `reposix-sim --ephemeral` (April 2026, target/debug).

## Stdout (key fragments)

```
[1/5] bootstrap two working trees
  target: 0001.md

[2/5] agent A: append a note + push
To reposix::http://127.0.0.1:7878/projects/demo
 * [new branch]      main -> main

[3/5] agent B (stale base): edit a different line + push -- expect rejection
issue 1 modified on backend at 2026-04-25T07:24:04+00:00 since last fetch (local base version: 1, backend version: 2). Run: git pull --rebase
To reposix::http://127.0.0.1:7878/projects/demo
 ! [rejected]        main -> main (fetch first)
error: failed to push some refs to 'reposix::http://127.0.0.1:7878/projects/demo'

[4/5] agent B reads stderr -> rebase onto the new tip
First, rewinding head to replay your work on top of it...
Applying: B: tag title

[5/5] agent B retries the push
To reposix::http://127.0.0.1:7878/projects/demo
 * [new branch]      main -> main
```

The line that matters most is in step [3/5]:

```
issue 1 modified on backend at ... since last fetch (local base version: 1, backend version: 2). Run: git pull --rebase
```

The literal substring `Run: git pull --rebase` is the dark-factory teaching string -- the dark-factory regression test (`scripts/dark-factory-test.sh`) asserts this string is present in the helper source so an stderr-reading agent always learns the recovery move.

## Audit-log rows

The cache DB is created lazily; its first row corresponds to the SECOND push (B's retry) because A's push runs before the cache directory exists. Real-world deployments have the cache dir pre-existing.

```bash
$ sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT id, ts, op, reason FROM audit_events_cache ORDER BY id"
1|2026-04-25T07:24:04Z|helper_push_started|refs/heads/main
2|2026-04-25T07:24:04Z|helper_push_sanitized_field|version
3|2026-04-25T07:24:04Z|helper_push_accepted|1
```

If the cache dir already exists at run time, you also see:

```
helper_push_rejected_conflict | issue 1 modified on backend at ... since last fetch
```

between A's push and B's retry. The op is enumerated in `audit_events_cache` per `crates/reposix-cache/migrations/`.

## Resulting commit on main

```bash
$ git -C /tmp/reposix-example-04-B log --oneline -3
6798d6d B: tag title
a4e08e6 Sync from REST snapshot
```

`6798d6d` is B's commit replayed on top of the new tip `a4e08e6` (which incorporates A's push). The history is linear -- no merge commit, no manual conflict resolution -- because A and B touched different lines. Had they touched the same line, `git rebase` would have stopped on a textual conflict and the agent would have edited the file + run `git rebase --continue`. That is the standard git story; reposix does not deviate from it.
