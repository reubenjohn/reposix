---
phase: 84
plan: 01
title: "DVCS-WEBHOOK-01..04 — Webhook-driven mirror sync: GH Action workflow + setup guide"
wave: 1
depends_on: [80, 83]
requirements: [DVCS-WEBHOOK-01, DVCS-WEBHOOK-02, DVCS-WEBHOOK-03, DVCS-WEBHOOK-04]
files_modified:
  - docs/guides/dvcs-mirror-setup-template.yml
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/webhook-trigger-dispatch.sh
  - quality/gates/agent-ux/webhook-cron-fallback.sh
  - quality/gates/agent-ux/webhook-force-with-lease-race.sh
  - quality/gates/agent-ux/webhook-first-run-empty-mirror.sh
  - quality/gates/agent-ux/webhook-latency-floor.sh
  - quality/gates/agent-ux/webhook-backends-without-webhooks.sh
  - scripts/webhook-latency-measure.sh
  - quality/reports/verifications/perf/webhook-latency.json
  - CLAUDE.md
autonomous: true
mode: standard
---

# Phase 84 Plan 01 — Webhook-driven mirror sync

Organize this plan by reading its chapters in sequence. Each chapter is self-contained and can be jumped to directly. Begin with the objective for architectural context, then work through the six sequential tasks.

## Chapters

### [Objective & Architecture Overview](./00-objective.md)

The core requirements and execution model. Read this first to understand the workflow's role in the DVCS topology, latency targets, and the six sequential tasks that comprise the phase.

### [Must-Haves & Constraints](./01-must-haves.md)

The detailed specification for the workflow YAML, catalog rows, shell harnesses, and latency verification. Load-bearing details for implementation.

### [Threat Analysis](./02-threat-analysis.md)

Trust boundaries and STRIDE threat register. Covers the new webhook dispatching surface and existing trust relationships between workflow, Confluence, and the mirror repo.

### [Task 01: Catalog-First (Part A — Setup)](./T01a-setup.md)

Mint 6 catalog rows in `quality/catalogs/agent-ux.json`. Read-first references and the overall action structure for authoring TINY verifier shells.

### [Task 01: Catalog-First (Part B — Verifier Scripts)](./T01b-verifiers.md)

The six TINY verifier shell scripts covering webhook trigger dispatch, cron fallback, race handling, first-run cases, latency floor, and backends without webhooks.

### [Task 02: Workflow YAML](./T02-workflow-yaml.md)

Author the workflow YAML in two locations: the canonical template copy in `docs/guides/dvcs-mirror-setup-template.yml` and the live copy in the mirror repo's `.github/workflows/reposix-mirror-sync.yml`. Both are byte-equal modulo whitespace.

### [Task 03: First-Run Harness](./T03-first-run-harness.md)

Shell harness exercising both first-run cases (fresh-but-readme mirror vs. truly-empty mirror) against file:// bare-repo fixtures. Tests graceful handling per Q4.3.

### [Task 04: Force-With-Lease Race](./T04-force-with-lease-race.md)

Shell harness simulating the race between a concurrent bus push and the workflow's `--force-with-lease` push. Asserts correct rejection and mirror-state preservation.

### [Task 05: Latency Artifact](./T05-latency-artifact.md)

Generate the synthetic-method latency JSON artifact and author the owner-runnable measurement script for real-TokenWorld n=10 validation post-phase.

### [Task 06: Catalog Flip & Close](./T06-catalog-flip.md)

Flip catalog rows FAIL → PASS, update CLAUDE.md with one paragraph + one QG-07 bullet, and execute the terminal per-phase push.

## Dependencies

- **Depends on:** Phase 80 (bus remote URL parser) and Phase 83 (bus write fan-out).
- **Required by:** Phase 85 (P85 setup guide references the workflow YAML template).
- **Parallel-safe:** No overlap with concurrent phases.

## Execution Mode

Sequential tasks (T01 → T02 → ... → T06). No cargo invocations. Pure YAML, shell, JSON, and catalog operations. Trivially satisfies CLAUDE.md "Build memory budget" (no compiled crates touched).
