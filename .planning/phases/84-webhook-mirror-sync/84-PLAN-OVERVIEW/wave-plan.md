# Wave plan + Plan summary

← [back to index](./index.md)

## Wave plan

Strictly sequential — one plan, six tasks. T01 → T02 → T03 → T04 →
T05 → T06 within the same plan body.

| Wave | Plans  | Cargo? | File overlap         | Notes                                                                                                                |
|------|--------|--------|----------------------|----------------------------------------------------------------------------------------------------------------------|
| 1    | 84-01  | NO     | none with prior phase | catalog + workflow YAML (live + template) + 3 shell verifier harnesses + latency JSON artifact + CLAUDE.md + close   |

`files_modified` audit (single-plan phase, no cross-plan overlap to
audit):

| Plan  | Files                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
|-------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 84-01 | `docs/guides/dvcs-mirror-setup-template.yml` (new — template copy in canonical repo), `<mirror-repo>/.github/workflows/reposix-mirror-sync.yml` (new — live copy in `reubenjohn/reposix-tokenworld-mirror`; pushed via separate `gh`/`git` flow in T02), `quality/catalogs/agent-ux.json` (6 new rows), `quality/gates/agent-ux/webhook-trigger-dispatch.sh` (new), `quality/gates/agent-ux/webhook-cron-fallback.sh` (new), `quality/gates/agent-ux/webhook-force-with-lease-race.sh` (new), `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh` (new), `quality/gates/agent-ux/webhook-latency-floor.sh` (new), `quality/gates/agent-ux/webhook-backends-without-webhooks.sh` (new), `scripts/webhook-latency-measure.sh` (new — owner-runnable real-TokenWorld measurement script for the headline number), `quality/reports/verifications/perf/webhook-latency.json` (new — synthetic-method artifact landed in T05), `CLAUDE.md` |

P84 has zero new cargo workspace operations. The workflow uses
`cargo binstall` (no compilation); local tests are shell exclusively.
The `Build memory budget` rule is trivially satisfied. T01 + T02 +
T03 + T04 + T05 + T06 share the executor with no parallel cargo
invocations to coordinate.

## Plan summary table

| Plan  | Goal                                                                                                            | Tasks | Cargo? | Catalog rows minted | Tests/artifacts added                                                                                                            | Files modified (count) |
|-------|-----------------------------------------------------------------------------------------------------------------|-------|--------|----------------------|---------------------------------------------------------------------------------------------------------------------------------|------------------------|
| 84-01 | Webhook-driven mirror sync workflow + 3 shell test harnesses + latency artifact + close                         | 6     | NO     | 6 (status FAIL → PASS at T06) | 0 Rust unit/integration tests; 3 shell-harness verifiers (`webhook-first-run-empty-mirror.sh`, `webhook-force-with-lease-race.sh`, `webhook-latency-floor.sh`) + 3 grep-shape verifiers (`webhook-trigger-dispatch.sh`, `webhook-cron-fallback.sh`, `webhook-backends-without-webhooks.sh`) + 1 measurement artifact (`webhook-latency.json`) + 1 owner-runnable measurement script (`scripts/webhook-latency-measure.sh`) | ~13 (1 template YAML in canonical repo + 1 live YAML in mirror repo + 6 new verifier shells + 1 measurement script + 1 JSON artifact + 1 catalog edit + CLAUDE.md) |

Total: 6 tasks across 1 plan. Wave plan: sequential.

Test count: 0 Rust unit, 0 Rust integration. 3 shell harnesses
(file://-bare-repo fixtures, all <2s wall time each) + 3 grep-shape
verifiers + 1 JSON-asset-exists verifier (`webhook-latency-floor.sh`).
The shell harnesses follow the pattern of `quality/gates/agent-ux/
dark-factory.sh` (existing 2-arm precedent — file:// + `git init
--bare` + temp-dir trap-cleanup). The grep-shape verifiers follow
the pattern of P82's `bus-fetch-not-advertised.sh` (TINY ~30-line
delegate to a single check).
