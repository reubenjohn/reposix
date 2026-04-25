# CLI reference

`reposix` is a thin orchestrator. The agent's day-to-day surface is plain `git` against a partial-clone working tree; the CLI exists to bootstrap that working tree, run the simulator, and surface a few backend-shaped queries with no clean git equivalent.

Built from `crates/reposix-cli`. Subcommands as of v0.9.0:

```text
reposix — git-native partial clone for autonomous agents

Usage: reposix <COMMAND>

Commands:
  init     Initialize a partial-clone working tree backed by reposix
  sim      Run the REST simulator (delegates to reposix-sim)
  list     List issues/pages in a project (prints JSON or table)
  refresh  Re-fetch all issues and write a commit (force delta sync)
  spaces   List readable Confluence spaces (Confluence backend only)
  version  Print the version
  help     Print this message or the help of the given subcommand(s)
```

## `reposix init`

Bootstrap a git working tree wired to a reposix backend via partial clone. Entry point for every agent workflow — see the [first-run tutorial](../tutorials/first-run.md).

```bash
reposix init sim::demo /tmp/repo
reposix init github::reubenjohn/reposix /tmp/issues
reposix init confluence::TokenWorld /tmp/space
reposix init jira::TEST /tmp/jira
```

| Argument | Form | Purpose |
|---|---|---|
| `<spec>` | `<backend>::<project>` | `sim`, `github`, `confluence`, or `jira` plus a project / repo / space / key. |
| `<path>` | filesystem path | Working-tree location (parents created as needed). |

`init` runs `git init`, four `git config` lines wiring the partial-clone promisor remote, and a best-effort `git fetch --filter=blob:none origin`. The working tree is plain git afterwards. Tree metadata is fetched eagerly; blobs are fetched on demand on first read.

`confluence::` and `jira::` specs require `REPOSIX_CONFLUENCE_TENANT` or `REPOSIX_JIRA_INSTANCE`. See [Confluence](confluence.md) and [JIRA](jira.md) for credential setup.

## `reposix sim`

Spawn the REST simulator as a child process. Default backend for tests and demos; see the [simulator reference](simulator.md) for the REST shape and seed semantics.

| Flag | Default | Purpose |
|---|---|---|
| `--bind` | `127.0.0.1:7878` | Listen address. |
| `--db` | `runtime/sim.db` | SQLite file (ignored when `--ephemeral`). |
| `--seed-file` | — | JSON seed path. |
| `--no-seed` | off | Skip seeding. |
| `--ephemeral` | off | In-memory SQLite. |
| `--rate-limit` | `100` | Per-agent requests per second. |

## `reposix list`

Query the backend's `list_records` and dump JSON or a fixed-width table. Useful for quick inspection from the shell; the agent UX prefers `git ls-files` once `reposix init` has run.

```bash
reposix list --backend sim --project demo
reposix list --backend confluence --project TokenWorld --format table
```

| Flag | Default | Purpose |
|---|---|---|
| `--backend` | `sim` | `sim`, `github`, `confluence`, `jira`. |
| `--project` | `demo` | Project slug or Confluence space key. |
| `--origin` | `http://127.0.0.1:7878` | Sim origin (ignored for non-sim). |
| `--format` | `json` | `json` or `table`. |
| `--no-truncate` | off | Confluence: error instead of capping at 500 pages. |

## `reposix refresh`

Force a delta sync. Re-fetches every record, writes Markdown files into the working tree, and creates a git commit so `git diff HEAD~1` shows backend changes since the last refresh. Agent UX equivalent: `git fetch`.

```bash
reposix refresh /tmp/repo --backend sim --project demo
```

| Flag | Default | Purpose |
|---|---|---|
| `<path>` | required | Working-tree directory. |
| `--origin` | `http://127.0.0.1:7878` | Backend origin. |
| `--project` | `demo` | Project slug / space key. |
| `--backend` | `sim` | Backend to query. |
| `--offline` | off | Reserved for the offline cache path; currently errors. |

## `reposix spaces`

List readable Confluence spaces (Confluence-only). Prints a table of space key, name, and web URL.

```bash
reposix spaces --backend confluence
```

Requires `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`, and `REPOSIX_ALLOWED_ORIGINS` — see the [Confluence reference](confluence.md).

## Removed subcommands

Two subcommands were removed in v0.9.0 alongside the pivot to git-native partial clone. Older docs, recordings, and scripts may still mention them; they no longer exist in the binary.

- `reposix mount` — removed; use [`reposix init`](#reposix-init). The working tree is now a regular partial-clone repository, so `git clone`, `git status`, and `git push` cover what `mount` used to do.
- `reposix demo` — removed; the canonical regression is `bash scripts/dark-factory-test.sh sim`, which exercises the full agent loop end-to-end against the simulator.

See the [v0.9.0 architecture-pivot summary](https://github.com/reubenjohn/reposix/blob/main/.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md) for the migration rationale.

## Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `REPOSIX_ALLOWED_ORIGINS` | `http://127.0.0.1:*,http://localhost:*` | Egress allowlist. Glob on port only. |
| `REPOSIX_CONFLUENCE_TENANT` | — | Tenant subdomain for `confluence::<space>`. |
| `REPOSIX_JIRA_INSTANCE` | — | Tenant subdomain for `jira::<key>`. |
| `GITHUB_TOKEN` | — | GitHub PAT. |
| `ATLASSIAN_EMAIL`, `ATLASSIAN_API_KEY` | — | Confluence / JIRA creds. |
| `RUST_LOG` | `info` | Tracing filter. |

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success. |
| 1 | Expected failure (e.g. SG-02 bulk-delete refusal, push-time conflict). |
| 2 | Unexpected error (backend unreachable, IO error, malformed spec). |
