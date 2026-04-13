# CLI reference

The top-level `reposix` binary orchestrates the simulator, FUSE mount, and the end-to-end demo. Built from `crates/reposix-cli`.

```text
reposix — git-backed FUSE filesystem for autonomous agents

Usage: reposix <COMMAND>

Commands:
  sim      Run the Phase-2 REST simulator (delegates to reposix-sim)
  mount    Mount the FUSE filesystem (delegates to reposix-fuse)
  demo     Run the canonical end-to-end demo
  version  Print the version
  help     Print this message or the help of the given subcommand(s)
```

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
