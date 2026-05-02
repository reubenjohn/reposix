---
phase: 84
title: "Webhook-driven mirror sync — GH Action workflow + setup guide"
milestone: v0.13.0
requirements: [DVCS-WEBHOOK-01, DVCS-WEBHOOK-02, DVCS-WEBHOOK-03, DVCS-WEBHOOK-04]
depends_on: [80, 83]
plans:
  - 84-01-PLAN.md  # 6 tasks: catalog → workflow YAML → first-run test → race test → latency artifact → CLAUDE.md + push
waves:
  1: [84-01]
---

# Phase 84 — Webhook-driven mirror sync: GH Action workflow + setup guide (overview)

This is the FIFTH DVCS-substantive phase of milestone v0.13.0 — the
**pull side** of the bus topology. The bus remote (P82+P83) handles
the SoT-first / mirror-best-effort write path; P84 ships the
**reverse-direction sync** that keeps the GH mirror current with
confluence-side edits an agent never made (web-UI edits by humans,
edits via other tooling, edits by other reposix clones). The
substrate is a GitHub Actions workflow living in the mirror repo
itself (`reubenjohn/reposix-tokenworld-mirror`, NOT
`reubenjohn/reposix`) — `repository_dispatch` for the webhook
trigger plus a `*/30 * * * *` cron safety net.

Per `decisions.md` Q4.1 (RATIFIED cron `*/30` default, configurable),
Q4.2 (RATIFIED backends-without-webhooks → cron-only mode is just
the workflow with the `repository_dispatch` block deleted; no extra
implementation), Q4.3 (RATIFIED first-run on empty mirror handled
gracefully — the YAML branches on `git show-ref --verify --quiet
refs/remotes/mirror/main` and falls back to plain `git push` when
the local mirror tracking ref is absent), and the architecture-
sketch's verbatim YAML skeleton at lines 210–236.

**Single plan, six sequential tasks** per RESEARCH.md § "Plan
Splitting". P84's risk surface is much narrower than P83's — no Rust
integration tests touching audit-table schemas, no fault-injection
matrices, no `BackendConnector` trait changes. The whole phase is
**YAML + shell + one JSON measurement artifact + a CLAUDE.md
update**. Total cargo touch in the phase: ZERO new compiles (the
workflow itself uses `cargo binstall reposix-cli`; local tests are
shell-only). The build-memory-budget rule is trivially satisfied.

**Workflow-repo split is load-bearing.** Per CARRY-FORWARD §
DVCS-MIRROR-REPO-01 P84 bullet (lines 152–155), the workflow file
lands in `reubenjohn/reposix-tokenworld-mirror` (the real GH mirror
repo created 2026-05-01), NOT in `reubenjohn/reposix`. The
canonical reposix repo's commit is the **template + setup-guide
forward-link**, NOT the live workflow. T01 surfaces this distinction
explicitly so an executor working from the canonical repo's working
tree does not accidentally commit `.github/workflows/reposix-mirror-sync.yml`
into the wrong place. The recommended shape (from RESEARCH.md
§ "Claude's Discretion"):

- **Live copy** at `<mirror-repo>/.github/workflows/reposix-mirror-sync.yml`
  — runs the cron + dispatch.
- **Template copy** at `docs/guides/dvcs-mirror-setup-template.yml`
  in this repo — referenced by P85's setup guide and **byte-equal
  (modulo whitespace)** to the live copy. A T06 verifier asserts the
  diff is whitespace-only or zero.

T02 ships BOTH files. T02's git push lands the template in
`reubenjohn/reposix` first; pushing the live copy into the mirror
repo is a SEPARATE git operation T02 performs against
`reubenjohn/reposix-tokenworld-mirror` (using `gh auth status`
confirmed scopes per CARRY-FORWARD). The verifier asserts (a) the
template exists in the canonical repo, AND (b) `gh api repos/
reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/
reposix-mirror-sync.yml` returns 200 — so the test fails if either
copy is missing.

**Six task sequence:**

- **T01 — Catalog-first.** Six rows mint BEFORE any workflow YAML
  edits in `quality/catalogs/agent-ux.json` (the existing dimension
  home alongside `agent-ux/dark-factory-sim`,
  `agent-ux/reposix-attach-against-vanilla-clone`, `agent-ux/mirror-refs-*`,
  `agent-ux/sync-reconcile-subcommand`, `agent-ux/bus-precheck-*`,
  `agent-ux/bus-write-*` from P79–P83). Five rows track DVCS-WEBHOOK-01..04
  + the Q4.2 backends-without-webhooks fallback path; the sixth row
  covers the `--force-with-lease` race-protection invariant
  (DVCS-WEBHOOK-02 split into trigger-shape vs lease-shape coverage).
  Six TINY shell verifiers under `quality/gates/agent-ux/`. Initial
  status `FAIL`. Hand-edited per documented gap (NOT Principle A) —
  same shape as P79/P80/P81/P82/P83 row annotations, citing
  GOOD-TO-HAVES-01.

- **T02 — Workflow YAML at `.github/workflows/reposix-mirror-sync.yml`
  in mirror repo + template copy in canonical repo.** Author the
  YAML per the verbatim skeleton in RESEARCH.md § "Workflow YAML
  shape" (lines 100–193) with the three Q1/Q2/Q3 ratifications
  applied (concurrency block YES; latency cadence pre-release;
  inline 5-line tldr + forward-link to P85 in YAML header).
  `cargo binstall reposix-cli` (NOT `reposix` — published crate name
  per RESEARCH.md Pitfall 2). Cron literal `'*/30 * * * *'` (NOT
  `${{ vars.MIRROR_SYNC_CRON }}` — RESEARCH.md Pitfall 3). First-run
  branch in the push step uses `git show-ref --verify --quiet
  refs/remotes/mirror/main` to choose between lease push and plain
  push (Q4.3). After authoring locally in the canonical repo, T02
  pushes the live copy to the mirror repo via `gh api`-style
  authenticated push or a `git clone reubenjohn/reposix-tokenworld-mirror
  /tmp/mirror-repo && cp ... && git push` flow (executor decides the
  smaller-blast-radius path). T02 closes the `webhook-trigger-dispatch`,
  `webhook-cron-fallback`, and `webhook-backends-without-webhooks`
  catalog rows (the three rows whose verifiers fire purely on the
  YAML's existence + structural grep — no test fixture needed).

- **T03 — First-run handling test (`webhook-first-run-empty-mirror.sh`).**
  Shell harness with two sub-fixtures per RESEARCH.md § "First-run
  Handling (Q4.3)": (a) **fresh-but-readme** mirror seeded with one
  README commit (`git init --bare && git push --initial-commit`) —
  asserts the lease-push branch fires successfully, replacing the
  README with the SoT's main; (b) **truly-empty** mirror (`git init
  --bare` only, no `main` ref) — asserts the plain-push branch fires
  successfully, creating `main`. Both sub-fixtures are file:// bare
  repos via `tempfile::tempdir`-equivalent in shell (`mktemp -d` +
  `trap "rm -rf"`). Each sub-fixture exercises the SAME code path
  the live workflow runs on its push step (the conditional `if git
  show-ref --verify --quiet ... ; then ... else ... fi` block). T03
  closes `webhook-first-run-empty-mirror`.

- **T04 — `--force-with-lease` race test (`webhook-force-with-lease-race.sh`).**
  Shell harness from RESEARCH.md § "`--force-with-lease` Semantics"
  walk-through, ~50 lines: bare-repo mirror seeded with `SHA-A`;
  workflow-side working tree fetches mirror (sees `SHA-A`); a SECOND
  working tree (simulating the bus push winning the race) pushes
  `SHA-B` directly to the bare mirror; original workflow-side tree
  attempts `git push --force-with-lease=refs/heads/main:SHA-A`;
  asserts (a) exit code 1, (b) stderr contains one of `{stale info,
  rejected, non-fast-forward}` (git's exact wording varies by
  version — assert the SET), (c) mirror's `main` is still `SHA-B`
  (untouched by the workflow's failed push). The test runs in <1s
  on file:// bares. T04 closes `webhook-force-with-lease-race`.

- **T05 — Latency measurement artifact (`webhook-latency.json`).**
  Generate the headline latency artifact at
  `quality/reports/verifications/perf/webhook-latency.json` per
  ROADMAP SC4 + RESEARCH.md § "Latency Measurement Strategy".
  Methodology: **synthetic harness on CI** (lower-bound number) plus
  **ten-edit real TokenWorld pass run by the developer post-phase**
  (headline number). T05 ships the SYNTHETIC measurement (writes the
  JSON artifact with `method: "synthetic-dispatch"`, n=10, p50/p95/max
  fields, `verdict: "PASS"` if p95 ≤ 120s per ROADMAP SC4 falsifiable
  threshold) AND the measurement script
  `scripts/webhook-latency-measure.sh` so the real-TokenWorld pass is
  reproducible. The catalog row's cadence is `pre-release` (Q2
  ratification — re-measured per release, not per-PR). The verifier
  shell `webhook-latency-floor.sh` asserts the artifact's `p95_seconds`
  ≤ 120 (the falsifiable threshold per ROADMAP SC4). T05 closes
  `webhook-latency-floor`.

- **T06 — CLAUDE.md update + flip catalog rows + per-phase push +
  verifier dispatch.** Run `python3 quality/runners/run.py --cadence
  pre-pr --tag webhook` to flip the 5 pre-pr rows FAIL → PASS. The
  pre-release-cadence `webhook-latency-floor` row flips on T05's
  artifact landing (verifier asserts file exists + p95 ≤ 120).
  CLAUDE.md update lands in the same commit per OP-7 (one paragraph
  in § Architecture documenting the workflow path + secrets
  convention; one bullet in § Commands showing the dispatch
  invocation `gh api repos/.../dispatches -f event_type=reposix-mirror-sync`).
  `git push origin main` with pre-push GREEN. The orchestrator then
  dispatches the verifier subagent.

Sequential — never parallel. T01 → T02 → T03 → T04 → T05 → T06. No
cargo invocations means the build-memory rule is satisfied trivially;
sequencing is for narrative clarity (T02 depends on T01's catalog
rows; T03/T04/T05 each depend on T02's YAML existing) plus the
per-phase push protocol (T06 must come last).

## Chapters

- [Wave plan + Plan summary](./wave-plan.md) — wave structure, `files_modified` audit, plan summary table, and test-count breakdown.
- [Decisions D-01..D-08](./decisions.md) — all eight plan-time ratifications: concurrency block, latency cadence, YAML header shape, catalog home, `cargo binstall` name, cron literal, first-run predicate, and dual-copy contract.
- [Architecture S1–S2 + Hard constraints](./architecture-constraints.md) — subtle architectural points (fresh-but-readme sub-case, `client_payload` untrusted), fourteen hard constraints, and threat model crosswalk.
- [Execution: phase-close, risks, delegation, verification](./execution.md) — phase-close protocol, risks + mitigations table, +2 reservation, subagent delegation table, and developer-facing verification approach.
