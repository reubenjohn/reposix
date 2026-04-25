---
title: Integrate reposix with your agent
---

# Integrate with your agent

reposix's product thesis is that an agent that already knows git and POSIX needs **zero new tools** to work an issue tracker. This page sketches three integration patterns — Claude Code, Cursor, and a custom SDK loop — at the level of "what to do" rather than "here is a 200-line recipe." Full vetted recipes (Claude Code, Cursor, Aider, Continue, Devin, SWE-agent CI fixtures) ship in v0.12.0; this is the pointer page.

The substrate beneath every pattern is the same: run [`reposix init`](../tutorials/first-run.md#3-bootstrap-the-working-tree) once to produce a working tree, then hand the agent that directory and let it work through `cat`, `grep`, `sed`, `git add`, `git commit`, `git push`. No MCP tool registration, no custom CLI bindings.

## Pattern 1 — Claude Code (skill)

Claude Code has a project-local skill at `.claude/skills/reposix-agent-flow/SKILL.md` that encodes the dark-factory regression test. Reuse it as the integration template.

**Setup.**

1. Run `reposix init <backend>::<project> /tmp/agent-workspace` once before launching the agent.
2. Hand the agent that path as its working directory.
3. The skill's `SKILL.md` doubles as a regression harness — invoking `/reposix-agent-flow` from a session validates that an agent given only `reposix init` can complete a task using pure git and POSIX, including the conflict-rebase and blob-limit recovery cycles.

The skill spec is the contract; treat the literal teaching strings (`git pull --rebase`, `git sparse-checkout`) in the helper's stderr as the agent's onboarding documentation. Agents read those strings and recover; you do not have to write a system prompt that explains them.

> **Gotcha.** Do **not** add a `reposix list` or `reposix get` tool to the agent's allow-list. They were the v0.1 dispatch verbs and have been removed. The substrate point of v0.9.0 is `git ls-files issues/` and `cat issues/<id>.md` — adding a custom verb undoes the dark-factory property.

## Pattern 2 — Cursor (shell loop)

Cursor agents are simpler: they `cd` into a directory and treat it as any other repo. The integration is the loop the user already runs.

**Setup.**

1. `reposix init sim::demo /tmp/cursor-repo` (or any backend).
2. Open `/tmp/cursor-repo` in Cursor.
3. Ask the agent natural-language questions ("Find issues mentioning 'database' and add a TODO comment to each"). The agent will reach for `grep -r`, `sed -i`, `git status`, and `git push` because those are the obvious tools when looking at a git repo.

No Cursor-specific configuration is required. The agent does not have to know reposix exists.

> **Gotcha.** The agent will sometimes try to construct a REST request against the backend directly. If `REPOSIX_ALLOWED_ORIGINS` is configured (it should be), those calls will be denied at the egress allowlist and surface a clear error — see [trust model §mitigations table](../how-it-works/trust-model.md#mitigations-table). Treat the allowlist denial as the signal that the agent skipped the substrate; the recovery is to push it back toward `cat` and `git push`.

## Pattern 3 — Custom SDK loop

For agents you build yourself (Python, Node, Go, anything-with-`subprocess`), the integration is two calls into the OS: spawn `reposix init` once, then `subprocess.run(...)` for each git or POSIX verb the agent wants.

**Sketch.**

```text
init        := subprocess.run(["reposix", "init", spec, path])
read_file   := subprocess.run(["cat", f"{path}/issues/{id}.md"])
search      := subprocess.run(["git", "-C", path, "grep", needle])
edit        := your-favorite-string-rewrite, then write back to file
sync_pull   := subprocess.run(["git", "-C", path, "pull", "--rebase"])
sync_push   := subprocess.run(["git", "-C", path, "push"])
```

Crucially, you do **not** register reposix tools with the model. The model's tool list is `bash`/`shell` (or whatever your harness already exposes); reposix is something it discovers via `cat .git/config` if it is curious. From the model's perspective, the working directory looks like a git repo because it IS a git repo.

> **Gotcha.** When `git push` rejects with `! [remote rejected] main -> main (fetch first)`, the recovery is `git pull --rebase && git push` — exactly what an experienced human would do. Make sure your harness surfaces the helper's stderr to the model verbatim. Truncating or suppressing stderr breaks the dark-factory teaching loop.

> **Gotcha.** When `git fetch` returns `error: refusing to fetch <N> blobs (limit: <M>)`, the recovery is `git sparse-checkout set <pathspec> && git checkout origin/main`. Same rule: surface the stderr verbatim. The error message is self-teaching by design.

## What "integration" is NOT

- **Not an MCP server.** reposix does not register tools with the model. It is a substrate that makes the tools the model already has (git, cat, grep) work against an issue tracker.
- **Not a custom CLI.** `reposix list` was removed in v0.9.0. There is `reposix init` (one-shot bootstrap) and that is the entire user-facing surface. Everything else is git or POSIX.
- **Not a sandbox.** An agent on the host can still `curl` the backend with the same token. The egress allowlist guards the helper and the cache; it does not guard the rest of the environment. See [trust model §what's NOT mitigated](../how-it-works/trust-model.md#whats-not-mitigated).

## See also

- [First-run tutorial](../tutorials/first-run.md) — the seven-step walkthrough every agent integration is built on top of.
- [Mental model in 60 seconds](../concepts/mental-model-in-60-seconds.md) — the three keys (clone IS a git working tree · frontmatter IS the schema · `git push` IS the sync verb) every agent should internalise.
- [Troubleshooting](troubleshooting.md) — what an agent should do when `git push` is rejected, when the blob limit fires, and how to read the audit log.
- [Trust model](../how-it-works/trust-model.md) — the taint and audit guarantees the agent loop runs inside.
- `.claude/skills/reposix-agent-flow/SKILL.md` — the Claude Code skill spec + regression test.
- v0.12.0 (planned) — vetted, tested recipes for Claude Code, Cursor, Aider, Continue, Devin, and SWE-agent CI fixtures.
