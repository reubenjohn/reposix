# Benchmark results -- token economy

*Measured: 2026-04-15 19:36 UTC*
*Tokenizer: Anthropic count_tokens API (requirements-bench.txt pins anthropic==0.72.0)*

Task held constant across both scenarios: **read 3 issues, edit 1, push the
change**. What differs is only the context the agent must ingest to get
started.

## Baseline comparison (MCP-mediated vs reposix)

| Scenario | Characters | Real tokens (`count_tokens`) |
|----------|-----------:|-----------------------------:|
| MCP-mediated (tool catalog + schemas) |     16,274 |      4,883 |
| **reposix** (shell session transcript) |      1,372 | **       531** |

**Reduction:** `reposix` uses **89.1%** fewer tokens than the
MCP-mediated baseline for the same task. Equivalently, MCP costs
**~9.2x** more context.

## Per-backend raw-JSON comparison (BENCH-02)

| Backend | Raw-API fixture | Characters | Real tokens | reposix tokens | Reduction |
|---------|-----------------|-----------:|------------:|---------------:|----------:|
| Jira (MCP) | mcp_jira_catalog.json | 16,274 | 4,883 | 531 | 89.1% |
| GitHub | github_issues.json | 10,100 | 3,661 | 531 | 85.5% |
| Confluence | confluence_pages.json | 6,647 | 2,251 | 531 | 76.4% |
| Jira (real adapter) | — | — | — | — | N/A (adapter not yet implemented) |

## What this does NOT measure

- Actual inference cost (token price depends on the frontier model).
- The agent's own reasoning tokens (they cancel out -- the task is identical).
- Tool-call output tokens (small and comparable).
- Re-fetch of schemas if the agent's context is compacted mid-session.

## What this DOES measure

- The raw bytes the agent's context window has to hold in order to be
  productive at minute 0.
- The cost of "learning the tool" vs "using what you already know".
- Token counts are produced by Anthropic's `count_tokens` endpoint (SDK pinned in `requirements-bench.txt`).

## Fixture provenance

- `benchmarks/fixtures/mcp_jira_catalog.json` -- a representative manifest of
  35 Jira tools, modeled on the public Atlassian Forge surface and the schemas
  produced by the `mcp-atlassian` server. Full schemas for each tool, shaped
  like real JSON-Schema input descriptors.
- `benchmarks/fixtures/reposix_session.txt` -- the ANSI-stripped excerpt of
  what an agent's shell actually contains after running the equivalent
  workflow through `scripts/demo.sh`.
- `benchmarks/fixtures/github_issues.json` -- a synthetic GitHub REST v3
  `/repos/{owner}/{repo}/issues` payload with 3 representative issues.
- `benchmarks/fixtures/confluence_pages.json` -- a synthetic Confluence v2
  `/wiki/api/v2/pages` payload with 3 pages including full ADF body content.

Reproduce: `python3 scripts/bench_token_economy.py --offline` (cache must be populated first)
