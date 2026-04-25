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
reposix sim --seed-file crates/reposix-sim/fixtures/seed.json &
# 2026-04-24T12:00:00.000Z  INFO reposix_sim: reposix-sim listening addr=127.0.0.1:7878
```

Backgrounded so the rest of the steps run in the same shell. The seed loads the canonical `demo` project — five issues, deterministic IDs.

> If you used the **Build from source** tab above without installing, run
> `target/debug/reposix sim --seed-file crates/reposix-sim/fixtures/seed.json &` instead.
> Every other step assumes `reposix` is on `PATH`.

## 3. Bootstrap the working tree

```bash
reposix init sim::demo /tmp/repo
# reposix init: configured `/tmp/repo` with remote.origin.url = reposix::http://127.0.0.1:7878/projects/demo
# Next: cd /tmp/repo && git checkout -B main refs/reposix/origin/main
```

What that did: `git init /tmp/repo`, then `git config extensions.partialClone origin`, set `remote.origin.url`, and ran `git fetch --filter=blob:none origin`. `/tmp/repo` is now a real git working tree — `git status`, `git diff`, and `git log` all work the way they do on any other repo. Cold init runs in `24 ms` against the sim ([benchmark](../benchmarks/v0.9.0-latency.md)).

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
# 0001.md  0002.md  0003.md  0004.md  0005.md

cat issues/0001.md
# ---
# id: 1
# title: Add user avatar upload
# status: open
# assignee: alice@acme.com
# labels: [backend, needs-review]
# version: 1
# ---
# ## Description
# Avatar uploads are blocked by S3 permissions...
```

Frontmatter is the schema; the body is plain Markdown. No special tools, no MCP servers — `cat`, `ls`, `grep -r` all behave normally because the working tree IS a git repo. Read [Mental model in 60 seconds](../concepts/mental-model-in-60-seconds.md) if this still feels surprising.

## 6. Edit an issue

Add a comment to issue 1 and flip its status:

```bash
cat >> issues/0001.md <<'EOF'

## Comment from tutorial
First-run tutorial — confirmed avatar upload is blocked, escalating.
EOF

sed -i 's/^status: .*/status: in_progress/' issues/0001.md
git diff issues/0001.md
```

Expected: a diff with two hunks — the status flip on the frontmatter line, and the comment appended at the bottom. No reposix-specific verbs were used to make the edit.

## 7. Commit and push

```bash
git add issues/0001.md
git commit -m "tutorial: add comment, move issue 1 to in_progress"
git push
# To reposix::http://127.0.0.1:7878/projects/demo
#  * [new branch]      main -> main
```

What just happened: `git push` handed your commit to the reposix git remote, which parsed the changed file, sanitized the frontmatter (server-controlled fields like `id` and `version` are stripped — see [trust model](../how-it-works/trust-model.md)), checked the backend version was still `1`, and applied the write via REST. The mechanics live in [git layer §push round-trip](../how-it-works/git-layer.md).

If a second writer had mutated issue 1 between your `init` and your `push`, the helper would have rejected the push with `! [remote rejected] main -> main (fetch first)` and you would have run `git pull --rebase && git push` to recover. That conflict-rebase loop is the dark-factory teaching mechanism — try it as an exercise.

## 8. Verify the audit row

```bash
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
    "SELECT ts, op, decision FROM audit_events_cache \
     WHERE op LIKE 'helper_push_%' ORDER BY ts DESC LIMIT 3"
# 2026-04-24T12:01:32Z|helper_push_accepted|ok
# 2026-04-24T12:01:32Z|helper_push_started|
```

Two rows: the push opened, the push accepted. No `helper_push_rejected_conflict`, no `helper_push_sanitized_field` (you did not try to overwrite a server field). Every push, accept or reject, writes one append-only audit row. `git log` is the agent's intent; `audit_events_cache` is the system's outcome.

## What did you do?

You used `cat`, `git`, `sed`, `git push`. No reposix-specific commands except `init`. **That is the dark factory pattern in action** — the substrate disappears once it is set up, and the agent works through tools it (and you) already know.

## Next

- [Mental model in 60 seconds](../concepts/mental-model-in-60-seconds.md) — the three keys that explain why every step above worked.
- [How reposix complements MCP and SDKs](../concepts/reposix-vs-mcp-and-sdks.md) — when to reach for reposix, when to reach for an MCP server.
- [Troubleshooting](../guides/troubleshooting.md) — what to do when `git push` is rejected, when `git fetch` hits the blob limit, and how to read the audit log.
- [Write your own connector](../guides/write-your-own-connector.md) — graduate from the sim to a backend that does not exist yet.
- [Testing targets](../reference/testing-targets.md) — when you are ready to point `reposix init` at GitHub, Confluence, or JIRA.
