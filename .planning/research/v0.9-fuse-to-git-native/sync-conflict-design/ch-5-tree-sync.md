# 5. Tree Sync Has No Limit

← [back to index](./index.md)

Tree metadata is structurally small:

| Scale | Tree Size | Fetch Time |
|-------|-----------|------------|
| 100 issues (typical GitHub project) | ~10 KB | <100ms |
| 1,000 issues (active Jira project) | ~100 KB | <500ms |
| 10,000 pages (large Confluence space) | ~1 MB | <2s |
| 50,000 issues (enterprise Jira) | ~5 MB | <5s |

A tree entry is approximately 100 bytes: mode (6) + space (1) + filename (60 avg) + null (1) + SHA-1 (20) + overhead (12). Even at enterprise scale, the tree fits in a single git packfile transfer that takes seconds.

No limit is applied to tree sync. The tree is always fully synced on every fetch. This gives the agent full awareness of every item in the project via:

```
$ ls issues/                     # see all issue filenames
$ git diff --name-only origin/main   # see what changed since last fetch
$ wc -l issues/*                 # count items without fetching content
```

The agent can make decisions about what to read based on filenames, paths, and directory structure -- all without downloading a single blob.
