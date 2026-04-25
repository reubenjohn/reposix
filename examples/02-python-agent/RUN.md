# Example 02 -- Python agent (stdlib only)

A Python script that does the same dark-factory loop as `01-shell-loop`, except in Python and with a non-trivial mutation: it walks every issue file, finds those whose body mentions "database", parses the frontmatter with regex (no `pyyaml`), inserts `severity: medium` into the frontmatter, and pushes.

Stdlib only -- no `requests`, no `anthropic`, no `pyyaml`. The only network-touching code path is `subprocess.run(["git", ...])`.

## What this demonstrates

- An agent harness in Python can drive reposix with two primitives: `subprocess.run(["reposix", "init", ...])` once, then `subprocess.run(["git", ...])` for the rest.
- Frontmatter is YAML but the agent does not need a YAML parser to add a field -- it can splice into the existing `---` fence with a regex (the file IS a string).
- The push round-trip emits one `helper_push_accepted` row in the audit log for every issue that was modified.

## Prerequisites

Same as `01-shell-loop`:

1. `cargo build -p reposix-cli -p reposix-sim -p reposix-remote` (workspace root).
2. Simulator running on `127.0.0.1:7878` with the demo seed.
3. Python 3.10+ on `PATH` (uses only stdlib).

## Run

```bash
python3 run.py
```

## What success looks like

See [`expected-output.md`](expected-output.md) for the captured stdout and the resulting `git diff`.
