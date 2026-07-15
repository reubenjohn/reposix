---
last_measured_at: 2026-07-15T21:36:36Z
---

# v0.9.0 Latency Envelope

**Generated:** 2026-07-15T21:36:36Z (commit `3278abc`)
**Reproducer:** `bash quality/gates/perf/latency-bench.sh`

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
| `reposix init` cold [^blob]                 | 155 ms             |               |               |               |
| List records [^N]                             | 20 ms (N=6)             |               |               |               |
| Get one record                                | 19 ms              |                |                |                |
| PATCH record (no-op)                          | 78 ms            |              |              |              |
| Helper `capabilities` probe                 | 10 ms              |                |                |                |

[^blob]: `reposix init` materializes blobs lazily (partial clone with
    `--filter=blob:none`). Blob counts at end of init: sim=0,
    github=0, confluence=0, jira=0.
    A non-zero count means the helper served `fetch` requests from git
    that pulled actual blob bytes during the bootstrap fetch.
[^N]: `N` = records returned by the canonical list endpoint:
    sim/github/jira issues, confluence pages in the configured space.
    **N values reflect live backend state at run time** — the configured
    Confluence space (`TokenWorld`) and `reubenjohn/reposix`
    issue count drift over time; expect
    ±20% wobble between runs. The `Helper capabilities probe` row is
    local-only (no network), so it's identical across columns and serves
    as a runner-variance control.

Real-backend cells are populated by the `bench-latency-v09` CI job
(see [`.github/workflows/ci.yml`](https://github.com/reubenjohn/reposix/blob/main/.github/workflows/ci.yml)
for cadence; the weekly cron variant lives in
[`.github/workflows/bench-latency-cron.yml`](https://github.com/reubenjohn/reposix/blob/main/.github/workflows/bench-latency-cron.yml)).

## Summary — authoritative cold-init figure (BENCH-01, P115)

Prior to this measurement, downstream docs (`docs/index.md`, `README.md`, and the
concept/tutorial pages) cited a split between **24 ms** and **27 ms** for sim
`reposix init` cold bootstrap, sourced from earlier bench runs and never fully
reconciled with each other. This live re-run of `quality/gates/perf/latency-bench.sh`
on 2026-07-15T21:36:36Z (commit `3278abc`) is the current authoritative measurement:

- **Cold init (authoritative): 155 ms** — sim `reposix init`, single-sample
  bootstrap, this run. This figure supersedes both the 24 ms and 27 ms values
  above as the current sim cold-init number; it remains well under the 500 ms
  soft threshold.
- **Cached read (proxy: `Get one record`): 19 ms** — median of 3 samples, this run.

Reconciling the superseded 24 ms/27 ms prose in `docs/index.md`, `README.md`, and
the concept/tutorial docs against this figure is explicitly out of scope for this
measurement pass — tracked as Phase 117/118 follow-up.

## Soft thresholds

- **sim cold init < 500ms** — regression-flagged via `WARN:` line on
  stderr; not CI-blocking. Tracked here so a sudden 5x regression
  surfaces in PR review.
- **real-backend step < 3s** — same WARN-only mechanism.

## Reproduce

```bash
bash quality/gates/perf/latency-bench.sh
```

The script regenerates this file in place. To capture real-backend
columns, export the relevant credential bundle before running:

```bash
# GitHub (reubenjohn/reposix issues)
export GITHUB_TOKEN=…
# Confluence (space key set via REPOSIX_CONFLUENCE_SPACE; default TokenWorld)
export ATLASSIAN_API_KEY=… ATLASSIAN_EMAIL=… REPOSIX_CONFLUENCE_TENANT=… REPOSIX_CONFLUENCE_SPACE=…
# JIRA (TEST project, overridable via JIRA_TEST_PROJECT)
export JIRA_EMAIL=… JIRA_API_TOKEN=… REPOSIX_JIRA_INSTANCE=…

export REPOSIX_ALLOWED_ORIGINS='https://api.github.com,https://reuben-john.atlassian.net'
bash quality/gates/perf/latency-bench.sh
```

See `docs/reference/testing-targets.md` for the canonical safe-to-mutate
test targets.
