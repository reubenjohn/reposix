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

## Decisions ratified at plan time

The four open questions surfaced by RESEARCH.md § "Open Questions"
are RATIFIED here so the executing subagent and the verifier
subagent both grade against the same contract. Decisions D-01..D-08
correspond to RESEARCH.md OQ#1..#4 plus four planner-discretion
calls.

### D-01 — Q1 (concurrency block): YES, add `concurrency` block to the workflow (RATIFIED)

**Decision:** the workflow YAML adds a top-level
`concurrency: { group: reposix-mirror-sync, cancel-in-progress: false }`
block. NOT cancel-in-progress (per below).

**Implementation (T02):** add to YAML at top-level (sibling of `on:`,
`permissions:`, `env:`, `jobs:`):

```yaml
concurrency:
  group: reposix-mirror-sync
  cancel-in-progress: false
```

**Why YES (vs leaving it off):** cron + `repository_dispatch` could
fire 2× near a 30-min boundary if a webhook fires within seconds of
a cron tick. While `--force-with-lease` makes the second push a
no-op (the first run's push advanced `main`; the second run's lease
check fails cleanly per the race walk-through in RESEARCH.md), the
cost-of-protecting is 5 lines of YAML; the cost-of-being-bitten is
twin runs racing through `reposix init` + cache build (~2-3 min of
runner time) and producing duplicate `audit_events_cache` blob-mat
rows. Idempotent at the mirror level, but wasteful at the runner
level.

**Why `cancel-in-progress: false` (NOT true):** `cancel-in-progress:
true` would mean a cron tick that fires while a webhook-dispatched
run is in flight CANCELS the in-flight run. That's the wrong
direction — the in-flight run is already syncing the mirror; killing
it mid-sync produces a partially-applied state (some blobs
materialized, no push). Better to QUEUE the second run behind the
first; the second run sees `main` already in sync and exits cleanly.

**Precedent:** `ci.yml:16-18` uses `concurrency: { group: ${{
github.workflow }}-${{ github.ref }}, cancel-in-progress: true }`.
The cancellation semantics are different there (CI is OK to cancel
because a new push supersedes the old one); for sync workflows
queue-don't-cancel is the idiomatic choice.

**Source:** RESEARCH.md § "Open Questions" Q2 ("Does the workflow
need `concurrency:` to prevent overlapping runs?"); `ci.yml:16-18`
precedent.

### D-02 — Q2 (latency measurement cadence): one-shot artifact with `cadence: pre-release` (RATIFIED)

**Decision:** the `webhook-latency-floor` catalog row has `cadence:
pre-release` (NOT `pre-pr`, NOT `weekly`). The `webhook-latency.json`
artifact is one-shot per release; T05 lands the v0.13.0 measurement;
re-measurement happens at v0.14.0 P0 or whenever a perf-related code
path changes.

**Why pre-release (NOT pre-pr):** the latency measurement is real
network I/O against TokenWorld + the GH Actions runner. Running it on
every PR is (a) flaky (TokenWorld rate limits, GH Actions runner
cold-start variance, atlas API hiccups), (b) costly (10 manual edits
per measurement; can't be automated against TokenWorld without
mock-edit infrastructure that doesn't exist), and (c) overkill —
the substrate doesn't change PR-to-PR. Tracking a per-release
number in `webhook-latency.json` as an asset-exists check is the
matching cadence.

**Why NOT `weekly`:** `weekly` cadence would fire `webhook-latency-floor`
in a cron context where the verifier needs to RE-RUN the measurement
to refresh the artifact. The synthetic-dispatch path (`gh api repos/
.../dispatches`) could in principle do this, but introduces noise
in the GH Actions tab and doesn't add signal — pre-release is when
the reader cares.

**Falsifiable threshold:** the verifier asserts `p95_seconds ≤ 120`
(per ROADMAP SC4 "if p95 > 120s, P85 docs document the constraint").
Aspirational target is `p95 ≤ 60`; failure threshold is `p95 ≤ 120`;
between-bands triggers a documentation entry but not a row failure.

**Open follow-up:** if owner load post-v0.13.0 surfaces value in
recurring measurement, a v0.14.0 GOOD-TO-HAVE row could change
cadence to `weekly` with a CI runner-side synthetic dispatch driver.
Not in scope for v0.13.0.

**Source:** RESEARCH.md § "Open Questions" Q1 ("Should the latency
measurement be a recurring CI check or one-shot artifact?"); ROADMAP
SC4 falsifiable threshold.

### D-03 — Q3 (workflow's setup-guide reference): inline 5-line tldr in YAML header + forward-link to P85 (RATIFIED)

**Decision:** the workflow YAML's header (top-of-file comment block)
includes a 5-line tldr summarizing what the workflow does, what
secrets it needs, and how to override the cron cadence — followed by
a forward-link to `docs/guides/dvcs-mirror-setup.md` (the P85
deliverable). The forward-link is fine because the docs file lands
before milestone close (P85 explicitly depends on P84 GREEN).

**Implementation (T02):** the YAML header block:

```yaml
# reposix-mirror-sync — v0.13.0 webhook-driven mirror sync
#
# What: sync the GH mirror with confluence-side edits (the pull
# direction; bus-remote handles the push direction).
# Triggers: repository_dispatch (event_type=reposix-mirror-sync) +
# cron */30min safety net + workflow_dispatch (manual).
# Secrets needed: ATLASSIAN_API_KEY, ATLASSIAN_EMAIL,
# REPOSIX_CONFLUENCE_TENANT (set via `gh secret set` on this repo).
# Cron override: edit this YAML directly (the schedule field cannot
# read ${{ vars.* }} — see Pitfall 3 in P84 RESEARCH).
# Full setup walk-through: docs/guides/dvcs-mirror-setup.md (P85).
```

**Why inline-tldr-plus-link (vs link-only or full-docs-inline):**
link-only assumes the reader has the canonical repo handy; an owner
hitting the workflow file from the mirror repo's GitHub UI may not
have that context — 5 lines of inline orientation is enough to
answer "what is this and what env does it need?". Full-docs-inline
duplicates information that gets stale; the tldr is the irreducible
minimum.

**Forward-reference acceptability:** P85 explicitly depends on P84
GREEN (ROADMAP P85 `Depends on: P79 + P80 + P81 + P82 + P83 + P84
ALL GREEN`). The link `docs/guides/dvcs-mirror-setup.md` will be
live by the time milestone v0.13.0 ships. If P85 slips, the link is
a 404 in the interim — acceptable because the workflow is owner-only
during the dev window; readers hitting the mirror repo UI in that
window are the owner who knows the doc is forthcoming.

**Source:** RESEARCH.md § "Open Questions" Q4 ("Should P84's plan
reference forward to P85's docs or vice-versa?"); ROADMAP P85
dependency.

### D-04 — `agent-ux.json` is the catalog home (NOT a new `webhook-sync.json`)

**Decision:** add the 6 new rows to the existing
`quality/catalogs/agent-ux.json` (joining the P79–P83 row family).
NOT a new `webhook-sync.json` or `release.json`.

**Why:** `agent-ux` is the existing dimension that P79–P83 already
populated; adding 6 more keeps the single-file shape readable.
P82's D-04 set the precedent ("dimension catalogs are routed to
`quality/gates/<dim>/` runner discovery — `agent-ux` is the existing
dimension"); P84 inherits.

**Why NOT a release-dimension row for `webhook-latency-floor`:** the
latency artifact is a perf claim about an agent-ux substrate (the
webhook sync is part of the agent UX — it's how the mirror stays
current with what the agent sees). Releases handle CHANGELOG +
asset-bytes claims; they don't track substrate-quality numbers.
`agent-ux` is the right dimension; `pre-release` is the right
cadence (D-02).

**Source:** RESEARCH.md § "Catalog Row Design" (recommends
`agent-ux.json`); P82 D-04 precedent.

### D-05 — `cargo binstall reposix-cli` (NOT `reposix`) — verbatim

**Decision:** the workflow YAML's install step uses `cargo binstall
reposix-cli`. NEVER `cargo binstall reposix`.

**Why:** the published crate name is `reposix-cli` (verified in
`crates/reposix-cli/Cargo.toml` `[package]` block). `reposix` is the
**workspace name**, not a crate name. The architecture-sketch's
"`cargo binstall reposix`" at line 223 is shorthand; it is incorrect
when literally executed in a workflow.

**Verification at T02:** add a comment in the YAML at the binstall
step pointing at the binstall metadata location (`crates/reposix-cli/Cargo.toml`
`[package.metadata.binstall]`). The verifier `webhook-trigger-dispatch.sh`
greps for `cargo binstall reposix-cli` (NOT `reposix`) in the
workflow YAML — failing the build if the wrong crate name appears.

**Source:** RESEARCH.md § "Standard Stack" + § "Common Pitfalls"
Pitfall 2; `crates/reposix-cli/Cargo.toml:19-25`.

### D-06 — Cron expression is a literal `'*/30 * * * *'` in the schedule field — NEVER `${{ vars.* }}`

**Decision:** the workflow `schedule:` block uses a literal
`'*/30 * * * *'` cron expression. The Q4.1 RATIFIED "configurable
via `vars.MIRROR_SYNC_CRON`" is satisfied by **editing the YAML
directly** when the owner wants a different cadence. NOT by
templating `${{ vars.MIRROR_SYNC_CRON }}` into the cron field.

**Why:** GitHub Actions parses the `schedule:` block at workflow-load
time, BEFORE the `${{ vars.* }}` context is evaluated. The cron
expression CANNOT be templated via `vars` — the templating is parsed
literally and the resulting cron is INVALID. RESEARCH.md Pitfall 3
documents this explicitly with the bench-cron precedent (which
hardcodes `'0 13 * * 1'`). The Q4.1 intent ("configurable") is
preserved by documenting the edit-YAML flow in P85's setup guide.

**Verification at T02:** the YAML's literal cron expression is
asserted by `webhook-cron-fallback.sh` via grep (`grep -F "*/30 * * *
*"` on the YAML file).

**Documentation (T06 + P85):** CLAUDE.md's P84 entry names the
constraint; P85's `dvcs-mirror-setup.md` provides the edit-the-YAML
instructions for owners who want a different cadence.

**Source:** RESEARCH.md § "Workflow YAML Shape" Note + Pitfall 3 +
Assumption A2; bench-latency-cron.yml precedent (line 18 hardcodes
its cron).

### D-07 — First-run handling: branch on `git show-ref --verify --quiet refs/remotes/mirror/main`

**Decision:** the workflow's "Push to mirror" step branches on the
LOCAL state of `refs/remotes/mirror/main` (set by the prior `git
fetch mirror` step). If the ref EXISTS locally → use `--force-with-lease`
push. If the ref is ABSENT locally → use plain `git push mirror main`
(no lease).

**Why this exact predicate (vs e.g. checking the remote directly):**
two reasons: (a) the `git fetch mirror` step IMMEDIATELY precedes
this branch — so `refs/remotes/mirror/main`'s presence is the
freshest possible signal of the remote's `main` existing or not.
(b) `git show-ref --verify --quiet` is a non-network local query —
zero round-trip cost, deterministic exit code (0 = present, 1 =
absent). Compare to a `git ls-remote` re-query at this point (one
extra network round-trip per workflow run, for no information gain
over what `git fetch` just told us).

**Race property:** between `git fetch mirror` and the push, a
concurrent bus push could change `mirror/main` from "present at
SHA-A" to "present at SHA-B". This is exactly the race that
`--force-with-lease=refs/heads/main:SHA-A` detects and rejects. The
race CANNOT, however, change `mirror/main` from absent → present in
this window UNLESS the bus push is itself the first-ever push — and
in that case the workflow's plain push will fail with "non-fast-forward"
and the next cron tick will see the present-state and lease-push.
Acceptable.

**Sub-case 4.3.a (fresh-but-readme):** `gh repo create --add-readme`
seeded the mirror with one README commit. After `git fetch mirror`,
`refs/remotes/mirror/main` IS present (pointing at the README
commit's SHA). The lease push branch fires; the workflow's `main`
(SoT content) replaces the README. Owner sees this on first
workflow run and is documented in P85.

**Sub-case 4.3.b (truly-empty):** `gh repo create` (no `--add-readme`)
or a `gh repo create` followed by an emptying push. After `git fetch
mirror`, `refs/remotes/mirror/main` is ABSENT (no remote `main`
exists to fetch). The plain-push branch fires; `git push mirror
main` creates the ref. Subsequent runs go through 4.3.a.

**Source:** RESEARCH.md § "First-run Handling (Q4.3)" + § "Workflow
YAML Shape" verbatim YAML; `git show-ref` man page for the predicate
semantics.

### D-08 — Live workflow + template copy are byte-equal modulo whitespace; T02 ships both

**Decision:** the workflow YAML exists in TWO places: (a)
**LIVE COPY** at `<mirror-repo>/.github/workflows/reposix-mirror-sync.yml`
(in `reubenjohn/reposix-tokenworld-mirror`); (b) **TEMPLATE COPY**
at `docs/guides/dvcs-mirror-setup-template.yml` (in `reubenjohn/reposix`).
T02 ships both atomically.

**Why both (vs only-live or only-template):**

- **Only-live:** P85's setup guide can't reference a file that lives
  in another repo without leaving a permanently dangling link or
  requiring readers to clone the mirror repo to read the YAML.
  Worse: catalog-first principle says we land the contract in this
  repo's catalog first; if the live copy is the only artifact and
  the verifier needs to see it, we'd need cross-repo verifier
  invocations (heavy; stateful in a way the rest of the catalog isn't).
- **Only-template:** T01–T05 work would land cleanly, but the actual
  GH Action would never run. The mirror would never sync. Phase
  ships catalog-green but the substrate is dead.

**Drift prevention:** T02's commit ships BOTH copies in the same
working session. The template path is `docs/guides/dvcs-mirror-setup-template.yml`
(NOT `.github/workflows/...`) so it doesn't accidentally activate as
a workflow in the canonical repo. The verifier
`webhook-trigger-dispatch.sh` asserts:

1. `docs/guides/dvcs-mirror-setup-template.yml` exists in the canonical
   repo and parses as YAML.
2. `gh api repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/
   workflows/reposix-mirror-sync.yml` returns 200 (live copy exists in
   mirror repo).
3. The two copies are byte-equal modulo whitespace (a `diff -w
   <template> <(gh api ... | jq -r .content | base64 -d)` check).

If the diff is non-zero modulo whitespace, the verifier fails — drift
is caught at the row level, NOT at runtime when the live workflow
breaks.

**Why-modulo-whitespace and not bit-for-bit:** YAML parsers tolerate
trailing-newline / indent-tab/space variations. Forcing bit-for-bit
would create false-positive drift on cosmetic diffs that don't
affect behavior. `diff -w` (ignore whitespace) is the standard
right-cardinality check for "logically identical".

**T02 cross-repo push protocol:** the executor performs the live-copy
push to the mirror repo via:

```bash
# Working tree is in canonical repo. Author template copy.
$EDITOR docs/guides/dvcs-mirror-setup-template.yml
git add docs/guides/dvcs-mirror-setup-template.yml
git commit -m "..."  # part of T02's atomic commit

# Push live copy to mirror repo via temp clone.
TMPDIR=$(mktemp -d); trap "rm -rf $TMPDIR" EXIT
git clone git@github.com:reubenjohn/reposix-tokenworld-mirror.git "$TMPDIR/mirror"
mkdir -p "$TMPDIR/mirror/.github/workflows"
cp docs/guides/dvcs-mirror-setup-template.yml \
   "$TMPDIR/mirror/.github/workflows/reposix-mirror-sync.yml"
cd "$TMPDIR/mirror"
git add .github/workflows/reposix-mirror-sync.yml
git commit -m "feat(workflow): add reposix-mirror-sync.yml (P84 / DVCS-WEBHOOK-01)"
git push origin main
```

This is a SEPARATE git operation from T02's commit-into-canonical-repo
flow. The two commits are NOT atomic across repos (no cross-repo
two-phase commit exists in git). If the canonical-repo commit lands
but the mirror-repo push fails, the verifier catches the drift on
next run; the executor fixes by retrying the mirror-repo push. This
is acceptable because the mirror-repo push is idempotent.

**Source:** RESEARCH.md § "Claude's Discretion" "Whether to ship the
workflow as a template..."; CARRY-FORWARD § DVCS-MIRROR-REPO-01
("workflow YAML lives in `reubenjohn/reposix-tokenworld-mirror`,
NOT in `reubenjohn/reposix`").

## Subtle architectural points (read before T02)

The two below are flagged because they are the most likely sources
of T02 review friction. Executor must internalize them before
authoring the YAML.

### S1 — `gh repo create --add-readme` produces sub-case 4.3.a, NOT 4.3.b

CARRY-FORWARD § DVCS-MIRROR-REPO-01 line 135–137 says the mirror is
"empty except auto-generated README from `gh repo create
--add-readme`." This means the mirror's `main` ref **exists** at the
README commit's SHA. RESEARCH.md A1 confirms this assumption. So
sub-case 4.3.a (`fresh-but-readme`) is the **actual** first-run state
of `reubenjohn/reposix-tokenworld-mirror` as of P84 launch.

**Why this matters for T02 + T03.** T02's YAML must handle BOTH
sub-cases (4.3.a + 4.3.b) regardless of which one is actually the
first-run state, because (a) future mirrors created by other owners
might be 4.3.b (no `--add-readme`), and (b) the catalog row asserts
the workflow handles BOTH gracefully (DVCS-WEBHOOK-03 verbatim text
mentions "no `refs/heads/main`, no `refs/mirrors/...`" — the
truly-empty case). T03's harness exercises BOTH sub-cases.

T02's YAML branch (`if git show-ref --verify --quiet ...`) handles
both sub-cases without distinguishing them at the YAML layer —
4.3.a goes through the lease-push branch, 4.3.b goes through the
plain-push branch. The branching predicate is the SAME for the live
workflow as for T03's test harness.

**First-run impact on `reposix-tokenworld-mirror`:** the first
real workflow run (after T02 ships the YAML) REPLACES the auto-
generated README on the mirror with the SoT's content (TokenWorld
pages exported as markdown). RESEARCH.md Pitfall 5 names this:
documented in P85 as expected behavior ("the mirror repo's README is
replaced on first sync").

### S2 — `client_payload` from `repository_dispatch` is UNTRUSTED — workflow IGNORES it

The `repository_dispatch` event carries an arbitrary `client_payload`
JSON object set by the dispatching party (Confluence's webhook
config, or `gh api ... -f client_payload=...`). RESEARCH.md § "Known
Threat Patterns" flags this as untrusted input.

**Why this matters for T02.** The workflow YAML's `run:` blocks must
NEVER interpolate `${{ github.event.client_payload.* }}` into shell
commands. All values consumed by the workflow are derived from
`secrets.*` and `vars.*` (which are repo-owner-controlled). The
verbatim YAML in RESEARCH.md § "Workflow YAML Shape" follows this
discipline — there's no `${{ github.event.client_payload.* }}`
reference anywhere.

**Verifier check (T02):** `webhook-trigger-dispatch.sh` greps the
YAML for `github.event.client_payload` and asserts ZERO matches.
Defense-in-depth: even if the YAML grows in the future, a regression
that reads payload into a `run:` block fires the verifier red.

## Hard constraints (carried into the plan body)

Per the orchestrator's instructions for P84 and CLAUDE.md operating
principles:

1. **Catalog-first (QG-06).** T01 mints SIX rows + SIX verifier shells
   BEFORE T02–T06 implementation. Initial status `FAIL`. Rows are
   hand-edited per documented gap (NOT Principle A) — annotated in
   commit message referencing GOOD-TO-HAVES-01. The agent-ux
   dimension's `bind` verb is not yet implemented; rows ship as
   hand-edits matching P79/P80/P81/P82/P83 precedent.
2. **No cargo invocations.** P84 has zero new Rust integration tests
   and no compilation. Local verifiers are shell scripts and
   `gh api`/`git`/`jq` invocations only. CLAUDE.md "Build memory
   budget" is trivially satisfied; no parallel-cargo coordination
   needed.
3. **Sequential task execution.** Tasks T01 → T02 → T03 → T04 → T05
   → T06 — never parallel. T02's YAML must exist before T03/T04/T05
   verifiers can target it; T06's catalog flip + push must come last.
4. **Workflow file lands in mirror repo, NOT canonical repo (D-08 +
   CARRY-FORWARD § DVCS-MIRROR-REPO-01).** T02 ships BOTH a template
   copy in `docs/guides/dvcs-mirror-setup-template.yml` (canonical)
   AND a live copy in `<mirror-repo>/.github/workflows/reposix-mirror-sync.yml`
   (mirror). The verifier checks both exist + are byte-equal modulo
   whitespace.
5. **`cargo binstall reposix-cli` (NOT `reposix`) per D-05.** Verifier
   greps the YAML for the correct crate name; wrong name fails the
   row.
6. **Literal cron `'*/30 * * * *'` per D-06.** NEVER `${{ vars.* }}`
   in the schedule field. Verifier greps for the literal string.
7. **First-run branch in the push step uses `git show-ref --verify
   --quiet refs/remotes/mirror/main` per D-07.** Handles both
   sub-cases 4.3.a + 4.3.b without distinguishing them at the YAML
   layer.
8. **Concurrency block YES, `cancel-in-progress: false` per D-01.**
   Defends against duplicate runs near a cron-vs-dispatch boundary;
   queues rather than cancels.
9. **Latency artifact at `quality/reports/verifications/perf/webhook-latency.json`
   per D-02 + ROADMAP SC4.** T05 ships the synthetic-method JSON
   with `verdict: "PASS"` if `p95_seconds ≤ 120` (falsifiable
   threshold). The catalog row's cadence is `pre-release`.
10. **Workflow YAML's `client_payload` MUST NOT be interpolated into
    `run:` blocks per S2.** Verifier greps for `github.event.client_payload`
    and asserts zero matches.
11. **Per-phase push BEFORE verifier (CLAUDE.md "Push cadence — per-phase",
    codified 2026-04-30).** T06 ends with `git push origin main`
    against the canonical repo; pre-push gate must pass; verifier
    subagent grades the six catalog rows AFTER push lands. Verifier
    dispatch is an orchestrator-level action AFTER this plan
    completes — NOT a plan task. The mirror-repo push (T02 step 2)
    is SEPARATE and follows its own pre-push (no pre-push hook on
    the mirror repo since it has no source code).
12. **CLAUDE.md update in same PR (QG-07).** T06 documents the
    workflow path (§ Architecture) + secrets convention + the
    `gh api ... dispatches` invocation form (§ Commands or new
    "Mirror sync" sub-section).
13. **Six-row catalog set in `agent-ux.json` (D-04).** NOT a new
    `webhook-sync.json`. Rows joining the existing
    `agent-ux/dark-factory-sim`, `agent-ux/reposix-attach-*`,
    `agent-ux/mirror-refs-*`, `agent-ux/sync-reconcile-*`,
    `agent-ux/bus-*` family.
14. **`--` separator + `-` rejection NOT in scope here.** P82's
    `git ls-remote` shell-out used these as Tampering mitigations
    for argument-injection via `mirror_url`. P84's workflow runs in
    a GH Actions context with `mirror_url` derived from
    `${{ github.server_url }}/${{ github.repository }}.git` — server-
    side fixed values, no user-input attack surface. The mitigation
    is unnecessary at the YAML layer. (P82's PRECHECK A still uses
    them; P84's workflow is a different threat model.)

## Threat model crosswalk

Per CLAUDE.md § "Threat model" — this phase introduces ONE new
trifecta surface (the workflow's `repository_dispatch` event-payload
boundary) and inherits two existing surfaces.

| Existing/new surface                      | What P84 changes                                                                                                                                                                                                                                                                                |
|-------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Helper outbound HTTP (cache → confluence) | UNCHANGED — workflow's `reposix init` step uses the same `BackendConnector` trait + `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist. The only delta is that `REPOSIX_ALLOWED_ORIGINS` is set to confluence-tenant-only in the workflow env. No new HTTP construction site introduced. |
| Cache prior-blob parse (Tainted bytes)    | UNCHANGED — workflow does not introduce new tainted-byte sources. The cache built by `reposix init` is ephemeral (deleted post-runner-shutdown).                                                                                                                                                |
| **`repository_dispatch` `client_payload` (NEW)** | NEW: the workflow receives a JSON `client_payload` from the dispatching party (Confluence webhook or any party with a PAT to dispatch). Threat: payload-injection into `run:` shell commands → arbitrary code execution on the runner. Mitigation: workflow IGNORES `client_payload` — derives all values from `secrets.*` and `vars.*` (per RESEARCH.md "Known Threat Patterns" § Untrusted client_payload injection). STRIDE: Tampering — mitigated by S2 + verifier grep check. |
| **GH PAT for cross-repo webhook dispatch (NEW)** | NEW: Confluence's outbound webhook needs a GH PAT with `repo` scope to dispatch into the mirror repo. The PAT is configured on the Atlassian webhook side (NOT in the workflow). RESEARCH.md Pitfall 7 documents the requirement; P85's setup guide walks through it. STRIDE: Authentication / Spoofing — mitigated by user-controlled PAT scope; not exposed to the workflow. |

`<threat_model>` STRIDE register addendum below the per-task threat
register in the plan body:

- **T-84-01 (Tampering — `client_payload` injection):** workflow
  IGNORES the payload (no `${{ github.event.client_payload.* }}`
  references); verifier grep check.
- **T-84-02 (Information Disclosure — secrets in workflow logs):**
  GH Actions auto-redacts `${{ secrets.* }}` in step output;
  workflow avoids `set -x` in `run:` blocks.
- **T-84-03 (Denial of Service — workflow-trigger amplification):**
  cron + dispatch combined could fire 2× per 30min; the `concurrency:
  cancel-in-progress: false` block (D-01) queues the second run; the
  second run sees `main` in sync via `--force-with-lease` and exits
  cleanly. Wasted runner time bounded by GH's 6-hour job limit per
  workflow run; for a 5-min sync, near-zero risk.
- **T-84-04 (Elevation of Privilege — non-owner pushes to mirror via
  workflow):** workflow's `permissions: contents: write` is scoped
  to the mirror repo only; only repo-write users can dispatch; PAT
  for cross-repo dispatch is owner-controlled. Documented in P85.

## Phase-close protocol

Per CLAUDE.md OP-7 + REQUIREMENTS.md § "Recurring success criteria
across every v0.13.0 phase":

1. **All commits pushed.** Plan terminates with `git push origin
   main` against the CANONICAL repo in T06 (per CLAUDE.md "Push
   cadence — per-phase", codified 2026-04-30, closes backlog 999.4).
   Pre-push gate-passing is part of the plan's close criterion. The
   mirror-repo push (T02) was an earlier independent operation; T06
   does not re-touch the mirror repo.
2. **Pre-push gate GREEN.** If pre-push BLOCKS: treat as plan-internal
   failure (fix, NEW commit, re-push). NO `--no-verify` per CLAUDE.md
   git safety protocol.
3. **Verifier subagent dispatched.** AFTER 84-01 pushes (i.e., after
   T06 completes), the orchestrator dispatches an unbiased verifier
   subagent per `quality/PROTOCOL.md` § "Verifier subagent prompt
   template" (verbatim copy). The subagent grades the six P84
   catalog rows from artifacts with zero session context.
4. **Verdict at `quality/reports/verdicts/p84/VERDICT.md`.** Format
   per `quality/PROTOCOL.md`. Phase loops back if verdict is RED.
5. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "P83 SHIPPED ... next P84" → "P84 SHIPPED 2026-MM-DD"
   (commit SHA cited).
6. **CLAUDE.md updated in T06.** T06's CLAUDE.md edit lands in the
   terminal commit (one § Architecture paragraph + one § Commands
   bullet per QG-07).
7. **REQUIREMENTS.md DVCS-WEBHOOK-01..04 checkboxes flipped.**
   Orchestrator (top-level) flips `[ ]` → `[x]` after verifier GREEN.
   NOT a plan task.

## Risks + mitigations

| Risk                                                                                                  | Likelihood | Mitigation                                                                                                                                                                                                                                                                                                |
|-------------------------------------------------------------------------------------------------------|------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Executor commits the workflow YAML to the WRONG repo** (RESEARCH.md Pitfall 1) — lands in `reubenjohn/reposix` instead of `reubenjohn/reposix-tokenworld-mirror`. | MEDIUM     | T02's first sub-step explicitly names the target repo as the mirror. T02's verifier asserts `gh api repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/reposix-mirror-sync.yml` returns 200 (not a local file check). T01's plan body and T02's `<read_first>` both name the constraint at the top.        |
| **`cargo binstall reposix` instead of `cargo binstall reposix-cli`** (RESEARCH.md Pitfall 2)         | MEDIUM     | D-05 RATIFIED. Verifier greps the YAML for `cargo binstall reposix-cli` (and asserts `cargo binstall reposix\b` does NOT appear). Comment in YAML cites `crates/reposix-cli/Cargo.toml` `[package.metadata.binstall]`.                                                                                  |
| **`${{ vars.MIRROR_SYNC_CRON }}` in schedule field** (RESEARCH.md Pitfall 3 / Q4.1 misimpl)          | LOW-MED    | D-06 RATIFIED. Verifier greps for the literal `'*/30 * * * *'`. CLAUDE.md (T06) + P85 docs (later) document the edit-the-YAML flow for cadence overrides. Bench-cron precedent (`bench-latency-cron.yml:18` hardcodes `'0 13 * * 1'`) cited.                                                                  |
| **`actions/checkout@v6` `fetch-depth: 0` missing** (RESEARCH.md Pitfall 4) — shallow clone breaks `--force-with-lease`'s lease comparison | LOW        | T02 explicitly sets `fetch-depth: 0` in the checkout step per the verbatim YAML in RESEARCH.md. Verifier `webhook-cron-fallback.sh` greps for `fetch-depth: 0`.                                                                                                                                          |
| **Mirror's auto-generated README replaced on first run causes owner confusion** (RESEARCH.md Pitfall 5) | LOW        | Documented in P85's setup guide. Not a P84-blocker — the behavior is correct; the documentation is the mitigation.                                                                                                                                                                                          |
| **Workflow runs every 30 min during phase development, cluttering the mirror repo's Actions tab** (RESEARCH.md Pitfall 6) | MEDIUM     | T02's mirror-repo push lands LATE (after T01 catalog work in canonical repo, after authoring in canonical repo, after CI on the canonical repo's commit succeeds). Owner can disable the workflow on the mirror repo via `gh workflow disable reposix-mirror-sync --repo reubenjohn/reposix-tokenworld-mirror` between T02 and milestone close if cron noise is bothersome. Document in T02's notes. |
| **GH Actions `repository_dispatch` requires a PAT, not the runner `GITHUB_TOKEN`** (RESEARCH.md Pitfall 7) | LOW        | Documented in P85's setup guide; NOT a P84-implementation issue (the workflow itself receives the dispatch event; it doesn't trigger workflows in other repos). T05's synthetic-dispatch test uses the executor's `gh auth status`-confirmed token, which has `repo` scope.                                |
| **`cargo binstall reposix-cli` fails because the latest published version lacks the confluence init path** (RESEARCH.md Assumption A5) | LOW        | T02 verifies empirically before declaring the row green: dry-run `cargo binstall --dry-run reposix-cli` resolves a recent version (the executor sanity-checks). If a release-lag-driven gap surfaces, file as P84 SURPRISES-INTAKE entry — eager-resolve by pinning the binstall version in YAML.                  |
| **Synthetic dispatch via `gh api ... /dispatches` doesn't fire the workflow on `main` because it requires the workflow file already on `main`** | LOW        | T05's synthetic harness runs AFTER T02's mirror-repo push. The workflow is on `main` of the mirror repo by then. If it isn't, T05 fails with a clear "no workflow runs found" error — fix is sequencing, not a substrate issue.                                                                          |
| **Pre-push hook BLOCKs on a pre-existing drift unrelated to P84**                                    | LOW        | Per CLAUDE.md § "Push cadence — per-phase": treat as phase-internal failure. Diagnose, fix, NEW commit (NEVER amend), re-push. Do NOT bypass with `--no-verify`.                                                                                                                                            |
| **Real-TokenWorld latency measurement (`scripts/webhook-latency-measure.sh`) requires manual edits in TokenWorld**                          | LOW        | The script is OWNER-RUNNABLE, NOT CI-runnable. T05 ships the SCRIPT and the SYNTHETIC-method JSON; the real-TokenWorld pass is post-phase, owner-driven, and reported in a separate JSON refresh. Catalog row's `verdict: PASS` requires synthetic p95 ≤ 120 — achievable. |
| **Atlassian Cloud webhook config doesn't support the `Authorization: token <PAT>` header shape** (RESEARCH.md Assumption A6) | LOW        | Atlassian's outbound-webhook docs claim arbitrary URL + headers. If empirically broken, file as P85 SURPRISES-INTAKE entry — fallback is a relay service, but P85 is the right phase to discover this; P84 implements the receiving side correctly regardless.                                          |

## +2 reservation: out-of-scope candidates

`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
`GOOD-TO-HAVES.md` exist already (created during P79). P84 surfaces
candidates only when they materialize during execution — none
pre-filed at planning time.

Anticipated candidates the plan flags (per OP-8):

- **LOW** — Recurring weekly latency measurement. D-02 ratified
  one-shot at `pre-release` cadence; if v0.14.0 owner-load surfaces
  value in recurring measurement, file as v0.14.0 GOOD-TO-HAVE.
- **LOW** — `cancel-in-progress: true` for the concurrency block
  IF empirical multi-run measurements show queue-depth saturation.
  D-01 ratified `false` (queue, don't cancel); revisit if real data
  shows the queue growing unbounded.
- **LOW-MED** — Workflow file at the mirror repo's `main` branch
  fires its cron on every commit to the mirror repo. If the mirror's
  `main` advances frequently (e.g., 100+ pushes/day from the bus
  remote in production), the cron could overlap heavily with the
  push activity. D-01's `concurrency: cancel-in-progress: false`
  handles this, but at high volume the queue could grow. Eager-resolve
  IF empirical measurement shows it; file otherwise.
- **MED** — `actionlint` (Go binary) static-analysis of the workflow
  YAML in the verifier shells. RESEARCH.md § "Test Infrastructure"
  recommends but defers to "install on demand." If the verifier
  catches a YAML bug in production that `actionlint` would have
  caught at write-time, file as v0.14.0 GOOD-TO-HAVE to add it to
  pre-push as a structure-dim row.

Items NOT in scope for P84 (deferred per the v0.13.0 ROADMAP):

- DVCS docs (P85). T06 only updates CLAUDE.md; the
  `dvcs-mirror-setup.md` walkthrough lives in P85.
- Confluence webhook configuration automation. Owner-side manual
  setup per RESEARCH.md § "Deferred Ideas"; P85 documents the manual
  flow.
- Bidirectional sync. Mirror is read-only from confluence's
  perspective. Out of scope per architecture-sketch.
- Multi-mirror fan-out. Workflow targets a single mirror. v0.14.0.
- Real-TokenWorld latency headline number (the n=10 manual-edit
  pass). Owner runs `scripts/webhook-latency-measure.sh` post-phase;
  the JSON refresh is a separate commit (likely milestone-close P88
  or a v0.14.0 P0 task). T05 ships the synthetic number.
- `actionlint` integration. Filed as a candidate above.

## Subagent delegation

Per CLAUDE.md "Subagent delegation rules" + the gsd-planner spec
"aggressive subagent delegation":

| Plan / Task                                                      | Delegation                                                                                                                                                                                                                  |
|------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 84-01 T01 (6 catalog rows + 6 verifier shells)                   | `gsd-executor` — catalog-first commit; **hand-edits agent-ux.json per documented gap (NOT Principle A)**.                                                                                                                  |
| 84-01 T02 (workflow YAML — template in canonical + live in mirror) | Same 84-01 executor. Two git-push operations (canonical repo via working tree; mirror repo via temp clone). NO cargo invocations.                                                                                                |
| 84-01 T03 (first-run shell harness)                              | Same 84-01 executor. Pure shell + `git init --bare` fixtures. No cargo.                                                                                                                                                       |
| 84-01 T04 (force-with-lease race shell harness)                  | Same 84-01 executor. Pure shell + `git init --bare` fixtures. No cargo.                                                                                                                                                       |
| 84-01 T05 (latency artifact + measurement script)                | Same 84-01 executor. Synthetic `gh api`-based dispatch + JSON write. No cargo.                                                                                                                                                |
| 84-01 T06 (catalog flip + CLAUDE.md + push)                      | Same 84-01 executor (terminal task).                                                                                                                                                                                          |
| Phase verifier (P84 close)                                       | Unbiased subagent dispatched by orchestrator AFTER 84-01 T06 pushes per `quality/PROTOCOL.md` § "Verifier subagent prompt template" (verbatim). Zero session context; grades the six catalog rows from artifacts.        |

Phase verifier subagent's verdict criteria (extracted for P84):

- **DVCS-WEBHOOK-01:** template copy at `docs/guides/dvcs-mirror-setup-template.yml`
  exists and parses as valid YAML; live copy reachable via `gh api
  repos/reubenjohn/reposix-tokenworld-mirror/contents/.github/workflows/reposix-mirror-sync.yml`
  (200 OK); `diff -w` of the two copies returns zero (byte-equal
  modulo whitespace); workflow YAML contains `repository_dispatch`
  with `types: [reposix-mirror-sync]` AND `schedule:` with cron
  `'*/30 * * * *'` AND `cargo binstall reposix-cli` (NOT `reposix`)
  AND `actions/checkout@v6` with `fetch-depth: 0` AND
  `concurrency: { group: reposix-mirror-sync, cancel-in-progress:
  false }` (D-01).
- **DVCS-WEBHOOK-02:** workflow YAML's "Push to mirror" step uses
  `git push --force-with-lease=refs/heads/main:${LEASE_SHA}` shape
  (grep-able); shell harness `webhook-force-with-lease-race.sh`
  exits 0 (asserting the rejection-on-race property). Mirror's
  `main` ref untouched after the simulated race.
- **DVCS-WEBHOOK-03:** workflow YAML branches on `git show-ref
  --verify --quiet refs/remotes/mirror/main`; shell harness
  `webhook-first-run-empty-mirror.sh` exits 0 (asserting both 4.3.a
  and 4.3.b sub-cases handled correctly).
- **DVCS-WEBHOOK-04:** `quality/reports/verifications/perf/webhook-latency.json`
  exists with required fields (`measured_at`, `method`, `n`,
  `p50_seconds`, `p95_seconds`, `max_seconds`, `target_seconds`,
  `verdict`); `webhook-latency-floor.sh` asserts `p95_seconds ≤ 120`.
- **Q4.2 backends-without-webhooks (5th catalog row):** workflow's
  `repository_dispatch` block is deletable without breaking the
  cron-only path; the verifier `webhook-backends-without-webhooks.sh`
  greps the YAML for the documented trim path AND validates that
  removing the `repository_dispatch:` block produces still-valid
  YAML.
- **`client_payload` ignored (S2):** verifier greps the YAML for
  `github.event.client_payload` and asserts ZERO matches.
- New catalog rows in `quality/catalogs/agent-ux.json` (6); each
  verifier exits 0; status PASS after T06.
- Recurring (per phase): catalog-first ordering preserved (T01 commits
  catalog rows BEFORE T02–T06 implementation); per-phase push
  completed; verdict file at `quality/reports/verdicts/p84/VERDICT.md`;
  CLAUDE.md updated in T06.

## Verification approach (developer-facing)

After T06 pushes and the orchestrator dispatches the verifier subagent:

```bash
# Verifier-equivalent invocations (informational; the verifier subagent runs from artifacts):
bash quality/gates/agent-ux/webhook-trigger-dispatch.sh
bash quality/gates/agent-ux/webhook-cron-fallback.sh
bash quality/gates/agent-ux/webhook-force-with-lease-race.sh
bash quality/gates/agent-ux/webhook-first-run-empty-mirror.sh
bash quality/gates/agent-ux/webhook-backends-without-webhooks.sh
bash quality/gates/agent-ux/webhook-latency-floor.sh
python3 quality/runners/run.py --cadence pre-pr --tag webhook         # re-grade pre-pr rows
python3 quality/runners/run.py --cadence pre-release --tag webhook    # re-grade webhook-latency-floor

# Synthetic dispatch + ref-arrival check (manual; not part of the catalog):
gh api repos/reubenjohn/reposix-tokenworld-mirror/dispatches \
  -f event_type=reposix-mirror-sync \
  -f client_payload='{"trigger":"manual-test"}'
gh run watch --repo reubenjohn/reposix-tokenworld-mirror \
  $(gh run list --repo reubenjohn/reposix-tokenworld-mirror \
     --workflow=reposix-mirror-sync --limit 1 --json databaseId -q '.[0].databaseId')
gh api repos/reubenjohn/reposix-tokenworld-mirror/git/refs/mirrors/confluence-synced-at \
  -q .object.sha
```

The fixtures for T03 + T04 use **two local bare repos**
(`mktemp -d` + `git init --bare` + `file://` URL) per RESEARCH.md
§ "Test Fixture Strategy". Same approach as
`scripts/dark-factory-test.sh`.

This is a **subtle point worth flagging**: success criterion 2
(`--force-with-lease` race) is satisfied by two simultaneous
contracts — (a) the workflow YAML's push step uses the lease syntax
(grep-able from the YAML), AND (b) the shell harness asserts that an
actual lease check rejects on simulated drift. The integration tests
assert BOTH.
