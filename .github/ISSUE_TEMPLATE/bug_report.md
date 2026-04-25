---
name: 🐛 Bug report
about: Report a bug in reposix
title: "[BUG] "
labels: bug
---

## Versions

- reposix git commit: `git -C /path/to/reposix rev-parse HEAD`
- Backend: sim / github / confluence / jira / other
- OS: 
- git --version: 
- rustc --version (if building from source): 

## What happened

Steps to reproduce:

1. 
2. 
3. 

## What you expected

## Relevant audit log rows

(See `docs/guides/troubleshooting.md` for the sqlite query.)

```sql
-- paste output of: sqlite3 ~/.cache/reposix/<backend>-<project>.git/cache.db \
--   "select ts, op, ... from audit_events_cache order by ts desc limit 10;"
```

## Stderr / stdout

```
paste here
```

## Anything else

(screenshots, config snippets, related issues)
