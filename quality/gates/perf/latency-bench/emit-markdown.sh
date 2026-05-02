#!/usr/bin/env bash
# quality/gates/perf/latency-bench/emit-markdown.sh -- format collected timings as docs/benchmarks/latency.md.
#
# Sourced by ../latency-bench.sh after all backend probe blocks have run.
# Reads: WORKSPACE_ROOT, OUT, all SIM_/GH_/CF_/JR_ timing variables.
# Writes: $OUT (the docs/benchmarks/latency.md artifact).

GENERATED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
GIT_SHA="$(git -C "$WORKSPACE_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"

SIM_INIT_CELL="$(fmt_ms "$SIM_INIT_MS")"
SIM_LIST_CELL="$(fmt_ms_n "$SIM_LIST_MS" "$SIM_N")"
SIM_GET_CELL="$(fmt_ms "$SIM_GET_MS")"
SIM_PATCH_CELL="$(fmt_ms "$SIM_PATCH_MS")"
SIM_CAP_CELL="$(fmt_ms "$SIM_CAP_MS")"

GH_INIT_CELL="$(fmt_ms "$GH_INIT_MS")"
GH_LIST_CELL="$(fmt_ms_n "$GH_LIST_MS" "$GH_N")"
GH_GET_CELL="$(fmt_ms "$GH_GET_MS")"
GH_PATCH_CELL="$(fmt_ms "$GH_PATCH_MS")"
GH_CAP_CELL="$(fmt_ms "$GH_CAP_MS")"

CF_INIT_CELL="$(fmt_ms "$CF_INIT_MS")"
CF_LIST_CELL="$(fmt_ms_n "$CF_LIST_MS" "$CF_N")"
CF_GET_CELL="$(fmt_ms "$CF_GET_MS")"
CF_PATCH_CELL="$(fmt_ms "$CF_PATCH_MS")"
CF_CAP_CELL="$(fmt_ms "$CF_CAP_MS")"

JR_INIT_CELL="$(fmt_ms "$JR_INIT_MS")"
JR_LIST_CELL="$(fmt_ms_n "$JR_LIST_MS" "$JR_N")"
JR_GET_CELL="$(fmt_ms "$JR_GET_MS")"
JR_PATCH_CELL="$(fmt_ms "$JR_PATCH_MS")"
JR_CAP_CELL="$(fmt_ms "$JR_CAP_MS")"

cat > "$OUT" <<MARKDOWN
---
last_measured_at: ${GENERATED_AT}
---

# v0.9.0 Latency Envelope

**Generated:** ${GENERATED_AT} (commit \`${GIT_SHA}\`)
**Reproducer:** \`bash scripts/latency-bench.sh\`

## How to read this

reposix v0.9.0 replaces the per-read FUSE round-trip with a partial-clone
working tree backed by a \`git-remote-reposix\` promisor remote. The
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

The MCP/REST baseline comparison sits in \`docs/benchmarks/token-economy.md\`
(token-economy benchmark, v0.7.0). v0.9.0's win is on the latency
axis, not the token axis: the cache-backed bare repo means an agent
can \`grep -r\` an issue tracker without re-hitting the API for every
match.

## Latency table

| Step                                          | sim                          | github                       | confluence                   | jira                         |
|-----------------------------------------------|------------------------------|------------------------------|------------------------------|------------------------------|
| \`reposix init\` cold [^blob]                 | ${SIM_INIT_CELL}             | ${GH_INIT_CELL}              | ${CF_INIT_CELL}              | ${JR_INIT_CELL}              |
| List records [^N]                             | ${SIM_LIST_CELL}             | ${GH_LIST_CELL}              | ${CF_LIST_CELL}              | ${JR_LIST_CELL}              |
| Get one record                                | ${SIM_GET_CELL}              | ${GH_GET_CELL}               | ${CF_GET_CELL}               | ${JR_GET_CELL}               |
| PATCH record (no-op)                          | ${SIM_PATCH_CELL}            | ${GH_PATCH_CELL}             | ${CF_PATCH_CELL}             | ${JR_PATCH_CELL}             |
| Helper \`capabilities\` probe                 | ${SIM_CAP_CELL}              | ${GH_CAP_CELL}               | ${CF_CAP_CELL}               | ${JR_CAP_CELL}               |

[^blob]: \`reposix init\` materializes blobs lazily (partial clone with
    \`--filter=blob:none\`). Blob counts at end of init: sim=${SIM_BLOBS:-0},
    github=${GH_BLOBS:-0}, confluence=${CF_BLOBS:-0}, jira=${JR_BLOBS:-0}.
    A non-zero count means the helper served \`fetch\` requests from git
    that pulled actual blob bytes during the bootstrap fetch.
[^N]: \`N\` = records returned by the canonical list endpoint:
    sim/github/jira issues, confluence pages in the configured space.
    **N values reflect live backend state at run time** — the configured
    Confluence space (\`${CF_PROJECT:-TokenWorld}\`) and \`reubenjohn/reposix\`
    issue count drift over time; expect
    ±20% wobble between runs. The \`Helper capabilities probe\` row is
    local-only (no network), so it's identical across columns and serves
    as a runner-variance control.

Real-backend cells are populated by the \`bench-latency-v09\` CI job
(see [\`.github/workflows/ci.yml\`](https://github.com/reubenjohn/reposix/blob/main/.github/workflows/ci.yml)
for cadence; the weekly cron variant lives in
[\`.github/workflows/bench-latency-cron.yml\`](https://github.com/reubenjohn/reposix/blob/main/.github/workflows/bench-latency-cron.yml)).

## Soft thresholds

- **sim cold init < 500ms** — regression-flagged via \`WARN:\` line on
  stderr; not CI-blocking. Tracked here so a sudden 5x regression
  surfaces in PR review.
- **real-backend step < 3s** — same WARN-only mechanism.

## Reproduce

\`\`\`bash
bash scripts/latency-bench.sh
\`\`\`

The script regenerates this file in place. To capture real-backend
columns, export the relevant credential bundle before running:

\`\`\`bash
# GitHub (reubenjohn/reposix issues)
export GITHUB_TOKEN=…
# Confluence (space key set via REPOSIX_CONFLUENCE_SPACE; default TokenWorld)
export ATLASSIAN_API_KEY=… ATLASSIAN_EMAIL=… REPOSIX_CONFLUENCE_TENANT=… REPOSIX_CONFLUENCE_SPACE=…
# JIRA (TEST project, overridable via JIRA_TEST_PROJECT)
export JIRA_EMAIL=… JIRA_API_TOKEN=… REPOSIX_JIRA_INSTANCE=…

export REPOSIX_ALLOWED_ORIGINS='https://api.github.com,https://reuben-john.atlassian.net'
bash scripts/latency-bench.sh
\`\`\`

See \`docs/reference/testing-targets.md\` for the canonical safe-to-mutate
test targets.
MARKDOWN

echo "latency-bench: regenerated $OUT" >&2
echo "  sim    init=${SIM_INIT_MS}ms list=${SIM_LIST_MS}ms get=${SIM_GET_MS}ms patch=${SIM_PATCH_MS}ms cap=${SIM_CAP_MS}ms (N=${SIM_N}, blobs=${SIM_BLOBS:-0})" >&2
[[ -n "$GH_INIT_MS" ]] && echo "  github init=${GH_INIT_MS}ms list=${GH_LIST_MS}ms get=${GH_GET_MS}ms patch=${GH_PATCH_MS}ms cap=${GH_CAP_MS}ms (N=${GH_N}, blobs=${GH_BLOBS:-0})" >&2 || true
[[ -n "$CF_INIT_MS" ]] && echo "  confluence init=${CF_INIT_MS}ms list=${CF_LIST_MS}ms get=${CF_GET_MS}ms patch=${CF_PATCH_MS}ms cap=${CF_CAP_MS}ms (N=${CF_N}, blobs=${CF_BLOBS:-0})" >&2 || true
[[ -n "$JR_INIT_MS" ]] && echo "  jira   init=${JR_INIT_MS}ms list=${JR_LIST_MS}ms get=${JR_GET_MS}ms patch=${JR_PATCH_MS}ms cap=${JR_CAP_MS}ms (N=${JR_N}, blobs=${JR_BLOBS:-0})" >&2 || true
