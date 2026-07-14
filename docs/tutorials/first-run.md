---
title: First run — five minutes from clone to a real edit
---

# First run

Five minutes. Eight steps. By the end you will have edited an issue, pushed it through `git push`, and seen the audit row that records the write. Every command is copy-pastable.

The tutorial targets the in-process [simulator](../reference/simulator.md) so you do not need credentials, an internet connection, or any of the [real test targets](../reference/testing-targets.md). The same flow works against GitHub, Confluence, and JIRA when you are ready — only the `reposix init` argument changes.

## 1. Install reposix

Pick one — all five install both the `reposix` CLI and the
`git-remote-reposix` helper that git needs to talk to a `reposix::`
remote.

=== "curl (Linux/macOS)"
    ```bash
    curl --proto '=https' --tlsv1.2 -LsSf \
        https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.sh | sh
    ```

=== "PowerShell (Windows)"
    ```powershell
    powershell -ExecutionPolicy Bypass -c "irm https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.ps1 | iex"
    ```

=== "Homebrew"
    ```bash
    brew install reubenjohn/reposix/reposix
    ```

=== "cargo binstall"
    ```bash
    cargo binstall reposix-cli reposix-remote
    ```

=== "Build from source"
    ```bash
    git clone https://github.com/reubenjohn/reposix
    cd reposix
    cargo install --path crates/reposix-cli --path crates/reposix-remote
    # Or for ad-hoc usage without installing:
    #   cargo build -p reposix-cli -p reposix-remote
    #   export PATH="$PWD/target/debug:$PATH"
    ```

After this step, `which reposix git-remote-reposix` should print two
paths.

## 2. Start the simulator

```bash
reposix sim --bind 127.0.0.1:7878 &
# reposix-sim: listening on http://127.0.0.1:7878 (seed: builtin, 6 issues) — Ctrl-C to stop
```

Backgrounded so the rest of the steps run in the same shell. With no `--seed-file`, the simulator loads its **compiled-in demo seed** — the canonical `demo` project, six issues, deterministic IDs — so this step works from any install with no download and no network. (Pass `--seed-file <path>` to load your own fixture instead, or `--no-seed` for an empty simulator.)

> The `reposix` binary runs the simulator **in-process** — a single shipped binary is all
> you need, no separate `reposix-sim`. If you used the **Build from source** tab without
> installing, prefix the command with `target/debug/`. Every other step assumes `reposix`
> is on `PATH`.

## 3. Bootstrap the working tree

```bash
reposix init sim::demo /tmp/repo
# reposix init: configured `/tmp/repo` with remote.origin.url = reposix::http://127.0.0.1:7878/projects/demo
# Next: cd /tmp/repo && git checkout -B main refs/reposix/origin/main (or git sparse-checkout set <pathspec> first)
```

What that did: `git init /tmp/repo`, then `git config extensions.partialClone origin`, set `remote.origin.url`, and ran `git fetch --filter=blob:none origin`. `/tmp/repo` is now a real git working tree — `git status`, `git diff`, and `git log` all work the way they do on any other repo. Cold init runs in `24 ms` against the sim ([benchmark](../benchmarks/latency.md)).

> If `reposix init` fails or you see surprising output, run `reposix doctor` — it lists each diagnostic with a copy-pastable Fix command. See the [doctor reference](../reference/cli.md#reposix-doctor) for the full check catalog.

## 4. Check out the seeded branch

```bash
cd /tmp/repo
git checkout -B main refs/reposix/origin/main
```

> Why `refs/reposix/origin/main` instead of plain `origin/main`? The
> `git-remote-reposix` helper namespaces fetched refs under
> `refs/reposix/*` so that `git fast-export` correctly emits the deltas
> on push. See [git-layer](../how-it-works/git-layer.md) for the
> mechanic; the takeaway is one line of friction at clone time, zero
> friction afterwards.

## 5. Inspect the project

```bash
ls issues/
# 1.md  2.md  3.md  4.md  5.md  6.md

cat issues/1.md
# ---
# id: 1
# title: database connection drops under load
# status: open
# labels:
# - bug
# - p1
# created_at: 2026-04-13T00:00:00Z
# updated_at: 2026-04-13T00:00:00Z
# version: 1
# ---
# The <script>alert(1)</script> test harness drops connections after ~500 concurrent requests.
# ...
```

Frontmatter is the schema; the body is plain Markdown. No special tools, no MCP servers — `cat`, `ls`, `grep -r` all behave normally because the working tree IS a git repo. Read [Mental model in 60 seconds](../concepts/mental-model-in-60-seconds.md) if this still feels surprising.

## 6. Edit an issue

Add a comment to issue 1 and flip its status:

```bash
cat >> issues/1.md <<'EOF'

## Comment from tutorial
First-run tutorial — confirmed avatar upload is blocked, escalating.
EOF

sed -i 's/^status: .*/status: in_progress/' issues/1.md
git diff issues/1.md
```

Expected: a diff with two hunks — the status flip on the frontmatter line, and the comment appended at the bottom. No reposix-specific verbs were used to make the edit.

!!! note "Comments are connector-specific"

    The simulator and the GitHub backend round-trip a `## Comment`
    section in the issue body verbatim. Other backends differ:

    | Backend     | `## Comment` block treatment                          |
    |-------------|-------------------------------------------------------|
    | sim         | round-tripped verbatim                                |
    | github      | round-tripped verbatim                                |
    | confluence  | round-tripped verbatim, as plain page-body text (Confluence's own native page-comments feature is a separate, unrelated concept reposix does not expose in the working tree — see [D91-05](https://github.com/reubenjohn/reposix/blob/main/.planning/milestones/v0.13.0-phases/91-attach-sync-real-backend-wiring/91-DECISIONS.md)) |
    | jira        | not yet supported (tracked as v0.12.0 carry-forward)  |

    See `docs/reference/<backend>.md` for the per-connector comment shape.

## 7. Commit and push

```bash
git add issues/1.md
git commit -m "tutorial: add comment, move issue 1 to in_progress"
git push
# To reposix::http://127.0.0.1:7878/projects/demo
#    5df9f45..3c0f29d  main -> main
```

The push is a **fast-forward** (`<old>..<new>`), not a `[new branch]` create — `reposix init` already seeded `main` on the remote, so your commit just advances it. The exact hashes are illustrative and will differ on every run.

What just happened: `git push` handed your commit to the reposix git remote, which parsed the changed file, sanitized the frontmatter (server-controlled fields like `id` and `version` are stripped — see [trust model](../how-it-works/trust-model.md)), checked the backend version was still `1`, and applied the write via REST. The mechanics live in [git layer §push round-trip](../how-it-works/git-layer.md).

If a second writer had mutated issue 1 between your `init` and your `push`, the helper would have rejected the push with `! [remote rejected] main -> main (fetch first)` and you would have run `git pull --rebase && git push` to recover. That conflict-rebase loop is the dark-factory teaching mechanism — try it as an exercise.

## 8. Verify the audit row

```bash
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT ts, op, reason FROM audit_events_cache \
     WHERE op LIKE 'helper_push_%' ORDER BY ts DESC LIMIT 3"
# 2026-07-07T23:06:07.162650444+00:00|helper_push_accepted|1
# 2026-07-07T23:06:07.159171609+00:00|helper_push_sanitized_field|version
# 2026-07-07T23:06:07.154427867+00:00|helper_push_started|refs/heads/main
```

(Timestamps are RFC3339 with nanosecond precision and a `+00:00` UTC offset; yours will differ.)

Three rows: the push opened, a server-controlled field got stripped, the push was accepted. That middle row is normal, not a warning sign — issue `1.md`'s frontmatter still carries the server-controlled `version` field reposix wrote when it materialized the file, and `git push` strips it before applying the write (server fields round-trip out, never in — see [trust model](../how-it-works/trust-model.md)). You'd only see `helper_push_rejected_conflict` if a second writer had raced you. Every push, accept or reject, writes one append-only audit row. `git log` is the agent's intent; `audit_events_cache` is the system's outcome.

## What did you do?

You used `cat`, `git`, `sed`, `git push`. No reposix-specific commands except `init`. **That is the dark factory pattern in action** — the substrate disappears once it is set up, and the agent works through tools it (and you) already know.

## Next

- [Mental model in 60 seconds](../concepts/mental-model-in-60-seconds.md) — the three keys that explain why every step above worked.
- [How reposix complements MCP and SDKs](../concepts/reposix-vs-mcp-and-sdks.md) — when to reach for reposix, when to reach for an MCP server.
- [Troubleshooting](../guides/troubleshooting.md) — what to do when `git push` is rejected, when `git fetch` hits the blob limit, and how to read the audit log.
- [Write your own connector](../guides/write-your-own-connector.md) — graduate from the sim to a backend that does not exist yet.
- [Testing targets](../reference/testing-targets.md) — when you are ready to point `reposix init` at GitHub, Confluence, or JIRA.
