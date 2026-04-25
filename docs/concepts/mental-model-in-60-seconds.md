---
title: Mental model in 60 seconds
---

# Mental model in 60 seconds

Three keys. One page. After this you can drive reposix from your shell.

## 1. Clone IS a git working tree

`reposix init sim::demo /tmp/repo` produces a real git working tree. `.git/` is real. Blobs are lazy. `git status` and `git diff` are upstream git.

```text
/tmp/repo
├── .git/         # real .git
└── issues/
    ├── 0001.md
    └── 0002.md
```

`ls`, `cat`, `grep -r`, `sed -i` — all of POSIX works as it always has. The bootstrap takes `24 ms` against the simulator ([`v0.9.0-latency.md`](../benchmarks/v0.9.0-latency.md)).

## 2. Frontmatter IS the schema

Each issue is one Markdown file. Structured fields are YAML frontmatter; the body is Markdown. Custom fields are just more YAML keys.

```bash
$ cat issues/0001.md
---
id: 1
title: Add user avatar upload
status: in_progress
assignee: alice@acme.com
labels: [backend, needs-review]
custom_fields: { customer_impact: medium }
---
## Description
Avatar uploads are blocked by S3 permissions…
```

Reading the schema takes 30 seconds; editing it takes the editor you already have.

## 3. `git push` IS the sync verb

Edits become REST writes when — and only when — you `git push`. The helper parses your commits, checks the backend version, applies the writes, or rejects with the standard git "fetch first" error so you `git pull --rebase && git push`.

```bash
sed -i 's/^status: .*/status: done/' issues/0001.md
git commit -am 'close 1' && git push

sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
  "SELECT ts, op, decision FROM audit ORDER BY ts DESC LIMIT 5"
```

Every push, accept or reject, writes one append-only audit row. `git log` is the intent; the audit table is the outcome.

## Ready in 60s? Run this:

```bash
cargo run -p reposix-cli -- init sim::demo /tmp/repo \
  && cd /tmp/repo \
  && git checkout origin/main \
  && cat issues/0001.md
```

That's the whole loop. Next:

- [How reposix complements MCP and SDKs →](reposix-vs-mcp-and-sdks.md)
- [Latency envelope (`8 ms` cache read · `24 ms` cold init) →](../benchmarks/v0.9.0-latency.md)
- [Sanctioned real-backend test targets →](../reference/testing-targets.md)
