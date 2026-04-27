---
last_measured_at: 2026-04-27T05:57:07Z
---

# v0.9.0 Latency Envelope

**Generated:** 2026-04-27T05:57:07Z (commit `8ec7322`)
**Reproducer:** `bash scripts/latency-bench.sh`

## How to read this

reposix v0.9.0 replaces the per-read FUSE round-trip with a partial-clone
working tree backed by a `git-remote-reposix` promisor remote. The
golden-path latencies below characterize the sim backend (in-process
HTTP simulator) and any real backends for which credentials were
available at run time. **What's measured:** end-to-end wall-clock for
each operation, single-threaded, against an ephemeral sim DB on
localhost or the corresponding real-backend REST API. Each step is the
**median of 3 samples** to absorb network jitter. **What's NOT measured:**
runner hardware variance and cold-cache vs warm-cache TLS reuse for HTTPS
backends (a single warm-up GET amortizes the TLS handshake before timing).
Take the sim column as a lower bound for transport overhead and the
real-backend columns as a proxy for "what an agent on a typical laptop
will see."

The MCP/REST baseline comparison sits in `docs/benchmarks/token-economy.md`
(token-economy benchmark, v0.7.0). v0.9.0's win is on the latency
axis, not the token axis: the cache-backed bare repo means an agent
can `grep -r` an issue tracker without re-hitting the API for every
match.

## Latency table

| Step                                          | sim                          | github                       | confluence                   | jira                         |
|-----------------------------------------------|------------------------------|------------------------------|------------------------------|------------------------------|
| `reposix init` cold [^blob]                 | 27 ms             | 26 ms              | 26 ms              | 26 ms              |
| List records [^N]                             | 9 ms (N=6)             | 489 ms (N=17)              |               | 263 ms (N=0)              |
| Get one record                                | 8 ms              | 253 ms               |                | n/a               |
| PATCH record (no-op)                          | 12 ms            | 471 ms             |              | n/a             |
| Helper `capabilities` probe                 | 6 ms              | 6 ms               | 5 ms               | 6 ms               |

[^blob]: `reposix init` materializes blobs lazily (partial clone with
    `--filter=blob:none`). Blob counts at end of init: sim=0,
    github=0, confluence=0, jira=0.
    A non-zero count means the helper served `fetch` requests from git
    that pulled actual blob bytes during the bootstrap fetch.
[^N]: `N` = records returned by the canonical list endpoint:
    sim/github/jira issues, confluence pages in the configured space.
    **N values reflect live backend state at run time** — the TokenWorld
    space and `reubenjohn/reposix` issue count drift over time; expect
    ±20% wobble between runs. The `Helper capabilities probe` row is
    local-only (no network), so it's identical across columns and serves
    as a runner-variance control.

Real-backend cells are populated by the `bench-latency-v09` CI job
(see [`.github/workflows/ci.yml`](https://github.com/reubenjohn/reposix/blob/main/.github/workflows/ci.yml)
for cadence; the weekly cron variant lives in
[`.github/workflows/bench-latency-cron.yml`](https://github.com/reubenjohn/reposix/blob/main/.github/workflows/bench-latency-cron.yml)).

## Soft thresholds

- **sim cold init < 500ms** — regression-flagged via `WARN:` line on
  stderr; not CI-blocking. Tracked here so a sudden 5x regression
  surfaces in PR review.
- **real-backend step < 3s** — same WARN-only mechanism.

## Reproduce

```bash
bash scripts/latency-bench.sh
```

The script regenerates this file in place. To capture real-backend
columns, export the relevant credential bundle before running:

```bash
# GitHub (reubenjohn/reposix issues)
export GITHUB_TOKEN=…
# Confluence (TokenWorld space)
export ATLASSIAN_API_KEY=… ATLASSIAN_EMAIL=… REPOSIX_CONFLUENCE_TENANT=…
# JIRA (TEST project, overridable via JIRA_TEST_PROJECT)
export JIRA_EMAIL=… JIRA_API_TOKEN=… REPOSIX_JIRA_INSTANCE=…

export REPOSIX_ALLOWED_ORIGINS='https://api.github.com,https://reuben-john.atlassian.net'
bash scripts/latency-bench.sh
```

See `docs/reference/testing-targets.md` for the canonical safe-to-mutate
test targets.
