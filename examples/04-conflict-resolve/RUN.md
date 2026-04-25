# Example 04 -- Conflict resolve (the dark-factory teaching loop)

Two agents (modeled here as two separate working-tree directories) both edit issue 1. Agent A pushes first. Agent B's push is rejected with `[remote rejected] main -> main (fetch first)` -- the standard git wire-protocol message for "your base is stale". Agent B reads the stderr, drops its stale tracking ref, re-fetches the new tip, rebases its local commit onto it, and pushes successfully on the retry.

This is the dark-factory teaching loop in action. The agent recovers from a real-world hazard using moves it already knows; reposix did not have to teach it anything new.

> **v0.9.0 caveat.** A plain `git pull --rebase` fails today because every helper-side fetch produces a fresh fast-import root commit -- there is no parent chain across fetches, so the new tip does not contain the old one. The recovery in this script (drop tracking ref, re-fetch, `git rebase --onto`) is the v0.9.0 workaround. v0.10 will make `git pull --rebase` Just Work.

## What this demonstrates

- Push-time conflict detection is what `git pull --rebase` recovers from. There is no bespoke conflict protocol -- the helper turns a backend `409 version_mismatch` into the same `[remote rejected]` git already produces against any remote.
- The audit log captures the rejection: one row with `op = helper_push_rejected_conflict`, then on retry one `helper_push_started` + one `helper_push_accepted`.

## Prerequisites

- Binaries built: `cargo build -p reposix-cli -p reposix-sim -p reposix-remote`.
- Simulator running on `127.0.0.1:7878` with the demo seed (any state is fine; the script picks a known-existing issue).

## Run

```bash
./run.sh
```

## What success looks like

See [`expected-output.md`](expected-output.md) for the captured stdout, the audit-log rejection row, and the resulting git history with the rebase commit.
