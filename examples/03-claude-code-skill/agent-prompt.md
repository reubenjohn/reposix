# Agent prompt -- 03-claude-code-skill

Drop this into the system prompt of any LLM agent (Claude Code in autonomous mode, a custom SDK loop, Cursor, etc.). It is what we hand the agent inside the dark-factory regression harness.

## Template

```text
You are a software-engineering agent. Your task is to {GOAL}.

The codebase is a git repo at /tmp/repo. You may use cat, grep, sed, ls,
git diff, git status, git add, git commit, and git push. Do NOT invoke
any 'reposix' subcommand -- it has already been run for you. Do not
try to call REST APIs directly -- the working tree IS the issue tracker.

If `git push` is rejected with "fetch first", run `git pull --rebase`
and try the push again. If `git fetch` errors with
"refusing to fetch <N> blobs", read the stderr -- it names the
recovery move (`git sparse-checkout set <pathspec>`) verbatim.

When you finish, run `git log --oneline -5` and `git status`. The task is
done when `git status` is clean and the last commit on `main` matches
your work.
```

Replace `{GOAL}` with whatever you want the agent to accomplish.

## Goal 1 -- "Add a severity label to every database-related issue"

```text
Goal: For every issue file whose body mentions the word "database" (case
insensitive), add a `severity: medium` line to the YAML frontmatter
(after the `version:` line, before the closing `---`). Then commit with
the message `label severity:medium on database-related issues` and push.
Skip any issue that already has a `severity:` line.
```

This is the same goal Example 02 hardcodes in Python. Handing it to an agent as natural language tests whether the agent reaches for `grep -lr database .` + `sed -i` (or its own equivalent) without being told to.

## Goal 2 -- "Find and close all TODO comments older than six months"

```text
Goal: Walk every issue file. If the body contains "TODO" AND the
`created_at:` frontmatter timestamp is older than 2025-10-24, set
`status: closed` in the frontmatter. Commit the changes with the
message `close stale TODOs (>6 months)` and push.
```

The agent has to compose three primitives -- `grep -l TODO`, parse a date out of frontmatter, splice a status field -- without any reposix-specific tool. If `git push` is rejected because another agent moved one of those issues meanwhile, the agent should run `git pull --rebase && git push` (and resolve any conflicts the standard way).

## How to bootstrap before handing the agent the prompt

```bash
# Once, before launching the agent.
cd /path/to/reposix
cargo build -p reposix-cli -p reposix-sim -p reposix-remote
export PATH="$PWD/target/debug:$PATH"
reposix-sim --bind 127.0.0.1:7878 \
    --seed-file crates/reposix-sim/fixtures/seed.json \
    --ephemeral &
reposix init sim::demo /tmp/repo
cd /tmp/repo
git fetch origin || true
git checkout -B main refs/reposix/origin/main
```

After that, hand the agent `/tmp/repo` as its working directory and the prompt above as its system prompt.

## See also

- [`.claude/skills/reposix-agent-flow/SKILL.md`](../../.claude/skills/reposix-agent-flow/SKILL.md) -- the contract the regression harness asserts.
- [`docs/guides/integrate-with-your-agent.md`](../../docs/guides/integrate-with-your-agent.md) -- Pattern 1 walks through the Claude Code integration end to end.
