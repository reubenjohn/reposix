# Twitter / X post

## Tweet 1 (main message)

Just had a "whoa, coding agents are cool" moment.

Idea: expose REST APIs (GitHub Issues, Confluence, JIRA…) as file systems. Agents are wildly more efficient on files than tool schemas.

I told Claude Code "you have until 8 AM. Go wild. Night!"

I woke up to this 👇

## Tweet 2 (details + link)

Part 2:
Result: **reposix** — a git-native partial clone + git-remote-helper for issue trackers.

In a live benchmark against the same GitHub backend (official GitHub MCP server vs a `reposix` git-native shell session, same task): **~94% fewer output tokens · ~75% cheaper** per session (median-of-3 — see the token-economy benchmark).

Only inputs I gave it: https://github.com/reubenjohn/reposix/blob/603dfa558dd1266515be47f7cd92376c861c34d5/InitialReport.md

Morning brief: https://github.com/reubenjohn/reposix/blob/main/MORNING-BRIEF.md

---

## Notes / alternates

**Alt opener** (if 280 is tight — count is fine as-is but keep this in reserve):
> "Coding agents are cool" moment last night. Idea: turn REST APIs (GitHub Issues, Confluence, JIRA) into file systems — agents are far more efficient on files than tool schemas. Told Claude Code "you have until 8 AM, go wild." Woke up to this 👇

**If you want to drop the emoji**, replace "👇" with "— links below" or just end the tweet at "I woke up to this."

**Character counts** (Twitter-weighted for the 280 limit, without the heading: each URL counts as 23 via t.co, 👇 counts as 2, markdown `**`/`` ` `` is not posted):
- Tweet 1: 267 (fits 280)
- Tweet 2: 413 — **over the 280 limit**; post as a thread or trim (the two GitHub links alone consume 46 of the budget)
