# Benchmark session-spend ledger

First-class, append-only record of every **live-MCP benchmark session** spent against
the owner's ceiling. One row per session; `running_total` is monotonic and must stay
**≤ 50**. Committed empty (schema only) *before* any session is spent — rows are appended
one-at-a-time immediately after each session completes, never backfilled.

Scope: this ledger tracks the **token-economy** benchmark rows (BENCH-01 / P115), which
are the only rows that consume live sessions. The latency-track rows run at zero session
budget (CI `bench-latency-v09`) and are **not** recorded here.

## Session unit — ruled 2026-07-15 (MANAGER ruling A1, recorded verbatim)

> One benchmark session = **one live agentic conversation / task run** (fresh context
> through task completion or abort), regardless of how many internal API calls or turns
> it makes. Failed/aborted runs still count against the 50. Re-runs are new sessions.

- **Ceiling:** ≤ **50** sessions total, on the existing subscription — no new API dollars.
- **Escalation:** past **50** sessions → **escalate to the MANAGER** (owner-directed).
- **Per-session record:** id, date, benchmark row, backend, arm, task, approximate token
  total (`unit_consumed`), outcome; `running_total` incremented each row.
- **Outlier guard:** any single session ballooning past **~5× the median** token cost of
  prior sessions is **flagged in the ledger**, not silently absorbed.

## Running total

**Sessions spent: 7 / 50.** The 6-session median-of-3 token-economy capture is
**COMPLETE** (2026-07-16, rows 2–7 below) after the **[SELF] pivot** from the infeasible
Jira/`atlassian-rovo` path (no issue-CRUD tool, token authz-denied, KAN empty) to the
**GitHub backend** (`reubenjohn/reposix` issues — a sanctioned OP-6 target). Server:
`github-probe` (official GitHub remote MCP). See
`.planning/phases/115-live-mcp-benchmark-re-measurement/115-MCP-SERVER-CHOICE.md`. Real
per-session token totals live in the `benchmarks/captures/*.json` extracts; no session
ballooned past ~5× the running median, so no outlier flag. (Row 1 remains the original
`atlassian-rovo` context-load smoke.)

## Ledger

| # | timestamp (UTC, ISO-8601) | backend | arm (mcp-mediated / reposix-mediated) | task | unit_consumed (per ruling) | running_total | artifact_produced (which fixture) |
|---|---|---|---|---|---|---|---|
<!-- Append one row per live-MCP session here, in order. Increment running_total each
     row; verify ≤ 50 BEFORE starting the next session. Flag any session > ~5× the
     running median token cost. Do not backfill. -->
| 1 | 2026-07-16T04:56:06Z | kan (context only) | mcp-mediated | smoke: list every available tool (verify mcp__atlassian-rovo__* load) | 1 | 1 | benchmarks/captures/mcp-kan-smoke.json + benchmarks/fixtures/mcp_jira_catalog.json |
| 2 | 2026-07-16T05:53:29Z | github | mcp-mediated | read 3 issues (#56,#57,#60), edit 1 (#60 marker), push | 1 | 2 | benchmarks/captures/mcp-github-run1.json + benchmarks/fixtures/mcp_github_catalog.json |
| 3 | 2026-07-16T05:57:26Z | github | mcp-mediated | read 3 issues (#56,#57,#60), edit 1 (#60 marker), push | 1 | 3 | benchmarks/captures/mcp-github-run2.json |
| 4 | 2026-07-16T05:59:18Z | github | mcp-mediated | read 3 issues (#56,#57,#60), edit 1 (#60 marker), push | 1 | 4 | benchmarks/captures/mcp-github-run3.json |
| 5 | 2026-07-16T06:00:50Z | github | reposix-mediated | read 3 issues (#56,#57,#60), edit 1 (#60 marker), push | 1 | 5 | benchmarks/captures/reposix-github-run1.json + benchmarks/fixtures/reposix_session.txt + reposix_trajectory.json |
| 6 | 2026-07-16T06:03:17Z | github | reposix-mediated | read 3 issues (#56,#57,#60), edit 1 (#60 marker), push | 1 | 6 | benchmarks/captures/reposix-github-run2.json |
| 7 | 2026-07-16T06:04:09Z | github | reposix-mediated | read 3 issues (#56,#57,#60), edit 1 (#60 marker), push | 1 | 7 | benchmarks/captures/reposix-github-run3.json |
