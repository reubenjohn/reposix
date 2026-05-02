← [back to index](./index.md) · phase 84 plan 01

# Objective & Architecture Overview

Land the **pull side** of the v0.13.0 DVCS topology — a GitHub
Actions workflow that keeps the GH mirror current with confluence-
side edits the agent never made (web-UI edits, edits via other
tooling, edits by other reposix clones). The workflow lives in the
mirror repo (`reubenjohn/reposix-tokenworld-mirror`), triggers on
`repository_dispatch` (event_type `reposix-mirror-sync`) plus a
`*/30 * * * *` cron safety net, runs `reposix init confluence::TokenWorld
/tmp/sot` to build the SoT cache, then `git push mirror main
--force-with-lease=...` against the mirror. First-run handling
(empty mirror or fresh-but-readme mirror) is graceful per Q4.3 —
the workflow branches on `git show-ref --verify --quiet
refs/remotes/mirror/main` and falls back to plain push when the
local tracking ref is absent. `--force-with-lease` race protection
defends against a bus push (P82+P83) landing between the workflow's
fetch and its push. Latency target: < 60s p95 (aspirational); 120s
p95 (falsifiable threshold per ROADMAP SC4).

## Task Structure

This is a **single plan, six sequential tasks** per RESEARCH.md
§ "Plan Splitting":

- **T01** — Catalog-first: 6 rows in `quality/catalogs/agent-ux.json`
  + 6 TINY verifier shells (status FAIL).
- **T02** — Workflow YAML at TWO locations: template copy at
  `docs/guides/dvcs-mirror-setup-template.yml` (canonical repo) AND
  live copy at `<mirror-repo>/.github/workflows/reposix-mirror-sync.yml`
  (`reubenjohn/reposix-tokenworld-mirror`). Pushed to mirror repo
  via separate `git clone /tmp + cp + git push` flow.
- **T03** — Shell harness `webhook-first-run-empty-mirror.sh` —
  exercises both Q4.3 sub-cases (4.3.a fresh-but-readme; 4.3.b
  truly-empty) against file:// bare-repo fixtures.
- **T04** — Shell harness `webhook-force-with-lease-race.sh` —
  ~50-line race walk-through fixture; bare-repo seeded with `SHA-A`,
  bus-push wins race to `SHA-B`, original workflow's lease-push
  attempt rejects cleanly.
- **T05** — Latency artifact at `quality/reports/verifications/perf/webhook-latency.json`
  + `scripts/webhook-latency-measure.sh` (owner-runnable script for
  the headline real-TokenWorld n=10 measurement). T05's commit
  ships the SYNTHETIC-method JSON (CI-runnable lower bound).
- **T06** — Catalog flip FAIL → PASS + CLAUDE.md update + per-phase
  push.

Sequential (T01 → T02 → T03 → T04 → T05 → T06). No cargo invocations
in any task — the workflow uses `cargo binstall` (no compilation);
local verifiers are shell-only. CLAUDE.md "Build memory budget" is
trivially satisfied; sequencing is for narrative + dependency
clarity.

## Architecture

The workflow YAML lives in the **mirror repo**, not the canonical
repo (CARRY-FORWARD § DVCS-MIRROR-REPO-01 P84 bullet). T02 ships
TWO copies: the LIVE copy (active workflow, in mirror repo's
`.github/workflows/`) and the TEMPLATE copy (referenced by P85's
setup guide, in canonical repo's `docs/guides/`). The two are
byte-equal modulo whitespace; T02's verifier asserts the diff is
zero (`diff -w`).

`State` is unmodified. `BackendConnector` is unmodified. There are
NO new Rust modules and NO compiled crates touched by P84. The
phase is YAML + shell + JSON + a CLAUDE.md paragraph.

The first-run-handling predicate is `git show-ref --verify --quiet
refs/remotes/mirror/main` (D-07 of OVERVIEW). If present → lease
push. If absent → plain push. The predicate runs in the workflow's
push step; T03's harness exercises BOTH branches against file://
bare repos.

The `--force-with-lease` semantics (D-04 of OVERVIEW; verbatim from
RESEARCH.md § "`--force-with-lease` Semantics"): the YAML's lease
expression is `--force-with-lease=refs/heads/main:${LEASE_SHA}`
where `${LEASE_SHA}` is `git rev-parse refs/remotes/mirror/main`'s
output. The lease check fails IFF the remote's `main` has moved
since the local fetch — which is exactly the bus-push-wins-race
case. T04's harness simulates this race and asserts the rejection
plus the mirror-state-untouched property.

The latency artifact at `quality/reports/verifications/perf/webhook-latency.json`
follows the per-release-cadence pattern (D-02 of OVERVIEW). T05
ships the synthetic-method measurement (CI-runnable; lower-bound);
the headline real-TokenWorld n=10 number is captured by the OWNER
running `scripts/webhook-latency-measure.sh` post-phase. The
verifier `webhook-latency-floor.sh` asserts `p95_seconds ≤ 120`
(falsifiable threshold) regardless of method.

The 6 catalog rows ALL live in `quality/catalogs/agent-ux.json`
(D-04 of OVERVIEW). 5 are `pre-pr` cadence; 1 (`webhook-latency-floor`)
is `pre-release` cadence. Initial status FAIL; flip to PASS via
`python3 quality/runners/run.py` in T06 BEFORE the per-phase push
commits.

## Execution Notes

**Best-effort vs hard-error semantics (workflow runtime):**

- **STEP 1 (`actions/checkout@v6` with `fetch-depth: 0`):** hard
  error → workflow run fails fast (GH Actions native).
- **STEP 2 (`cargo binstall reposix-cli`):** hard error → workflow
  run fails fast (binstall non-zero exit).
- **STEP 3 (`reposix init confluence::TokenWorld /tmp/sot`):** hard
  error → workflow run fails fast (Atlassian credentials missing,
  TokenWorld unreachable, etc.).
- **STEP 4 (`git remote add mirror $MIRROR_URL` + `git fetch mirror
  main`):** the fetch may fail with "couldn't find remote ref main"
  on a truly-empty mirror — the YAML's `2>/dev/null || echo "first-run:
  ..."` handles this gracefully (D-07).
- **STEP 5 (`git push --force-with-lease=...`):** rejection on race
  is the CORRECT outcome — workflow exits 1, GH Actions logs the
  failed run, next cron tick re-attempts cleanly (the bus push
  already updated the mirror; second run sees no drift).
- **STEP 6 (mirror-lag refs push):** namespace-pushed in a SEPARATE
  invocation; failure is non-fatal (`|| echo warn`); next cron tick
  re-attempts.

This plan **runs no cargo** per CLAUDE.md "Build memory budget" —
trivially satisfied. Per-crate fallback rules don't apply (no
crates touched).

This plan terminates with `git push origin main` against the
canonical repo (per CLAUDE.md push cadence) with pre-push GREEN.
The catalog rows' initial FAIL status is acceptable through T01–T05
because the rows' verifiers fail-fast on the missing artifacts; the
runner re-grades to PASS during T06 BEFORE the push commits.
