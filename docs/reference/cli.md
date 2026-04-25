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
  doctor   Diagnose a reposix working tree and print fix commands
  history  List sync tags (time-travel snapshots) for a working tree
  at       Find the closest sync tag at-or-before a timestamp
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

## `reposix doctor`

Audit a reposix working tree and print copy-pastable fix commands for every issue found. Inspired by `flutter doctor` / `brew doctor`. Exits 0 if no ERROR-severity finding, 1 otherwise — wire into CI as a gate.

```bash
reposix doctor                  # diagnose current dir
reposix doctor /tmp/repo
reposix doctor --fix /tmp/repo  # also apply safe fixes inline
```

| Flag | Default | Purpose |
|---|---|---|
| `<path>` | cwd | Working tree to audit. |
| `--fix` | off | Apply deterministic, non-destructive fixes (today: `git config extensions.partialClone origin`). Never mutates cache, audit log, or backend. |

Checks performed (each finding is OK / INFO / WARN / ERROR):

- Working tree is a git repo.
- `extensions.partialClone=origin` set.
- `remote.origin.url` uses `reposix::` scheme + parses cleanly.
- `git-remote-reposix` helper binary on PATH.
- `git --version >= 2.34` (>=2.27 minimum).
- Cache DB exists and opens cleanly.
- `audit_events_cache` table present + non-empty.
- `audit_cache_no_update` / `audit_cache_no_delete` append-only triggers present (security guardrail).
- `meta.last_fetched_at` not older than 24h.
- `REPOSIX_ALLOWED_ORIGINS` sane for the configured remote.
- `REPOSIX_BLOB_LIMIT` not set to `0` on a non-sim backend.
- Sparse-checkout pattern count.
- `rustc --version` (informational, contributors only).

## `reposix history`

List sync tags (time-travel snapshots) for a `reposix init`'d working tree. Every `Cache::sync` writes a private tag under `refs/reposix/sync/<ISO8601-no-colons>` in the cache's bare repo, pointing at the synthesis commit. `reposix history` prints them most-recent first. See [time travel](../how-it-works/time-travel.md) for the design.

```bash
reposix history /tmp/repo
reposix history /tmp/repo --limit 25
```

| Flag | Default | Purpose |
|---|---|---|
| `<path>` | cwd | Working-tree directory. |
| `--limit` | `10` | Cap on entries printed (most-recent first). |

Output format: `<slug>   commit <short>   <op> (<n> record(s) in this sync)` per line, plus a trailer summarising the total tag count and a copy-pastable `git -C <cache> checkout` invocation.

## `reposix at`

Print the closest sync tag at-or-before a given RFC-3339 timestamp. Useful for "what did reposix observe when this bug was filed?".

```bash
reposix at 2026-04-25T01:00:00Z /tmp/repo
```

| Flag | Default | Purpose |
|---|---|---|
| `<timestamp>` | required | RFC-3339 (e.g. `2026-04-25T01:00:00Z`). |
| `<path>` | cwd | Working-tree directory. |

Prints the matching `refs/reposix/sync/<slug>` ref name, the synthesis commit short OID, and a copy-pastable `git -C <cache> checkout` invocation. If the target predates every sync tag, prints a not-found line and exits 0.

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
