---
title: reposix vs MCP and REST SDKs
---

# reposix vs MCP and REST SDKs

reposix **complements** REST and MCP — it does not stand in their place. The 80% of tracker operations an agent does a hundred times a day (status changes, comments, field edits, label adds, link creation) absorb into `cat` + `git push`. The other 20% — complex JQL, bulk imports, admin operations, reporting queries — keep using REST or MCP directly. This page is for the skeptic asking "where exactly does each tool earn its keep."

## Numbers, side by side

| Axis | reposix | MCP | Raw REST SDK |
|---|---|---|---|
| **Tokens before first useful op** | `~0` (POSIX is in pre-training) | `~100k` schema discovery per server | `~5k` SDK boilerplate per backend |
| **Latency, cached read** | `8 ms` ([sim](../benchmarks/v0.9.0-latency.md)) | `200–500 ms` tool dispatch + LLM | `100–300 ms` HTTPS round-trip |
| **Latency, cold init / first call** | `24 ms` cold init ([sim](../benchmarks/v0.9.0-latency.md)) | one tool-list at session start | per-call HTTPS handshake |
| **Conflict semantics** | native git merge conflicts; standard `git pull --rebase` recovery | 409 retry in tool runtime | 409 retry in caller code |
| **Pre-training overlap** | ~100% — POSIX + git are in every model | ~0% — protocol is post-training | ~80% — REST patterns are pre-trained |
| **Egress surface** | one binary (`git-remote-reposix`); audited | tool runtime + transport (varies per server) | per-call, untyped |
| **Audit trail** | append-only SQLite + `git log` | per-server, varies | per-call, depends on caller |

The latency cells for reposix are measured ([`docs/benchmarks/v0.9.0-latency.md`](../benchmarks/v0.9.0-latency.md)). The MCP and SDK cells are characterized from public-API behaviour and the reposix project's own [agentic-engineering reference](../research/agentic-engineering-reference.md); they are not measured by reposix's harness.

## When each one earns its keep

### Use reposix for…

- **The hundred small edits.** Status changes, comments, field edits, label adds, link creation, custom-field touches.
- **`grep`-able read access.** "Find me every issue mentioning *database*." `grep -r database issues/` returns instantly from the cache.
- **Multi-agent contention you want to debug.** Conflicts surface as `git merge` conflicts on text files — twenty years of tooling already understands that shape.
- **Audit trails as a first-class artifact.** `git log` plus an append-only SQLite audit table; both are committed-or-fixture artifacts, not session state.

### Use REST or MCP for…

- **Complex JQL or CQL queries.** reposix doesn't pretend to model arbitrary search; the API does it natively. `gh issue list --search 'state:open author:@me'` and `jira jql '…'` keep working alongside reposix.
- **Bulk imports.** Posting a thousand new issues via `git push` is technically possible but the REST API is the right tool.
- **Admin operations.** Permissions, workflows, custom-field configuration — admin endpoints stay where they are.
- **Reporting and analytics.** Aggregation lives at the API tier.

The reposix CLI doesn't shadow these. `gh issue view --json` and `jira issue list` are still in your `$PATH`; they work on the same backends reposix talks to. The simulator audit log records reposix's writes regardless of which tool the agent reaches for first.

## How "complement" plays out in practice

A typical agent session looks like this:

```bash
# Read and edit — pure git, lands in reposix's cache + audit log.
cd /tmp/jira && git pull && grep -rl 'database' issues/ | head -5
sed -i 's/^status: .*/status: in_progress/' issues/PROJ-42.md
git commit -am 'PROJ-42 in progress' && git push

# Reach for REST when the abstraction does not fit the question.
gh issue list --search 'created:>2026-04-01 -label:wontfix' --json number,title
jira jql 'project = PROJ AND sprint in openSprints() AND assignee = currentUser()'
```

Both paths route through the same `REPOSIX_ALLOWED_ORIGINS` egress allowlist when run under reposix's audit harness; both leave audit rows. The difference is only which surface is more legible for the question being asked.

## The thesis in one sentence

> reposix absorbs the ceremony around the operations every agent does constantly. The API stays exactly where it is for the rest.

Next:

- [Mental model in 60 seconds →](mental-model-in-60-seconds.md)
- [Latency envelope →](../benchmarks/v0.9.0-latency.md)
- [Sanctioned real-backend test targets →](../reference/testing-targets.md)
