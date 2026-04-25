# Expected output -- 02-python-agent

Captured by running `python3 run.py` against a freshly-seeded `reposix-sim --ephemeral` (April 2026, target/debug).

## Stdout

```
To reposix::http://127.0.0.1:7878/projects/demo
 * [new branch]      main -> main
matched 1 issue(s) mentioning 'database': ['0001.md']
  0001.md: severity=medium added
```

The `* [new branch]` line is the push response; it's printed first because Python flushes the subprocess's stderr-on-stdout earlier than the agent's own log lines. With the demo seed the substring "database" appears in exactly one issue body (issue 1, "database connection drops under load").

## Frontmatter after the run

```yaml
---
id: 1
title: database connection drops under load
status: open
labels:
- bug
- p1
created_at: 2026-04-13T00:00:00Z
updated_at: 2026-04-13T00:00:00Z
version: 1
severity: medium
---
```

Note that `severity: medium` was inserted at the end of the existing block. Server-controlled fields (`id`, `version`, `created_at`, `updated_at`) are still present locally; the helper strips them on the egress path before the REST PATCH.

## Resulting commit

```bash
$ git -C /tmp/reposix-example-02 log --oneline -3
fb8d266 label severity:medium on 1 issue(s)
b58a861 Sync from REST snapshot
```

## Audit log rows from the push

```bash
$ sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT id, ts, op, reason FROM audit_events_cache ORDER BY id"
1|2026-04-25T07:20:35Z|helper_push_started|refs/heads/main
2|2026-04-25T07:20:35Z|helper_push_accepted|
```

Two rows: started + accepted. No `helper_push_sanitized_field` this time -- the agent did not write back any server-owned fields, so nothing got stripped. (Compare with `01-shell-loop` where the comment-append included the `version: 1` line and triggered the sanitiser.)
