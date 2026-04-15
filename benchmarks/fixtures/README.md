# Benchmarks fixtures — provenance + offline contract

These files are the inputs consumed by `scripts/bench_token_economy.py`. Each fixture is a
synthetic, anonymized, deterministic artifact constructed to represent the *shape* of a real
API response — not scraped from any live system. The script reads each fixture, counts tokens
via the Anthropic `count_tokens` API, and caches the result in a `*.tokens.json` sidecar file
committed alongside the fixture. Subsequent runs (including CI) pass `--offline` to read cached
counts and skip all network calls.

## Fixture inventory

| File | Size (bytes) | Shape | Purpose | Backend row |
|------|-------------|-------|---------|-------------|
| `mcp_jira_catalog.json` | 19362 | JSON object with `tools[]` | MCP-mediated baseline (35-tool Jira manifest) | MCP |
| `reposix_session.txt` | 1372 | ANSI-stripped shell transcript | reposix POSIX session (read 3 issues, edit 1) | reposix |
| `github_issues.json` | 11579 | JSON array, GitHub REST v3 `/repos/{owner}/{repo}/issues` | GitHub `/issues` raw payload for token-count comparison | github (BENCH-02) |
| `confluence_pages.json` | 7281 | JSON object with `results[]`, Confluence v2 `/wiki/api/v2/pages` shape | Confluence pages raw payload for token-count comparison | confluence (BENCH-02) |

## Synthetic data disclaimer

Every fixture in this directory is constructed, not scraped. No real user data, no real tenant
references, no production credentials. The GitHub fixture uses `example-org/example-repo` and
the Confluence fixture uses `example.atlassian.net` — these are deliberately fictional identifiers
that make provenance obvious at a glance. User login names (`alice-bot`, `benchmark-ci`,
`fuse-agent`) and Confluence author IDs are invented. Anyone tracing a token-reduction claim
back to these files must understand they represent *shape*, not *size of any real production
payload*. The fixtures are designed to exercise the full verbosity of each API response format,
not to match the volume of any specific real-world repository or Confluence space.

## Offline-reproducibility contract

Token counts are expensive to regenerate (each requires an Anthropic API call) but cheap to
cache. The workflow is:

1. **First run (developer with API key):** Set `ANTHROPIC_API_KEY` and run
   `python3 scripts/bench_token_economy.py`. The script calls `client.messages.count_tokens()`
   for each fixture whose sidecar does not yet exist or whose `content_hash` is stale. It writes
   sidecar files of the form:
   ```json
   {
     "content_hash": "<sha256 hex of fixture bytes>",
     "input_tokens": 1234,
     "source": "github_issues.json",
     "model": "claude-3-haiku-20240307",
     "counted_at": "2026-04-15T12:00:00Z"
   }
   ```
2. **Sidecars are committed to git.** Every `*.tokens.json` file belongs in version control so
   CI can run the benchmark without an API key. If a sidecar is missing from the repo, CI will
   fail with an explicit message directing the developer to regenerate it.
3. **Subsequent runs (CI and offline developers):** Pass `--offline` to
   `python3 scripts/bench_token_economy.py --offline`. The script reads each sidecar, verifies
   the fixture's current `sha256` matches the cached `content_hash`, and uses the cached
   `input_tokens` count. No Anthropic API calls are made.
4. **Cache invalidation:** If a fixture's bytes change (even a single character), its `sha256`
   will differ from the cached `content_hash`. Under `--offline`, the script exits non-zero with
   a message explaining which fixture is stale and how to regenerate the sidecar. Do not manually
   edit sidecar files to fix a mismatch — regenerate them with the API key.

## Adding a new fixture

1. Create `benchmarks/fixtures/<name>.{json,txt}` with synthetic, anonymized content shaped
   like the real API response you want to benchmark.
2. Run `ANTHROPIC_API_KEY=<your-key> python3 scripts/bench_token_economy.py` to populate
   `benchmarks/fixtures/<name>.{json,txt}.tokens.json`.
3. Commit both files (`<name>.{json,txt}` and `<name>.{json,txt}.tokens.json`) and update the
   fixture inventory table in this README with the measured byte count from `wc -c`.

`scripts/bench_token_economy.py` is the canonical consumer. See its `--help` output for the
full CLI reference.
