# reposix examples

Each example is a self-contained agent loop that exercises reposix end-to-end against the simulator. They are the "show, don't tell" companions to [`docs/tutorials/first-run.md`](../docs/tutorials/first-run.md) and [`docs/guides/integrate-with-your-agent.md`](../docs/guides/integrate-with-your-agent.md).

| Example | Language | Loop | Expected wall time |
|---|---|---|---|
| [`01-shell-loop`](01-shell-loop/) | Bash | Triage TODO -> append review comment | ~10s |
| [`02-python-agent`](02-python-agent/) | Python (stdlib only) | Find issues mentioning "database" -> add severity label | ~15s |
| [`03-claude-code-skill`](03-claude-code-skill/) | Markdown | Bridges to `.claude/skills/reposix-agent-flow` | N/A -- invoke from Claude Code |
| [`04-conflict-resolve`](04-conflict-resolve/) | Bash | Two agents touch same issue; second handles `fetch first` | ~20s |
| [`05-blob-limit-recovery`](05-blob-limit-recovery/) | Bash | Agent unaware of sparse-checkout learns from stderr -> retries | ~15s |

## How to run any example

1. Build the binaries once (workspace root):

    ```bash
    cargo build -p reposix-cli -p reposix-sim -p reposix-remote
    export PATH="$PWD/target/debug:$PATH"
    ```

2. Start the simulator in another terminal:

    ```bash
    reposix-sim --bind 127.0.0.1:7878 \
        --seed-file crates/reposix-sim/fixtures/seed.json \
        --ephemeral
    ```

3. `cd` into the example directory.
4. Read its `RUN.md`.
5. Run `./run.sh` (or `python3 run.py` for example 02).

## What you should see

Every example completes WITHOUT invoking any `reposix` subcommand other than `init`. Each one is a textbook dark-factory loop: the agent reaches for `cat`, `grep`, `sed`, `git add`, `git commit`, `git push` because that is what the working tree obviously is.

The audit log gets a row for every network op. Inspect with the sqlite query in [`docs/guides/troubleshooting.md`](../docs/guides/troubleshooting.md#read-the-audit-log):

```bash
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT ts, op, decision FROM audit_events_cache ORDER BY ts DESC LIMIT 10"
```

## See also

- [`docs/tutorials/first-run.md`](../docs/tutorials/first-run.md) -- the seven-step happy path each example builds on.
- [`docs/guides/integrate-with-your-agent.md`](../docs/guides/integrate-with-your-agent.md) -- Claude Code, Cursor, custom-SDK integration patterns.
- [`.claude/skills/reposix-agent-flow/SKILL.md`](../.claude/skills/reposix-agent-flow/SKILL.md) -- the dark-factory regression harness referenced by example 03.
- [`docs/guides/troubleshooting.md`](../docs/guides/troubleshooting.md) -- audit-log queries and recovery moves for every error message.
