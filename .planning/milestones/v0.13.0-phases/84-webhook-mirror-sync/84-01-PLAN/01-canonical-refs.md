← [index](./index.md)

# Canonical Refs

<canonical_refs>
**Spec sources:**
- `.planning/REQUIREMENTS.md` DVCS-WEBHOOK-01..04 (lines 94-97) —
  verbatim acceptance.
- `.planning/ROADMAP.md` § Phase 84 (lines 167-188) — phase goal +
  8 success criteria.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Webhook-
  driven mirror sync" (lines 204-247) — verbatim YAML skeleton +
  the webhook setup discussion + Q4.1/Q4.2/Q4.3 open questions.
- `.planning/research/v0.13.0-dvcs/decisions.md` Q4.1 (cron `*/30`
  configurable), Q4.2 (backends without webhooks → cron-only mode),
  Q4.3 (first-run on empty mirror handled gracefully).
- `.planning/phases/84-webhook-mirror-sync/84-RESEARCH.md` — full
  research bundle (especially § "Workflow YAML Shape", § "First-run
  Handling (Q4.3)", § "`--force-with-lease` Semantics", § "Latency
  Measurement Strategy", § "Common Pitfalls" 1-7).
- `.planning/phases/84-webhook-mirror-sync/84-PLAN-OVERVIEW.md`
  § "Decisions ratified at plan time" (D-01..D-08).
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md` §
  DVCS-MIRROR-REPO-01 (lines 126-183) — the real GH mirror's
  existence + scopes + workflow lands in mirror repo.

**Workflow YAML precedents (T02):**
- `.github/workflows/bench-latency-cron.yml` — cron-driven workflow
  donor pattern (literal cron string at line 18; `actions/checkout@v6`
  + `dtolnay/rust-toolchain@stable` + secrets shape).
- `.github/workflows/ci.yml:16-18` — `concurrency:` block precedent
  (different cancellation semantics; D-01 chooses
  `cancel-in-progress: false` for sync workflows).
- `.github/workflows/ci.yml:114-120` — `gh secret set ATLASSIAN_*`
  one-time-owner-action precedent.
- `.github/workflows/release.yml` — real-backend smoke-test pattern
  (env-block + secrets-into-env shape).

**First-run + race fixtures (T03 + T04):**
- `quality/gates/agent-ux/dark-factory.sh` — file:// bare-repo
  fixture donor pattern (existing 2-arm precedent that uses `git
  init --bare` + temp-dir trap-cleanup).
- `scripts/dark-factory-test.sh` (pre-migration ancestor) — the
  legacy file:// fixture pattern; same style.

**Latency artifact + measurement (T05):**
- `quality/reports/verifications/agent-ux/dark-factory-sim.json` —
  agent-ux verification artifact JSON shape donor (status,
  measured_at, etc.).
- RESEARCH.md § "Latency Measurement Strategy" — the n=10 + p95
  computation methodology + the JSON template.

**Quality Gates:**
- `quality/catalogs/agent-ux.json` — existing file with rows from
  P79–P83 (P79 attach, P80 mirror-refs, P81 sync-reconcile, P82 +
  P83 bus-* row family); the 6 new rows join.
- `quality/gates/agent-ux/sync-reconcile-subcommand.sh` (P81 TINY
  verifier precedent — 30-line shape).
- `quality/gates/agent-ux/bus-fetch-not-advertised.sh` (P82 TINY
  verifier precedent — grep-shape).
- `quality/PROTOCOL.md` § "Verifier subagent prompt template" + §
  "Principle A".

**Operating principles:**
- `CLAUDE.md` § "Build memory budget" — trivially satisfied (no
  cargo).
- `CLAUDE.md` § "Push cadence — per-phase" — terminal push
  protocol.
- `CLAUDE.md` § Operating Principles OP-1 (simulator-first — note:
  P84 is real-backend by definition; the synthetic harness is the
  CI-runnable surrogate), OP-3 (audit log unchanged in P84 — the
  workflow's `reposix init` step writes its own audit rows; phase
  itself adds none), OP-7 (verifier subagent), OP-8 (+2
  reservation).
- `CLAUDE.md` § "Threat model" — `<threat_model>` section below
  enumerates the new boundary (the `repository_dispatch`
  `client_payload` surface).
- `CLAUDE.md` § Quality Gates — 9 dimensions / 6 cadences / 5
  kinds.

This plan introduces ZERO new HTTP construction sites in code (the
workflow's `reposix init` step uses the existing `client()` factory
+ `REPOSIX_ALLOWED_ORIGINS` allowlist). The new threat surface is
the `repository_dispatch` event payload, mitigated by NEVER
interpolating `client_payload` into shell commands (S2 of OVERVIEW).
</canonical_refs>
