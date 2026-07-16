---
last_measured_at: 2026-07-15T21:45:40Z
---

# v0.9.0 Latency Envelope

**Corrected:** 2026-07-15 — supersedes commit `9384ca6`, which reported a
machine-load noise outlier (155 ms) as the sim cold-init figure.
**Canonical source:** CI job `bench-latency-v09`, run
[`29452237641`](https://github.com/reubenjohn/reposix/actions/runs/29452237641),
commit `3278abc`, GitHub Actions `ubuntu-24.04` hosted runner, 2026-07-15.
**Reproducer:** `bash quality/gates/perf/latency-bench.sh` (regenerates the
dev-machine figures locally; the canonical figures below come from the
`bench-latency-v09` CI job log, not a local run).

## How to read this

reposix v0.9.0 replaces the per-read FUSE round-trip with a partial-clone
working tree backed by a `git-remote-reposix` promisor remote. The
golden-path latencies below characterize the sim backend (in-process
HTTP simulator) and the three real backends, all measured in a single
controlled run of the `bench-latency-v09` CI job. **What's measured:**
end-to-end wall-clock for each operation, single-threaded, against an
ephemeral sim DB or the corresponding real-backend REST API. **What's
NOT measured:** per-step sample variance within the run (see the
Provenance section below for why CI, not a dev machine, is the reference
environment) and cold-cache vs warm-cache TLS reuse for HTTPS backends (a
single warm-up GET amortizes the TLS handshake before timing).

The MCP/REST baseline comparison sits in `docs/benchmarks/token-economy.md`
(token-economy benchmark, v0.7.0). v0.9.0's win is on the latency
axis, not the token axis: the cache-backed bare repo means an agent
can `grep -r` an issue tracker without re-hitting the API for every
match.

## Latency table

| Step                                          | sim                   | github                 | confluence             | jira                   |
|-----------------------------------------------|-----------------------|-------------------------|-------------------------|-------------------------|
| `reposix init` cold [^blob]                   | 278 ms                | 830 ms                  | 1136 ms                 | 329 ms                  |
| List records [^N]                             | 7 ms (N=6)            | 779 ms (N=75)            | 215 ms (N=3)             | 226 ms (N=0)             |
| Get one record                                | 6 ms                  | 320 ms                   | 202 ms                   | n/a                      |
| PATCH record (no-op) [^patch]                 | 10 ms                 | 662 ms                   | 183 ms                   | n/a                      |
| Helper `capabilities` probe                   | 5 ms                  | 5 ms                     | 7 ms                     | 6 ms                     |

[^blob]: `reposix init` materializes blobs lazily (partial clone with
    `--filter=blob:none`). A non-zero blob count at end of init would mean
    the helper served `fetch` requests from git that pulled actual blob
    bytes during the bootstrap fetch — the steady-state expectation across
    all four backends is zero.
[^N]: `N` = records returned by the canonical list endpoint at run time:
    sim/github/jira issues, confluence pages in the configured space.
    jira's `N=0` in this run means the configured JIRA project returned no
    issues, which is why its `Get`/`PATCH` steps have no record to operate
    on and read `n/a`. **N values reflect live backend state at run time** —
    the configured Confluence space (`TokenWorld`) and `reubenjohn/reposix`
    issue count drift over time; expect wobble between runs. The `Helper
    capabilities probe` row is local-only (no network), so it's comparable
    across columns and serves as a runner-variance control.
[^patch]: **Caveat — do not read this as a clean PATCH figure.** The
    bench's PATCH probe sends an unsupported `expected_version` field,
    which the sim's issue-update handler rejects with a 400. The sim
    `patch=10 ms` figure above times that rejection path, not a
    successful patch. See "PATCH figures — known caveat" below.

Real-backend cells are populated by the `bench-latency-v09` CI job
(see [`.github/workflows/ci.yml`](https://github.com/reubenjohn/reposix/blob/main/.github/workflows/ci.yml)
for cadence; the weekly cron variant lives in
[`.github/workflows/bench-latency-cron.yml`](https://github.com/reubenjohn/reposix/blob/main/.github/workflows/bench-latency-cron.yml)).

## Provenance & methodology

These figures are measured by the CI job `bench-latency-v09`
([workflow run `29452237641`](https://github.com/reubenjohn/reposix/actions/runs/29452237641)),
executing at commit `3278abc` on a GitHub Actions `ubuntu-24.04` hosted
runner, 2026-07-15. The run log is the reproducible source of truth —
anyone can re-open that run, or trigger a fresh `bench-latency-v09` run,
and read the same numbers.

**Latency is environment-dependent — there is no single fixed number.**
A dev-VM warm sample of sim `init` (N=3 consecutive runs, no other load
on the box) measured 42/45/42 ms — roughly 6-7x faster than the CI
figure of 278 ms above. Neither number is "wrong"; they measure the same
code path under different hardware and contention profiles. **CI is the
canonical reference** for this document because (a) it is reproducible
from a committed, replayable artifact (the run log), not a one-off
sample on a machine whose load state isn't recorded, and (b) it runs on
every push, so a regression is caught continuously instead of relying on
someone remembering to re-run a benchmark by hand.

## PATCH figures — known caveat

The `PATCH record (no-op)` row above times the bench's PATCH probe
end-to-end, but the sim PATCH probe currently sends an unsupported
`expected_version` field in its request body, which `reposix-sim`'s
issue-update handler rejects with a 400
(`unknown field 'expected_version'`). **The sim `patch=10 ms` figure
therefore times an error-rejection path, not a successful patch** — it
is not a clean measurement of PATCH latency and should not be read as
one until the underlying bug is fixed. See the filed defect in
`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (2026-07-15,
"latency-bench PATCH probe sends unsupported `expected_version`"). The
github/confluence/jira PATCH figures come from the same probe code path,
so they should be cross-checked once the sim-side bug is resolved rather
than assumed clean by default.

## Soft thresholds

- **sim cold init < 500ms** — regression-flagged via `WARN:` line on
  stderr; not CI-blocking. Tracked here so a sudden 5x regression
  surfaces in PR review.
- **real-backend step < 3s** — same WARN-only mechanism.

## Reproduce

```bash
bash quality/gates/perf/latency-bench.sh
```

**Regen-clobber guard.** This file carries a `reposix:regen-guard:protected-*`
marker comment near the end (deliberately placed there, not wrapping the
content above, so it never shifts line numbers the docs-alignment catalog
cites into the sections above). `emit-markdown.sh` refuses to regenerate a
file that carries that marker unless you set
`REPOSIX_LATENCY_BENCH_ALLOW_CANONICAL_OVERWRITE=1` — the refusal message
names the exact recovery commands. To eyeball a local/sim run without
touching this tracked file, redirect the output instead:

```bash
OUT=/tmp/latency-preview.md bash quality/gates/perf/latency-bench.sh
```

To capture real-backend columns locally, export the relevant credential
bundle before running:

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

<!-- reposix:regen-guard:protected-begin -- this document is a hand-curated
     CI-canonical latency record (bench-latency-v09 run log figures above,
     plus the Provenance/PATCH-caveat write-ups). quality/gates/perf/
     latency-bench/emit-markdown.sh refuses to regenerate a file carrying
     this marker, because its template cannot reproduce the curated prose
     above and would silently drop it -- see that script's header comment
     (and latency-bench/regen-guard.sh) for the teaching error + recovery
     steps if you hit the guard. Deliberately placed at end-of-file (not
     wrapping the content above) so it never shifts the line numbers the
     docs-alignment catalog cites into this file's earlier sections.
     reposix:regen-guard:protected-end -->
