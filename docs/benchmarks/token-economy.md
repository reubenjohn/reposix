# Benchmark results -- token economy

*Measured: 2026-07-16, from 6 live agentic sessions captured during P115 Task 4.*
*Source: committed session-usage records in `benchmarks/captures/*.json` (median-of-3 per arm).*

## Methodology

These numbers are the honest **end-to-end cost of a real agentic run**, not a
token-count of a static fixture. Two arms ran the **same task** against the
**same live backend**, 3 times each (median-of-3); each
session's Claude Code JSONL **usage record** (output tokens, cache-creation
tokens, total input-context tokens, and end-to-end USD cost) was scrubbed to an
offline-CI-stable extract and committed.

- **MCP-mediated arm** -- the agent reaches the backend through the official GitHub MCP server (`github/github-mcp-server`, GA), registered locally as `github-probe`.
- **reposix-mediated arm** -- the agent reaches the SAME backend content through
  a `reposix` git-native checkout, using only `cat` / `grep` / `git` (run under
  `--strict-mcp-config` with zero MCP servers loaded, so its usage carries no
  MCP tool-loading cost).
- **Backend:** GitHub (reubenjohn/reposix) (public RUSTSEC-advisory issues; no private data).
- **Task (held constant):** read 3 GitHub issues (#56, #57, #60), edit 1 (#60 body marker), push.
- **Model:** claude-sonnet-5 (`--model sonnet`).

Runs are offline-reproducible: `python3 quality/gates/perf/bench_token_economy.py --offline`
recomputes the medians from the committed captures with no `ANTHROPIC_API_KEY`
and no network, and regenerates this file byte-for-byte.

## Headline: reposix is ~94% fewer output tokens and ~75% cheaper per session

For the identical task against the identical live GitHub backend, the
git-native (`reposix`) arm is cheaper on **every** axis than the GitHub-MCP arm.
The two lead numbers are **output tokens** (what the agent has to generate) and
**end-to-end USD cost** (what the run actually costs):

| Axis | reposix (median) | GitHub MCP (median) | reposix advantage |
|------|-----------------:|--------------------:|:------------------|
| Output tokens (agent generates) | 1,213 | 21,171 | **~94.3% fewer** |
| Cache-creation tokens (new context cached) | 19,068 | 56,129 | ~66.0% fewer |
| Total input-context tokens | 244,556 | 550,219 | ~55.6% smaller |
| Cost per session (USD) | $0.2076 | $0.8278 | ~74.9% cheaper |

Equivalently: the MCP arm costs **~4.0x** more per session and
carries **~2.25x** the total input-context for the same result.

## What retired the old 89.1% / 85.5% figures

The previous token-economy figures (an **89.1%** headline and a per-backend
**85.5%** GitHub number) came from a *different, synthetic* methodology: running
Anthropic's `count_tokens` over a static, hand-constructed JSON fixture that
stood in for an MCP tool catalog. That measured the size of a fixture, not the
cost of a live agent run. It is **retired here** in favour of the live
session-usage medians above -- real sessions, a real GitHub backend, and the
GitHub MCP server's real tool surface. The synthetic fixtures remain in the repo
only as provenance for that retired estimate; they no longer back any published
number.

## What this DOES measure

- The real, end-to-end token + dollar cost of an agent completing the task, as
  recorded by Claude Code's own per-session usage accounting.
- The cost of "learning + calling the tool surface" (MCP) vs "using POSIX + git
  you already know" (reposix), against a live backend.

## What this does NOT measure (honest caveats)

- **reposix write-back on GitHub is read-only in this build cut.** The
  reposix arm read + locally edited + attempted a push; the push was correctly
  rejected by the documented read-only GitHub adapter. The token comparison is
  unaffected -- it measures agent context size, not write capability -- but these
  numbers must not be read as a claim that reposix persists writes to GitHub.
- **Fidelity note (factual):** during capture, the GitHub MCP `issue_read`
  HTML-escaped body content (`>=` -> `&gt;=`) and dropped literal angle-bracket
  text, so an MCP read-modify-write round-trip altered raw markdown; the reposix
  arm round-tripped bytes unchanged. Recorded for accuracy, not as a headline.
- Absolute numbers vary run-to-run with backend content and agent path; the
  medians above smooth 3 runs per arm but are not a guarantee for any
  single session.

## Capture provenance

- `benchmarks/captures/reposix-github-run{1,2,3}.json`,
  `benchmarks/captures/mcp-github-run{1,2,3}.json` -- the six scrubbed
  session-usage extracts (session id, per-axis token counts, turn count,
  end-to-end USD cost, tool-call names). No backend body content, no secrets.
  Captured live during P115 Task 4 against GitHub (reubenjohn/reposix).
  `benchmarks/captures/mcp-kan-smoke.json` is a context-only smoke probe and is
  **excluded** from the medians.
- `benchmarks/fixtures/mcp_github_catalog.json` -- the real 44-tool GitHub MCP
  tool surface recorded at capture time (provenance for the MCP arm's tool set).
- `benchmarks/fixtures/reposix_session.txt` -- an ANSI-stripped transcript of the
  reposix arm's git-native shell session against the live GitHub backend.

Reproduce: `python3 quality/gates/perf/bench_token_economy.py --offline`
(deterministic; no API key, no network).
