# CLI reference

`reposix` is a thin orchestrator. The agent's day-to-day surface is plain `git` against a partial-clone working tree; the CLI exists to bootstrap that working tree, run the simulator, and surface a few backend-shaped queries with no clean git equivalent.

Built from `crates/reposix-cli`. Subcommands as of v0.9.0:

```text
reposix — git-native partial clone for autonomous agents

Usage: reposix <COMMAND>

Commands:
  init     Initialize a partial-clone working tree backed by reposix
  attach   Attach an existing checkout to a SoT backend (DVCS-ATTACH-01..04)
  sim      Run the REST simulator (delegates to reposix-sim)
  list     List issues/pages in a project (prints JSON or table)
  refresh  Re-fetch all issues and write a commit (force delta sync)
  spaces   List readable Confluence spaces (Confluence backend only)
  sync     On-demand cache reconciliation against the SoT (--reconcile)
  doctor   Diagnose a reposix working tree and print fix commands
  history  List sync tags (time-travel snapshots) for a working tree
  log      Time-travel log alias (--time-travel) for sync history
  at       Find the closest sync tag at-or-before a timestamp
  gc       Evict materialized blobs (or detect orphan caches) from reposix state
  tokens   Print a token-economy ledger from the audit log
  cost     Per-op cost table over the token-cost ledger (Markdown)
  version  Print the version
  help     Print this message or the help of the given subcommand(s)
```

## Common workflows

After every `reposix init`, run `reposix doctor` to verify your setup. If anything fails, copy the `Fix:` line from the finding and run it. The doctor output is the single answer to "is reposix wired correctly here?".

For long-running deployments:

- `reposix history` (or `reposix log --time-travel`) shows the cache's sync points; pair with [`reposix at <ts>`](#reposix-at) to find the snapshot for a specific time.
- `reposix init --since=<ts>` bootstraps a fresh working tree pinned to a historical snapshot — useful for "show me what this issue looked like when the bug was filed".
- `reposix cost --since 7d` aggregates the last week of token spend by op kind for billing or cost-monitoring dashboards.
- `reposix gc --orphans` enumerates caches whose owning working trees have been deleted; pair with `--purge` to actually reclaim disk space.

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
| `--since=<RFC3339>` | optional flag | Rewind the working tree to the closest cache sync tag at-or-before `<ts>` (time-travel init). Errors clearly when no such tag exists. |

`init` runs `git init`, four `git config` lines wiring the partial-clone promisor remote, and a best-effort `git fetch --filter=blob:none origin`. The working tree is plain git afterwards. Tree metadata is fetched eagerly; blobs are fetched on demand on first read.

After `init` succeeds, the absolute working-tree path is recorded in `cache.db::meta.worktrees` so [`reposix gc --orphans`](#reposix-gc) can later detect caches whose owning working tree has been deleted.

With `--since=<RFC3339>`, `init` reads sync tags from the cache's bare repo, picks the closest one at-or-before the timestamp, runs `git fetch --filter=blob:none <cache-path> <commit-oid>` from the working tree to bring the historical commit's tree into the local object store, and rewinds `refs/heads/main` + `refs/remotes/origin/main` to that commit. `git checkout main` then puts the agent at the historical snapshot. Errors with a clear message when no tag exists at-or-before the target.

`confluence::` and `jira::` specs require `REPOSIX_CONFLUENCE_TENANT` or `REPOSIX_JIRA_INSTANCE`. See [Confluence](confluence.md) and [JIRA](jira.md) for credential setup.

## `reposix attach`

Adopt an existing checkout — a vanilla `git clone` mirror, a hand-edited tree, or a prior `reposix init` — and bind it to a `SoT` backend (DVCS-ATTACH-01..04, v0.13.0). Builds a cache from REST against the `SoT`, reconciles the current `HEAD` tree against backend records by frontmatter `id`, and adds a new reposix-equipped remote configured for partial clone. The existing `origin` remote (if any) is left untouched — plain-git mirror semantics are preserved; `extensions.partialClone` is set to the *new* remote, never to `origin`.

```bash
reposix attach sim::demo                    # attach CWD to sim
reposix attach confluence::SPACE /tmp/repo  # attach a specific path
reposix attach sim::demo --no-bus           # single-SoT remote URL (skip ?mirror=)
```

| Argument / flag | Default | Purpose |
|---|---|---|
| `<spec>` | required | `<backend>::<project>`, same form as `reposix init`. |
| `<path>` | cwd | Working tree to attach (must already be a git repo — `.git/` must exist). |
| `--no-bus` | off | Skip the `?mirror=` query param; configure a single-`SoT` remote URL instead of the bus form. |
| `--mirror-name` | `origin` | Existing plain-git remote to fold into the bus URL's `?mirror=` half. |
| `--remote-name` | `reposix` | Name of the new reposix-equipped remote this command adds. |
| `--orphan-policy` | `abort` | What to do when a local record's `id` isn't found on the backend: `delete-local` (destructive), `fork-as-new` (treat as a new record to create on next push), or `abort` (refuse and report). |
| `--ignore` | `.git,.github` | Comma-separated directory names skipped during the reconciliation walk. |

Re-attaching against the *same* `SoT` is idempotent (refreshes the cache + reconciliation table); re-attaching against a *different* `SoT` is rejected. Every attach writes an unconditional `attach_walk` row to `audit_events_cache` (OP-3). See [DVCS topology](../concepts/dvcs-topology.md) for the mental model and [`docs/guides/troubleshooting.md`](../guides/troubleshooting.md) for reconciliation-warning recovery.

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

## `reposix sync`

On-demand cache reconciliation against the `SoT` — the L1 escape hatch (DVCS-PERF-L1-02). Use this when a push reject suggests cache desync (e.g. the backend deleted a record that surfaces as a REST 404 on `PATCH`). Without `--reconcile`, the bare form prints a one-line hint and exits 0 — it's reserved for future flag combinations, not an error.

```bash
reposix sync                        # prints a hint; no-op
reposix sync --reconcile            # rebuild cache from SoT (cwd)
reposix sync --reconcile /tmp/repo
```

| Flag | Default | Purpose |
|---|---|---|
| `--reconcile` | off | Run a full `list_records` walk + cache rebuild + cursor bump. Without this flag, `sync` only prints a hint. |
| `<path>` | cwd | Working-tree directory. Resolves the reposix remote the same partial-clone-aware way `doctor` / `gc` do (works on both `reposix init` and `reposix attach` trees). |

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

Checks performed (each finding is OK / INFO / WARN / ERROR; copy-pastable `Fix:` line below the message when applicable):

- `git.repo` — working tree is a git repo.
- `git.extensions.partialClone` — `extensions.partialClone` names a remote (`origin` for `reposix init`, or the `--remote-name` used by `reposix attach`) whose URL is a reposix remote. **Auto-fixable** with `--fix` only when the setting is entirely unset (defaults to `origin`); if it points at a non-reposix or missing remote, `doctor` reports a WARN with a copy-pastable fix instead of silently rewriting it (QL-004 — rewriting to `origin` would corrupt an attached tree).
- `git.remote.origin.url` — despite the check name (kept for compatibility), this resolves the *reposix* remote (partialClone-aware: `origin` for `reposix init` trees, the attach remote name for `reposix attach` trees) and confirms it uses the `reposix::` scheme and parses cleanly.
- `helper.binary` — `git-remote-reposix` is on PATH.
- `git.version` — `git --version >= 2.34` (>=2.27 minimum).
- `backend.registered` — the backend named by the URL scheme (sim/github/confluence/jira) is registered in this build.
- `cache.db` — cache DB exists at the expected path.
- `cache.db.readable` — cache DB opens cleanly.
- `cache.integrity` — `PRAGMA integrity_check = ok` (detects on-disk corruption that opens cleanly through).
- `cache.audit.table` — `audit_events_cache` table present + non-empty.
- `cache.audit.triggers` — `audit_cache_no_update` / `audit_cache_no_delete` append-only triggers present (security guardrail).
- `cache.freshness` — `meta.last_fetched_at` not older than 24h.
- `cache.refs.main` — cache's bare repo has at least one commit on `refs/heads/main`.
- `worktree.head.drift` — working-tree HEAD matches the cache's `refs/heads/main`. WARN with ahead/behind counts when they diverge.
- `env.REPOSIX_ALLOWED_ORIGINS` — env-var allowlist actually covers the configured remote (port-glob `:*` honoured for loopback). WARN when it doesn't.
- `env.REPOSIX_BLOB_LIMIT` — not set to `0` on a non-sim backend.
- `git.sparse-checkout` — pattern count (informational).
- `rustc` — Rust toolchain version (informational, contributors only).

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

## `reposix log --time-travel`

Alias for [`reposix history`](#reposix-history) using the `reposix log --time-travel` framing from v0.11.0 §3b. Prints the cache's sync tags in reverse chronological order. Without `--time-travel`, the subcommand errors — the bare `reposix log` form is reserved for a future commit-graph view.

```bash
reposix log --time-travel /tmp/repo
reposix log --time-travel /tmp/repo --limit 25
```

| Flag | Default | Purpose |
|---|---|---|
| `--time-travel` | required today | Switch to the sync-tag listing. |
| `<path>` | cwd | Working-tree directory. |
| `--limit` | `10` | Cap on entries printed. |

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

## `reposix gc`

Evict materialized blobs from a reposix cache. Tree/commit objects, refs, and sync tags are NEVER touched — only loose blob objects under `.git/objects/<2>/<38>` are eligible. Blobs re-fetch transparently on next read. See [v0.11.0 §3j](https://github.com/reubenjohn/reposix/blob/main/.planning/research/v0.11.0/vision-and-innovations.md#3j-reposix-archive--reposix-gc--bounded-disk-usage).

```bash
reposix gc                                       # LRU evict to 500 MB cap, current dir
reposix gc --strategy ttl --max-age-days 7       # evict blobs not touched in a week
reposix gc --strategy all --dry-run /tmp/repo    # plan, don't execute
```

| Flag | Default | Purpose |
|---|---|---|
| `<path>` | cwd | Working tree to gc. |
| `--strategy` | `lru` | `lru`, `ttl`, or `all`. |
| `--max-size-mb` | `500` | Cap for `--strategy=lru`. |
| `--max-age-days` | `30` | Cutoff for `--strategy=ttl`. |
| `--dry-run` | off | Print what would be evicted; don't touch disk. |
| `--orphans` | off | Cross-cache orphan mode (see below). Ignores `--strategy` / `--max-*-*` flags. |
| `--purge` | off | With `--orphans`: actually delete the orphan cache directories (default is dry-run/list-only). |
| `--include-sim` | off | With `--orphans --purge`: also remove sim-prefixed caches (`sim-*.git`); off by default to preserve simulator state. |
| `--include-untracked` | off | With `--orphans --purge`: also remove caches with no recorded owning worktree (pre-v0.11.0 caches and helper-only opens). |

Each eviction (real or dry-run) appends an `op='cache_gc'` row to `audit_events_cache` in the cache DB.

### `reposix gc --orphans`

Walks `<XDG_CACHE_HOME>/reposix/*.git/`, opens each `cache.db`, reads `meta.worktrees`, and reports caches whose recorded owning working trees no longer exist on disk. Two reasons surface:

- `all_worktrees_missing` — every recorded path is absent. Safe to purge with `--purge`.
- `no_worktrees_recorded` — meta row was never written (pre-v0.11.0 caches, simulator caches opened by `cargo run -p reposix-sim`, or helper-only opens). Preserved by default; pass `--include-untracked` to purge.

```bash
reposix gc --orphans                                  # list orphans (default)
reposix gc --orphans --purge                          # actually remove them
reposix gc --orphans --purge --include-sim            # also remove sim-*.git
reposix gc --orphans --purge --include-untracked      # remove untracked caches too
```

Each orphan line shows `<path>  <size>  reason=<reason>  <status>` plus, when applicable, each recorded-but-missing worktree path indented underneath for forensics.

## `reposix cost`

Per-op cost table over the `op='token_cost'` audit log, rendered as a pipe-friendly Markdown table. Pairs with [`reposix tokens`](#reposix-tokens) (which surfaces a back-of-envelope MCP comparison); `cost` is the raw aggregate suitable for piping into a spreadsheet or `awk`. See [v0.11.0 §3c](https://github.com/reubenjohn/reposix/blob/main/.planning/research/v0.11.0/vision-and-innovations.md#3c-token-cost-ledger--built-in-cost-telemetry).

```bash
reposix cost                                            # all-time
reposix cost --since 7d                                 # last 7 days
reposix cost --since 1m --chars-per-token 4             # last ~30 days, 4 chars/token
reposix cost --since 2026-04-25T01:00:00Z /tmp/repo     # explicit RFC-3339 cutoff
```

| Flag | Default | Purpose |
|---|---|---|
| `<path>` | cwd | Working tree whose cache to read. |
| `--since` | all-time | Filter to rows newer than the cutoff. Duration shortcuts `7d` / `30d` / `1m` / `1y` / `12h` / `30min` / `2w`, or full RFC-3339 timestamp. |
| `--chars-per-token` | `3.5` | Heuristic divisor for the token-estimate columns. Lower values inflate the estimate; reasonable values are 2.5–4. |

Output:

```text
| op       | bytes_in | bytes_out | est_input_tokens | est_output_tokens |
| -------- | -------- | --------- | ---------------- | ----------------- |
| fetch    |   12,345 |    67,890 |            3,527 |            19,397 |
| push     |      512 |       128 |              146 |                36 |
| TOTAL    |   12,857 |    68,018 |            3,673 |            19,433 |
```

Both estimates are heuristic; the chars/token divisor over-estimates for binary protocol-v2 packfile frames and under-estimates for English-text-heavy issue bodies.

## `reposix tokens`

Print a token-economy ledger derived from the cache's audit log. Reads `op='token_cost'` rows (one per helper RPC turn — `fetch` or `push`), sums them, prints totals plus an honest comparison against a back-of-envelope MCP-equivalent estimate. See [v0.11.0 §3c](https://github.com/reubenjohn/reposix/blob/main/.planning/research/v0.11.0/vision-and-innovations.md#3c-token-cost-ledger--built-in-cost-telemetry).

```bash
reposix tokens /tmp/repo
```

| Flag | Default | Purpose |
|---|---|---|
| `<path>` | cwd | Working tree whose cache to read. |

The token estimate is `chars / 4` over the WIRE bytes (incl. protocol-v2 framing). The MCP baseline is a conservative 100k schema discovery + 5k per tool call. Both are heuristic — actual savings vary by workload (blob-heavy reads favour reposix; metadata-only calls favour MCP).

## `reposix spaces`

List readable Confluence spaces (Confluence-only). Prints a table of space key, name, and web URL.

```bash
reposix spaces --backend confluence
```

Requires `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`, and `REPOSIX_ALLOWED_ORIGINS` — see the [Confluence reference](confluence.md).

## Removed subcommands

Two subcommands were removed in v0.9.0 alongside the pivot to git-native partial clone. Older docs, recordings, and scripts may still mention them; they no longer exist in the binary.

- `reposix mount` — removed; use [`reposix init`](#reposix-init). The working tree is now a regular partial-clone repository, so `git clone`, `git status`, and `git push` cover what `mount` used to do.
- `reposix demo` — removed; the canonical regression is `bash quality/gates/agent-ux/dark-factory.sh sim`, which exercises the full agent loop end-to-end against the simulator.

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
