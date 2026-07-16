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

**Sessions spent: 1 / 50.** (One live session spent: the P115 Task-4 MCP-arm smoke
test, 2026-07-15. The planned 6-session median-of-3 capture is **BLOCKED** — see
`.planning/phases/115-live-mcp-benchmark-re-measurement/115-MCP-SERVER-CHOICE.md` and the
v0.15.0 `SURPRISES-INTAKE.md` BLOCKER entry: the ratified `atlassian-rovo` MCP has no
Jira issue-CRUD tool and its API token is permission-denied on `tools/call`, and Jira
project KAN currently holds 0 issues. Awaiting an owner ruling before spending the
remaining capture sessions.)

## Ledger

| # | timestamp (UTC, ISO-8601) | backend | arm (mcp-mediated / reposix-mediated) | task | unit_consumed (per ruling) | running_total | artifact_produced (which fixture) |
|---|---|---|---|---|---|---|---|
<!-- Append one row per live-MCP session here, in order. Increment running_total each
     row; verify ≤ 50 BEFORE starting the next session. Flag any session > ~5× the
     running median token cost. Do not backfill. -->
| 1 | 2026-07-16T04:56:06Z | kan (context only) | mcp-mediated | smoke: list every available tool (verify mcp__atlassian-rovo__* load) | 1 | 1 | benchmarks/captures/mcp-kan-smoke.json + benchmarks/fixtures/mcp_jira_catalog.json |
