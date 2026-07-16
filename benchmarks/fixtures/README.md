# Benchmarks fixtures — provenance + offline contract

> **Headline source moved (P115 T5, 2026-07-16).** The published token-economy
> numbers no longer come from these fixtures. They are computed from committed
> **live session-usage records** under `benchmarks/captures/*.json` (see
> `docs/benchmarks/token-economy.md` § Methodology). The regenerator is
> `quality/gates/perf/bench_token_economy.py --offline` — deterministic, offline,
> no `ANTHROPIC_API_KEY`. The files in *this* directory are retained as
> **provenance + an optional per-artifact enrichment input**, not as the headline.

## Fixture inventory

| File | Shape | Role after P115 T5 |
|------|-------|--------------------|
| `mcp_github_catalog.json` | JSON, 44-tool GitHub MCP tool surface | **Real** capture of the GitHub MCP server's tool surface at P115 T4 capture time. Provenance for the MCP arm's tool set. |
| `reposix_session.txt` | ANSI-stripped shell transcript | **Real** git-native reposix session against the live GitHub backend (`reubenjohn/reposix`), captured P115 T4. No `/mnt/` paths, no `scripts/demo.sh`. |
| `mcp_jira_catalog.json` | JSON object with `tools[]` | Honest 3-tool `atlassian-rovo` record retained from the infeasible-Jira attempt (see `115-MCP-SERVER-CHOICE.md`). Synthetic-shape; **not** a published number. |
| `github_issues.json` | JSON array, GitHub REST v3 `/issues` shape | Synthetic raw-payload sample for optional per-artifact enrichment only. |
| `confluence_pages.json` | JSON object, Confluence v2 `/wiki/api/v2/pages` shape | Synthetic raw-payload sample for optional per-artifact enrichment only. |

## Provenance disclaimer

`reposix_session.txt` and `mcp_github_catalog.json` were **live-captured** during
the P115 Task 4 benchmark against the sanctioned public OP-6 target
`reubenjohn/reposix` (public RUSTSEC-advisory issues — no private data, no
credentials; secret-scrubbed before commit). The remaining fixtures
(`github_issues.json`, `confluence_pages.json`, `mcp_jira_catalog.json`) are
synthetic, anonymized samples constructed to represent the *shape* of an API
response — the GitHub sample uses `example-org/example-repo` and the Confluence
sample uses `example.atlassian.net`, deliberately fictional identifiers. Anyone
tracing a published token-economy number must look at `benchmarks/captures/*.json`
(the live session-usage records), not these shape samples.

## Optional enrichment: `*.tokens.json` sidecars

The `*.tokens.json` sidecars cache Anthropic `count_tokens` results for the
synthetic raw-payload samples. This path is **optional per-artifact enrichment**,
demoted from the headline by P115 amendment #10; it is not required to reproduce
any published figure. When wired, it runs offline from committed sidecars via
`python3 quality/gates/perf/bench_token_economy.py` with an `ANTHROPIC_API_KEY`
only on first population (SHA-256 content-hash keyed; committed for offline CI).

> The stale `reposix_session.txt.tokens.json` sidecar (left over from when this
> transcript backed the retired count_tokens headline) was **deleted** in P115 T5
> — nothing consumes it under the captures methodology (GTH-V15-26).

## Adding a new synthetic sample

1. Create `benchmarks/fixtures/<name>.{json,txt}` with synthetic, anonymized
   content shaped like the real API response.
2. (Optional enrichment only) populate its sidecar with
   `ANTHROPIC_API_KEY=<key> python3 quality/gates/perf/bench_token_economy.py`.
3. Commit the fixture (and, if used, its sidecar).

The canonical consumer of the token-economy HEADLINE is
`quality/gates/perf/bench_token_economy.py` reading `benchmarks/captures/*.json`.
See its module docstring for the full methodology.
