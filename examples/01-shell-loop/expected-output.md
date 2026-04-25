# Expected output -- 01-shell-loop

Captured by running `bash run.sh` against `reposix-sim --bind 127.0.0.1:7878 --seed-file crates/reposix-sim/fixtures/seed.json --ephemeral` (April 2026, target/debug).

## Stdout

```
WARN reposix::init: git fetch --filter=blob:none failed with status exit status: 128 -- local repo is configured but not yet synced. Stderr: fatal: could not read ref refs/reposix/main
reposix init: configured `/tmp/reposix-example-01` with remote.origin.url = reposix::http://127.0.0.1:7878/projects/demo
Next: cd /tmp/reposix-example-01 && git checkout origin/main (or git sparse-checkout set <pathspec> first)
fatal: could not read ref refs/reposix/main
triaging: ./0004.md
[main bd10226] review: 0004.md
 1 file changed, 3 insertions(+)
To reposix::http://127.0.0.1:7878/projects/demo
 * [new branch]      main -> main

Done. Inspect the audit log with:
  sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT ts, op, decision FROM audit_events_cache ORDER BY ts DESC LIMIT 5"
```

The two `fatal: could not read ref refs/reposix/main` lines are benign noise from the trailing `git fetch` step (init is best-effort on fetch, and the fetch DOES write `refs/reposix/origin/main` correctly before returning non-zero). The next `git checkout -B main refs/reposix/origin/main` in `run.sh` works because the ref is in place.

## Resulting commit on `main`

```bash
$ git -C /tmp/reposix-example-01 log --oneline -3
bd10226 review: 0004.md
4cf4ee8 Sync from REST snapshot
```

## Audit log rows from the push

```bash
$ sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT id, ts, op, reason FROM audit_events_cache ORDER BY id"
1|2026-04-25T07:18:37Z|helper_push_started|refs/heads/main
2|2026-04-25T07:18:37Z|helper_push_sanitized_field|version
3|2026-04-25T07:18:37Z|helper_push_accepted|2,4
```

Three rows for one push:

- `helper_push_started` -- the helper accepted the `export` command from git.
- `helper_push_sanitized_field` -- a server-controlled field (`version`) was stripped on egress. The diff includes `version: 1` from the seed; the helper enforces SG-03 by refusing to round-trip it.
- `helper_push_accepted` -- the REST PATCH succeeded; the `reason` field lists the issue ids that changed (here, ids 2 and 4 because the bd10226 commit's diff touches issue 4 plus the helper's diff representation includes the 0002.md baseline).

## What changed on the server

```bash
$ curl -s http://127.0.0.1:7878/projects/demo/issues/4 | head -20
{"id":4,"title":"flaky integration test on CI","status":"open","labels":["bug","ci","flaky"],"created_at":"2026-04-13T00:00:00Z","updated_at":"2026-04-25T07:18:37Z","version":2,"body":"...\n\n## Comment from shell-loop example\nReviewed by reposix shell-loop example at 2026-04-25T07:18:07+00:00\n"}
```

`version` advanced from `1` to `2`; the body now includes the appended comment block.
