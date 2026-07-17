# reposix token-economy benchmark

The reposix architecture paper ([`docs/research/initial-report.md`](../docs/research/initial-report.md) §"Token Economics of Filesystem Interaction") argues that exposing cloud services as a filesystem sharply cuts the token volume an agent must ingest, versus learning and calling an MCP tool surface for the same work.

This benchmark puts a real number on that claim — not by speculating, but by capturing live agentic runs.

## Where the published numbers live

The headline result and its full methodology live in the canonical, machine-regenerated results page **[`docs/benchmarks/token-economy.md`](../docs/benchmarks/token-economy.md)** (do NOT hand-edit it — see "Running the benchmark" below). In short: for the *same* task against the *same* live GitHub backend, the git-native (`reposix`) arm generates **~94% fewer output tokens** and costs **~75% less per session** than the GitHub-MCP arm.

Those figures come from **committed live session-usage records** — real Claude Code JSONL usage extracts (output tokens, cache-creation tokens, total input-context tokens, and end-to-end USD cost), from 6 sessions captured during P115 Task 4, median-of-3 per arm — under [`captures/*.json`](captures/). They are the honest end-to-end cost of a real run, NOT a `count_tokens` of a static fixture.

## What we compare

| Scenario | What the agent reads | Provenance fixture |
|----------|---------------------|--------------------|
| **MCP-mediated** | The official GitHub MCP server's tool surface | `fixtures/mcp_github_catalog.json` — a real 44-tool GitHub MCP surface recorded at capture time |
| **reposix** | A `cat` / `grep` / `git` session over a git-native checkout | `fixtures/reposix_session.txt` — a real captured session (see provenance below) |

Both arms run the same task against the same live backend: read 3 GitHub issues, edit 1, push the change.

## Fixture provenance

`fixtures/reposix_session.txt` is a real, ANSI-stripped git-native `reposix` shell session captured against the live GitHub backend (`reubenjohn/reposix`) during P115 Task 4 — no `/mnt/` paths and no demo script; it is the actual bytes the agent's shell placed in context. Full inventory plus the synthetic-vs-real split for every fixture is in [`fixtures/README.md`](fixtures/README.md); anyone tracing a published number must look at `captures/*.json` (the live session-usage records), not the shape-sample fixtures.

## Running the benchmark

```bash
# Offline (reads the committed captures; zero network, no API key):
python3 quality/gates/perf/bench_token_economy.py --offline
```

This recomputes the medians from `captures/*.json` and regenerates `docs/benchmarks/token-economy.md` byte-for-byte, so the published page and the committed captures can never silently diverge. The honest caveats (GitHub write-back is read-only in this build cut, the MCP-fidelity note, run-to-run variance) live in that page's "What this does NOT measure" section.

## Gate location (post-SIMPLIFY-11)

Per SIMPLIFY-11 (P59), the perf scripts that operate on these fixtures
were migrated to `quality/gates/perf/`. The fixtures themselves stay
here -- they are test inputs, not gates. The legacy `scripts/...`
entry points keep working via thin shims for OP-5 reversibility.

- `quality/gates/perf/latency-bench.sh` (was `scripts/latency-bench.sh`)
- `quality/gates/perf/bench_token_economy.py` (was `scripts/bench_token_economy.py`)
- `quality/gates/perf/test_bench_token_economy.py` (was `scripts/test_bench_token_economy.py`)

Cross-references: `quality/catalogs/perf-targets.json` (3 rows WAIVED until 2026-07-26 per MIGRATE-03; full gate logic lands v0.12.1) + `quality/gates/perf/README.md` (dimension-level overview).
