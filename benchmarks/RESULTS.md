# Benchmark results — token economy

*Measured: 2026-04-14 05:31 UTC*

Task held constant across both scenarios: **read 3 issues, edit 1, push the
change**. What differs is only the context the agent must ingest to get
started.

| Scenario | Characters | Estimated tokens (`chars / 4`) |
|----------|-----------:|-------------------------------:|
| MCP-mediated (tool catalog + schemas) |     16,274 |      4,068 |
| **reposix** (shell session transcript) |      1,260 | **       315** |

**Reduction:** `reposix` uses **92.3%** fewer tokens than the
MCP-mediated baseline for the same task. Equivalently, MCP costs
**~12.9×** more context.

## What this does NOT measure

- Actual inference cost (token price depends on the frontier model).
- The agent's own reasoning tokens (they cancel out — the task is identical).
- Tool-call output tokens (small and comparable).
- Re-fetch of schemas if the agent's context is compacted mid-session.

## What this DOES measure

- The raw bytes the agent's context window has to hold in order to be
  productive at minute 0.
- The cost of "learning the tool" vs "using what you already know".

## Fixture provenance

- `benchmarks/fixtures/mcp_jira_catalog.json` — a representative manifest of
  35 Jira tools, modeled on the public Atlassian Forge surface and the schemas
  produced by the `mcp-atlassian` server. Full schemas for each tool, shaped
  like real JSON-Schema input descriptors.
- `benchmarks/fixtures/reposix_session.txt` — the ANSI-stripped excerpt of
  what an agent's shell actually contains after running the equivalent
  workflow through `scripts/demo.sh`.

Reproduce: `python3 scripts/bench_token_economy.py`
