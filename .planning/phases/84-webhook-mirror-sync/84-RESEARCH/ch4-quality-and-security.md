# Phase 84 Research — Catalog Row Design, Test Infrastructure, Plan Splitting

← [back to index](./index.md)

## Catalog Row Design

Per ROADMAP SC7, six rows in `quality/catalogs/agent-ux.json`. Each follows the precedent shape from `agent-ux/bus-precheck-a-mirror-drift-emits-fetch-first` (`agent-ux.json`); each verifier under `quality/gates/agent-ux/` follows the shape of `bus-precheck-a-mirror-drift-emits-fetch-first.sh`. **All six rows are minted hand-edited** per the existing P79 footnote (`"_provenance_note": "Hand-edit per documented gap (NOT Principle A): reposix-quality bind only supports the docs-alignment dimension..."`); P84 inherits this constraint.

| Row ID | Verifier | Cadence | Kind | Sources |
|---|---|---|---|---|
| `agent-ux/webhook-trigger-dispatch` | `quality/gates/agent-ux/webhook-trigger-dispatch.sh` | pre-pr | mechanical | `.github/workflows/reposix-mirror-sync.yml` (in mirror repo); REQUIREMENTS DVCS-WEBHOOK-01 |
| `agent-ux/webhook-cron-fallback` | `quality/gates/agent-ux/webhook-cron-fallback.sh` | pre-pr | mechanical | same workflow file; verifies cron block parses + matches `*/30 * * * *` default |
| `agent-ux/webhook-force-with-lease-race` | `quality/gates/agent-ux/webhook-force-with-lease-race.sh` | pre-pr | mechanical | shell harness or `tests/webhook_force_with_lease_race.rs`; REQUIREMENTS DVCS-WEBHOOK-02 |
| `agent-ux/webhook-first-run-empty-mirror` | `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh` | pre-pr | mechanical | shell harness exercising both 4.3.a + 4.3.b sub-cases; REQUIREMENTS DVCS-WEBHOOK-03 |
| `agent-ux/webhook-latency-floor` | `quality/gates/agent-ux/webhook-latency-floor.sh` (asserts JSON p95 ≤ 120s) | pre-release | asset-exists | `quality/reports/verifications/perf/webhook-latency.json`; REQUIREMENTS DVCS-WEBHOOK-04 |
| `agent-ux/webhook-backends-without-webhooks` | `quality/gates/agent-ux/webhook-backends-without-webhooks.sh` (asserts the trim path produces a valid YAML by stripping `repository_dispatch:` block) | pre-pr | mechanical | workflow YAML + Q4.2 doc snippet |

**Catalog row order:** rows land in commit 1 (per CLAUDE.md "catalog-first rule"); verifiers are stubs that exit non-zero with a clear "not yet implemented" message until the corresponding implementation tasks land. After P84 closes, all six are GREEN.

The `webhook-latency-floor` row is the only one with cadence `pre-release` (others are `pre-pr`). Rationale: the latency artifact freshness TTL should match the perf-targets dimension's general cadence (re-measured per release, not per-PR). The walker's freshness check fires on the artifact's `measured_at` timestamp.

## Test Infrastructure

**Recommendation: shell harnesses, NOT `act`.**

Three test categories:

1. **YAML lint + structure:** `actionlint` (Go binary, fast) verifies the workflow file parses and uses valid action references. Add to `quality/gates/agent-ux/webhook-trigger-dispatch.sh` as a sub-step.

2. **`--force-with-lease` race protection:** shell harness using `git init --bare` for the "mirror." Walk-through above. ~50 lines of shell. Place at `quality/gates/agent-ux/webhook-force-with-lease-race.sh`. Runs in <1s.

3. **First-run handling:** shell harness with two sub-fixtures (4.3.a fresh-but-readme, 4.3.b truly-empty). Each fixture is a `git init --bare` with deterministic seed. Place at `quality/gates/agent-ux/webhook-first-run-empty-mirror.sh`. Runs in <2s.

**Why NOT `act`:**
- Pulls Docker images (~500MB initial) — heavy on the dev VM.
- Requires Docker daemon — adds setup friction.
- Doesn't run on the CI runner faithfully (different environment).
- The substrates we care about (`reposix init`, `git push --force-with-lease`) are testable WITHOUT a workflow runner — we test them directly.

**Real-backend test (gated by secrets, milestone-close):** the existing `agent_flow_real` integration tests in `crates/reposix-cli/tests/agent_flow_real.rs` are the precedent. Add `webhook_real_dispatch` as a new `#[ignore]`-gated test that uses `gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches -f event_type=reposix-mirror-sync` to trigger the workflow + polls for ref-update. Skip cleanly when `GITHUB_TOKEN` is absent. Adds < 30 lines of Rust.

## Plan Splitting Recommendation

**Single plan, ~6 tasks.** P84 is much narrower than P83's risk surface — no Rust code paths to refactor, no audit-table schema changes, no fault-injection coverage. The riskiest parts (race protection, first-run) are isolated test fixtures, not architecture.

Recommended task sequence:

| # | Task | Output |
|---|---|---|
| 1 | Catalog rows + stub verifiers | 6 rows in `agent-ux.json`; 6 shell stubs under `quality/gates/agent-ux/webhook-*.sh` (each `exit 1` with TODO message) |
| 2 | Workflow YAML at `.github/workflows/reposix-mirror-sync.yml` (in mirror repo) | YAML file landed; `webhook-trigger-dispatch.sh` + `webhook-cron-fallback.sh` + `webhook-backends-without-webhooks.sh` flip GREEN |
| 3 | First-run handling test | `webhook-first-run-empty-mirror.sh` flips GREEN; covers 4.3.a + 4.3.b |
| 4 | `--force-with-lease` race test | `webhook-force-with-lease-race.sh` flips GREEN |
| 5 | Latency measurement (real TokenWorld + synthetic harness) | `webhook-latency.json` artifact; `webhook-latency-floor.sh` flips GREEN |
| 6 | CLAUDE.md update + phase-close push | CLAUDE.md "v0.13.0 — in flight" section gains P84 entry; `git push origin main`; verifier subagent dispatch |

**Why no split:** total task surface is ~6 commits; no single task is risky enough to warrant isolation. Compare to P83's split which was driven by build-memory-budget concerns (fault-injection tests are heavy linkage) — P84 has zero new Rust integration tests; only YAML + shell + a few API polls.
