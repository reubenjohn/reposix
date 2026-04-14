# Phase 22 CONTEXT — Honest-tokenizer benchmarks: replace `len/4` with `count_tokens` API (OP-8)

> Status: scoped in session 5, 2026-04-14.
> Author: planning agent, session 6 prep.
> Can run in parallel with other v0.7.0 phases (disjoint files — Python scripts + docs only).

## Phase identity

**Name:** Honest-tokenizer benchmarks — replace `len/4` with Claude `count_tokens` API (OP-8).

**Scope tag:** v0.7.0 (benchmark methodology upgrade + docs honesty fix).

**Addresses:** OP-8 from HANDOFF.md. `scripts/bench_token_economy.py` already exists; this phase upgrades its measurement methodology and updates the headline number in docs.

## Goal (one paragraph)

The current `scripts/bench_token_economy.py` approximates token counts as `len(text)/4` — accurate to within ±10% for English+code but not honest enough for published benchmarks. This phase replaces the approximation with real calls to Anthropic's `client.messages.count_tokens()` API, caches the results in `benchmarks/fixtures/*.tokens.json` for offline reproducibility, extends the benchmark to per-backend comparison tables (GitHub, Confluence, and placeholder for Jira), measures cold-mount time-to-first-ls across a 4-backend × 3-size matrix, and re-states the token-reduction headline in `docs/why.md` with the real number — even if it is lower than the current estimate.

## Source design context

From HANDOFF.md §OP-8 (verbatim bullet list):

- **Use Claude's `count_tokens` API.** Anthropic SDK exposes `client.messages.count_tokens()`. Replace the `len/4` in `bench_token_economy.py` with a real call. Cache results in `benchmarks/fixtures/*.tokens.json` so the bench is still offline-reproducible.
- **Per-backend comparison tables.** Three runs against the same agent task:
  - (a) `gh api /repos/X/Y/issues` JSON payload ingested by an MCP agent vs `reposix list --backend github` → `cat` pipeline.
  - (b) `curl /wiki/api/v2/spaces/X/pages` JSON vs `reposix mount --backend confluence` + `cat`.
  - (c) Jira REST v3 `/issues/search` JSON vs `reposix mount --backend jira` (once that adapter exists).
  Headline number per backend. Likely range: 85%–98% reduction, depending on JSON verbosity.
- **Cold-mount time-to-first-ls.** Matrix: 4 backends × [10, 100, 500] issues. For each cell: measure wall-clock from `reposix mount` spawn to first non-empty `ls`. Expected: sim ~50 ms; github ~800 ms; confluence ~1.5 s (2 round-trips for space-resolve + page-list).
- **Git-push round-trip latency.** `echo "---\nstatus: done\n---" > 0001.md; git push` — time from `git push` to audit-row visible. Baseline for future optimisations (transaction batching, persistent HTTP).
- **Honest-framing section in `docs/why.md`.** Today's benchmark claims 92.3%; when we upgrade to real tokenisation, re-state the number. If it's lower, say so. Dishonest-but-flattering beats honest only if you don't care about the project.

## Design questions

1. **API key for `count_tokens`.** The benchmark script needs `ANTHROPIC_API_KEY`. Guard with `if not os.environ.get("ANTHROPIC_API_KEY"): sys.exit("set ANTHROPIC_API_KEY")` and document in the script's `--help`. CI can skip this gate cleanly.
2. **Cache invalidation.** `benchmarks/fixtures/*.tokens.json` should include a content hash of the input text so stale cache entries are detected. Define the cache key format before coding.
3. **Jira backend placeholder.** The Jira adapter does not exist yet. The per-backend table should have a Jira row with `N/A (adapter not yet implemented)` rather than omitting it — sets the expectation for when the adapter lands.
4. **Cold-mount matrix scope.** The 500-issue matrix cell for real backends (github, confluence) requires live credentials. Gate these cells behind `REPOSIX_BENCH_LIVE=1` so the benchmark is still runnable offline against the simulator.

## Canonical refs

- `scripts/bench_token_economy.py` — existing benchmark; upgrade in place, do not create a new script.
- `benchmarks/fixtures/` — create this directory for cached token counts (create if it does not exist).
- `docs/why.md` — honest-framing section to update with real number post-run.
- Anthropic SDK: `client.messages.count_tokens()` — see `claude-api` skill for correct usage and prompt-caching patterns.
- `HANDOFF.md §OP-8` — original design capture.
