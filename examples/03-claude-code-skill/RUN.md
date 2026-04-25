# Example 03 -- Claude Code skill pointer

Unlike examples 01, 02, 04, 05 this directory does NOT contain a runnable script. It is a pointer to the project-local Claude Code skill at:

> [`.claude/skills/reposix-agent-flow/SKILL.md`](../../.claude/skills/reposix-agent-flow/SKILL.md)

That skill is the dark-factory regression harness for v0.9.0. Invoking it from a Claude Code session spawns a subprocess agent that is given only `reposix init` and a goal, and must complete its task using pure git/POSIX -- no MCP tool registration, no in-context CLI training, no system-prompt nudging beyond the goal itself.

## How to use

From any Claude Code session in this repo:

```text
/reposix-agent-flow
```

That runs `bash scripts/dark-factory-test.sh sim` -- the regression script lives at the top of the repo.

## Two example goals you can hand the agent

The skill itself runs a fixed sanity-check against `sim::demo`. To turn it into a real-work harness, pass the agent a goal of your own. See [`agent-prompt.md`](agent-prompt.md) for the full prompt template; two ready-to-use goals are baked in.

1. "Add a severity label to every database-related issue."
2. "Find and close all TODO comments older than six months."

Either goal works against the simulator out of the box once `reposix init sim::demo /tmp/agent-workspace` has run.

## What this demonstrates

- The agent's tool list is `bash`/`shell` (or whatever your harness already exposes). Reposix is something the agent finds via `cat .git/config`, not something it has to be taught about.
- The `git pull --rebase` and `git sparse-checkout` recovery moves are baked into the helper's stderr -- the agent learns them without prompt engineering. See `.claude/skills/reposix-agent-flow/SKILL.md` §"What it asserts" for the byte-identical regression contract.

## See also

- [`docs/guides/integrate-with-your-agent.md`](../../docs/guides/integrate-with-your-agent.md) Pattern 1 (Claude Code skill).
- [`scripts/dark-factory-test.sh`](../../scripts/dark-factory-test.sh) -- the underlying script the skill invokes.
