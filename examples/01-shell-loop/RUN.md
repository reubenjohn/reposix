# Example 01 -- Bash shell loop

A short bash script that bootstraps a working tree, finds the first open issue (`grep -lr '^status: open' .`), appends a review-comment block to the file, commits, and pushes.

> **Layout note.** The simulator's working tree puts each issue under the canonical `issues/` bucket as `issues/1.md`, `issues/2.md`, ... (unpadded id, matching the [first-run tutorial](../../docs/tutorials/first-run.md) and the QL-001 canonical path shape). This script uses a recursive `grep -lr` predicate, so it finds records wherever they live.

## What this demonstrates

- `reposix init sim::demo <path>` is the only reposix-specific command.
- After init, the agent uses `grep -r`, `cat`, `git add`, `git commit`, `git push`.
- The audit log records both a `helper_push_started` row and a `helper_push_accepted` row for the round trip.

## Prerequisites

1. Binaries built (from workspace root): `cargo build -p reposix-cli -p reposix-sim -p reposix-remote`.
2. `target/debug/` on `PATH`: `export PATH="$PWD/target/debug:$PATH"`.
3. Simulator running on `127.0.0.1:7878` in another terminal:

    ```bash
    reposix-sim --bind 127.0.0.1:7878 \
        --seed-file crates/reposix-sim/fixtures/seed.json \
        --ephemeral
    ```

## Run

```bash
./run.sh
```

## What success looks like

See [`expected-output.md`](expected-output.md) for the captured stdout, the resulting `git log`, and the audit-log rows the push produced.
