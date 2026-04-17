# CLI reference

The top-level `reposix` binary orchestrates the simulator, FUSE mount, and the end-to-end demo. Built from `crates/reposix-cli`.

```text
reposix — git-backed FUSE filesystem for autonomous agents

Usage: reposix <COMMAND>

Commands:
  sim      Run the Phase-2 REST simulator (delegates to reposix-sim)
  mount    Mount the FUSE filesystem (delegates to reposix-fuse)
  list     List issues/pages in a project (prints JSON or table)
  spaces   List all readable Confluence spaces (Confluence backend only)
  demo     Run the canonical end-to-end demo
  version  Print the version
  help     Print this message or the help of the given subcommand(s)
```

## `reposix list`

List issues or pages in a project. Prints JSON by default; use `--format table` for human-readable output.

```bash
reposix list --project demo
reposix list --backend confluence --project <SPACE_KEY> --format table
reposix list --backend confluence --project <SPACE_KEY> --no-truncate
```

| Flag | Default | Purpose |
|------|---------|---------|
| `--backend` | `sim` | Backend to query (`sim`, `github`, `confluence`). |
| `--project` | `demo` | Project slug (sim/GitHub) or Confluence space key. |
| `--format` | `json` | Output format: `json` or `table`. |
| `--no-truncate` | off | (Confluence only) Fail with a non-zero exit if the backend would have returned a truncated list (>500 pages). Uses `list_issues_strict` internally. |

## `reposix spaces`

List all Confluence spaces readable by the configured credentials. Confluence backend only.

```bash
reposix spaces --backend confluence
```

Prints a table of space key, space name, and Confluence web URL. Requires all four Confluence env vars (`ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`, `REPOSIX_ALLOWED_ORIGINS`). See [Confluence backend reference](../reference/confluence.md) for credential setup.

## `reposix sim`

Spawn the REST simulator as a subprocess.

| Flag | Default | Purpose |
|------|---------|---------|
| `--bind` | `127.0.0.1:7878` | Listen address. |
| `--db` | `runtime/sim.db` | SQLite file. |
| `--seed-file` | — | Path to JSON seed (e.g. `crates/reposix-sim/fixtures/seed.json`). |
| `--no-seed` | off | Don't seed even if `--seed-file` is given. |
| `--ephemeral` | off | Use in-memory SQLite instead of `--db`. |
| `--rate-limit` | `100` | Per-agent requests/sec. |

## `reposix mount`

Foreground FUSE mount. Ctrl-C unmounts cleanly via `UmountOnDrop`.

```bash
reposix mount /tmp/reposix-mnt \
    --backend http://127.0.0.1:7878 \
    --project demo
```

| Flag | Default | Purpose |
|------|---------|---------|
| `--backend` | `http://127.0.0.1:7878` | Origin of the reposix-compatible backend. Must pass the egress allowlist. |
| `--project` | `demo` | Which project's issues to present at the mount root. |
| `--read-only` | off | Forward-compat alias. v0.1 is always read-only-capable and write-capable. |

Set `REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:*,http://localhost:*` (the default) or a custom allowlist. The mount will start; individual `read`/`write` calls that hit non-allowlisted origins return EIO.

## `reposix demo`

Runs the full 9-step demo end-to-end. Used by CI and by `scripts/demo.sh` under `script(1)` for the recording.

| Flag | Default | Purpose |
|------|---------|---------|
| `--keep-running` | off | After scripted steps, block on Ctrl-C. Useful when narrating on camera. |

Output is structured and colored where helpful. Each step prints a banner (`[1/9]` … `[9/9]`) so the recorded typescript reads like a narration.

## Environment variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `REPOSIX_ALLOWED_ORIGINS` | `http://127.0.0.1:*,http://localhost:*` | Comma-separated allowlist. Glob only on port (`:*` matches any port). |
| `REPOSIX_BACKEND` | `http://127.0.0.1:7878` | Default `--backend` for `reposix mount`. |
| `RUST_LOG` | `info` | Tracing filter for all binaries. |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success. |
| 1 | Expected failure (e.g. SG-02 bulk-delete refusal). |
| 2 | Unexpected error (backend unreachable, IO error, etc.). |
